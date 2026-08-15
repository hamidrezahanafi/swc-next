//! ES2021: logical-assignment-operators — Convert `a ||= b`, `a &&= b` to `a || (a = b)`, `a && (a = b)`.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Expr, GetSpan, LogicalOperator,
};

use crate::state::TransformerCtx;

pub(super) struct LogicalAssignment;

impl LogicalAssignment {
    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        let logical_op = match op {
            AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
            AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
            _ => return,
        };

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);

        // Only handle simple identifier: x ||= y → x || (x = y)
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
        let assign = ctx.ast.expr_assignment_expression(
            span,
            left,
            right,
            AssignmentOperator::Assign,
        );

        let result = ctx.ast.expr_logical_expression(span, ident_expr, assign, logical_op);
        unsafe { ctx.ast.replace_data(node, result) };
    }
}