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
    GetSpan, Program, Stmt,
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

        // Collect indices to transform
        let mut to_transform: Vec<usize> = Vec::new();
        for i in 0..len {
            let stmt = body.get_node(ctx.ast, i).unwrap();
            if stmt.is_export_all_declaration(ctx.ast) {
                let export_all = stmt.as_export_all_declaration(ctx.ast).unwrap();
                if export_all.exported(ctx.ast).is_some() {
                    to_transform.push(i);
                }
            }
        }

        if to_transform.is_empty() {
            return;
        }

        // Collect all stmts and rebuild, replacing export * as ns with import + export
        let mut new_stmts: Vec<Stmt> = Vec::new();
        let mut transform_idx = 0;

        for i in 0..len {
            if transform_idx < to_transform.len() && i == to_transform[transform_idx] {
                let stmt = body.get_node(ctx.ast, i).unwrap();
                let export_all = stmt.as_export_all_declaration(ctx.ast).unwrap();
                let span = export_all.span(ctx.ast);
                let source = export_all.source(ctx.ast);
                let exported = export_all.exported(ctx.ast).unwrap();
                let exported_str = if exported.is_identifier_name(ctx.ast) {
                    let id = exported.as_identifier_name(ctx.ast).unwrap();
                    ctx.ast.get_utf8(id.name(ctx.ast)).to_owned()
                } else {
                    continue;
                };

                // Create bridge identifier: _ns
                let empty_decorators = ctx.ast.add_typed_sub_range(Vec::new());
                let bridge_name_str = format!("_{}", exported_str);
                let bridge_name = ctx.ast.add_utf8(&bridge_name_str);
                let bridge_binding = ctx.ast.binding_identifier(
                    span,
                    bridge_name,
                    empty_decorators,
                    None,
                    false,
                );

                // Create: import * as _ns from "mod"
                let empty_import_assertions = ctx.ast.add_typed_sub_range(Vec::new());
                let import_ns = ctx.ast.import_declaration_specifier_import_namespace_specifier(
                    span,
                    bridge_binding,
                );
                let import_specifiers = ctx.ast.add_typed_sub_range([import_ns]);
                let import_decl = ctx.ast.import_declaration(
                    span,
                    import_specifiers,
                    source,
                    empty_import_assertions,
                    None,
                    swc_next_ecma_ast::ImportOrExportKind::Value,
                );
                new_stmts.push(Stmt::ImportDeclaration(import_decl));

                // Create: export { _ns as ns }
                let local_name = ctx.ast.add_utf8(&bridge_name_str);
                let local = ctx.ast.module_export_name_identifier_name(span, local_name);
                let export_spec = ctx.ast.export_specifier(
                    span, local, exported,
                    swc_next_ecma_ast::ImportOrExportKind::Value,
                );
                let export_specifiers = ctx.ast.add_typed_sub_range([export_spec]);
                let empty_export_assertions = ctx.ast.add_typed_sub_range(Vec::new());
                let export_named = ctx.ast.export_named_declaration(
                    span,
                    None,
                    export_specifiers,
                    None,
                    empty_export_assertions,
                    swc_next_ecma_ast::ImportOrExportKind::Value,
                );
                new_stmts.push(Stmt::ExportNamedDeclaration(export_named));

                transform_idx += 1;
            } else {
                let stmt = body.get_node(ctx.ast, i).unwrap();
                new_stmts.push(stmt);
            }
        }

        let new_body = ctx.ast.add_typed_sub_range(new_stmts);
        node.set_body(ctx.ast, new_body);
    }
}