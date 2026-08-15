use std::collections::HashSet;

use swc_next_ecma_ast::{GetSpan, ObjectExpression, ObjectPropertyKind};

use crate::state::TransformerCtx;

/// ES2015: duplicate-keys — Convert duplicate keys in object literals to
/// computed properties to avoid runtime collision semantics.
///
/// ```js
/// { a: 1, a: 2 } → { a: 1, ["a"]: 2 }
/// ```
pub(super) struct DuplicateKeys;

impl DuplicateKeys {
    pub fn exit_object_expression(
        &mut self,
        object: ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let properties = object.properties(ctx.ast);
        let len = properties.len();
        if len < 2 {
            return;
        }

        // Phase 1: collect key text for all properties
        let mut keys: Vec<Option<KeyId>> = Vec::with_capacity(len);
        let mut seen = HashSet::new();
        let mut has_duplicate = false;

        for i in 0..len {
            let Some(prop_kind) = properties.get_node(ctx.ast, i) else {
                keys.push(None);
                continue;
            };
            let key_id = property_key_of(prop_kind, ctx.ast);
            if let Some(ref key_id) = key_id {
                if !seen.insert(key_id.clone()) {
                    has_duplicate = true;
                }
            }
            keys.push(key_id);
        }

        if !has_duplicate {
            return;
        }

        // Phase 2: convert duplicates to computed keys
        seen.clear();
        for i in 0..len {
            let Some(ref key_id) = keys[i] else {
                continue;
            };
            if !seen.insert(key_id.clone()) {
                // This is a duplicate — convert to computed
                let Some(prop_kind) = properties.get_node(ctx.ast, i) else {
                    continue;
                };
                convert_property_to_computed(prop_kind, ctx.ast);
            }
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum KeyId {
    Ident(Vec<u8>),
    String(Vec<u8>),
}

fn property_key_of(prop: ObjectPropertyKind, ast: &swc_next_ecma_ast::Ast) -> Option<KeyId> {
    let obj_prop = prop.as_object_property(ast)?;
    let key = obj_prop.key(ast);
    if key.is_identifier_name(ast) {
        let ident = key.as_identifier_name(ast).unwrap();
        let name = ast.get_utf8(ident.name(ast));
        return Some(KeyId::Ident(name.as_bytes().to_vec()));
    }
    if key.is_string_literal(ast) {
        let string = key.as_string_literal(ast).unwrap();
        let value = ast.get_wtf8(string.value(ast));
        return Some(KeyId::String(value.as_bytes().to_vec()));
    }
    None
}

fn convert_property_to_computed(
    prop_kind: ObjectPropertyKind,
    ast: &mut swc_next_ecma_ast::Ast,
) {
    let Some(obj_prop) = prop_kind.as_object_property(ast) else {
        return;
    };
    let key = obj_prop.key(ast);
    let span = key.span(ast);
    let (string_value, string_raw) = if let Some(ident) = key.as_identifier_name(ast) {
        let name = ast.get_utf8(ident.name(ast)).to_string();
        (ast.add_wtf8(name.as_str().into()), ast.add_utf8(&name))
    } else if let Some(string) = key.as_string_literal(ast) {
        (string.value(ast), string.raw(ast))
    } else {
        return;
    };
    let new_key = ast.property_key_string_literal(span, string_value, string_raw);
    obj_prop.set_key(ast, new_key);
    obj_prop.set_computed(ast, true);
}