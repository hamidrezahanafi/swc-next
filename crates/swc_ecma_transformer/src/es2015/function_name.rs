use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Class, Function, GetSpan, ObjectProperty,
    PropertyKey, ReferenceId, ScopeId, VariableDeclarator,
};
use swc_next_ecma_semantic::{SymbolFlags, scope};

use crate::state::{TransformerCtx, TransformerCtxExt};

/// ES2015: function-name — Infer function/class names from binding patterns
/// and assignment targets.
///
/// ```js
/// var f = function() {}   → var f = function f() {}
/// obj.f = function() {}   → obj.f = function f() {}
/// { f: function() {} }    → { f: function f() {} }
/// ```
pub(super) struct FunctionName;

impl FunctionName {
    pub fn exit_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let init = declarator.init(ctx.ast);
        let Some(init) = init else {
            return;
        };

        let name = {
            let ast = &*ctx.ast;
            let id = declarator.id(ast);
            match id.as_binding_identifier(ast) {
                Some(id) => id.name(ast),
                None => return,
            }
        };

        if let Some(function) = init.as_function(ctx.ast) {
            set_function_name(function, ctx, name);
        } else if let Some(class) = init.as_class(ctx.ast) {
            set_class_name(class, ctx, name);
        }
    }

    pub fn exit_assignment_expression(
        &mut self,
        expr: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if expr.operator(ctx.ast) != AssignmentOperator::Assign {
            return;
        }

        let (left, right) = {
            let ast = &*ctx.ast;
            (expr.left(ast), expr.right(ast))
        };

        let name = {
            let ast = &*ctx.ast;
            left.as_simple_assignment_target(ast)
                .and_then(|simple| simple.as_identifier_reference(ast))
                .map(|ident| ident.name(ast))
        };
        let Some(name) = name else {
            return;
        };

        if let Some(function) = right.as_function(ctx.ast) {
            set_function_name(function, ctx, name);
        } else if let Some(class) = right.as_class(ctx.ast) {
            set_class_name(class, ctx, name);
        }
    }

    pub fn exit_object_property(
        &mut self,
        property: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let (value, property_key) = {
            let ast = &*ctx.ast;
            (property.value(ast), property.key(ast))
        };

        let name = property_key_name(property_key, ctx.ast);
        let Some(name) = name else {
            return;
        };

        if let Some(function) = value.as_function(ctx.ast) {
            set_function_name(function, ctx, name);
        } else if let Some(class) = value.as_class(ctx.ast) {
            set_class_name(class, ctx, name);
        }
    }
}

fn set_function_name(function: Function, ctx: &mut TransformerCtx<'_, '_, '_>, name: AstUtf8) {
    if function.id(ctx.ast).is_some() {
        return;
    }
    publish_expression_name(
        ExpressionNameTarget::Function(function),
        ctx,
        name,
        SymbolFlags::FUNCTION,
    );
}

fn set_class_name(class: Class, ctx: &mut TransformerCtx<'_, '_, '_>, name: AstUtf8) {
    if class.id(ctx.ast).is_some() {
        return;
    }
    publish_expression_name(
        ExpressionNameTarget::Class(class),
        ctx,
        name,
        SymbolFlags::CLASS,
    );
}

#[derive(Clone, Copy)]
enum ExpressionNameTarget {
    Function(Function),
    Class(Class),
}

impl ExpressionNameTarget {
    fn node_id(self) -> swc_next_ecma_ast::NodeId {
        match self {
            Self::Function(node) => node.node_id(),
            Self::Class(node) => node.node_id(),
        }
    }

    fn span(self, ast: &swc_next_ecma_ast::Ast) -> swc_next_ecma_ast::Span {
        match self {
            Self::Function(node) => node.span(ast),
            Self::Class(node) => node.span(ast),
        }
    }
}

fn publish_expression_name(
    target: ExpressionNameTarget,
    ctx: &mut TransformerCtx<'_, '_, '_>,
    name: AstUtf8,
    flags: SymbolFlags,
) {
    let owner = target.node_id();
    let body_scope = ctx.semantic().scope_of(owner);
    let body_scope_data = ctx.semantic().scope(body_scope);
    let outer_scope = body_scope_data
        .parent
        .expect("function/class expression scope has an outer scope");
    let references = references_shadowed_by_expression_name(ctx, body_scope, name);
    let semantic_name = ctx.ast.canonicalize_utf8(name);
    let span = target.span(ctx.ast);

    ctx.transaction(owner, |tx| {
        let expression_name_scope = tx.create_scope(
            outer_scope,
            owner,
            scope::ScopeKind::ExpressionName,
            body_scope_data.flags,
        );
        tx.reparent_scope(body_scope, expression_name_scope);

        match target {
            ExpressionNameTarget::Function(function) => {
                let clone = tx.clone_detached(function);
                let binding = tx.build_detached(|ast| {
                    let binding = ast.binding_identifier(
                        span,
                        name,
                        swc_next_ecma_ast::TypedSubRange::empty(),
                        None,
                        false,
                    );
                    clone.root.set_id(ast, Some(binding));
                    binding
                });
                let symbol = tx.declare(
                    expression_name_scope,
                    binding.node_id(),
                    semantic_name,
                    flags,
                );
                tx.replace_from_transformed_clone(function, clone, body_scope, &[]);
                for reference in references {
                    tx.rebind_reference(reference, Some(symbol));
                }
            }
            ExpressionNameTarget::Class(class) => {
                let clone = tx.clone_detached(class);
                let binding = tx.build_detached(|ast| {
                    let binding = ast.binding_identifier(
                        span,
                        name,
                        swc_next_ecma_ast::TypedSubRange::empty(),
                        None,
                        false,
                    );
                    clone.root.set_id(ast, Some(binding));
                    binding
                });
                let symbol = tx.declare(
                    expression_name_scope,
                    binding.node_id(),
                    semantic_name,
                    flags,
                );
                tx.replace_from_transformed_clone(class, clone, body_scope, &[]);
                for reference in references {
                    tx.rebind_reference(reference, Some(symbol));
                }
            }
        }
    })
    .expect("function-name mutation must pass preflight");
}

fn references_shadowed_by_expression_name(
    ctx: &TransformerCtx<'_, '_, '_>,
    body_scope: ScopeId,
    name: AstUtf8,
) -> Vec<ReferenceId> {
    let name = ctx.ast.canonicalize_utf8(name);
    ctx.semantic()
        .iter_references()
        .filter(|entry| {
            entry.reference.name == name
                && ctx
                    .semantic()
                    .scope_is_ancestor(body_scope, entry.reference.scope)
                && entry.reference.symbol.is_none_or(|symbol| {
                    !ctx.semantic()
                        .scope_is_ancestor(body_scope, ctx.semantic().symbol(symbol).scope)
                })
        })
        .map(|entry| entry.id)
        .collect()
}

type AstUtf8 = swc_next_ecma_ast::AstUtf8;

fn property_key_name(key: PropertyKey, ast: &swc_next_ecma_ast::Ast) -> Option<AstUtf8> {
    if let Some(ident) = key.as_identifier_name(ast) {
        return Some(ident.name(ast));
    }
    None
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze, scope::ScopeKind};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn inferred_names_create_expression_scopes_and_rebind_inner_references() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "let f = function() { return f; }; let C = class { static self() { return C; } }; f; C;",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.function_name = true;
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

        let expression_symbols: Vec<_> = semantic
            .iter_symbols()
            .filter(|entry| semantic.scope(entry.symbol.scope).kind == ScopeKind::ExpressionName)
            .collect();
        assert_eq!(expression_symbols.len(), 2);
        assert!(expression_symbols.iter().any(|entry| {
            ast.get_ast_str_bytes(entry.symbol.name) == b"f"
                && entry.symbol.flags.contains(SymbolFlags::FUNCTION)
        }));
        assert!(expression_symbols.iter().any(|entry| {
            ast.get_ast_str_bytes(entry.symbol.name) == b"C"
                && entry.symbol.flags.contains(SymbolFlags::CLASS)
        }));

        let named_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| matches!(ast.get_ast_str_bytes(entry.reference.name), b"f" | b"C"))
            .collect();
        assert_eq!(named_references.len(), 4);
        assert_eq!(
            named_references
                .iter()
                .filter(|entry| {
                    let symbol = semantic.symbol(entry.reference.symbol.unwrap());
                    semantic.scope(symbol.scope).kind == ScopeKind::ExpressionName
                })
                .count(),
            2
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn inferred_name_updates_dynamic_reference_fallback() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "var obj = {}; var f = function () { with (obj) { return f; } };",
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
        options.es2015.function_name = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let reference = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"f")
            .unwrap()
            .id;
        assert!(semantic.reference(reference).flags.is_dynamic());

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let reference = semantic.reference(reference);
        assert!(reference.flags.is_dynamic());
        let symbol = semantic.symbol(reference.symbol.unwrap());
        assert_eq!(
            semantic.scope(symbol.scope).kind,
            swc_next_ecma_semantic::scope::ScopeKind::ExpressionName
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
