//! ES2020: optional-chaining — Transform `?.` to conditional expressions.

use swc_next_ecma_ast::{
    BinaryOperator, Expr, ExprData, GetSpan, LogicalOperator, NumericLiteralKind, Span,
    UnaryOperator,
};

use crate::{
    mutation::{DetachedClone, MutationTransaction},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct OptionalChaining;

impl OptionalChaining {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_expression(&self, node: Expr, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let ExprData::ChainExpression(chain) = ctx.ast.expr_data(node) else {
            return;
        };
        let span = chain.span(ctx.ast);
        let source_inner = chain.expression(ctx.ast);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(node);
            let chain = clone.root.as_chain_expression(tx.ast()).unwrap();
            let cloned_inner = chain.expression(tx.ast());
            let mut copies = Vec::new();
            let result = self.lower_chain(source_inner, cloned_inner, tx, span, &mut copies);
            clone.replace_root(result);
            let layout = tx.replace_expression_from_transformed_clone(node, clone, scope, &[]);
            for copy in &copies {
                tx.duplicate_semantics_from_detached_clone(copy, &layout);
            }
        })
        .expect("optional-chaining mutation must pass preflight");
    }

    fn lower_chain<'ast>(
        &self,
        source: Expr,
        expr: Expr,
        tx: &mut MutationTransaction<'_, 'ast>,
        span: Span,
        copies: &mut Vec<DetachedClone<Expr>>,
    ) -> Expr {
        match (tx.ast().expr_data(source), tx.ast().expr_data(expr)) {
            (ExprData::MemberExpression(source_member), ExprData::MemberExpression(member)) => {
                let object = member.object(tx.ast());
                let property = member.property(tx.ast());
                if member.optional(tx.ast()) {
                    let second = tx.clone_detached(source_member.object(tx.ast()));
                    let third = tx.clone_detached(source_member.object(tx.ast()));
                    let result = tx.build_detached(|ast| {
                        let null_check = self.build_null_check(span, object, second.root, ast);
                        let void_node = self.build_void_0(span, ast);
                        let member_access = ast.expr_member_expression(
                            span,
                            third.root,
                            property,
                            member.computed(ast),
                            false,
                        );
                        ast.expr_conditional_expression(span, null_check, void_node, member_access)
                    });
                    copies.push(second);
                    copies.push(third);
                    result
                } else {
                    let lowered =
                        self.lower_chain(source_member.object(tx.ast()), object, tx, span, copies);
                    tx.build_detached(|ast| {
                        ast.expr_member_expression(
                            span,
                            lowered,
                            property,
                            member.computed(ast),
                            false,
                        )
                    })
                }
            }
            (ExprData::CallExpression(source_call), ExprData::CallExpression(call)) => {
                let callee = call.callee(tx.ast());
                let args = call.arguments(tx.ast());
                let type_args = call.type_arguments(tx.ast());
                if call.optional(tx.ast()) {
                    let second = tx.clone_detached(source_call.callee(tx.ast()));
                    let third = tx.clone_detached(source_call.callee(tx.ast()));
                    let result = tx.build_detached(|ast| {
                        let null_check = self.build_null_check(span, callee, second.root, ast);
                        let void_node = self.build_void_0(span, ast);
                        let call_expr =
                            ast.expr_call_expression(span, third.root, args, false, type_args);
                        ast.expr_conditional_expression(span, null_check, void_node, call_expr)
                    });
                    copies.push(second);
                    copies.push(third);
                    result
                } else {
                    let lowered =
                        self.lower_chain(source_call.callee(tx.ast()), callee, tx, span, copies);
                    tx.build_detached(|ast| {
                        ast.expr_call_expression(span, lowered, args, false, type_args)
                    })
                }
            }
            _ => expr,
        }
    }

    fn build_void_0(&self, span: Span, ast: &mut swc_next_ecma_ast::Ast<'_>) -> Expr {
        let zero_str = ast.add_utf8("0");
        let zero = ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero_str);
        ast.expr_unary_expression(span, zero, UnaryOperator::Void)
    }

    fn build_null_check(
        &self,
        span: Span,
        first: Expr,
        second: Expr,
        ast: &mut swc_next_ecma_ast::Ast<'_>,
    ) -> Expr {
        let null_node = ast.expr_null_literal(span);
        let void_node = self.build_void_0(span, ast);
        let eq_null =
            ast.expr_binary_expression(span, first, null_node, BinaryOperator::StrictEqual);
        let eq_void =
            ast.expr_binary_expression(span, second, void_node, BinaryOperator::StrictEqual);
        ast.expr_logical_expression(span, eq_null, eq_void, LogicalOperator::Or)
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

    #[test]
    fn chain_replacement_and_copied_occurrences_are_transactional() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function f(value) { return value?.method?.(() => value); }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2020.optional_chaining = true;
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
}
