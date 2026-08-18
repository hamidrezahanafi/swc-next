use swc_next_ecma_ast::{Ast, GetSpan, ObjectProperty, PropertyKey};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct Shorthand;

impl Shorthand {
    pub fn exit_object_property(
        &mut self,
        property: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let (is_shorthand, is_method) = {
            let ast = &*ctx.ast;
            (property.shorthand(ast), property.method(ast))
        };
        if !is_shorthand && !is_method {
            return;
        }

        let is_proto_key = self.is_proto_key(property, ctx.ast);
        let scope = ctx.semantic().scope_of(property.node_id());
        ctx.transaction(property.node_id(), |tx| {
            let clone = tx.clone_detached(property);
            tx.build_detached(|ast| {
                if is_proto_key {
                    let span = clone.root.key(ast).span(ast);
                    let value = ast.add_wtf8("__proto__".into());
                    let key = ast.property_key_string_literal(span, value, ast.empty_utf8());
                    clone.root.set_key(ast, key);
                    clone.root.set_computed(ast, true);
                }
                if is_shorthand {
                    clone.root.set_shorthand(ast, false);
                }
                if is_method {
                    clone.root.set_method(ast, false);
                }
            });
            tx.replace_from_transformed_clone(property, clone, scope, &[]);
        })
        .expect("shorthand mutation must pass preflight");
    }

    fn is_proto_key(&self, property: ObjectProperty, ast: &Ast) -> bool {
        let key: PropertyKey = property.key(ast);
        if let Some(identifier) = key.as_identifier_name(ast) {
            return ast.get_utf8(identifier.name(ast)) == "__proto__";
        }
        if let Some(string) = key.as_string_literal(ast) {
            return ast.get_wtf8(string.value(ast)).as_bytes() == b"__proto__";
        }
        false
    }
}
