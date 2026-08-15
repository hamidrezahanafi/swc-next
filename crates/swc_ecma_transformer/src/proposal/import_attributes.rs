//! Proposal: import-attributes — Strip `with` clauses from import/export.
//!
//! Older runtimes don't support `import json from "file" with { type: "json" }`.
//! This transform removes the attributes clause.
//!
//! ```js
//! // Input:
//! import json from "file" with { type: "json" };
//!
//! // Output:
//! import json from "file";
//! ```

use swc_next_ecma_ast::{
    Ast, ExportAllDeclaration, ExportNamedDeclaration, ImportAttribute, ImportDeclaration,
    TypedSubRange, VisitMut, VisitMutWith,
};

use crate::state::TransformerCtx;

pub(super) struct ImportAttributes;

impl ImportAttributes {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let empty = ctx.ast.add_typed_sub_range(Vec::new());
        let mut visitor = ImportAttributesVisitor {
            ast: ctx.ast,
            empty,
        };
        program.visit_mut_children_with(&mut visitor);
    }
}

struct ImportAttributesVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    empty: TypedSubRange<ImportAttribute>,
}

impl<'ast> VisitMut<'ast> for ImportAttributesVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_import_declaration(
        &mut self,
        node: ImportDeclaration,
    ) -> Option<ImportDeclaration> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        node.set_attributes(self.ast, self.empty);
        Some(node)
    }

    fn visit_mut_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
    ) -> Option<ExportNamedDeclaration> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        node.set_attributes(self.ast, self.empty);
        Some(node)
    }

    fn visit_mut_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
    ) -> Option<ExportAllDeclaration> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);
        node.set_attributes(self.ast, self.empty);
        Some(node)
    }
}