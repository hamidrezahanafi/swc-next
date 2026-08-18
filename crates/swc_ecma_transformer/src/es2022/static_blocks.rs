//! ES2022: static-blocks — Transform static blocks to private-field IIFEs.
//!
//! ```js
//! class C { static { foo(); } }
//! // becomes
//! class C { static #_ = (() => { foo(); })(); }
//! ```

use std::collections::HashSet;

use swc_next_ecma_ast::{Class, ClassElement, GetSpan, Stmt, TypedSubRange};
use swc_next_ecma_semantic::scope;

use crate::{
    mutation::{DetachedScopeRegion, IdentityPolicy},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct StaticBlocks;

impl StaticBlocks {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn exit_class(&self, class: Class, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let body = class.body(ctx.ast);
        let elements: Vec<_> = body
            .body(ctx.ast)
            .iter()
            .map(|element| ctx.ast.get_node_in_sub_range(element))
            .collect();
        if !elements
            .iter()
            .any(|element| element.is_static_block(ctx.ast))
        {
            return;
        }

        let mut used_private_names: HashSet<String> = elements
            .iter()
            .filter_map(|element| element.as_property_definition(ctx.ast))
            .filter_map(|property| property.key(ctx.ast).as_private_identifier(ctx.ast))
            .map(|private| ctx.ast.get_utf8(private.name(ctx.ast)).to_owned())
            .collect();
        let class_scope = ctx.semantic().scope_of(body.node_id());
        let body_span = body.span(ctx.ast);

        ctx.transaction(body.node_id(), |tx| {
            let mut output = Vec::with_capacity(elements.len());
            for element in elements {
                let Some(block) = element.as_static_block(tx.ast()) else {
                    output.push(element);
                    continue;
                };
                let statements: Vec<Stmt> = block
                    .body(tx.ast())
                    .iter()
                    .map(|statement| tx.ast().get_node_in_sub_range(statement))
                    .collect();
                if statements.is_empty() {
                    tx.retire_subtree(block.node_id());
                    continue;
                }

                let block_scope = tx.semantic().scope_of(block.node_id());
                let block_flags = tx.semantic().scope(block_scope).flags;
                let private_name = next_private_name(&mut used_private_names);
                let (property, arrow) = tx.build_detached(|ast| {
                    let body_statements = ast.add_typed_sub_range(statements);
                    let arrow_body = ast.arrow_function_body_function_body(
                        swc_next_ecma_ast::DUMMY_SP,
                        TypedSubRange::empty(),
                        body_statements,
                    );
                    let params = ast.formal_parameters(
                        swc_next_ecma_ast::DUMMY_SP,
                        TypedSubRange::empty(),
                        None,
                        swc_next_ecma_ast::FormalParameterKind::FormalParameters,
                    );
                    let arrow = ast.arrow_function_expression(
                        swc_next_ecma_ast::DUMMY_SP,
                        false,
                        false,
                        params,
                        arrow_body,
                        None,
                        None,
                    );
                    let call = ast.expr_call_expression(
                        swc_next_ecma_ast::DUMMY_SP,
                        swc_next_ecma_ast::Expr::ArrowFunctionExpression(arrow),
                        TypedSubRange::empty(),
                        false,
                        None,
                    );
                    let private_name = ast.add_utf8(&private_name);
                    let key = ast
                        .property_key_private_identifier(swc_next_ecma_ast::DUMMY_SP, private_name);
                    let property = ast.property_definition(
                        swc_next_ecma_ast::DUMMY_SP,
                        TypedSubRange::empty(),
                        key,
                        Some(call),
                        false,
                        true,
                        false,
                        None,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        swc_next_ecma_ast::Accessibility::None,
                    );
                    (property, arrow)
                });

                let region = DetachedScopeRegion {
                    root: arrow.node_id(),
                    scope: block_scope.into(),
                };
                let layout =
                    tx.create_generated_scope_regions(property, class_scope, &[region], &[]);
                tx.adopt_generated_subtree_with_layout(property, Some(body.node_id()), &layout);
                tx.retag_scope(block_scope, scope::ScopeKind::Function, block_flags);
                tx.reowner_scope(block_scope, arrow.node_id());
                tx.retire_node(block.node_id());
                output.push(ClassElement::PropertyDefinition(property));
            }

            let replacement = tx.build_detached(|ast| {
                let elements = ast.add_typed_sub_range(output);
                ast.class_body(body_span, elements)
            });
            tx.replace(
                body.node_id(),
                replacement.node_id(),
                IdentityPolicy::Preserve,
            );
        })
        .expect("static-blocks mutation must pass preflight");
    }
}

fn next_private_name(used: &mut HashSet<String>) -> String {
    let mut suffix = 1;
    loop {
        let candidate = if suffix == 1 {
            "_".to_owned()
        } else {
            format!("_{suffix}")
        };
        if used.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, NodeKind};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze, scope::ScopeKind};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn static_block_scopes_are_reused_as_arrow_function_scopes() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C { static #_ = 0; static { let x = 1; use(x); } static { this.y = 2; } static {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let class = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_class(&ast))
            .unwrap();
        let blocks: Vec<_> = class
            .body(&ast)
            .body(&ast)
            .iter()
            .map(|element| ast.get_node_in_sub_range(element))
            .filter_map(|element| element.as_static_block(&ast))
            .collect();
        let moved_statement = blocks[0].body(&ast).get_node(&ast, 0).unwrap();

        let mut options = TransformerOptions::default();
        options.es2022.static_blocks = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let block_scopes: Vec<_> = blocks
            .iter()
            .map(|block| semantic.scope_of(block.node_id()))
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let elements: Vec<_> = class
            .body(&ast)
            .body(&ast)
            .iter()
            .map(|element| ast.get_node_in_sub_range(element))
            .collect();
        assert!(
            elements
                .iter()
                .all(|element| !element.is_static_block(&ast))
        );
        let private_names: Vec<_> = elements
            .iter()
            .filter_map(|element| element.as_property_definition(&ast))
            .filter_map(|property| property.key(&ast).as_private_identifier(&ast))
            .map(|private| ast.get_utf8(private.name(&ast)).to_owned())
            .collect();
        assert_eq!(private_names, ["_", "_2", "_3"]);
        assert!(
            blocks
                .iter()
                .all(|block| !semantic.is_node_active(block.node_id()))
        );
        for scope in &block_scopes[..2] {
            assert!(semantic.is_scope_active(*scope));
            let scope = semantic.scope(*scope);
            assert_eq!(scope.kind, ScopeKind::Function);
            assert_eq!(ast.node_kind(scope.node), NodeKind::ArrowFunctionExpression);
        }
        assert!(!semantic.is_scope_active(block_scopes[2]));
        assert_eq!(
            ast.node_kind(semantic.parent_of(moved_statement.node_id()).unwrap()),
            NodeKind::FunctionBody
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
