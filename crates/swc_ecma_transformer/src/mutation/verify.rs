use std::collections::HashSet;

use swc_next_ecma_ast::{Ast, NodeId, Program, ScopeId, SymbolId, Visit, VisitWith};
use swc_next_ecma_semantic::{DeclarationId, ReferenceId, Semantic};
#[cfg(test)]
use swc_next_ecma_semantic::{SemanticFeatureFlags, SubtreeEffect};

use super::{MutationError, MutationTransaction};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InvariantViolation {
    EpochMismatch {
        ast: u64,
        semantic: u64,
    },
    ReachableNodeInactive(NodeId),
    ActiveNodeUnreachable(NodeId),
    ParentMismatch {
        node: NodeId,
        expected: Option<NodeId>,
        actual: Option<NodeId>,
    },
    InactiveNodeScope {
        node: NodeId,
        scope: ScopeId,
    },
    DeclarationBacklink {
        declaration: DeclarationId,
        node: NodeId,
    },
    ReferenceBacklink {
        reference: ReferenceId,
        node: NodeId,
    },
    InactiveSymbolTarget {
        reference: ReferenceId,
        symbol: SymbolId,
    },
    MissingSymbolUse {
        reference: ReferenceId,
        symbol: SymbolId,
    },
    InactiveScopeOwner {
        scope: ScopeId,
        node: NodeId,
    },
    InactiveScopeParent {
        scope: ScopeId,
        parent: ScopeId,
    },
    InactiveSymbolScope {
        symbol: SymbolId,
        scope: ScopeId,
    },
}

pub fn verify_program<'ast>(
    program: Program,
    ast: &Ast<'ast>,
    semantic: &Semantic<'ast>,
) -> Result<(), Vec<InvariantViolation>> {
    let mut verifier = TreeVerifier {
        ast,
        semantic,
        stack: Vec::new(),
        reachable: HashSet::new(),
        violations: Vec::new(),
    };
    program.visit_with(&mut verifier);

    if ast.mutation_epoch() != semantic.mutation_epoch() {
        verifier.violations.push(InvariantViolation::EpochMismatch {
            ast: ast.mutation_epoch(),
            semantic: semantic.mutation_epoch(),
        });
    }
    for node in semantic.iter_nodes() {
        if !verifier.reachable.contains(&node) {
            verifier
                .violations
                .push(InvariantViolation::ActiveNodeUnreachable(node));
        }
    }
    for entry in semantic.iter_declarations() {
        let node = entry.declaration.node;
        if !semantic.is_node_active(node) || semantic.declaration_of(node) != Some(entry.id) {
            verifier
                .violations
                .push(InvariantViolation::DeclarationBacklink {
                    declaration: entry.id,
                    node,
                });
        }
    }
    for entry in semantic.iter_references() {
        let reference = entry.reference;
        if !semantic.is_node_active(reference.node)
            || semantic.reference_of(reference.node) != Some(entry.id)
        {
            verifier
                .violations
                .push(InvariantViolation::ReferenceBacklink {
                    reference: entry.id,
                    node: reference.node,
                });
        }
        if let Some(symbol) = reference.symbol {
            if !semantic.is_symbol_active(symbol) {
                verifier
                    .violations
                    .push(InvariantViolation::InactiveSymbolTarget {
                        reference: entry.id,
                        symbol,
                    });
            } else if !semantic.uses(symbol).contains(&entry.id) {
                verifier
                    .violations
                    .push(InvariantViolation::MissingSymbolUse {
                        reference: entry.id,
                        symbol,
                    });
            }
        }
    }
    for entry in semantic.iter_scopes() {
        if !semantic.is_node_active(entry.scope.node) {
            verifier
                .violations
                .push(InvariantViolation::InactiveScopeOwner {
                    scope: entry.id,
                    node: entry.scope.node,
                });
        }
        if let Some(parent) = entry.scope.parent
            && !semantic.is_scope_active(parent)
        {
            verifier
                .violations
                .push(InvariantViolation::InactiveScopeParent {
                    scope: entry.id,
                    parent,
                });
        }
    }
    for entry in semantic.iter_symbols() {
        if !semantic.is_scope_active(entry.symbol.scope) {
            verifier
                .violations
                .push(InvariantViolation::InactiveSymbolScope {
                    symbol: entry.id,
                    scope: entry.symbol.scope,
                });
        }
    }

    if verifier.violations.is_empty() {
        Ok(())
    } else {
        Err(verifier.violations)
    }
}

struct TreeVerifier<'a, 'ast> {
    ast: &'a Ast<'ast>,
    semantic: &'a Semantic<'ast>,
    stack: Vec<NodeId>,
    reachable: HashSet<NodeId>,
    violations: Vec<InvariantViolation>,
}

impl<'ast> Visit<'ast> for TreeVerifier<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeId) {
        let expected_parent = self.stack.last().copied();
        self.reachable.insert(node);
        if !self.semantic.is_node_active(node) {
            self.violations
                .push(InvariantViolation::ReachableNodeInactive(node));
        } else {
            let actual_parent = self.semantic.parent_of(node);
            if actual_parent != expected_parent {
                self.violations.push(InvariantViolation::ParentMismatch {
                    node,
                    expected: expected_parent,
                    actual: actual_parent,
                });
            }
            let scope = self.semantic.scope_of(node);
            if !self.semantic.is_scope_active(scope) {
                self.violations
                    .push(InvariantViolation::InactiveNodeScope { node, scope });
            }
        }
        self.stack.push(node);
    }

    fn leave_node(&mut self, node: NodeId) {
        assert_eq!(self.stack.pop(), Some(node));
    }
}

impl MutationTransaction<'_, '_> {
    /// Checks invariants that must hold at every pass boundary.
    pub fn verify_epochs(&self) -> Result<(), MutationError> {
        let ast = self.ast.mutation_epoch();
        let semantic = self.semantic.mutation_epoch();
        if ast == semantic {
            Ok(())
        } else {
            Err(MutationError::EpochMismatch { ast, semantic })
        }
    }
}

#[cfg(test)]
pub fn assert_semantic_matches_reanalysis<'ast>(
    program: Program,
    ast: &Ast<'ast>,
    incremental: &Semantic<'ast>,
) {
    use swc_next_ecma_semantic::{
        AnalyzeOptions, analyze,
        scope::{ScopeFlags, ScopeKind},
    };

    let rebuilt = analyze(
        program,
        ast,
        AnalyzeOptions {
            features: incremental.features(),
            ..AnalyzeOptions::default()
        },
    )
    .semantic;

    let incremental_nodes: HashSet<_> = incremental.iter_nodes().collect();
    let rebuilt_nodes: HashSet<_> = rebuilt.iter_nodes().collect();
    assert_eq!(incremental_nodes, rebuilt_nodes, "active node set differs");
    for node in incremental_nodes {
        assert_eq!(
            incremental.parent_of(node),
            rebuilt.parent_of(node),
            "parent differs for {node:?}"
        );
        assert_eq!(
            scope_path(incremental, incremental.scope_of(node)),
            scope_path(&rebuilt, rebuilt.scope_of(node)),
            "scope path differs for {node:?} ({:?})",
            ast.node_kind(node)
        );

        let incremental_declaration = incremental.declaration_of(node).map(|id| {
            let declaration = incremental.declaration(id);
            let symbol = incremental.symbol(declaration.symbol);
            (
                declaration.flags,
                symbol.name,
                symbol.flags,
                scope_path(incremental, symbol.scope),
                declaration_nodes(incremental, declaration.symbol),
            )
        });
        let rebuilt_declaration = rebuilt.declaration_of(node).map(|id| {
            let declaration = rebuilt.declaration(id);
            let symbol = rebuilt.symbol(declaration.symbol);
            (
                declaration.flags,
                symbol.name,
                symbol.flags,
                scope_path(&rebuilt, symbol.scope),
                declaration_nodes(&rebuilt, declaration.symbol),
            )
        });
        assert_eq!(
            incremental_declaration, rebuilt_declaration,
            "declaration differs for {node:?}"
        );

        let incremental_reference = incremental.reference_of(node).map(|id| {
            let reference = incremental.reference(id);
            (
                reference.name,
                reference.flags,
                scope_path(incremental, reference.scope),
                reference
                    .symbol
                    .map(|symbol| declaration_nodes(incremental, symbol)),
            )
        });
        let rebuilt_reference = rebuilt.reference_of(node).map(|id| {
            let reference = rebuilt.reference(id);
            (
                reference.name,
                reference.flags,
                scope_path(&rebuilt, reference.scope),
                reference
                    .symbol
                    .map(|symbol| declaration_nodes(&rebuilt, symbol)),
            )
        });
        assert_eq!(
            incremental_reference, rebuilt_reference,
            "reference differs for {node:?}"
        );
    }

    if let (Ok(incremental_classes), Ok(rebuilt_classes)) = (
        incremental.classes_post_order(),
        rebuilt.classes_post_order(),
    ) {
        assert_eq!(
            incremental_classes
                .iter()
                .map(|fact| fact.class.node_id())
                .collect::<Vec<_>>(),
            rebuilt_classes
                .iter()
                .map(|fact| fact.class.node_id())
                .collect::<Vec<_>>(),
            "class post-order differs"
        );
    }

    if let (Ok(incremental_generators), Ok(rebuilt_generators)) = (
        incremental.generator_functions_post_order(),
        rebuilt.generator_functions_post_order(),
    ) {
        assert_eq!(
            incremental_generators
                .iter()
                .map(|function| function.node_id())
                .collect::<Vec<_>>(),
            rebuilt_generators
                .iter()
                .map(|function| function.node_id())
                .collect::<Vec<_>>(),
            "generator post-order differs"
        );
    }

    if let (Ok(incremental_async), Ok(rebuilt_async)) = (
        incremental.async_functions_post_order(),
        rebuilt.async_functions_post_order(),
    ) {
        assert_eq!(incremental_async, rebuilt_async, "async post-order differs");
    }

    if incremental
        .features()
        .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE)
    {
        for node in incremental.iter_nodes() {
            let incremental_captures = incremental.function_captures(node).unwrap();
            let rebuilt_captures = rebuilt.function_captures(node).unwrap();
            assert_eq!(
                incremental_captures
                    .arguments
                    .iter()
                    .map(|reference| incremental.reference(*reference).node)
                    .collect::<Vec<_>>(),
                rebuilt_captures
                    .arguments
                    .iter()
                    .map(|reference| rebuilt.reference(*reference).node)
                    .collect::<Vec<_>>(),
                "arguments captures differ for {node:?}"
            );
            assert_eq!(
                incremental_captures.new_targets, rebuilt_captures.new_targets,
                "new.target captures differ for {node:?}"
            );
        }
    }

    for (feature, effect) in [
        (SemanticFeatureFlags::YIELD, SubtreeEffect::Yield),
        (SemanticFeatureFlags::AWAIT, SubtreeEffect::Await),
    ] {
        if incremental.features().contains(feature) {
            for node in incremental.iter_nodes() {
                assert_eq!(
                    incremental.contains_effect(node, effect).unwrap(),
                    rebuilt.contains_effect(node, effect).unwrap(),
                    "{effect:?} effect differs for {node:?}"
                );
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    enum NormalExportTarget {
        Local {
            declarations: Vec<NodeId>,
            declaration: Option<NodeId>,
            reference: Option<NodeId>,
        },
        External {
            source: swc_next_ecma_semantic::ModuleName,
            imported: swc_next_ecma_semantic::ExternalExportName,
        },
        AnonymousDeclaration(NodeId),
        Expression {
            node: NodeId,
            reference: Option<NodeId>,
            declarations: Option<Vec<NodeId>>,
        },
        UnresolvedLocal {
            name: swc_next_ecma_semantic::ModuleName,
            reference: Option<NodeId>,
        },
    }

    #[derive(Debug, Eq, PartialEq)]
    struct NormalExportEntry {
        node: NodeId,
        statement: NodeId,
        exported: Option<swc_next_ecma_semantic::ExportName>,
        origin: swc_next_ecma_semantic::ExportOrigin,
        target: NormalExportTarget,
        type_only: bool,
    }

    fn normalize_exports(semantic: &Semantic<'_>) -> Vec<NormalExportEntry> {
        use swc_next_ecma_semantic::ExportTarget;

        semantic
            .module_record()
            .exports()
            .iter()
            .map(|entry| NormalExportEntry {
                node: entry.node,
                statement: entry.statement,
                exported: entry.exported,
                origin: entry.origin,
                target: match entry.target {
                    ExportTarget::Local {
                        symbol,
                        declaration,
                        reference,
                    } => NormalExportTarget::Local {
                        declarations: declaration_nodes(semantic, symbol),
                        declaration: declaration.map(|id| semantic.declaration(id).node),
                        reference: reference.map(|id| semantic.reference(id).node),
                    },
                    ExportTarget::External { source, imported } => {
                        NormalExportTarget::External { source, imported }
                    }
                    ExportTarget::AnonymousDeclaration { node } => {
                        NormalExportTarget::AnonymousDeclaration(node)
                    }
                    ExportTarget::Expression {
                        node,
                        reference,
                        symbol,
                    } => NormalExportTarget::Expression {
                        node,
                        reference: reference.map(|id| semantic.reference(id).node),
                        declarations: symbol.map(|id| declaration_nodes(semantic, id)),
                    },
                    ExportTarget::UnresolvedLocal { name, reference } => {
                        NormalExportTarget::UnresolvedLocal {
                            name,
                            reference: reference.map(|id| semantic.reference(id).node),
                        }
                    }
                },
                type_only: entry.type_only,
            })
            .collect()
    }

    assert_eq!(
        normalize_exports(incremental),
        normalize_exports(&rebuilt),
        "module record differs"
    );

    fn scope_path(
        semantic: &Semantic<'_>,
        mut scope: ScopeId,
    ) -> Vec<(NodeId, ScopeKind, ScopeFlags, NodeId, ScopeKind)> {
        let mut path = Vec::new();
        loop {
            let current = semantic.scope(scope);
            let hoist_target = semantic.scope(current.hoist_target);
            path.push((
                current.node,
                current.kind,
                current.flags,
                hoist_target.node,
                hoist_target.kind,
            ));
            let Some(parent) = current.parent else {
                break;
            };
            scope = parent;
        }
        path
    }

    fn declaration_nodes(semantic: &Semantic<'_>, symbol: SymbolId) -> Vec<NodeId> {
        semantic
            .declarations(symbol)
            .iter()
            .map(|declaration| declaration.node)
            .collect()
    }
}
