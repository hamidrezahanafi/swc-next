//! Safari 10.3 bugfix: async arrow functions in class methods throw.
//!
//! Converts async arrow functions inside class methods to regular function
//! expressions. See https://bugs.webkit.org/show_bug.cgi?id=166879

use swc_next_ecma_ast::{
    ArrowFunctionExpression, Expr, ExprData, FunctionType, GetSpan, NodeId, NodeKindData, Stmt,
    TypedSubRange,
};
use swc_next_ecma_semantic::scope::{ScopeFlags, ScopeKind};

use crate::{
    mutation::{DetachedScopeRegion, ScopeHandle},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct AsyncArrowsInClass {
    class_method_depth: usize,
}

impl AsyncArrowsInClass {
    pub fn new() -> Self {
        Self {
            class_method_depth: 0,
        }
    }

    pub fn enter_method_definition(&mut self) {
        self.class_method_depth += 1;
    }

    pub fn exit_method_definition(&mut self) {
        self.class_method_depth -= 1;
    }

    pub fn exit_expression(&mut self, node: Expr, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if self.class_method_depth == 0 {
            return;
        }
        let ExprData::ArrowFunctionExpression(arrow) = ctx.ast.expr_data(node) else {
            return;
        };
        if !arrow.r#async(ctx.ast) {
            return;
        }

        self.lower_arrow(arrow, ctx);
    }

    fn lower_arrow(&self, arrow: ArrowFunctionExpression, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let span = arrow.span(ctx.ast);
        let old = Expr::ArrowFunctionExpression(arrow);
        let function_scope = ctx.semantic().scope_of(arrow.node_id());
        ctx.transaction(arrow.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let cloned_arrow = clone.root.as_arrow_function_expression(tx.ast()).unwrap();
            let params = cloned_arrow.params(tx.ast());
            let old_body = cloned_arrow.body(tx.ast());
            let mut generated_body = None;
            let body = match tx.ast().arrow_function_body_data(old_body) {
                swc_next_ecma_ast::ArrowFunctionBodyData::FunctionBody(body) => body,
                swc_next_ecma_ast::ArrowFunctionBodyData::Expr(expr) => {
                    let body = tx.build_detached(|ast| {
                        let return_stmt = ast.return_statement(span, Some(expr));
                        let items = ast.add_typed_sub_range([Stmt::ReturnStatement(return_stmt)]);
                        ast.function_body(span, TypedSubRange::empty(), items)
                    });
                    generated_body = Some(body);
                    body
                }
            };
            let function = tx.build_detached(|ast| {
                ast.function(
                    span,
                    FunctionType::FunctionExpression,
                    None,
                    false,
                    true,
                    params,
                    body,
                    None,
                    None,
                )
            });
            clone.replace_root(Expr::Function(function));

            let mut regions = Vec::new();
            if let Some(body) = generated_body
                && parameters_contain_expression(tx.ast(), params)
            {
                let flags = ScopeFlags {
                    strict: tx.semantic().scope(function_scope).flags.strict,
                    ..ScopeFlags::default()
                };
                let scope = tx.create_scope(
                    function_scope,
                    body.node_id(),
                    ScopeKind::FunctionBody,
                    flags,
                );
                regions.push(DetachedScopeRegion {
                    root: body.node_id(),
                    scope: ScopeHandle::Pending(scope),
                });
            }
            tx.replace_expression_from_transformed_clone(old, clone, function_scope, &regions);
        })
        .expect("async arrow mutation must pass preflight");
    }
}

fn parameters_contain_expression(
    ast: &swc_next_ecma_ast::Ast<'_>,
    params: swc_next_ecma_ast::FormalParameters,
) -> bool {
    params.items(ast).iter().any(|parameter| {
        let parameter = ast.get_node_in_sub_range(parameter).node_id();
        let pattern = match ast.kind_data(parameter) {
            NodeKindData::FormalParameter(parameter) => parameter.pattern(ast).node_id(),
            NodeKindData::TsParameterProperty(parameter) => parameter.parameter(ast).node_id(),
            _ => parameter,
        };
        pattern_contains_expression(ast, Some(pattern))
    }) || params
        .rest(ast)
        .is_some_and(|rest| pattern_contains_expression(ast, Some(rest.node_id())))
}

fn pattern_contains_expression(ast: &swc_next_ecma_ast::Ast<'_>, pattern: Option<NodeId>) -> bool {
    let Some(pattern) = pattern else { return false };
    match ast.kind_data(pattern) {
        NodeKindData::AssignmentPattern(_) => true,
        NodeKindData::BindingRestElement(rest) => {
            pattern_contains_expression(ast, Some(rest.argument(ast).node_id()))
        }
        NodeKindData::ArrayPattern(array) => {
            array.elements(ast).iter().any(|element| {
                ast.get_node_in_sub_range(element).is_some_and(|element| {
                    pattern_contains_expression(ast, Some(element.node_id()))
                })
            }) || array
                .rest(ast)
                .is_some_and(|rest| pattern_contains_expression(ast, Some(rest.node_id())))
        }
        NodeKindData::ObjectPattern(object) => object.properties(ast).iter().any(|property| {
            let property = ast.get_node_in_sub_range(property);
            property.computed(ast)
                || pattern_contains_expression(ast, Some(property.value(ast).node_id()))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    fn lower_and_assert(source: &str) {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.bugfixes.async_arrows_in_class = true;
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
    }

    #[test]
    fn lowers_block_and_concise_async_arrows_transactionally() {
        lower_and_assert(
            "class C { method() { const a = async x => x; const b = async (x = value) => x; const c = async x => { return x; }; } }",
        );
    }

    #[test]
    fn nested_class_methods_do_not_clear_the_outer_method_state() {
        lower_and_assert(
            "class C { method() { class D { nested() { return async x => x; } } return async y => y; } }",
        );
    }
}
