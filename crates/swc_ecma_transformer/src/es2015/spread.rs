//! ES2015: spread — Transform spread elements in arrays and calls.
//!
//! ```js
//! // Array spread:
//! [...arr] → [].concat(arr)
//! [a, ...arr] → [].concat([a], arr)
//! [a, ...arr, b] → [].concat([a], arr, [b])
//!
//! // Call spread (not yet implemented):
//! fn(...args) → fn.apply(void 0, args)
//! ```

use swc_next_ecma_ast::{
    Argument, Expr, GetSpan, SpreadElement,
};

use crate::state::TransformerCtx;

pub(super) struct Spread;

impl Spread {
    pub fn new() -> Self {
        Self
    }

    /// Transform array expressions with spread elements.
    /// [...arr] → [].concat(arr)
    pub fn exit_array_expression_for_spread(
        &self,
        node: swc_next_ecma_ast::ArrayExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Option<Expr> {
        // Check if any element is a spread
        // SWC stores spread elements as a separate type, so we need to check
        // the element types. Currently we handle the simple case where we
        // detect spreads through the parent's expression handling.
        // For now, this is a placeholder.
        None
    }
}