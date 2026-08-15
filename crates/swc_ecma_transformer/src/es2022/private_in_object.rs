//! ES2022: private-in-object — Transform `#x in obj` to `_checkInRHS(obj)`.
//!
//! ```js
//! #x in obj → _checkInRHS(obj)
//! ```

use swc_next_ecma_ast::{Argument, BinaryExpression, BinaryOperator, GetSpan};

use crate::state::TransformerCtx;

pub(super) struct PrivateInObject;

impl PrivateInObject {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if op != BinaryOperator::In {
            return;
        }

        let left = node.left(ctx.ast);
        if !left.is_private_identifier(ctx.ast) {
            return;
        }

        // #x in obj → _checkInRHS(obj)
        let span = node.span(ctx.ast);
        let right = node.right(ctx.ast);

        let helper_name = ctx.ast.add_utf8("_checkInRHS");
        let callee = ctx.ast.expr_identifier_reference(span, helper_name);
        let args = ctx.ast.add_typed_sub_range([Argument::Expr(right)]);
        let call = ctx.ast.expr_call_expression(span, callee, args, false, None);

        unsafe { ctx.ast.replace_data(node, call) };
    }
}