//! ES3: member-expression-literals — Convert between computed and dot-notation
//! member expressions based on identifier validity.
//!
//! ```js
//! obj["foo"] → obj.foo           // valid identifier → dot notation
//! obj.const  → obj["const"]      // reserved word → computed
//! ```

use swc_next_ecma_ast::{MemberExpression, PropertyKey};

use crate::state::TransformerCtx;

pub(super) struct MemberExprLits;

impl MemberExprLits {
    pub fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let computed = node.computed(ctx.ast);
        let property = node.property(ctx.ast);

        if computed {
            // obj["foo"] → obj.foo if the string is a valid identifier
            let str_value = string_literal_value(property, ctx);
            if let Some(ref str) = str_value {
                if is_valid_ident(str) && !is_es3_reserved(str) {
                    let name = ctx.ast.add_utf8(str);
                    let ident_name = ctx.ast.identifier_name(DUMMY_SP, name);
                    node.set_computed(ctx.ast, false);
                    node.set_property(ctx.ast, PropertyKey::IdentifierName(ident_name));
                }
            }
        } else {
            // obj.foo → obj["foo"] if the identifier is reserved
            if let Some(ident_name) = property.as_identifier_name(ctx.ast) {
                let name = ctx.ast.get_utf8(ident_name.name(ctx.ast)).to_owned();
                if !is_valid_ident(&name) || is_es3_reserved(&name) {
                    let value = ctx.ast.add_wtf8(name.as_str().into());
                    let raw = ctx.ast.add_utf8(&format!("\"{}\"", name));
                    let str_lit = ctx.ast.string_literal(DUMMY_SP, value, raw);
                    node.set_computed(ctx.ast, true);
                    node.set_property(ctx.ast, PropertyKey::StringLiteral(str_lit));
                }
            }
        }
    }
}

fn string_literal_value(property: PropertyKey, ctx: &TransformerCtx<'_, '_>) -> Option<String> {
    if let Some(str_lit) = property.as_string_literal(ctx.ast) {
        let wtf8 = ctx.ast.get_wtf8(str_lit.value(ctx.ast));
        return wtf8.as_str().map(|s| s.to_owned());
    }
    if let Some(expr) = property.as_expr(ctx.ast) {
        if let Some(str_lit) = expr.as_string_literal(ctx.ast) {
            let wtf8 = ctx.ast.get_wtf8(str_lit.value(ctx.ast));
            return wtf8.as_str().map(|s| s.to_owned());
        }
    }
    None
}

fn is_valid_ident(s: &str) -> bool {
    swc_next_ecma_ast::is_identifier(s.as_bytes())
}

fn is_es3_reserved(s: &str) -> bool {
    matches!(
        s,
        "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float"
            | "goto" | "int" | "long" | "native" | "short" | "synchronized"
            | "throws" | "transient" | "volatile"
    )
}

use swc_next_ecma_ast::DUMMY_SP;