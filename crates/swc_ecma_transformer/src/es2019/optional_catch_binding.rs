//! ES2019: optional-catch-binding — Add a binding to catch clauses without one.
//!
//! ```js
//! try {} catch {} → try {} catch (_unused) {}
//! ```

use swc_next_ecma_ast::{CatchClause, TypedSubRange, DUMMY_SP};

use crate::state::TransformerCtx;

pub(super) struct OptionalCatchBinding;

impl OptionalCatchBinding {
    pub fn enter_catch_clause(
        &mut self,
        node: CatchClause,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if node.param(ctx.ast).is_some() {
            return;
        }

        let name = ctx.ast.add_utf8("_unused");
        let param = ctx.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name,
            TypedSubRange::empty(),
            None,
            false,
        );
        node.set_param(ctx.ast, Some(param));
    }
}