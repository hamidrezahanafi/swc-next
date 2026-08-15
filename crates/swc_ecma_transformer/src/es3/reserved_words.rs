//! ES3: reserved-words — Rename identifiers that are ES3 reserved words.
//!
//! ```js
//! var abstract = 1;  → var _abstract = 1;
//! ```

use crate::state::TransformerCtx;

pub(super) struct ReservedWords;

impl ReservedWords {
    pub fn exit_assignment_expression(
        &mut self,
        node: swc_next_ecma_ast::AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
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
                ident.set_name(ctx.ast, new_name);
            }
        }
    }
}

fn is_es3_reserved_word(s: &str) -> bool {
    matches!(
        s,
        "abstract" | "boolean" | "byte" | "char" | "double" | "final" | "float"
            | "goto" | "int" | "long" | "native" | "short" | "synchronized"
            | "throws" | "transient" | "volatile"
    )
}