//! ES2015: object-super — Transform `super` in object literal methods.
//!
//! Input:
//! ```js
//! let obj = {
//!     foo() {
//!         super.bar();
//!     }
//! }
//! ```
//!
//! Output:
//! ```js
//! let obj = (() => {
//!     var _obj = {
//!         foo() {
//!             _get(_get_prototype_of(_obj), "bar", this).call(this);
//!         }
//!     };
//!     return _obj;
//! })();
//! ```
//!
//! See docs/design/object-super.md for full design.

use swc_next_ecma_ast::ObjectExpression;

use crate::state::TransformerCtx;

pub(super) struct ObjectSuper;

impl ObjectSuper {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_object_expression(
        &mut self,
        _node: ObjectExpression,
        _ctx: &mut TransformerCtx<'_, '_>,
    ) {
        // TODO: Full implementation.
        // Requires walking method bodies to detect and replace `super.foo` with
        // `_get(_get_prototype_of(_obj), "foo", this)` and wrapping the object
        // literal in an IIFE with a `_obj` variable.
        // See docs/design/object-super.md for the complete design.
    }
}