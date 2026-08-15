//! ES2016: Exponentiation operator transform.
//!
//! ```js
//! a ** b → Math.pow(a, b)
//! a **= b → a = Math.pow(a, b)
//! ```

use swc_next_ecma_ast::{
    Argument, AssignmentExpression, AssignmentOperator, BinaryExpression, BinaryOperator, Expr,
    GetSpan, PropertyKey,
};

use crate::state::TransformerCtx;

pub(super) struct Exponentiation;

impl Exponentiation {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.operator(ctx.ast) != BinaryOperator::Exponent {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);
        let right = node.right(ctx.ast);

        let pow_call = create_math_pow(span, left, right, ctx);
        unsafe { ctx.ast.replace_data(node, pow_call) };
    }

    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.operator(ctx.ast) != AssignmentOperator::ExponentAssign {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);
        let right = node.right(ctx.ast);

        if left.is_simple_assignment_target(ctx.ast) {
            let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
            if sat.is_identifier_reference(ctx.ast) {
                let ident = sat.as_identifier_reference(ctx.ast).unwrap();
                let pow_call = create_math_pow(
                    span,
                    Expr::IdentifierReference(ident),
                    right,
                    ctx,
                );
                let assign = ctx.ast.expr_assignment_expression(
                    span,
                    left,
                    pow_call,
                    AssignmentOperator::Assign,
                );
                unsafe { ctx.ast.replace_data(node, assign) };
            } else if sat.is_member_expression(ctx.ast) {
                let member = sat.as_member_expression(ctx.ast).unwrap();
                let obj = member.object(ctx.ast);
                let prop = member.property(ctx.ast);
                let computed = member.computed(ctx.ast);

                // obj.prop = Math.pow(obj.prop, right)
                let pow_left =
                    ctx.ast
                        .expr_member_expression(span, obj, prop, computed, false);
                let pow_call = create_math_pow(span, pow_left, right, ctx);
                let assign = ctx.ast.expr_assignment_expression(
                    span,
                    left,
                    pow_call,
                    AssignmentOperator::Assign,
                );
                unsafe { ctx.ast.replace_data(node, assign) };
            }
        }
    }
}

fn create_math_pow(
    span: swc_next_ecma_ast::Span,
    left: Expr,
    right: Expr,
    ctx: &mut TransformerCtx<'_, '_>,
) -> Expr {
    let math_name = ctx.ast.add_utf8("Math");
    let pow_name = ctx.ast.add_utf8("pow");
    let math = ctx.ast.expr_identifier_reference(span, math_name);
    let pow_prop = PropertyKey::IdentifierName(ctx.ast.identifier_name(span, pow_name));
    let callee = ctx.ast.expr_member_expression(span, math, pow_prop, false, false);
    let args = ctx.ast.add_typed_sub_range([Argument::Expr(left), Argument::Expr(right)]);
    ctx.ast.expr_call_expression(span, callee, args, false, None)
}