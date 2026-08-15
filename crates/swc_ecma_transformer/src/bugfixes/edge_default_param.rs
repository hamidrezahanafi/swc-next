//! Bugfix: edge-default-param — Convert destructured parameters with default
//! values to non-shorthand syntax in arrow functions.
//!
//! This fixes the only arguments-related bug in ES Modules-supporting browsers
//! (Edge 16 & 17).
//!
//! Input:
//! ```js
//! const f = ({ a = 1 }) => a;
//! ```
//!
//! Output:
//! ```js
//! const f = ({ a: a = 1 }) => a;
//! ```

use swc_next_ecma_ast::BindingProperty;

use crate::state::TransformerCtx;

pub(super) struct EdgeDefaultParam;

impl EdgeDefaultParam {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_binding_property(
        &self,
        node: BindingProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        // Check if the property is shorthand and has a default value
        if node.shorthand(ctx.ast)
            && node.value(ctx.ast).is_assignment_pattern(ctx.ast)
        {
            // Set shorthand to false: `{a = 1}` → `{a: a = 1}`
            node.set_shorthand(ctx.ast, false);
        }
    }
}