use std::collections::HashSet;

use swc_next_ecma_ast::{GetSpan, ObjectExpression, ObjectProperty, ObjectPropertyKind};

use crate::state::{TransformerCtx, TransformerCtxExt};

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
        ctx: &mut TransformerCtx<'_, '_, '_>,
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
            if let Some(ref key_id) = key_id
                && !seen.insert(key_id.clone())
            {
                has_duplicate = true;
            }
            keys.push(key_id);
        }

        if !has_duplicate {
            return;
        }

        // Phase 2: collect duplicates before opening the atomic transaction.
        seen.clear();
        let mut duplicates = Vec::new();
        for (i, key_id) in keys.iter().enumerate().take(len) {
            let Some(key_id) = key_id else {
                continue;
            };
            if !seen.insert(key_id.clone()) {
                // This is a duplicate — convert to computed
                let Some(prop_kind) = properties.get_node(ctx.ast, i) else {
                    continue;
                };
                let Some(property) = prop_kind.as_object_property(ctx.ast) else {
                    continue;
                };
                duplicates.push((property, ctx.semantic().scope_of(property.node_id())));
            }
        }

        ctx.transaction(object.node_id(), |tx| {
            for (property, scope) in duplicates {
                let clone = tx.clone_detached(property);
                tx.build_detached(|ast| convert_property_to_computed(clone.root, ast));
                tx.replace_from_transformed_clone(property, clone, scope, &[]);
            }
        })
        .expect("duplicate-keys mutation must pass preflight");
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

fn convert_property_to_computed(obj_prop: ObjectProperty, ast: &mut swc_next_ecma_ast::Ast) {
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
