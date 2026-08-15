//! Safari bugfix: destructuring parameter collision in function expressions.
//!
//! When a function expression has a parameter with the same name as the
//! function expression itself, the destructured binding shadows the function
//! name. This transform renames the destructured binding.
//!
//! Input:
//! ```js
//! (function a([a]) { a });
//! ```
//! Output:
//! ```js
//! (function a([_a]) { a });
//! ```
//!
//! TODO: This transform requires a rename pass and is non-trivial. It needs
//! a proper design document.

use swc_next_ecma_ast::{Expr, TraverseAction};

use crate::state::TransformerCtx;

pub(super) struct SafariIdDestructuringCollisionInFunctionExpression;

impl SafariIdDestructuringCollisionInFunctionExpression {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_expression(
        &self,
        _node: Expr,
        _ctx: &mut TransformerCtx<'_, '_>,
    ) -> TraverseAction<Expr> {
        // TODO: Implement proper rename of destructured parameters that collide
        // with the function expression name.
        // This requires:
        // 1. Tracking when we're inside a function expression's parameter list
        // 2. Detecting destructured bindings that match the function name
        // 3. Renaming them with a unique prefix (e.g., `_` prefix)
        // 4. Updating all references to the renamed binding
        TraverseAction::Proceed
    }
}