//! Bugfix: Safari id destructuring collision in function expression.
//!
//! Safari 15 has a bug where destructuring parameters in a named function
//! expression can shadow the function name binding. This transform renames
//! destructured parameters that collide with the function expression name.
//!
//! ```js
//! // Input:
//! (function a([a]) { a });
//! // Output:
//! (function a([_a]) { _a });
//! ```
//!
//! ## Implementation notes
//!
//! This transform requires SyntaxContext/SymbolId tracking to distinguish
//! references to the function name binding from references to the destructured
//! parameter. In swc-next, this requires access to the semantic layer's
//! `rename_symbol()` / `rename_reference()` APIs.
//!
//! **Current status**: Stub. The full implementation needs:
//! 1. Visit each `Function` node (function expressions)
//! 2. Scan destructured params for bindings matching the function name
//! 3. Collect all other identifiers in the function body to avoid collisions
//! 4. Use `semantic.rename_symbol()` to rename the colliding binding
//! 5. All references to the renamed symbol are automatically updated

use swc_next_ecma_ast::Program;
use crate::state::TransformerCtx;

pub(super) struct SafariIdDestructuring;

impl SafariIdDestructuring {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        _program: Program,
        _ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        // TODO: Implement when VisitMut has access to SyntaxContext/semantic renames.
    }
}