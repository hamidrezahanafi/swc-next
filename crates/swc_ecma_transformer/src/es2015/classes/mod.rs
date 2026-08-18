mod analysis;
mod helpers;
mod lower;
mod plan;

use swc_next_ecma_ast::Program;

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct Classes;

impl Classes {
    pub(super) fn transform<'ast>(
        &mut self,
        program: Program,
        ctx: &mut TransformerCtx<'_, '_, 'ast>,
    ) {
        debug_assert_eq!(
            ctx.current_scope(),
            Some(ctx.semantic().scope_of(program.node_id()))
        );
        let plans = analysis::analyze_program(program, ctx.ast, ctx.state.semantic);
        let expression_plans = analysis::analyze_named_expressions(ctx.ast, ctx.state.semantic);
        let anonymous_expression_plans =
            analysis::analyze_anonymous_expressions(ctx.ast, ctx.state.semantic);
        let anonymous_default_plans =
            analysis::analyze_anonymous_default_exports(program, ctx.ast, ctx.state.semantic);
        if plans.is_empty()
            && expression_plans.is_empty()
            && anonymous_expression_plans.is_empty()
            && anonymous_default_plans.is_empty()
        {
            return;
        }
        lower::lower_program(
            program,
            plans,
            expression_plans,
            anonymous_expression_plans,
            anonymous_default_plans,
            ctx,
        )
        .expect("class lowering mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, ExprData, FunctionType, NodeKind, NodeKindData, SourceType};
    use swc_next_ecma_codegen::{Options as PrintOptions, print};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, ExportTarget, SymbolFlags, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    fn transform_and_print(source: &str) -> String {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        print(&allocator, &ast, parse.program, PrintOptions::default())
            .code
            .to_owned()
    }

    fn assert_node_output(source: &str, expected: &str) {
        let source = transform_and_print(source);
        let output = Command::new("node")
            .args(["--input-type=module", "--eval", &source])
            .output()
            .expect("class runtime tests require node");
        assert!(
            output.status.success(),
            "transformed source failed:\n{}\nstdout:\n{}\nstderr:\n{}",
            source,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
    }

    const BABEL_EXPECT_SHIM: &str = r#"
        function expect(actual) {
            return {
                toBe(expected) {
                    if (!Object.is(actual, expected)) {
                        throw new Error(`Expected ${actual} to be ${expected}`);
                    }
                },
                toThrow(expected) {
                    let thrown;
                    try { actual(); } catch (error) { thrown = error; }
                    if (!thrown || !String(thrown.message).includes(expected)) {
                        throw new Error(`Expected function to throw ${expected}, got ${thrown}`);
                    }
                }
            };
        }
    "#;

    const TEST262_ASSERT_SHIM: &str = r#"
        const assert = {
            sameValue(actual, expected) {
                if (!Object.is(actual, expected)) {
                    throw new Error(`Expected SameValue(${actual}, ${expected})`);
                }
            },
            throws(ExpectedError, callback) {
                let thrown;
                try { callback(); } catch (error) { thrown = error; }
                if (!(thrown instanceof ExpectedError)) {
                    throw new Error(`Expected ${ExpectedError.name}, got ${thrown}`);
                }
            }
        };
    "#;

    #[test]
    fn lowers_empty_class_declaration_with_stable_binding_identity() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class C {}", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let analyze_options = AnalyzeOptions {
            features: options.semantic_features(),
            ..AnalyzeOptions::default()
        };
        let mut semantic = analyze(parse.program, &ast, analyze_options).semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class = fact.class;
        let class_scope = fact.class_scope;
        let binding = class.id(&ast).unwrap();
        let declaration = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(declaration).symbol;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        let statement = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let function = statement
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_function(&ast))
            .unwrap();
        assert_eq!(function.id(&ast), Some(binding));
        assert_eq!(
            semantic.declaration_of(binding.node_id()),
            Some(declaration)
        );
        assert_eq!(semantic.declaration(declaration).symbol, symbol);
        assert!(
            semantic
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::FUNCTION)
        );
        assert!(!semantic.symbol(symbol).flags.contains(SymbolFlags::CLASS));
        assert!(!semantic.is_scope_active(class_scope));
        let function_scope = semantic.scope_of(function.node_id());
        assert_eq!(semantic.scope(function_scope).node, function.node_id());
        assert_eq!(ast.node_kind(function.node_id()), NodeKind::Function);
        assert!(matches!(
            ast.kind_data(function.body(&ast).node_id()),
            NodeKindData::FunctionBody(_)
        ));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        assert_eq!(ast.mutation_epoch(), 1);
        assert_eq!(semantic.mutation_epoch(), 1);
    }

    #[test]
    fn lowers_named_empty_class_expression_with_stable_semantics() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const value = class Inner {};", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let (class, class_scope) = {
            let fact = &semantic.classes_post_order().unwrap()[0];
            (fact.class, fact.class_scope)
        };
        let name_scope = semantic.scope(class_scope).parent.unwrap();
        let binding = class.id(&ast).unwrap();
        let declaration = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(declaration).symbol;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::Function);
        assert_eq!(semantic.scope_of(class.node_id()), class_scope);
        assert_eq!(
            semantic.scope(class_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::Function
        );
        assert_eq!(semantic.scope(class_scope).parent, Some(name_scope));
        assert_eq!(
            semantic.declaration_of(binding.node_id()),
            Some(declaration)
        );
        assert_eq!(semantic.declaration(declaration).symbol, symbol);
        assert!(
            semantic
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::FUNCTION)
        );
        assert!(!semantic.symbol(symbol).flags.contains(SymbolFlags::CLASS));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_named_class_expression_methods_through_iife() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class Inner { method() { return Inner; } static make() { return new Inner(); } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class = fact.class;
        let class_scope = fact.class_scope;
        let name_scope = semantic.scope(class_scope).parent.unwrap();
        let outer_scope = semantic.scope(name_scope).parent.unwrap();
        let method_scopes: Vec<_> = fact
            .methods
            .iter()
            .map(|method| method.function_scope)
            .collect();
        let binding = class.id(&ast).unwrap();
        let declaration = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(declaration).symbol;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::CallExpression);
        let NodeKindData::CallExpression(call) = ast.kind_data(class.node_id()) else {
            unreachable!()
        };
        let ExprData::Function(iife) = ast.expr_data(call.callee(&ast)) else {
            panic!("class expression must lower to a function IIFE")
        };
        assert_eq!(iife.body(&ast).body(&ast).len(), 4);
        assert!(!semantic.is_scope_active(class_scope));
        assert!(!semantic.is_scope_active(name_scope));
        assert_eq!(
            semantic.declaration_of(binding.node_id()),
            Some(declaration)
        );
        assert_eq!(semantic.declaration(declaration).symbol, symbol);
        let iife_scope = semantic.symbol(symbol).scope;
        assert_eq!(semantic.scope(iife_scope).parent, Some(outer_scope));
        assert_eq!(
            semantic.scope(iife_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::Function
        );
        for method_scope in method_scopes {
            assert_eq!(semantic.scope(method_scope).parent, Some(iife_scope));
        }
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_named_derived_expression_with_ordered_heritage_and_key_captures() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class Inner extends makeBase() { [key()]() { return Inner; } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class = fact.class;
        let heritage = fact.heritage.unwrap();
        let key = fact.computed_keys[0];
        let outer_scope = semantic
            .scope(semantic.scope(fact.class_scope).parent.unwrap())
            .parent
            .unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let NodeKindData::CallExpression(call) = ast.kind_data(class.node_id()) else {
            panic!("derived class expression must lower to an IIFE call")
        };
        assert_eq!(call.arguments(&ast).len(), 2);
        let ExprData::Function(iife) = ast.expr_data(call.callee(&ast)) else {
            panic!("derived class expression callee must be a function")
        };
        assert_eq!(iife.params(&ast).items(&ast).len(), 2);
        assert_eq!(iife.body(&ast).body(&ast).len(), 4);
        assert!(semantic.is_node_active(heritage));
        assert!(semantic.is_node_active(key));
        assert_eq!(semantic.scope_of(heritage), outer_scope);
        assert_eq!(semantic.scope_of(key), outer_scope);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_named_class_expression_heritage_tdz() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const value = class Inner extends Inner {};", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let class = semantic.classes_post_order().unwrap()[0].class;
        let program_scope = semantic.scope_of(parse.program.node_id());

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(matches!(
            ast.kind_data(class.node_id()),
            NodeKindData::CallExpression(_)
        ));
        assert!(
            semantic
                .binding_text(&ast, program_scope, "_class_name_tdz_ref")
                .is_some()
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn rewrites_immediate_and_deferred_heritage_closure_references() {
        for source in [
            "const value = class Inner extends (() => Inner)() {};",
            "let read; const value = class Inner extends (read = () => Inner, Base) {};",
        ] {
            let allocator = Allocator::new();
            let mut ast = Ast::new(source, &allocator);
            let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
            assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
            let mut options = TransformerOptions::default();
            options.es2015.classes = true;
            let mut semantic = analyze(
                parse.program,
                &ast,
                AnalyzeOptions {
                    features: options.semantic_features(),
                    ..AnalyzeOptions::default()
                },
            )
            .semantic;
            let program_scope = semantic.scope_of(parse.program.node_id());

            Transformer::transform(parse.program, &mut ast, &mut semantic, options);

            assert!(
                semantic
                    .binding_text(&ast, program_scope, "_class_name_tdz_ref")
                    .is_some(),
                "missing temporal class-name helper for {source}"
            );
            assert!(semantic.classes_post_order().unwrap().is_empty());
            verify_program(parse.program, &ast, &semantic).unwrap();
            assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        }
    }

    #[test]
    fn rewrites_direct_immediate_and_deferred_computed_key_tdz_references() {
        for source in [
            "const value = class Inner { [Inner]() {} };",
            "const value = class Inner { [(() => Inner)()]() {} };",
            "let read; const value = class Inner { [(read = () => Inner, 'key')]() {} };",
        ] {
            let allocator = Allocator::new();
            let mut ast = Ast::new(source, &allocator);
            let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
            assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
            let mut options = TransformerOptions::default();
            options.es2015.classes = true;
            let mut semantic = analyze(
                parse.program,
                &ast,
                AnalyzeOptions {
                    features: options.semantic_features(),
                    ..AnalyzeOptions::default()
                },
            )
            .semantic;
            let program_scope = semantic.scope_of(parse.program.node_id());

            Transformer::transform(parse.program, &mut ast, &mut semantic, options);

            assert!(
                semantic
                    .binding_text(&ast, program_scope, "_class_name_tdz_ref")
                    .is_some(),
                "missing temporal class-name helper for {source}"
            );
            assert!(semantic.classes_post_order().unwrap().is_empty());
            verify_program(parse.program, &ast, &semantic).unwrap();
            assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        }
    }

    #[test]
    fn lowers_nested_class_expression_in_heritage() {
        for source in [
            "const value = class Outer extends (class extends Base {}) {};",
            "class Outer extends (class extends Base {}) {}",
            "export default class extends (class extends Base {}) {}",
        ] {
            let allocator = Allocator::new();
            let mut ast = Ast::new(source, &allocator);
            let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
            assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
            let mut options = TransformerOptions::default();
            options.es2015.classes = true;
            let mut semantic = analyze(
                parse.program,
                &ast,
                AnalyzeOptions {
                    features: options.semantic_features(),
                    ..AnalyzeOptions::default()
                },
            )
            .semantic;
            assert_eq!(semantic.classes_post_order().unwrap().len(), 2);
            Transformer::transform(parse.program, &mut ast, &mut semantic, options);

            assert!(semantic.classes_post_order().unwrap().is_empty());
            verify_program(parse.program, &ast, &semantic).unwrap();
            assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        }
    }

    #[test]
    fn preserves_outer_class_tdz_through_nested_class_heritage() {
        for source in [
            "const value = class Outer extends (class extends Outer {}) {};",
            "const value = class Outer extends (class Inner extends Outer {}) {};",
            "let read; const value = class Outer extends (class extends (read = () => Outer, Base) {}) {};",
            "const value = class Outer { [(class extends Outer {})]() {} };",
        ] {
            let allocator = Allocator::new();
            let mut ast = Ast::new(source, &allocator);
            let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
            assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
            let mut options = TransformerOptions::default();
            options.es2015.classes = true;
            let mut semantic = analyze(
                parse.program,
                &ast,
                AnalyzeOptions {
                    features: options.semantic_features(),
                    ..AnalyzeOptions::default()
                },
            )
            .semantic;
            let program_scope = semantic.scope_of(parse.program.node_id());

            Transformer::transform(parse.program, &mut ast, &mut semantic, options);

            assert!(
                semantic
                    .binding_text(&ast, program_scope, "_class_name_tdz_ref")
                    .is_some(),
                "missing nested temporal helper for {source}"
            );
            assert!(semantic.classes_post_order().unwrap().is_empty());
            verify_program(parse.program, &ast, &semantic).unwrap();
            assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        }
    }

    #[test]
    fn preserves_heritage_before_computed_key_order_with_class_tdz() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class Inner extends (events.push('heritage'), Inner) { [events.push('key')]() {} };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let heritage = fact.heritage.unwrap();
        let key = fact.computed_keys[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let argument_parent = semantic.parent_of(heritage).unwrap();
        assert_eq!(semantic.parent_of(key), Some(argument_parent));
        let NodeKindData::CallExpression(call) = ast.kind_data(argument_parent) else {
            panic!("heritage and computed key must be arguments of the inner class IIFE")
        };
        assert_eq!(call.arguments(&ast).len(), 2);
        assert_eq!(
            call.arguments(&ast).get_node(&ast, 0).unwrap().node_id(),
            heritage
        );
        assert_eq!(
            call.arguments(&ast).get_node(&ast, 1).unwrap().node_id(),
            key
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn avoids_user_names_for_class_tdz_helper_and_sentinel() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function _class_name_tdz_ref() {} const _class_ref = 0; const value = class Inner extends Inner {};",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let program_scope = semantic.scope_of(parse.program.node_id());

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            semantic
                .binding_text(&ast, program_scope, "_class_name_tdz_ref1")
                .is_some()
        );
        assert!(
            semantic
                .iter_symbols()
                .any(|entry| { ast.get_ast_str_bytes(entry.symbol.name) == b"_class_ref1" })
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_derived_expression_with_explicit_constructor() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class extends Base { constructor(flag) { if (flag) super(flag); else return {}; this.flag = flag; } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class = fact.class;
        let super_call = fact.super_calls[0];
        let this_node = fact.this_nodes[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let NodeKindData::CallExpression(call) = ast.kind_data(class.node_id()) else {
            panic!("anonymous derived expression must lower to an IIFE call")
        };
        assert_eq!(call.arguments(&ast).len(), 1);
        let ExprData::Function(iife) = ast.expr_data(call.callee(&ast)) else {
            panic!("anonymous derived expression callee must be a function")
        };
        assert_eq!(iife.body(&ast).body(&ast).len(), 3);
        assert!(!semantic.is_node_active(super_call));
        assert!(!semantic.is_node_active(this_node));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_empty_class_expression_with_generated_name_scope() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const value = class {};", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let (class, class_scope, outer_scope) = {
            let fact = &semantic.classes_post_order().unwrap()[0];
            (
                fact.class,
                fact.class_scope,
                semantic.scope(fact.class_scope).parent.unwrap(),
            )
        };

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::Function);
        assert_eq!(
            semantic.scope(class_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::Function
        );
        let name_scope = semantic.scope(class_scope).parent.unwrap();
        assert_eq!(
            semantic.scope(name_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::ExpressionName
        );
        assert_eq!(semantic.scope(name_scope).parent, Some(outer_scope));
        assert_eq!(semantic.scope(name_scope).node, class.node_id());
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_class_expression_methods_through_iife() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class { constructor(input) { this.input = input; } method() { return this.input; } static make() { return 1; } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class = fact.class;
        let class_scope = fact.class_scope;
        let outer_scope = semantic.scope(class_scope).parent.unwrap();
        let constructor_scope = semantic.scope_of(fact.constructor.unwrap().node_id());
        let method_scopes: Vec<_> = fact
            .methods
            .iter()
            .filter(|method| method.function != fact.constructor.unwrap())
            .map(|method| method.function_scope)
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::CallExpression);
        let NodeKindData::CallExpression(call) = ast.kind_data(class.node_id()) else {
            unreachable!()
        };
        let ExprData::Function(iife) = ast.expr_data(call.callee(&ast)) else {
            panic!("class expression must lower to a function IIFE")
        };
        assert_eq!(iife.body(&ast).body(&ast).len(), 4);
        assert!(!semantic.is_scope_active(class_scope));
        let iife_scope = semantic.scope(constructor_scope).parent.unwrap();
        assert_eq!(semantic.scope(iife_scope).parent, Some(outer_scope));
        assert_eq!(
            semantic.scope(iife_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::Function
        );
        for method_scope in method_scopes {
            assert_eq!(semantic.scope(method_scope).parent, Some(iife_scope));
        }
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn moves_named_class_expression_constructor_scope_to_output_function() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class Inner { constructor(input) { this.input = input; } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let (class, class_scope, constructor_scope, name_scope) = {
            let fact = &semantic.classes_post_order().unwrap()[0];
            (
                fact.class,
                fact.class_scope,
                semantic.scope_of(fact.constructor.unwrap().node_id()),
                semantic.scope(fact.class_scope).parent.unwrap(),
            )
        };

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::Function);
        assert!(!semantic.is_scope_active(class_scope));
        assert_eq!(semantic.scope_of(class.node_id()), constructor_scope);
        assert_eq!(semantic.scope(constructor_scope).node, class.node_id());
        assert_eq!(semantic.scope(constructor_scope).parent, Some(name_scope));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn moves_anonymous_class_expression_constructor_into_generated_name_scope() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const value = class { constructor(input) { this.input = input; } };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let (class, class_scope, constructor_scope, outer_scope) = {
            let fact = &semantic.classes_post_order().unwrap()[0];
            (
                fact.class,
                fact.class_scope,
                semantic.scope_of(fact.constructor.unwrap().node_id()),
                semantic.scope(fact.class_scope).parent.unwrap(),
            )
        };

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(class.node_id()), NodeKind::Function);
        assert!(!semantic.is_scope_active(class_scope));
        assert_eq!(semantic.scope_of(class.node_id()), constructor_scope);
        assert_eq!(semantic.scope(constructor_scope).node, class.node_id());
        let name_scope = semantic.scope(constructor_scope).parent.unwrap();
        assert_eq!(semantic.scope(name_scope).parent, Some(outer_scope));
        assert_eq!(
            semantic.scope(name_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::ExpressionName
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn composes_arrow_generator_and_class_transactions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { method() { const get = () => this; return get(); } *items() { yield 1; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.arrow_functions =
            Some(crate::es2015::arrow::ArrowFunctionsOptions::default());
        options.es2015.generator = true;
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(semantic.classes_post_order().unwrap().is_empty());
        assert!(
            semantic
                .generator_functions_post_order()
                .unwrap()
                .is_empty()
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_exported_class_without_staling_module_record() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("export class C { method() { return C; } }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_export_named_declaration(&ast)
            .unwrap();
        let module_entry = semantic.module_record().exports()[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let declaration = export.declaration(&ast).unwrap();
        assert!(declaration.as_function(&ast).is_some());
        assert_eq!(semantic.module_record().exports()[0], module_entry);
        assert_eq!(
            semantic.parent_of(declaration.node_id()),
            Some(export.node_id())
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_named_default_export_class_without_staling_module_record() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "export default class C { method() { return C; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_export_default_declaration(&ast)
            .unwrap();
        let module_entry = semantic.module_record().exports()[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(export.declaration(&ast).as_function(&ast).is_some());
        assert_eq!(semantic.module_record().exports()[0], module_entry);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn retargets_anonymous_default_class_to_generated_function_binding() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("export default class {}", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::AnonymousDeclaration { .. }
        ));
        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_export_default_declaration(&ast)
            .unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let function = export.declaration(&ast).as_function(&ast).unwrap();
        let binding = function.id(&ast).unwrap();
        let declaration = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(declaration).symbol;
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::Local {
                symbol: target,
                declaration: Some(target_declaration),
                reference: None,
            } if target == symbol && target_declaration == declaration
        ));
        assert!(!semantic.is_scope_active(class_scope));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_default_class_constructor_and_methods() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "export default class { constructor(input) { this.input = input; } method() { return this.input; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let constructor_scope = semantic.scope_of(fact.constructor.unwrap().node_id());
        let outer_scope = semantic.scope(class_scope).parent.unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 2);
        assert!(!semantic.is_scope_active(class_scope));
        assert_eq!(semantic.scope(constructor_scope).parent, Some(outer_scope));
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::Local { .. }
        ));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_default_derived_class_with_synthetic_constructor() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function makeBase() { return function Base() {}; } export default class extends makeBase() { method() { return this; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let outer_scope = semantic.scope(class_scope).parent.unwrap();
        let heritage = fact.heritage.unwrap();
        let heritage_reference = semantic
            .node_children(heritage)
            .find_map(|node| semantic.reference_of(node))
            .unwrap();
        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 1)
            .unwrap()
            .as_export_default_declaration(&ast)
            .unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 12);
        assert!(export.declaration(&ast).as_function(&ast).is_some());
        assert!(semantic.is_node_active(heritage));
        assert_eq!(semantic.scope_of(heritage), outer_scope);
        assert_eq!(semantic.reference(heritage_reference).scope, outer_scope);
        assert!(!semantic.is_scope_active(class_scope));
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::Local { .. }
        ));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_anonymous_default_derived_class_with_explicit_constructor() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function Base() {} export default class extends Base { constructor(flag) { if (flag) super(flag); else return {}; this.flag = flag; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let outer_scope = semantic.scope(class_scope).parent.unwrap();
        let constructor = fact.constructor.unwrap();
        let constructor_scope = semantic.scope_of(constructor.node_id());
        let super_call = fact.super_calls[0];
        let this_node = fact.this_nodes[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 11);
        assert!(semantic.is_node_active(constructor.node_id()));
        assert_eq!(semantic.scope(constructor_scope).parent, Some(outer_scope));
        assert!(!semantic.is_node_active(super_call));
        assert!(!semantic.is_node_active(this_node));
        assert!(!semantic.is_scope_active(class_scope));
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::Local { .. }
        ));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_new_target_and_direct_eval_in_derived_class_functions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base {} class Derived extends Base { constructor(value) { super(value); this.target = new.target; } method() { return eval('Derived'); } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .find(|fact| ast.get_utf8(fact.class.id(&ast).unwrap().name(&ast)) == "Derived")
            .unwrap();
        let constructor = fact.constructor.unwrap();
        let constructor_scope = semantic.scope_of(constructor.node_id());
        let method_scope = fact
            .methods
            .iter()
            .find(|method| method.function != constructor)
            .unwrap()
            .function_scope;
        assert_eq!(fact.new_targets.len(), 1);
        assert!(semantic.scope(method_scope).flags.direct_eval);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(semantic.scope_of(constructor.node_id()), constructor_scope);
        assert!(semantic.scope(method_scope).flags.direct_eval);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_class_declaration_inside_with_scope_without_weakening_dynamic_lookup() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "with (environment) { class C { method() { return C; } } consume(C); }",
            &allocator,
        );
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Script,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let class_scope = semantic.classes_post_order().unwrap()[0].class_scope;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(!semantic.is_scope_active(class_scope));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_string_and_numeric_method_keys() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { 'quoted'() { return 1; } 2() { return 2; } get 'value'() { return 3; } set 'value'(input) {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 4);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn moves_computed_method_keys_with_stable_reference_ids() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const key = value => value; class C { [key('instance')]() {} static [key('static')]() {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let outer_scope = semantic.scope(fact.class_scope).parent.unwrap();
        let computed_keys: Vec<_> = fact.computed_keys.iter().copied().collect();
        let references: Vec<_> = computed_keys
            .iter()
            .map(|key| {
                semantic
                    .node_children(*key)
                    .find_map(|node| semantic.reference_of(node))
                    .unwrap()
            })
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 4);
        for (key, reference) in computed_keys.into_iter().zip(references) {
            assert!(semantic.is_node_active(key));
            assert_eq!(semantic.scope_of(key), outer_scope);
            assert_eq!(semantic.reference(reference).scope, outer_scope);
        }
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn captures_class_expression_computed_key_outside_iife() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const key = value => value; const C = class { [key('method')]() {} };",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let outer_scope = semantic.scope(fact.class_scope).parent.unwrap();
        let class = fact.class;
        let key = fact.computed_keys[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(semantic.scope_of(key), outer_scope);
        let NodeKindData::CallExpression(call) = ast.kind_data(class.node_id()) else {
            unreachable!()
        };
        let ExprData::Function(iife) = ast.expr_data(call.callee(&ast)) else {
            unreachable!()
        };
        let iife_scope = semantic.scope_of(iife.node_id());
        assert_eq!(semantic.scope(iife_scope).parent, Some(outer_scope));
        assert_eq!(
            semantic.scope(iife_scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::Function
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_outer_this_in_named_expression_computed_key() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function make() { return class Inner { [this.key]() {} }; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let name_scope = semantic.scope(fact.class_scope).parent.unwrap();
        let outer_scope = semantic.scope(name_scope).parent.unwrap();
        let key = fact.computed_keys[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(semantic.scope_of(key), outer_scope);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_separate_computed_accessor_evaluation() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const key = value => value; class C { get [key('get')]() { return 1; } set [key('set')](value) {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        assert_eq!(
            semantic.classes_post_order().unwrap()[0]
                .computed_keys
                .len(),
            2
        );

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 4);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_synthetic_derived_constructor_and_moves_heritage_once() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function makeBase() { return function Base() {}; } class Derived extends makeBase() { method() { return this; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let heritage = fact.heritage.unwrap();
        let output_scope = semantic.scope(fact.class_scope).parent.unwrap();
        let heritage_reference = semantic
            .node_children(heritage)
            .find_map(|node| semantic.reference_of(node))
            .unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 12);
        assert!(semantic.is_node_active(heritage));
        assert_eq!(semantic.scope_of(heritage), output_scope);
        assert_eq!(semantic.reference(heritage_reference).scope, output_scope);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn injects_class_helpers_once_and_avoids_nested_user_bindings() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function _inherits() {} function outer() { let _call_super; class A extends Base {} } class B extends Base {}",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let program_scope = semantic.scope_of(parse.program.node_id());

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let inherits = semantic
            .binding_text(&ast, program_scope, "_inherits1")
            .unwrap();
        let call_super = semantic
            .binding_text(&ast, program_scope, "_call_super1")
            .unwrap();
        assert_eq!(semantic.declarations(inherits).len(), 1);
        assert_eq!(semantic.declarations(call_super).len(), 1);
        assert_eq!(semantic.uses(inherits).len(), 2);
        assert_eq!(semantic.uses(call_super).len(), 2);
        assert_eq!(
            parse
                .program
                .body(&ast)
                .iter()
                .filter_map(|statement| {
                    ast.get_node_in_sub_range(statement)
                        .as_declaration(&ast)
                        .and_then(|declaration| declaration.as_function(&ast))
                        .and_then(|function| function.id(&ast))
                })
                .filter(|binding| {
                    matches!(
                        ast.get_utf8(binding.name(&ast)),
                        "_set_prototype_of"
                            | "_inherits1"
                            | "_get_prototype_of"
                            | "_type_of"
                            | "_assert_this_initialized"
                            | "_possible_constructor_return"
                            | "_is_native_reflect_construct"
                            | "_call_super1"
                    )
                })
                .count(),
            8
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_conditional_super_calls_this_and_explicit_returns() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Derived extends Base { constructor(value) { if (value) super(value); else super(); this.value = value; if (value < 0) return { value }; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let constructor = fact.constructor.unwrap();
        let constructor_scope = semantic.scope_of(constructor.node_id());
        let body = constructor.body(&ast);
        let super_calls: Vec<_> = fact.super_calls.iter().copied().collect();
        let this_nodes: Vec<_> = fact.this_nodes.iter().copied().collect();
        assert_eq!(super_calls.len(), 2);
        assert_eq!(this_nodes.len(), 1);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(constructor.node_id()), NodeKind::Function);
        assert_eq!(semantic.scope_of(constructor.node_id()), constructor_scope);
        assert!(semantic.is_node_active(body.node_id()));
        assert!(
            super_calls
                .iter()
                .all(|node| !semantic.is_node_active(*node))
        );
        assert!(
            this_nodes
                .iter()
                .all(|node| !semantic.is_node_active(*node))
        );
        let this_symbol = semantic
            .binding_text(&ast, constructor_scope, "_this")
            .unwrap();
        assert!(semantic.uses(this_symbol).len() >= 5);
        let program_scope = semantic.scope_of(parse.program.node_id());
        let call_super = semantic
            .binding_text(&ast, program_scope, "_call_super")
            .unwrap();
        let possible_return = semantic
            .binding_text(&ast, program_scope, "_possible_constructor_return")
            .unwrap();
        assert_eq!(semantic.uses(call_super).len(), 2);
        assert_eq!(semantic.uses(possible_return).len(), 2);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_derived_constructor_return_completion_through_finally() {
        assert_node_output(
            r#"
                const events = [];
                class Base {
                    constructor() {
                        events.push("super");
                    }
                }
                class FinallyReturn extends Base {
                    constructor() {
                        try {
                            return 1;
                        } finally {
                            events.push("finally-return");
                            return { kind: "object" };
                        }
                    }
                }
                class FinallyNormal extends Base {
                    constructor() {
                        try {
                            return 1;
                        } finally {
                            super();
                            events.push("finally-normal");
                        }
                    }
                }
                class FinallyPrimitive extends Base {
                    constructor() {
                        try {
                            super();
                            return { kind: "ignored" };
                        } finally {
                            return 1;
                        }
                    }
                }
                const first = new FinallyReturn();
                let normalError;
                try { new FinallyNormal(); } catch (error) { normalError = error.name; }
                let primitiveError;
                try { new FinallyPrimitive(); } catch (error) { primitiveError = error.name; }
                console.log(JSON.stringify([
                    first.kind,
                    normalError,
                    primitiveError,
                    events
                ]));
            "#,
            r#"["object","TypeError","TypeError",["finally-return","super","finally-normal","super"]]"#,
        );
    }

    #[test]
    fn preserves_break_and_continue_overrides_of_pending_constructor_returns() {
        assert_node_output(
            r#"
                class Base {}
                class ContinueOverride extends Base {
                    constructor() {
                        let iteration = 0;
                        outer: while (iteration++ < 2) {
                            try {
                                return 1;
                            } finally {
                                if (iteration === 1) continue outer;
                                super();
                                return { iteration };
                            }
                        }
                    }
                }
                class BreakOverride extends Base {
                    constructor() {
                        outer: while (true) {
                            try {
                                return 1;
                            } finally {
                                break outer;
                            }
                        }
                        super();
                        this.kind = "break";
                    }
                }
                class InternalBreak extends Base {
                    constructor() {
                        const result = { kind: "internal" };
                        try {
                            return result;
                        } finally {
                            while (true) break;
                        }
                    }
                }
                console.log(JSON.stringify([
                    new ContinueOverride().iteration,
                    new BreakOverride().kind,
                    new InternalBreak().kind
                ]));
            "#,
            r#"[2,"break","internal"]"#,
        );
    }

    #[test]
    fn passes_ported_babel_class_runtime_fixtures() {
        for fixture in [
            include_str!(
                "../../../tests/fixtures/classes/babel/derived-constructor-no-primitive-return.js"
            ),
            include_str!(
                "../../../tests/fixtures/classes/babel/derived-constructor-must-call-super.js"
            ),
            include_str!(
                "../../../tests/fixtures/classes/babel/super-reference-before-in-lambda.js"
            ),
            include_str!("../../../tests/fixtures/classes/babel/computed-super-method.js"),
        ] {
            assert_node_output(&format!("{BABEL_EXPECT_SHIM}\n{fixture}"), "");
        }
    }

    #[test]
    fn passes_ported_test262_derived_constructor_runtime_fixtures() {
        for (name, fixture) in [
            (
                "catch-finally",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-catch-finally.js"
                ),
            ),
            (
                "catch-finally-arrow",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-catch-finally-arrow.js"
                ),
            ),
            (
                "finally-super",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-finally-super.js"
                ),
            ),
            (
                "finally-super-arrow",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-finally-super-arrow.js"
                ),
            ),
            (
                "for-of-arrow",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-for-of-arrow.js"
                ),
            ),
            (
                "with-number",
                include_str!(
                    "../../../tests/fixtures/classes/test262/derived-class-return-override-with-number.js"
                ),
            ),
        ] {
            eprintln!("running Test262 fixture {name}");
            assert_node_output(&format!("{TEST262_ASSERT_SHIM}\n{fixture}"), "");
        }
    }

    #[test]
    fn lowers_super_reads_and_calls_in_instance_static_and_arrow_methods() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base { method(value) { return value; } static make(value) { return value; } get value() { return 1; } } class Derived extends Base { method(value) { return super.method(value) + super.value; } static make(key, value) { return super[key](value); } arrow(key) { return () => super[key]; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let derived = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .find(|fact| ast.get_utf8(fact.class.id(&ast).unwrap().name(&ast)) == "Derived")
            .unwrap();
        let super_members: Vec<_> = derived.super_members.iter().copied().collect();
        assert_eq!(super_members.len(), 4);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            super_members
                .iter()
                .all(|member| !semantic.is_node_active(*member))
        );
        let program_scope = semantic.scope_of(parse.program.node_id());
        assert!(semantic.binding_text(&ast, program_scope, "_get").is_some());
        assert!(semantic.binding_text(&ast, program_scope, "_set").is_none());
        assert!(
            semantic
                .binding_text(&ast, program_scope, "_super_prop_base")
                .is_some()
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_direct_super_assignments_in_constructor_instance_and_static_methods() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base {} class Derived extends Base { constructor(key, value) { super(); super[key] = value; } method(key) { return super[key] = super.value; } static setValue(value) { return super.value = value; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let derived = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .find(|fact| ast.get_utf8(fact.class.id(&ast).unwrap().name(&ast)) == "Derived")
            .unwrap();
        let super_members: Vec<_> = derived.super_members.iter().copied().collect();
        assert_eq!(super_members.len(), 4);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            super_members
                .iter()
                .all(|member| !semantic.is_node_active(*member))
        );
        let program_scope = semantic.scope_of(parse.program.node_id());
        assert!(semantic.binding_text(&ast, program_scope, "_get").is_some());
        assert!(semantic.binding_text(&ast, program_scope, "_set").is_some());
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_compound_logical_and_update_super_writes_with_isolated_temps() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base {} class Derived extends Base { constructor(key) { super(); super[key] ??= 1; } method(first, second, value) { super[first()] += super[second()]++; super.value &&= value; return ++super[first()]; } static update(key) { return super[key]--; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let derived = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .find(|fact| ast.get_utf8(fact.class.id(&ast).unwrap().name(&ast)) == "Derived")
            .unwrap();
        let super_members: Vec<_> = derived.super_members.iter().copied().collect();
        assert_eq!(super_members.len(), 6);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            super_members
                .iter()
                .all(|member| !semantic.is_node_active(*member))
        );
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_super_read_and_call_inside_derived_constructor() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base { static prepare(value) { return value; } } class Derived extends Base { constructor(value) { super(value); this.prepared = super.prepare(value); } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let derived = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .find(|fact| ast.get_utf8(fact.class.id(&ast).unwrap().name(&ast)) == "Derived")
            .unwrap();
        let super_call = derived.super_calls[0];
        let super_member = derived.super_members[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(!semantic.is_node_active(super_call));
        assert!(!semantic.is_node_active(super_member));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_super_reads_across_named_anonymous_and_default_class_expressions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Base { static make() {} } const Anonymous = class extends Base { method() { return super.value; } }; const Named = class Inner extends Base { static make() { return super.make(); } }; export default class extends Base { method(key) { return super[key]; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let super_members: Vec<_> = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .flat_map(|fact| fact.super_members.iter().copied())
            .collect();
        assert_eq!(super_members.len(), 3);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            super_members
                .iter()
                .all(|member| !semantic.is_node_active(*member))
        );
        assert!(matches!(
            semantic.module_record().exports()[0].target,
            ExportTarget::Local { .. }
        ));
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_exported_synthetic_derived_class_without_staling_module_record() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function Base() {} export class Derived extends Base {}",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let module_entry = semantic.module_record().exports()[0];

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(semantic.module_record().exports()[0], module_entry);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_extends_null_through_the_inheritance_helper() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class Nil extends null {}", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 10);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn moves_explicit_constructor_and_its_scope_without_renumbering() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class C { constructor(x) { this.x = x; } }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let analyze_options = AnalyzeOptions {
            features: options.semantic_features(),
            ..AnalyzeOptions::default()
        };
        let mut semantic = analyze(parse.program, &ast, analyze_options).semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let constructor = fact.constructor.unwrap();
        let constructor_scope = semantic.scope_of(constructor.node_id());
        let outer_scope = semantic.scope(class_scope).parent.unwrap();
        let original_body = constructor.body(&ast);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
        let function = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_function(&ast))
            .unwrap();
        assert_eq!(function, constructor);
        assert_eq!(function.body(&ast), original_body);
        assert_eq!(
            semantic.parent_of(function.node_id()),
            Some(parse.program.node_id())
        );
        assert_eq!(semantic.scope_of(function.node_id()), constructor_scope);
        assert_eq!(semantic.scope(constructor_scope).parent, Some(outer_scope));
        assert_eq!(semantic.scope(constructor_scope).node, function.node_id());
        assert!(!semantic.is_scope_active(class_scope));
        assert_eq!(function.body(&ast).body(&ast).len(), 2);
        assert!(semantic.classes_post_order().unwrap().is_empty());
    }

    #[test]
    fn batches_sibling_class_declarations_into_one_transaction() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class A {} class B {}", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 2);
        for index in 0..2 {
            assert!(
                parse
                    .program
                    .body(&ast)
                    .get_node(&ast, index)
                    .unwrap()
                    .as_declaration(&ast)
                    .and_then(|declaration| declaration.as_function(&ast))
                    .is_some()
            );
        }
        assert_eq!(ast.mutation_epoch(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_class_declaration_in_function_body_statement_list() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function make(input) { class Inner { constructor(value) { this.value = value; } method() { return this.value; } } return new Inner(input); }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let class = fact.class;
        let owner = semantic.parent_of(class.node_id()).unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(!semantic.is_scope_active(class_scope));
        let NodeKindData::FunctionBody(body) = ast.kind_data(owner) else {
            unreachable!()
        };
        assert_eq!(body.body(&ast).len(), 3);
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_class_declarations_in_block_and_switch_case_lists() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function make(flag) { 'use strict'; if (flag) { class Blocked { method() { return 1; } } return Blocked; } switch (flag) { case 1: class Cased {} return Cased; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        assert_eq!(semantic.classes_post_order().unwrap().len(), 2);
        let class_scopes: Vec<_> = semantic
            .classes_post_order()
            .unwrap()
            .iter()
            .map(|fact| fact.class_scope)
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        for scope in class_scopes {
            assert!(!semantic.is_scope_active(scope));
        }
        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn composes_nested_class_statement_and_method_body_edits() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class Outer { make() { class Inner { method() { return 1; } } return Inner; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        assert_eq!(semantic.classes_post_order().unwrap().len(), 2);

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(semantic.classes_post_order().unwrap().is_empty());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_instance_and_static_methods_with_preserved_function_scopes() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { method(x) { return x; } static make() { return new C(); } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let fact = &semantic.classes_post_order().unwrap()[0];
        let class_scope = fact.class_scope;
        let methods: Vec<_> = fact
            .methods
            .iter()
            .map(|method| {
                (
                    method.definition,
                    method.definition.key(&ast).node_id(),
                    method.function,
                    method.function_scope,
                )
            })
            .collect();
        let outer_scope = semantic.scope(class_scope).parent.unwrap();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 3);
        assert!(
            parse
                .program
                .body(&ast)
                .get_node(&ast, 0)
                .unwrap()
                .as_declaration(&ast)
                .and_then(|declaration| declaration.as_function(&ast))
                .is_some()
        );
        for index in 1..3 {
            assert!(
                parse
                    .program
                    .body(&ast)
                    .get_node(&ast, index)
                    .unwrap()
                    .as_expression_statement(&ast)
                    .is_some()
            );
        }
        for (definition, key, function, scope) in methods {
            assert!(!semantic.is_node_active(definition.node_id()));
            assert!(!semantic.is_node_active(key));
            assert!(semantic.is_node_active(function.node_id()));
            assert_eq!(
                function.function_type(&ast),
                FunctionType::FunctionExpression
            );
            assert_eq!(semantic.scope_of(function.node_id()), scope);
            assert_eq!(semantic.scope(scope).parent, Some(outer_scope));
        }
        assert!(!semantic.is_scope_active(class_scope));
        assert_eq!(ast.mutation_epoch(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_unpaired_getters_and_setters_as_accessor_descriptors() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { get value() { return 1; } static set current(value) {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let method_scopes: Vec<_> = semantic.classes_post_order().unwrap()[0]
            .methods
            .iter()
            .map(|method| method.function_scope)
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 3);
        for scope in method_scopes {
            assert!(semantic.is_scope_active(scope));
        }
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn groups_getter_and_setter_into_one_accessor_descriptor() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { get value() { return 1; } set value(next) {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let method_scopes: Vec<_> = semantic.classes_post_order().unwrap()[0]
            .methods
            .iter()
            .map(|method| method.function_scope)
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(parse.program.body(&ast).len(), 2);
        assert!(
            parse
                .program
                .body(&ast)
                .get_node(&ast, 1)
                .unwrap()
                .as_expression_statement(&ast)
                .is_some()
        );
        for scope in method_scopes {
            assert!(semantic.is_scope_active(scope));
        }
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn preserves_implicit_class_strictness_in_script_methods() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class C { method() { return this; } }", &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Script,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let method = semantic.classes_post_order().unwrap()[0].methods[0].function;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(method.body(&ast).directives(&ast).len(), 1);
        assert_eq!(
            ast.get_utf8(
                method
                    .body(&ast)
                    .directives(&ast)
                    .get_node(&ast, 0)
                    .unwrap()
                    .value(&ast)
            ),
            "use strict"
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn leaves_non_simple_method_parameters_for_the_parameters_prerequisite() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class C { method(value = 1) {} }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let mut options = TransformerOptions::default();
        options.es2015.classes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            parse
                .program
                .body(&ast)
                .get_node(&ast, 0)
                .unwrap()
                .as_declaration(&ast)
                .and_then(|declaration| declaration.as_class(&ast))
                .is_some()
        );
        assert_eq!(ast.mutation_epoch(), 0);
        verify_program(parse.program, &ast, &semantic).unwrap();
    }
}
