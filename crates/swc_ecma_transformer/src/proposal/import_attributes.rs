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
    TypedSubRange, Visit, VisitWith,
};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct ImportAttributes;

impl ImportAttributes {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        let mut collector = ImportAttributesCollector {
            ast: &*ctx.ast,
            targets: Vec::new(),
        };
        program.visit_children_with(&mut collector);
        let targets: Vec<_> = collector
            .targets
            .into_iter()
            .map(|target| {
                let scope = ctx.semantic().scope_of(target.node_id());
                (target, scope)
            })
            .collect();
        if targets.is_empty() {
            return;
        }

        ctx.transaction(program.node_id(), |tx| {
            for (target, scope) in targets {
                match target {
                    ImportAttributesTarget::Import(node) => {
                        let clone = tx.clone_detached(node);
                        tx.build_detached(|ast| {
                            clone
                                .root
                                .set_attributes(ast, TypedSubRange::<ImportAttribute>::empty());
                        });
                        tx.replace_from_transformed_clone(node, clone, scope, &[]);
                    }
                    ImportAttributesTarget::ExportNamed(node) => {
                        let clone = tx.clone_detached(node);
                        tx.build_detached(|ast| {
                            clone
                                .root
                                .set_attributes(ast, TypedSubRange::<ImportAttribute>::empty());
                        });
                        tx.replace_from_transformed_clone(node, clone, scope, &[]);
                    }
                    ImportAttributesTarget::ExportAll(node) => {
                        let clone = tx.clone_detached(node);
                        tx.build_detached(|ast| {
                            clone
                                .root
                                .set_attributes(ast, TypedSubRange::<ImportAttribute>::empty());
                        });
                        tx.replace_from_transformed_clone(node, clone, scope, &[]);
                    }
                }
            }
        })
        .expect("import attributes mutation must pass preflight");
    }
}

#[derive(Clone, Copy)]
enum ImportAttributesTarget {
    Import(ImportDeclaration),
    ExportNamed(ExportNamedDeclaration),
    ExportAll(ExportAllDeclaration),
}

impl ImportAttributesTarget {
    fn node_id(self) -> swc_next_ecma_ast::NodeId {
        match self {
            Self::Import(node) => node.node_id(),
            Self::ExportNamed(node) => node.node_id(),
            Self::ExportAll(node) => node.node_id(),
        }
    }
}

struct ImportAttributesCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    targets: Vec<ImportAttributesTarget>,
}

impl<'ast> Visit<'ast> for ImportAttributesCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_import_declaration(&mut self, node: ImportDeclaration) {
        if !node.attributes(self.ast).is_empty() {
            self.targets.push(ImportAttributesTarget::Import(node));
        }
    }

    fn visit_export_named_declaration(&mut self, node: ExportNamedDeclaration) {
        if !node.attributes(self.ast).is_empty() {
            self.targets.push(ImportAttributesTarget::ExportNamed(node));
        }
    }

    fn visit_export_all_declaration(&mut self, node: ExportAllDeclaration) {
        if !node.attributes(self.ast).is_empty() {
            self.targets.push(ImportAttributesTarget::ExportAll(node));
        }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn strips_attributes_and_retires_their_semantic_nodes_transactionally() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "import data from 'data.json' with { type: 'json' }; export * from 'other.json' with { type: 'json' }; use(data);",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let import = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_import_declaration(&ast)
            .unwrap();
        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 1)
            .unwrap()
            .as_export_all_declaration(&ast)
            .unwrap();
        let removed: Vec<_> = import
            .attributes(&ast)
            .iter()
            .chain(export.attributes(&ast).iter())
            .map(|attribute| ast.get_node_in_sub_range(attribute))
            .collect();
        assert_eq!(removed.len(), 2);
        let mut options = TransformerOptions::default();
        options.proposal.import_attributes = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let transformed_import = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_import_declaration(&ast)
            .unwrap();
        let transformed_export = parse
            .program
            .body(&ast)
            .get_node(&ast, 1)
            .unwrap()
            .as_export_all_declaration(&ast)
            .unwrap();
        assert!(transformed_import.attributes(&ast).is_empty());
        assert!(transformed_export.attributes(&ast).is_empty());
        assert!(
            removed
                .iter()
                .all(|attribute| !semantic.is_node_active(attribute.node_id()))
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
