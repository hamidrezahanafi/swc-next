//! ES3: member-expression-literals — Convert between computed and dot-notation
//! member expressions based on identifier validity.
//!
//! ```js
//! obj["foo"] → obj.foo           // valid identifier → dot notation
//! obj.const  → obj["const"]      // reserved word → computed
//! ```

use swc_next_ecma_ast::{MemberExpression, PropertyKey};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct MemberExprLits;

impl MemberExprLits {
    pub fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let computed = node.computed(ctx.ast);
        let property = node.property(ctx.ast);

        let replacement = if computed {
            // obj["foo"] → obj.foo if the string is a valid identifier
            let str_value = string_literal_value(property, ctx);
            if let Some(ref str) = str_value {
                if is_valid_ident(str) && !is_es3_reserved(str) {
                    Some(MemberPropertyReplacement::Identifier(str.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // obj.foo → obj["foo"] if the identifier is reserved
            if let Some(ident_name) = property.as_identifier_name(ctx.ast) {
                let name = ctx.ast.get_utf8(ident_name.name(ctx.ast)).to_owned();
                if !is_valid_ident(&name) || is_es3_reserved(&name) {
                    Some(MemberPropertyReplacement::String(name))
                } else {
                    None
                }
            } else {
                None
            }
        };
        let Some(replacement) = replacement else {
            return;
        };

        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let clone = tx.clone_detached(node);
            tx.build_detached(|ast| match replacement {
                MemberPropertyReplacement::Identifier(name) => {
                    let name = ast.add_utf8(&name);
                    let ident_name = ast.identifier_name(DUMMY_SP, name);
                    clone.root.set_computed(ast, false);
                    clone
                        .root
                        .set_property(ast, PropertyKey::IdentifierName(ident_name));
                }
                MemberPropertyReplacement::String(name) => {
                    let value = ast.add_wtf8(name.as_str().into());
                    let raw = ast.add_utf8(&format!("\"{}\"", name));
                    let str_lit = ast.string_literal(DUMMY_SP, value, raw);
                    clone.root.set_computed(ast, true);
                    clone
                        .root
                        .set_property(ast, PropertyKey::StringLiteral(str_lit));
                }
            });
            tx.replace_from_transformed_clone(node, clone, scope, &[]);
        })
        .expect("member-expression-literals mutation must pass preflight");
    }
}

enum MemberPropertyReplacement {
    Identifier(String),
    String(String),
}

fn string_literal_value(property: PropertyKey, ctx: &TransformerCtx<'_, '_, '_>) -> Option<String> {
    if let Some(str_lit) = property.as_string_literal(ctx.ast) {
        let wtf8 = ctx.ast.get_wtf8(str_lit.value(ctx.ast));
        return wtf8.as_str().map(|s| s.to_owned());
    }
    if let Some(expr) = property.as_expr(ctx.ast)
        && let Some(str_lit) = expr.as_string_literal(ctx.ast)
    {
        let wtf8 = ctx.ast.get_wtf8(str_lit.value(ctx.ast));
        return wtf8.as_str().map(|s| s.to_owned());
    }
    None
}

fn is_valid_ident(s: &str) -> bool {
    swc_next_ecma_ast::is_identifier(s.as_bytes())
}

fn is_es3_reserved(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "boolean"
            | "byte"
            | "char"
            | "double"
            | "final"
            | "float"
            | "goto"
            | "int"
            | "long"
            | "native"
            | "short"
            | "synchronized"
            | "throws"
            | "transient"
            | "volatile"
    )
}

use swc_next_ecma_ast::DUMMY_SP;
