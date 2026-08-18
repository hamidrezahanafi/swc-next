//! Proposal: export-default-from — `export X from "module"`.
//!
//! This transform converts `export { default as X } from "module"` (using the
//! `ExportDefaultSpecifier` form) into `export { default as X } from "module"`
//! (using `ExportNamedSpecifier`).
//!
//! In swc-next, the AST uses `ExportSpecifier` with `ModuleExportName` for both
//! `local` and `exported`, and there is no separate `ExportDefaultSpecifier`
//! type. The parser does not currently support the `export X from "module"`
//! syntax (without braces), so this transform is a no-op stub.
//!
//! When parser support is added (e.g., parsing `export X from "mod"` as
//! `ExportSpecifier { local: "default", exported: "X" }`), this module can be
//! updated to convert any specifier with `local == "default"` into a named
//! specifier if needed.

use crate::state::TransformerCtx;
use swc_next_ecma_ast::Program;

pub(super) struct ExportDefaultFrom;

impl ExportDefaultFrom {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        _program: Program,
        _ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        // No-op: parser doesn't support the source syntax yet.
    }
}
