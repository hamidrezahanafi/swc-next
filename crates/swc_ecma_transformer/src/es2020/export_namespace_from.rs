//! ES2020: export-namespace-from — Transform `export * as ns from "mod"`.
//!
//! ```js
//! export * as ns from "mod";
//! // becomes
//! import * as _ns from "mod";
//! export { _ns as ns };
//! ```

use std::collections::HashSet;

use swc_next_ecma_ast::{GetSpan, ImportOrExportKind, Program, Stmt, TypedSubRange};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace, SymbolFlags};

use crate::{
    mutation::StatementListOwner,
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct ExportNamespaceFrom;

impl ExportNamespaceFrom {
    pub fn new() -> Self {
        Self
    }

    pub fn transform_program(&self, program: Program, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let statements: Vec<_> = program
            .body(ctx.ast)
            .iter()
            .map(|statement| ctx.ast.get_node_in_sub_range(statement))
            .collect();
        let targets: HashSet<_> = statements
            .iter()
            .filter_map(|statement| {
                let export = statement.as_export_all_declaration(ctx.ast)?;
                export
                    .exported(ctx.ast)?
                    .is_identifier_name(ctx.ast)
                    .then_some(statement.node_id())
            })
            .collect();
        if targets.is_empty() {
            return;
        }

        let module_scope = ctx.semantic().scope_of(program.node_id());
        ctx.transaction(program.node_id(), |tx| {
            let mut output = Vec::with_capacity(statements.len() + targets.len());
            for statement in statements {
                if !targets.contains(&statement.node_id()) {
                    output.push(statement);
                    continue;
                }

                let export = statement.as_export_all_declaration(tx.ast()).unwrap();
                let span = export.span(tx.ast());
                let source = export.source(tx.ast());
                let source_span = source.span(tx.ast());
                let source_value = source.value(tx.ast());
                let source_raw = source.raw(tx.ast());
                let exported = export.exported(tx.ast()).unwrap();
                let exported = exported.as_identifier_name(tx.ast()).unwrap();
                let exported_name = exported.name(tx.ast());
                let exported_span = exported.span(tx.ast());
                let bridge_base = format!("_{}", tx.ast().get_utf8(exported_name));
                let bridge = tx.declare_private(module_scope, &bridge_base, SymbolFlags::IMPORT);

                let (import, export, specifier, local, exported) = tx.build_detached(|ast| {
                    let source = ast.string_literal(source_span, source_value, source_raw);
                    let import_namespace = ast
                        .import_declaration_specifier_import_namespace_specifier(
                            span,
                            bridge.declaration,
                        );
                    let import_specifiers = ast.add_typed_sub_range([import_namespace]);
                    let import = ast.import_declaration(
                        span,
                        import_specifiers,
                        source,
                        TypedSubRange::empty(),
                        None,
                        ImportOrExportKind::Value,
                    );

                    let local = ast.module_export_name_identifier_name(span, bridge.text);
                    ast.set_syntax_context(local.node_id(), bridge.context);
                    let exported =
                        ast.module_export_name_identifier_name(exported_span, exported_name);
                    let specifier =
                        ast.export_specifier(span, local, exported, ImportOrExportKind::Value);
                    let export_specifiers = ast.add_typed_sub_range([specifier]);
                    let export = ast.export_named_declaration(
                        span,
                        None,
                        export_specifiers,
                        None,
                        TypedSubRange::empty(),
                        ImportOrExportKind::Value,
                    );
                    (import, export, specifier, local, exported)
                });

                tx.adopt_generated_subtree(import, Some(program.node_id()), module_scope);
                tx.adopt_generated_subtree(export, Some(program.node_id()), module_scope);
                let reference = tx.reference(
                    module_scope,
                    local.node_id(),
                    tx.ast().canonicalize_utf8(bridge.text),
                    ReferenceFlags::default().with_space(ReferenceSpace::Any),
                    Some(bridge.symbol),
                );
                tx.replace_namespace_reexport(
                    statement.node_id(),
                    export.node_id(),
                    specifier.node_id(),
                    local.node_id(),
                    exported.node_id(),
                    bridge.symbol,
                    reference,
                );
                tx.retire_subtree(statement.node_id());
                output.push(Stmt::ImportDeclaration(import));
                output.push(Stmt::ExportNamedDeclaration(export));
            }
            tx.replace_statements(StatementListOwner::Program(program), output);
        })
        .expect("export-namespace-from mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, SymbolFlags, analyze,
    };

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn namespace_reexport_updates_binding_reference_and_module_record() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "const _ns = 0; export * as ns from 'mod'; use(_ns);",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2020.export_namespace_from = true;
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

        let import = parse
            .program
            .body(&ast)
            .get_node(&ast, 1)
            .unwrap()
            .as_import_declaration(&ast)
            .unwrap();
        let namespace = import
            .specifiers(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_import_namespace_specifier(&ast)
            .unwrap();
        let binding = namespace.local(&ast);
        assert_eq!(ast.get_utf8(binding.name(&ast)), "_ns1");
        let declaration = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(declaration).symbol;
        assert!(semantic.symbol(symbol).flags.contains(SymbolFlags::IMPORT));

        let export = parse
            .program
            .body(&ast)
            .get_node(&ast, 2)
            .unwrap()
            .as_export_named_declaration(&ast)
            .unwrap();
        let specifier = export.specifiers(&ast).get_node(&ast, 0).unwrap();
        let local = specifier.local(&ast);
        assert_eq!(
            ast.get_utf8(local.as_identifier_name(&ast).unwrap().name(&ast)),
            "_ns1"
        );
        let reference = semantic.reference_of(local.node_id()).unwrap();
        assert_eq!(semantic.reference(reference).symbol, Some(symbol));
        assert_eq!(
            semantic.reference(reference).flags.space,
            ReferenceSpace::Any
        );

        let entry = semantic.module_record().exports()[0];
        assert_eq!(entry.origin, ExportOrigin::LocalSpecifier);
        assert_eq!(entry.statement, export.node_id());
        assert!(matches!(
            entry.target,
            ExportTarget::Local {
                symbol: target,
                reference: Some(target_reference),
                ..
            } if target == symbol && target_reference == reference
        ));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
