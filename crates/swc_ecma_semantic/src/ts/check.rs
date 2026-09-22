use swc_next_ecma_ast::{
    Ast, ExportDefaultDeclaration, ExportDefaultDeclarationKindData, NodeId, NodeKindData,
    PropertyKeyData, TsExportAssignment, TsModuleNameData,
};

use crate::binder::{PreparedDeclaration, TsImportAliasIssue, TsNamespaceExportAliasIssue};
use crate::builder::ReportOptions;
use crate::check as js_check;
use crate::ecmascript::module_export_name;
use crate::ts::builder::SemanticBuilder;

pub fn check_export_default_declaration(
    builder: &mut SemanticBuilder<'_, '_>,
    declaration: ExportDefaultDeclaration,
) {
    let export = declaration.declaration(builder.ast());
    if let ExportDefaultDeclarationKindData::Expr(expression) =
        builder.ast().export_default_declaration_kind_data(export)
    {
        check_ts_ambient_export_assignment_expression(builder, expression.node_id());
    }
}

pub fn check_prepared_redeclaration(
    builder: &mut SemanticBuilder<'_, '_>,
    prepared: PreparedDeclaration,
) {
    let name = builder.ast().get_wtf8(prepared.name()).as_bytes();
    js_check::check_redeclaration(&mut builder.base, prepared.node(), name, prepared);
}

pub fn check_ts_alias_issues(
    builder: &mut SemanticBuilder<'_, '_>,
    export_alias_issues: impl IntoIterator<Item = TsNamespaceExportAliasIssue>,
    import_alias_issues: impl IntoIterator<Item = TsImportAliasIssue>,
) {
    for issue in export_alias_issues {
        match issue {
            TsNamespaceExportAliasIssue::Missing { name, node } => {
                check_ts_namespace_export_alias_missing(builder, name, node);
            }
            TsNamespaceExportAliasIssue::NonLocal { name, node } => {
                check_ts_namespace_export_alias_non_local(builder, name, node);
            }
            TsNamespaceExportAliasIssue::Duplicate { node, previous } => {
                check_ts_namespace_export_alias_duplicate(builder, node, previous);
            }
            TsNamespaceExportAliasIssue::Conflict { node, previous } => {
                check_ts_namespace_export_alias_conflict(builder, node, previous);
            }
        }
    }
    for issue in import_alias_issues {
        match issue {
            TsImportAliasIssue::Conflict(issue) => {
                check_ts_import_alias_conflict(builder, issue);
            }
            TsImportAliasIssue::TypeOnlyExport { node, export } => {
                check_ts_import_alias_type_only_export(builder, node, export);
            }
        }
    }
}

pub fn check_ts_namespace_export_alias_missing(
    builder: &mut SemanticBuilder<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    let name = builder.base.ast.get_utf8(name).as_bytes();
    builder.base.report_unresolved_export(name, node);
}

pub fn check_ts_namespace_export_alias_non_local(
    builder: &mut SemanticBuilder<'_, '_>,
    name: swc_next_ecma_ast::AstUtf8,
    node: NodeId,
) {
    let name = builder.base.ast.get_utf8(name).as_bytes();
    builder.base.report_non_local_export(name, node);
}

pub fn check_ts_namespace_export_alias_duplicate(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    check_ts_namespace_export_alias_conflict_with_kind(builder, node, previous, true);
}

pub fn check_ts_namespace_export_alias_conflict(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
) {
    check_ts_namespace_export_alias_conflict_with_kind(builder, node, previous, false);
}

pub fn check_ts_namespace_export_alias_conflict_with_kind(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    previous: NodeId,
    duplicate: bool,
) {
    let NodeKindData::ExportSpecifier(specifier) = builder.ast().kind_data(node) else {
        unreachable!("namespace export aliases are export specifiers")
    };
    let exported = specifier.exported(builder.ast()).node_id();
    let name = module_export_name(builder.ast(), exported);
    builder.report_ts_namespace_export_alias_conflict(name, node, previous, duplicate);
}

pub fn check_ts_import_alias_conflict(
    builder: &mut SemanticBuilder<'_, '_>,
    issue: crate::binder::TsImportAliasConflictIssue,
) {
    let name = builder.ast().get_wtf8(issue.name).as_bytes();
    builder.report_redeclaration(name, issue.node, issue.previous, issue.previous_flags);
}

pub fn check_ts_import_alias_type_only_export(
    builder: &mut SemanticBuilder<'_, '_>,
    node: NodeId,
    export: NodeId,
) {
    builder.report_ts_import_alias_type_only_export(node, export);
}

pub fn check_ts_export_assignment(
    builder: &mut SemanticBuilder<'_, '_>,
    assignment: TsExportAssignment,
) {
    let expression = assignment.expression(builder.ast());
    check_ts_ambient_export_assignment_expression(builder, expression.node_id());
}

pub fn check_ts_ambient_export_assignment_expression(
    builder: &mut SemanticBuilder<'_, '_>,
    expression: NodeId,
) {
    if !builder.symbols.in_ambient_context()
        || is_ts_entity_name_expression(builder.ast(), expression)
        || !is_export_assignment_container(builder)
    {
        return;
    }
    builder.report(
        builder.ast().span(expression),
        "The expression of an export assignment must be an identifier or qualified name in an ambient context",
        ReportOptions::default(),
    );
}

pub fn is_ts_entity_name_expression(tree: &Ast<'_>, expression: NodeId) -> bool {
    let mut current = expression;
    loop {
        current = crate::ecmascript::unwrap_parentheses(tree, current);
        match tree.kind_data(current) {
            NodeKindData::IdentifierReference(_) => return true,
            NodeKindData::ChainExpression(chain) => {
                current = chain.expression(tree).node_id();
            }
            NodeKindData::MemberExpression(member) if !member.computed(tree) => {
                if !matches!(
                    tree.property_key_data(member.property(tree)),
                    PropertyKeyData::IdentifierName(_)
                ) {
                    return false;
                }
                current = member.object(tree).node_id();
            }
            _ => return false,
        }
    }
}

pub fn is_export_assignment_container(builder: &SemanticBuilder<'_, '_>) -> bool {
    let Some(parent) = builder.parent() else {
        return false;
    };
    match builder.ast().kind_data(parent) {
        NodeKindData::Program(_) => true,
        NodeKindData::TsModuleBlock(_) => {
            let Some(container) = builder.ancestors().nth(2) else {
                return false;
            };
            match builder.ast().kind_data(container) {
                NodeKindData::TsGlobalDeclaration(_) => true,
                NodeKindData::TsModuleDeclaration(declaration) => matches!(
                    builder
                        .ast()
                        .ts_module_name_data(declaration.id(builder.ast())),
                    TsModuleNameData::StringLiteral(_)
                ),
                _ => false,
            }
        }
        _ => false,
    }
}
