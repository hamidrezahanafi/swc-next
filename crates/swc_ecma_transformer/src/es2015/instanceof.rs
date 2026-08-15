//! ES2015: instanceof — Convert `a instanceof B` to `_instanceof(a, B)`.
//!
//! ```js
//! foo instanceof Bar → _instanceof(foo, Bar)
//! ```

use swc_next_ecma_ast::{Argument, BinaryExpression, BinaryOperator, GetSpan};

use crate::state::TransformerCtx;

pub(super) struct InstanceOf;

impl InstanceOf {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.operator(ctx.ast) != BinaryOperator::Instanceof {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);
        let right = node.right(ctx.ast);
        let helper_name = ctx.ast.add_utf8("_instanceof");

        let callee = ctx.ast.expr_identifier_reference(span, helper_name);
        let args = ctx
            .ast
            .add_typed_sub_range([Argument::Expr(left), Argument::Expr(right)]);
        let call_expr = ctx.ast.expr_call_expression(span, callee, args, false, None);
        unsafe { ctx.ast.replace_data(node, call_expr) };
    }
}