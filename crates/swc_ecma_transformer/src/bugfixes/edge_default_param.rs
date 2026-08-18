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

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct EdgeDefaultParam;

impl EdgeDefaultParam {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_binding_property(
        &self,
        node: BindingProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        // Check if the property is shorthand and has a default value
        if node.shorthand(ctx.ast) && node.value(ctx.ast).is_assignment_pattern(ctx.ast) {
            // Set shorthand to false: `{a = 1}` → `{a: a = 1}`
            let scope = ctx.semantic().scope_of(node.node_id());
            ctx.transaction(node.node_id(), |tx| {
                let clone = tx.clone_detached(node);
                tx.build_detached(|ast| clone.root.set_shorthand(ast, false));
                tx.replace_from_transformed_clone(node, clone, scope, &[]);
            })
            .expect("edge-default-param mutation must pass preflight");
        }
    }
}
