//! ES2015: template-literals — Convert template literals to `.concat()` calls.
//!
//! ```js
//! `hello ${name}` → "hello ".concat(name)
//! ```

use swc_next_ecma_ast::{
    Argument, Expr, GetSpan, TemplateLiteral,
};

use crate::state::TransformerCtx;

pub(super) struct TemplateLiteralTransform;

impl TemplateLiteralTransform {
    pub fn exit_template_literal(
        &mut self,
        node: TemplateLiteral,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let quasis = node.quasis(ctx.ast);
        let exprs = node.expressions(ctx.ast);

        if quasis.is_empty() {
            return;
        }

        let first = quasis.get_node(ctx.ast, 0).unwrap();
        let mut result = quasi_to_expr(first, ctx);

        for i in 0..exprs.len() {
            let quasi = quasis.get_node(ctx.ast, i + 1).unwrap();
            let quasi_expr = quasi_to_expr(quasi, ctx);

            let callee = ctx.ast.expr_member_expression(
                DUMMY_SP,
                result,
                ctx.ast.property_key_identifier_name(DUMMY_SP, ctx.ast.add_utf8("concat")),
                false,
                false,
            );
            result = ctx.ast.expr_call_expression(
                DUMMY_SP,
                callee,
                ctx.ast.add_typed_sub_range([Argument::Expr(quasi_expr)]),
                false,
                None,
            );
        }

        unsafe { ctx.ast.replace_data(node, result) };
    }
}

fn quasi_to_expr(quasi: swc_next_ecma_ast::TemplateElement, ctx: &mut TransformerCtx<'_, '_>) -> Expr {
    if quasi.is_cooked_undefined(ctx.ast) {
        return Expr::IdentifierReference(ctx.ast.identifier_reference(
            ctx.ast.add_utf8("undefined"),
            DUMMY_SP,
        ));
    }
    let cooked = quasi.cooked(ctx.ast);
    let raw = quasi.raw(ctx.ast);
    let span = quasi.span(ctx.ast);
    Expr::StringLiteral(ctx.ast.string_literal(span, cooked, raw))
}

use swc_next_ecma_ast::DUMMY_SP;