//! ES2020: nullish-coalescing — Convert `a ?? b` to `a !== null && a !== void 0 ? a : b`.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, BinaryOperator, Expr, GetSpan, LogicalExpression,
    LogicalOperator, NumericLiteralKind, UnaryOperator,
};

use crate::state::TransformerCtx;

pub(super) struct NullishCoalescing;

impl NullishCoalescing {
    pub fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.operator(ctx.ast) != LogicalOperator::NullishCoalescing {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);
        let right = node.right(ctx.ast);

        // Build: void 0 (fresh literal)
        let zero = ctx.ast.add_utf8("0");
        let zero_lit = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero);
        let void0 = ctx.ast.expr_unary_expression(span, zero_lit, UnaryOperator::Void);

        // Build: left !== null
        let null_val = ctx.ast.expr_null_literal(span);
        let not_null = ctx.ast.expr_binary_expression(
            span, left, null_val, BinaryOperator::StrictNotEqual,
        );

        // Build: left !== void 0
        let not_void = ctx.ast.expr_binary_expression(
            span, left, void0, BinaryOperator::StrictNotEqual,
        );

        // Build: left !== null && left !== void 0
        let test =
            ctx.ast
                .expr_logical_expression(span, not_null, not_void, LogicalOperator::And);

        // Build: test ? left : right
        let ternary = ctx.ast.expr_conditional_expression(span, test, left, right);
        unsafe { ctx.ast.replace_data(node, ternary) };
    }

    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.operator(ctx.ast) != AssignmentOperator::NullishAssign {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);

        if !left.is_simple_assignment_target(ctx.ast) {
            return;
        }
        let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
        if !sat.is_identifier_reference(ctx.ast) {
            return;
        }
        let ident = sat.as_identifier_reference(ctx.ast).unwrap();
        let right = node.right(ctx.ast);

        let ident_expr = Expr::IdentifierReference(ident);

        // Build: void 0
        let zero = ctx.ast.add_utf8("0");
        let zero_lit = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero);
        let void0 = ctx.ast.expr_unary_expression(span, zero_lit, UnaryOperator::Void);

        // Build: ident !== null
        let null_val = ctx.ast.expr_null_literal(span);
        let not_null = ctx.ast.expr_binary_expression(
            span, ident_expr, null_val, BinaryOperator::StrictNotEqual,
        );

        // Build: ident !== void 0
        let not_void = ctx.ast.expr_binary_expression(
            span, ident_expr, void0, BinaryOperator::StrictNotEqual,
        );

        // Build: ident !== null && ident !== void 0
        let test =
            ctx.ast
                .expr_logical_expression(span, not_null, not_void, LogicalOperator::And);

        // Build: test ? ident : right
        let ident_expr = Expr::IdentifierReference(ident);
        let ternary = ctx.ast.expr_conditional_expression(
            span, test, ident_expr, right,
        );

        // Build: left = ternary
        let assign = ctx.ast.expr_assignment_expression(
            span, left, ternary, AssignmentOperator::Assign,
        );
        unsafe { ctx.ast.replace_data(node, assign) };
    }
}