//! ES2015: typeof-symbol — Convert `typeof` to handle Symbol type.
//!
//! ```js
//! typeof sym → typeof sym === "undefined" ? "undefined" : _typeof(sym)
//! typeof Symbol() → _typeof(Symbol())
//! ```

use swc_next_ecma_ast::{
    Argument, BinaryExpression, BinaryOperator, Expr, GetSpan, UnaryExpression, UnaryOperator,
};

use crate::state::TransformerCtx;

pub(super) struct TypeofSymbol {
    /// When non-zero, skip transforming typeof expressions inside binary
    /// comparisons like `typeof x === "string"`.
    skip_depth: u32,
}

impl TypeofSymbol {
    pub fn new() -> Self {
        Self { skip_depth: 0 }
    }

    pub fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if !matches!(
            op,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictEqual
                | BinaryOperator::StrictNotEqual
        ) {
            return;
        }

        let left = node.left(ctx.ast);
        if left.is_unary_expression(ctx.ast) {
            let unary = left.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let right = node.right(ctx.ast);
                if is_non_symbol_literal(right, ctx.ast) {
                    self.skip_depth += 1;
                    return;
                }
            }
        }

        let right = node.right(ctx.ast);
        if right.is_unary_expression(ctx.ast) {
            let unary = right.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let left = node.left(ctx.ast);
                if is_non_symbol_literal(left, ctx.ast) {
                    self.skip_depth += 1;
                }
            }
        }
    }

    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if !matches!(
            op,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictEqual
                | BinaryOperator::StrictNotEqual
        ) {
            return;
        }

        let left = node.left(ctx.ast);
        if left.is_unary_expression(ctx.ast) {
            let unary = left.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let right = node.right(ctx.ast);
                if is_non_symbol_literal(right, ctx.ast) {
                    self.skip_depth = self.skip_depth.saturating_sub(1);
                    return;
                }
            }
        }

        let right = node.right(ctx.ast);
        if right.is_unary_expression(ctx.ast) {
            let unary = right.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let left = node.left(ctx.ast);
                if is_non_symbol_literal(left, ctx.ast) {
                    self.skip_depth = self.skip_depth.saturating_sub(1);
                }
            }
        }
    }

    pub fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.skip_depth > 0 {
            return;
        }
        if node.operator(ctx.ast) != UnaryOperator::Typeof {
            return;
        }

        let span = node.span(ctx.ast);
        let arg = node.argument(ctx.ast);
        let is_ident = arg.is_identifier_reference(ctx.ast);

        // Build _typeof(x) call
        let helper_name = ctx.ast.add_utf8("_typeof");
        let callee = ctx.ast.expr_identifier_reference(span, helper_name);
        let call_args = ctx.ast.add_typed_sub_range([Argument::Expr(arg)]);
        let call = ctx.ast.expr_call_expression(span, callee, call_args, false, None);

        if is_ident {
            // typeof x → typeof x === "undefined" ? "undefined" : _typeof(x)
            let undefined_value = ctx.ast.add_wtf8("undefined".into());
            let undefined_raw = ctx.ast.add_utf8("\"undefined\"");
            let undefined_str = ctx.ast.expr_string_literal(span, undefined_value, undefined_raw);

            // Build: typeof x
            let typeof_arg = ctx.ast.expr_unary_expression(span, node.argument(ctx.ast), UnaryOperator::Typeof);

            // Build: typeof x === "undefined"
            let test = ctx.ast.expr_binary_expression(
                span,
                typeof_arg,
                undefined_str,
                BinaryOperator::StrictEqual,
            );

            // Build: "undefined" (for consequent)
            let uv2 = ctx.ast.add_wtf8("undefined".into());
            let ur2 = ctx.ast.add_utf8("\"undefined\"");
            let cons = ctx.ast.expr_string_literal(span, uv2, ur2);

            let ternary = ctx.ast.expr_conditional_expression(span, test, cons, call);
            unsafe { ctx.ast.replace_data(node, ternary) };
        } else {
            // typeof expr → _typeof(expr)
            unsafe { ctx.ast.replace_data(node, call) };
        }
    }
}

fn is_non_symbol_literal(expr: Expr, ast: &swc_next_ecma_ast::Ast) -> bool {
    let Some(s) = expr.as_string_literal(ast) else {
        return false;
    };
    let value = ast.get_wtf8(s.value(ast));
    matches!(
        value.as_str(),
        Some("undefined" | "boolean" | "number" | "string" | "function")
    )
}