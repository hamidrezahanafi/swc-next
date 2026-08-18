//! ES3: reserved-words — Rename identifiers that are ES3 reserved words.
//!
//! ```js
//! var abstract = 1;  → var _abstract = 1;
//! ```

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct ReservedWords;

impl ReservedWords {
    pub fn exit_assignment_expression(
        &mut self,
        node: swc_next_ecma_ast::AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let left = node.left(ctx.ast);
        if !left.is_simple_assignment_target(ctx.ast) {
            return;
        }
        let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
        if let Some(ident) = sat.as_identifier_reference(ctx.ast) {
            let name = ctx.ast.get_utf8(ident.name(ctx.ast)).to_owned();
            if is_es3_reserved_word(&name) {
                let new_name = ctx.ast.add_utf8(&format!("_{}", name));
                let semantic_name = ctx.ast.canonicalize_utf8(new_name);
                let scope = ctx.semantic().scope_of(ident.node_id());
                let reference = ctx
                    .semantic()
                    .reference_of(ident.node_id())
                    .expect("assignment identifier has a semantic reference");
                let reference_data = ctx.semantic().reference(reference);
                let symbol = ctx.semantic().lookup(
                    reference_data.scope,
                    semantic_name,
                    reference_data.flags.space,
                );
                ctx.transaction(ident.node_id(), |tx| {
                    let clone = tx.clone_detached(ident);
                    tx.build_detached(|ast| clone.root.set_name(ast, new_name));
                    tx.replace_from_transformed_clone(ident, clone, scope, &[]);
                    tx.rename_reference(reference, semantic_name);
                    tx.rebind_reference(reference, symbol);
                })
                .expect("reserved-words mutation must pass preflight");
            }
        }
    }
}

fn is_es3_reserved_word(s: &str) -> bool {
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
