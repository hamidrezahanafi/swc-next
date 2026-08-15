//! ES2020: optional-chaining — Transform `?.` to conditional expressions.
//!
//! Input:
//! ```js
//! a?.b
//! ```
//!
//! Output:
//! ```js
//! a === null || a === void 0 ? void 0 : a.b
//! ```

use swc_next_ecma_ast::{
    BinaryOperator, Expr, ExprData, GetSpan, LogicalOperator, NumericLiteralKind, Span,
    UnaryOperator,
};

use crate::state::TransformerCtx;

pub(super) struct OptionalChaining;

impl OptionalChaining {
    pub fn new() -> Self {
        Self
    }

    /// Called from `exit_expression`. If the expression is a ChainExpression,
    /// lower it to nested conditional expressions.
    pub fn exit_expression(
        &self,
        node: Expr,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Option<Expr> {
        // Only process ChainExpression
        if !matches!(ctx.ast.expr_data(node), ExprData::ChainExpression(_)) {
            return None;
        }
        let (span, inner) = {
            let ExprData::ChainExpression(chain) = ctx.ast.expr_data(node) else {
                unreachable!()
            };
            (chain.span(ctx.ast), chain.expression(ctx.ast))
        };
        let result = self.lower_chain(inner, ctx, span);
        Some(result)
    }

    /// Lower a chain expression to nested ternary operators.
    fn lower_chain(&self, expr: Expr, ctx: &mut TransformerCtx<'_, '_>, span: Span) -> Expr {
        match ctx.ast.expr_data(expr) {
            ExprData::MemberExpression(member) => {
                let (object, property, _computed, optional) = (
                    member.object(ctx.ast),
                    member.property(ctx.ast),
                    member.computed(ctx.ast),
                    member.optional(ctx.ast),
                );
                if optional {
                    let null_check = self.build_null_check(span, object, ctx);
                    let void_node = self.build_void_0(span, ctx);
                    let member_access = ctx.ast.expr_member_expression(
                        span,
                        object,
                        property,
                        false,
                        false,
                    );
                    ctx.ast.expr_conditional_expression(
                        span,
                        null_check,
                        void_node,
                        member_access,
                    )
                } else {
                    let lowered = self.lower_chain(object, ctx, span);
                    ctx.ast.expr_member_expression(span, lowered, property, false, false)
                }
            }
            ExprData::CallExpression(call) => {
                let (callee, args, optional, type_args) = (
                    call.callee(ctx.ast),
                    call.arguments(ctx.ast),
                    call.optional(ctx.ast),
                    call.type_arguments(ctx.ast),
                );
                if optional {
                    let null_check = self.build_null_check(span, callee, ctx);
                    let void_node = self.build_void_0(span, ctx);
                    let call_expr =
                        ctx.ast
                            .expr_call_expression(span, callee, args, false, type_args);
                    ctx.ast.expr_conditional_expression(
                        span,
                        null_check,
                        void_node,
                        call_expr,
                    )
                } else {
                    let lowered = self.lower_chain(callee, ctx, span);
                    ctx.ast.expr_call_expression(span, lowered, args, false, type_args)
                }
            }
            _ => expr,
        }
    }

    /// Build `void 0`
    fn build_void_0(&self, span: Span, ctx: &mut TransformerCtx<'_, '_>) -> Expr {
        let zero_str = ctx.ast.add_utf8("0");
        let zero = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero_str);
        ctx.ast.expr_unary_expression(span, zero, UnaryOperator::Void)
    }

    /// Build `obj === null || obj === void 0`
    fn build_null_check(
        &self,
        span: Span,
        obj: Expr,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let null_node = ctx.ast.expr_null_literal(span);
        let void_node = self.build_void_0(span, ctx);
        let eq_null = ctx.ast.expr_binary_expression(
            span,
            obj,
            null_node,
            BinaryOperator::StrictEqual,
        );
        let eq_void = ctx.ast.expr_binary_expression(
            span,
            obj,
            void_node,
            BinaryOperator::StrictEqual,
        );
        ctx.ast
            .expr_logical_expression(span, eq_null, eq_void, LogicalOperator::Or)
    }
}