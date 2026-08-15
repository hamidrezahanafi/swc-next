//! Safari 10/11 bugfix: cached tagged template literals.
//!
//! Safari 10/11 uses the same cached Strings value when the string parts are
//! the same, which violates the spec. This transform wraps each tagged template
//! with an inline cache.
//!
//! Input:
//! ```js
//! tag`hello ${name}`
//! ```
//! Output:
//! ```js
//! var _t, _identity = t => t;
//! tag(_t || (_t = _identity`hello ${0}`), name)
//! ```
//!
//! TODO: This transform requires hoisting declarations to the program level and
//! creating helper functions. It needs a proper design document.

use swc_next_ecma_ast::{Expr, ExprData, TraverseAction};

use crate::state::TransformerCtx;

pub(super) struct TemplateLiteralCaching;

impl TemplateLiteralCaching {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_expression(
        &self,
        _node: Expr,
        _ctx: &mut TransformerCtx<'_, '_>,
    ) -> TraverseAction<Expr> {
        // TODO: Implement proper inline caching for tagged template literals.
        // This requires:
        // 1. Creating an identity function helper (_identity = t => t) at program level
        // 2. For each tagged template, creating a cache variable (_tN)
        // 3. Building: _tN || (_tN = _identity`template${0}${0}...`)
        // 4. Hoisting variable declarations to the program
        TraverseAction::Proceed
    }
}