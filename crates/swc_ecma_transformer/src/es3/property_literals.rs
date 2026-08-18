//! ES3: property-literals — Convert between string-valued and identifier-valued
//! property names.
//!
//! ```js
//! { "bar": 1 } → { bar: 1 }  // valid identifier → dot notation
//! { default: 1 } → { "default": 1 }  // reserved word → string literal
//! ```

use swc_next_ecma_ast::{ObjectProperty, PropertyKey};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct PropertyLiterals;

impl PropertyLiterals {
    pub fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.computed(ctx.ast) {
            return;
        }
        let key = node.key(ctx.ast);

        // { "bar": 1 } → { bar: 1 } if the string is a valid non-reserved identifier
        let replacement = if let Some(str_lit) = key.as_string_literal(ctx.ast) {
            let value = ctx.ast.get_wtf8(str_lit.value(ctx.ast));
            let str = value.as_str().map(|s| s.to_owned());
            if let Some(str) = str {
                if is_valid_ident(&str) && !is_es3_reserved(&str) {
                    Some(PropertyReplacement::Identifier(str))
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Some(ident_name) = key.as_identifier_name(ctx.ast) {
            // { default: 1 } → { "default": 1 } if the identifier is reserved
            let name = ctx.ast.get_utf8(ident_name.name(ctx.ast)).to_owned();
            if !is_valid_ident(&name) || is_es3_reserved(&name) {
                Some(PropertyReplacement::String(name))
            } else {
                None
            }
        } else {
            None
        };
        let Some(replacement) = replacement else {
            return;
        };

        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let clone = tx.clone_detached(node);
            tx.build_detached(|ast| match replacement {
                PropertyReplacement::Identifier(name) => {
                    let name = ast.add_utf8(&name);
                    let ident_name = ast.identifier_name(DUMMY_SP, name);
                    clone
                        .root
                        .set_key(ast, PropertyKey::IdentifierName(ident_name));
                }
                PropertyReplacement::String(name) => {
                    let value = ast.add_wtf8(name.as_str().into());
                    let raw = ast.add_utf8(&format!("\"{}\"", name));
                    let str_lit = ast.string_literal(DUMMY_SP, value, raw);
                    clone.root.set_key(ast, PropertyKey::StringLiteral(str_lit));
                }
            });
            tx.replace_from_transformed_clone(node, clone, scope, &[]);
        })
        .expect("property-literals mutation must pass preflight");
    }
}

enum PropertyReplacement {
    Identifier(String),
    String(String),
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
