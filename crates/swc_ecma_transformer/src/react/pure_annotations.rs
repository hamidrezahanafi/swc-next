//! React: pure-annotations — Add `/*#__PURE__*/` annotations to pure React calls.
//!
//! Adds `/*#__PURE__*/` annotations to calls to known pure React methods
//! (createElement, cloneElement, createContext, memo, forwardRef, lazy, etc.)
//! so that minifiers can safely remove them during dead code elimination.
//!
//! ## Implementation notes
//!
//! This transform requires adding synthetic `/* @__PURE__ */` block comments
//! to the AST. In swc-next, the `Comment` struct stores a `span` referencing
//! source text, so adding a synthetic comment requires:
//!
//! 1. Registering the `/* @__PURE__ */` text in the source
//! 2. Building a `Comment` with `CommentKind::Block` and the correct span
//! 3. Pushing it to `Ast.attached_comments`
//!
//! The codegen (`printer.rs`) already handles `@__PURE__` / `#__PURE__` in
//! block comments, so the output format is correct.
//!
//! **Current status**: Stub. The full implementation needs:
//! 1. Scan imports to track which local names refer to React methods
//! 2. In `visit_mut_call_expression`, check if the callee is a known pure call
//! 3. Add a `/* @__PURE__ */` comment attached to the call expression

use crate::state::TransformerCtx;
use swc_next_ecma_ast::Program;

pub(super) struct PureAnnotations;

impl PureAnnotations {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        _program: Program,
        _ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        // TODO: Implement when synthetic comment attachment is available.
    }
}
