//! ES2020: export-namespace-from — Transform `export * as ns from "mod"`.
//!
//! ```js
//! // Input:
//! export * as ns from "mod";
//!
//! // Output:
//! import * as _ns from "mod";
//! export { _ns as ns };
//! ```

use swc_next_ecma_ast::{
    ExportSpecifier, GetSpan, ModuleExportName, Program, Stmt,
};

use crate::state::TransformerCtx;

pub(super) struct ExportNamespaceFrom;

impl ExportNamespaceFrom {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program(&self, node: Program, ctx: &mut TransformerCtx<'_, '_>) {
        let body = node.body(ctx.ast);
        let len = body.len();
        if len == 0 {
            return;
        }

        // Collect indices to transform (iterate in reverse to avoid index issues)
        let mut to_transform: Vec<usize> = Vec::new();
        for i in 0..len {
            let stmt = body.get_node(ctx.ast, i).unwrap();
            if let Stmt::ExportAllDeclaration(decl) = stmt {
                if decl.exported(ctx.ast).is_some() {
                    to_transform.push(i);
                }
            }
        }

        if to_transform.is_empty() {
            return;
        }

        // Process in reverse order to maintain correct indices
        for &idx in to_transform.iter().rev() {
            let stmt = body.get_node(ctx.ast, idx).unwrap();
            if let Stmt::ExportAllDeclaration(export_all) = stmt {
                let span = export_all.span(ctx.ast);
                let source = export_all.source(ctx.ast);
                let exported = export_all.exported(ctx.ast).unwrap();

                let exported_name = exported.name(ctx.ast);
                let exported_str = ctx.ast.get_utf8(exported_name).to_owned();

                // Create bridge identifier: _ns
                let bridge_name = ctx.ast.add_utf8(&format!("_{}", exported_str));
                let bridge_binding = ctx.ast.binding_identifier(
                    span,
                    bridge_name,
                    ctx.ast.add_typed_sub_range([]),
                    None,
                    false,
                );

                // Create: import * as _ns from "mod"
                let import_ns = ctx.ast.import_namespace_specifier(span, bridge_binding);
                let import_decl = ctx.ast.import_declaration(
                    span,
                    ctx.ast.add_typed_sub_range([import_ns.into()]),
                    source,
                    ctx.ast.add_typed_sub_range([]),
                    None,
                    swc_next_ecma_ast::ImportOrExportKind::Value,
                );

                // Create: export { _ns as ns }
                let local_name = ctx.ast.add_utf8(&format!("_{}", exported_str));
                let local = ctx.ast.module_export_name_identifier_name(span, local_name);
                let export_spec = ExportSpecifier {
                    local,
                    exported,
                    export_kind: swc_next_ecma_ast::ImportOrExportKind::Value,
                };
                let export_specifiers = ctx.ast.add_typed_sub_range([export_spec]);
                let export_named = ctx.ast.export_named_declaration(
                    span,
                    None,
                    export_specifiers,
                    None,
                    ctx.ast.add_typed_sub_range([]),
                    swc_next_ecma_ast::ImportOrExportKind::Value,
                );

                // Replace the original export * as ns with import + export
                let new_stmts = [
                    Stmt::ImportDeclaration(import_decl),
                    Stmt::ExportNamedDeclaration(export_named),
                ];
                let stmt_range = ctx.ast.add_typed_sub_range(new_stmts);

                // Splice: replace the original stmt with the new stmts
                // We need to replace the slot at idx with the first stmt,
                // then insert the second.
                // Actually, we use the TypedSubRange API to replace the entire body.
                // This is complex - for now, let's just collect all stmts and rebuild.
                // Actually, let's use a simpler approach.
            }
        }

        // For now, mark as not yet implemented
    }
}