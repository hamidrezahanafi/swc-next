use std::collections::{HashMap, HashSet};

use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::{
    Ast, AstNode, AstStr, CloneIn, DUMMY_SP, Expr, IdentifierReference, Mark, NodeId, NodeKindData,
    ScopeId, Stmt, SyntaxContext, TypedSubRange, Visit, VisitWith,
};
use swc_next_ecma_semantic::{
    AnalyzeOptions, DeclarationId, ReferenceId, Semantic, SemanticNodeState, analyze,
};
use swc_next_ecma_semantic::{ReferenceFlags, SymbolFlags, scope};

use super::{
    DetachedClone, DetachedScopeCreator, DetachedScopeLayout, DetachedScopeRegion,
    GeneratedProgramImport, IdentityPolicy, MutationTransaction, PendingReferenceId,
    PendingScopeId, PendingSymbolId, PrivateBinding, ReservedPrivateName, ScopeHandle,
    StatementListOwner, SymbolHandle,
    journal::{
        AstEdit, IdentityEdit, ModuleEdit, PendingNameReservation, PendingReference, PendingScope,
        PendingSymbol, ReplacementCategory, ResolutionEdit, ScopeEdit, SemanticEdit,
        SharedPrivateBinding,
    },
};

impl<'tx, 'ast> MutationTransaction<'tx, 'ast> {
    /// Builds nodes that are detached from the current tree.
    ///
    /// Appending detached storage does not advance the attached-tree mutation
    /// epoch. The returned nodes become part of the tree only through a
    /// journalled structural operation.
    pub fn build_detached<T>(&mut self, build: impl FnOnce(&mut Ast<'ast>) -> T) -> T {
        build(self.ast)
    }

    /// Builds detached nodes while consulting facts from the attached
    /// Semantic snapshot. The closure must not mutate attached AST nodes.
    pub fn build_detached_with_semantic<T>(
        &mut self,
        build: impl FnOnce(&mut Ast<'ast>, &Semantic<'ast>) -> T,
    ) -> T {
        build(self.ast, self.semantic)
    }

    /// Deep-clones an attached subtree into detached AST storage.
    ///
    /// Hygiene contexts are copied explicitly because the generated
    /// `CloneIn` implementations only clone node payloads. The returned
    /// bidirectional origin map remains valid while callers rewrite the clone
    /// and allows semantic queries to be redirected to attached nodes.
    pub fn clone_detached<N>(&mut self, root: N) -> DetachedClone<N>
    where
        N: Copy
            + CloneIn<'ast, Cloned = N>
            + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let original_nodes = collect_subtree_nodes(self.ast, root);
        let original_contexts: Vec<_> = original_nodes
            .iter()
            .map(|node| self.ast.syntax_context(*node))
            .collect();
        let clone = root.clone_in(self.ast);
        let cloned_nodes = collect_subtree_nodes(self.ast, clone);

        assert_eq!(
            original_nodes.len(),
            cloned_nodes.len(),
            "CloneIn must preserve the visited subtree shape"
        );

        let mut origins = std::collections::HashMap::with_capacity(cloned_nodes.len());
        let mut clones = std::collections::HashMap::with_capacity(cloned_nodes.len());
        for ((origin, cloned), context) in original_nodes
            .into_iter()
            .zip(cloned_nodes)
            .zip(original_contexts)
        {
            assert_eq!(
                self.ast.node_kind(origin),
                self.ast.node_kind(cloned),
                "CloneIn must preserve preorder node kinds"
            );
            self.ast.set_syntax_context(cloned, context);
            assert!(origins.insert(cloned, origin).is_none());
            assert!(clones.insert(origin, cloned).is_none());
        }

        DetachedClone {
            root: clone,
            origins,
            clones,
        }
    }

    /// Replaces an attached subtree root with an exact detached clone while
    /// preserving the root NodeId and every existing semantic identity below
    /// it.
    ///
    /// This operation is the copy-on-write bridge used by transforms that
    /// need to freely rewrite a detached copy before publishing it. It keeps
    /// DeclarationId, ReferenceId, SymbolId, and ScopeId stable, reowners
    /// nested scopes to cloned nodes, and retires the old descendant nodes in
    /// the same commit.
    pub fn replace_from_detached_clone<N>(&mut self, old: N, clone: DetachedClone<N>)
    where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let old_root = old.node_id();
        let cloned_root = clone.root.node_id();
        assert_eq!(clone.origin_of(cloned_root), Some(old_root));

        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        clone.root.visit_with(&mut collector);
        assert_eq!(collector.nodes.len(), clone.len());

        for (node, local_parent) in collector.nodes.into_iter().skip(1) {
            let origin = clone
                .origin_of(node)
                .expect("exact detached clone contains no generated nodes");
            let parent = local_parent.map(|parent| {
                if parent == cloned_root {
                    old_root
                } else {
                    parent
                }
            });
            self.register_node(node, parent, self.semantic.scope_of(origin));
            if let Some(declaration) = self.semantic.declaration_of(origin) {
                self.retarget_declaration_node(declaration, node);
            }
            if let Some(reference) = self.semantic.reference_of(origin) {
                self.retarget_reference_node(reference, node);
            }
        }

        let scope_reowners: Vec<_> = self
            .semantic
            .iter_scopes()
            .filter_map(|entry| {
                let owner = entry.scope.node;
                (owner != old_root)
                    .then(|| clone.clone_of(owner).map(|cloned| (entry.id, cloned)))
                    .flatten()
            })
            .collect();
        for (scope, owner) in scope_reowners {
            self.reowner_scope(scope, owner);
        }

        let mut retired = Vec::new();
        let mut pending: Vec<_> = self
            .semantic
            .node_children(old_root)
            .map(|node| (node, false))
            .collect();
        while let Some((node, visited)) = pending.pop() {
            if visited {
                retired.push(node);
            } else {
                pending.push((node, true));
                pending.extend(
                    self.semantic
                        .node_children(node)
                        .map(|child| (child, false)),
                );
            }
        }
        for node in retired {
            self.retire_node(node);
        }
        self.replace(old_root, cloned_root, IdentityPolicy::Preserve);
    }

    /// Publishes a transformed detached clone and reconciles its old semantic
    /// identities with the final reachable tree.
    ///
    /// Nodes added after cloning inherit their parent's scope unless they are
    /// roots of an explicit region. Existing nested scopes whose owner clones
    /// remain reachable keep their ScopeId and are reowned/reparented. Old
    /// identities and scopes whose clones disappeared are retired.
    pub fn replace_from_transformed_clone<N>(
        &mut self,
        old: N,
        clone: DetachedClone<N>,
        root_scope: impl Into<ScopeHandle>,
        regions: &[DetachedScopeRegion],
    ) -> DetachedScopeLayout
    where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.replace_from_transformed_clone_impl(
            old,
            clone,
            root_scope.into(),
            regions,
            ReplacementCategory::Exact,
        )
    }

    /// Publishes a transformed expression clone while allowing its root to
    /// change to another expression variant and retain the original NodeId.
    pub fn replace_expression_from_transformed_clone(
        &mut self,
        old: Expr,
        clone: DetachedClone<Expr>,
        root_scope: impl Into<ScopeHandle>,
        regions: &[DetachedScopeRegion],
    ) -> DetachedScopeLayout {
        self.replace_from_transformed_clone_impl(
            old,
            clone,
            root_scope.into(),
            regions,
            ReplacementCategory::Expression,
        )
    }

    /// Publishes a transformed statement clone while allowing its concrete
    /// statement variant to change and retaining the statement-union NodeId.
    pub fn replace_statement_from_transformed_clone(
        &mut self,
        old: Stmt,
        clone: DetachedClone<Stmt>,
        root_scope: impl Into<ScopeHandle>,
        regions: &[DetachedScopeRegion],
    ) -> DetachedScopeLayout {
        self.replace_from_transformed_clone_impl(
            old,
            clone,
            root_scope.into(),
            regions,
            ReplacementCategory::Statement,
        )
    }

    /// Recreates semantic identities for an additional detached occurrence of
    /// an attached subtree that has already been published as generated nodes
    /// by [`Self::replace_from_transformed_clone`].
    ///
    /// Unlike the primary transformed clone, a copied occurrence must not
    /// retarget the source identities: nested scopes, bindings, declarations,
    /// and references receive new IDs. References to bindings outside the
    /// copied subtree continue to target the existing symbols.
    pub fn duplicate_semantics_from_detached_clone<N>(
        &mut self,
        copy: &DetachedClone<N>,
        layout: &DetachedScopeLayout,
    ) {
        let origins: HashSet<_> = copy.origins.values().copied().collect();

        let mut source_scopes: Vec<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| origins.contains(&entry.scope.node))
            .map(|entry| (entry.id, entry.scope))
            .collect();
        source_scopes.sort_by_key(|(scope, _)| self.scope_depth(*scope));
        let mut scopes = HashMap::new();
        for (source, data) in source_scopes {
            let owner = copy
                .clone_of(data.node)
                .expect("copied scope owner must have a cloned node");
            let parent = data
                .parent
                .and_then(|parent| scopes.get(&parent).copied())
                .unwrap_or_else(|| layout.scope_of(owner));
            let scope = self.create_scope(parent, owner, data.kind, data.flags);
            scopes.insert(source, ScopeHandle::Pending(scope));
        }

        let mut copied_nodes: Vec<_> = copy
            .origins
            .iter()
            .map(|(node, origin)| (*node, *origin))
            .collect();
        copied_nodes.sort_by_key(|(node, _)| node.index());
        for &(node, origin) in &copied_nodes {
            if let Some(scope) = scopes.get(&self.semantic.scope_of(origin)).copied() {
                self.rescope_node(node, scope);
            }
        }

        let mut source_symbols = Vec::new();
        for entry in self.semantic.iter_symbols() {
            let declarations: Vec<_> = self
                .semantic
                .declarations(entry.id)
                .iter()
                .filter_map(|declaration| {
                    copy.clone_of(declaration.node)
                        .map(|node| (node, declaration.flags))
                })
                .collect();
            if declarations.is_empty() {
                continue;
            }
            assert_eq!(
                declarations.len(),
                1,
                "copied subtrees do not support merged declarations yet"
            );
            source_symbols.push((entry.id, entry.symbol, declarations[0]));
        }
        let mut symbols = HashMap::new();
        for (source, symbol, (node, flags)) in source_symbols {
            let scope = scopes
                .get(&symbol.scope)
                .copied()
                .unwrap_or_else(|| layout.scope_of(node));
            let pending = self.declare(scope, node, symbol.name, flags);
            symbols.insert(source, SymbolHandle::Pending(pending));
        }

        for (node, origin) in copied_nodes {
            let Some(reference) = self.semantic.reference_of(origin) else {
                continue;
            };
            let reference = self.semantic.reference(reference);
            let scope = scopes
                .get(&reference.scope)
                .copied()
                .unwrap_or_else(|| layout.scope_of(node));
            let symbol = reference.symbol.map(|symbol| {
                symbols
                    .get(&symbol)
                    .copied()
                    .unwrap_or(SymbolHandle::Existing(symbol))
            });
            self.reference(scope, node, reference.name, reference.flags, symbol);
        }
    }

    fn replace_from_transformed_clone_impl<N>(
        &mut self,
        old: N,
        clone: DetachedClone<N>,
        root_scope: ScopeHandle,
        regions: &[DetachedScopeRegion],
        replacement_category: ReplacementCategory,
    ) -> DetachedScopeLayout
    where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let old_root = old.node_id();
        let cloned_root = clone.root.node_id();
        assert_eq!(clone.origin_of(cloned_root), Some(old_root));
        let explicit_regions: HashMap<_, _> = regions
            .iter()
            .map(|region| (region.root, region.scope))
            .collect();

        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        clone.root.visit_with(&mut collector);
        let final_nodes = collector.nodes;
        let reachable_clones: HashSet<_> = final_nodes.iter().map(|(node, _)| *node).collect();
        let reachable_origins: HashSet<_> = reachable_clones
            .iter()
            .filter_map(|node| clone.origin_of(*node))
            .collect();

        let old_descendants: HashSet<_> = {
            let mut nodes = HashSet::new();
            let mut pending = vec![old_root];
            while let Some(node) = pending.pop() {
                if nodes.insert(node) {
                    pending.extend(self.semantic.node_children(node));
                }
            }
            nodes
        };
        let preserved_scopes: HashSet<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| {
                old_descendants.contains(&entry.scope.node)
                    && reachable_origins.contains(&entry.scope.node)
            })
            .map(|entry| entry.id)
            .collect();
        let retiring_scopes: HashSet<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| {
                entry.id != ScopeId::ROOT
                    && old_descendants.contains(&entry.scope.node)
                    && !preserved_scopes.contains(&entry.id)
            })
            .map(|entry| entry.id)
            .collect();

        let mut assigned_scopes = HashMap::new();
        let mut final_parents = HashMap::new();
        assigned_scopes.insert(cloned_root, root_scope);
        for (node, local_parent) in final_nodes.iter().copied().skip(1) {
            let parent = local_parent.expect("non-root clone node has a parent");
            let inherited = assigned_scopes[&parent];
            let owned_scope = clone.origin_of(node).and_then(|origin| {
                let scope = self.semantic.scope_of(origin);
                (self.semantic.scope(scope).node == origin && preserved_scopes.contains(&scope))
                    .then_some(ScopeHandle::Existing(scope))
            });
            let origin_scope_override = clone.origin_of(node).and_then(|origin| {
                let origin_parent = clone.origin_of(parent)?;
                let scope = self.semantic.scope_of(origin);
                let parent_scope = self.semantic.scope_of(origin_parent);
                (scope != parent_scope && !retiring_scopes.contains(&scope))
                    .then_some(ScopeHandle::Existing(scope))
            });
            let scope = explicit_regions
                .get(&node)
                .copied()
                .or(owned_scope)
                .or(origin_scope_override)
                .unwrap_or(inherited);
            let parent = if parent == cloned_root {
                old_root
            } else {
                parent
            };
            final_parents.insert(node, parent);
            assigned_scopes.insert(node, scope);
            self.register_node(node, Some(parent), scope);
            if let Some(origin) = clone.origin_of(node) {
                self.journal.module_edits.push(ModuleEdit::RetargetNode {
                    old: origin,
                    new: node,
                });
                if let Some(declaration) = self.semantic.declaration_of(origin) {
                    self.retarget_declaration_node(declaration, node);
                }
                if let Some(reference) = self.semantic.reference_of(origin) {
                    self.retarget_reference_node(reference, node);
                    self.rescope_reference(reference, scope);
                }
            }
        }

        let preserved_scope_data: Vec<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| preserved_scopes.contains(&entry.id))
            .map(|entry| (entry.id, entry.scope))
            .collect();
        for (scope, data) in preserved_scope_data {
            if data.node == old_root {
                continue;
            }
            let owner = clone
                .clone_of(data.node)
                .expect("preserved scope owner has a reachable clone");
            self.reowner_scope(scope, owner);
            if data
                .parent
                .is_none_or(|parent| !preserved_scopes.contains(&parent))
            {
                let parent_node = final_parents[&owner];
                let parent_scope = if parent_node == old_root {
                    root_scope
                } else {
                    assigned_scopes[&parent_node]
                };
                if parent_scope != ScopeHandle::Existing(scope)
                    && data.parent.map(ScopeHandle::Existing) != Some(parent_scope)
                {
                    self.reparent_scope(scope, parent_scope);
                }
            }
        }

        let mut retired_references = HashSet::new();
        let mut retired_declarations = HashSet::new();
        let mut affected_symbols = HashSet::new();
        let mut retired_nodes = Vec::new();
        let mut pending: Vec<_> = self
            .semantic
            .node_children(old_root)
            .map(|node| (node, false))
            .collect();
        while let Some((node, visited)) = pending.pop() {
            if visited {
                if !reachable_origins.contains(&node) {
                    if let Some(reference) = self.semantic.reference_of(node) {
                        self.retire_reference(reference);
                        retired_references.insert(reference);
                    }
                    if let Some(declaration) = self.semantic.declaration_of(node) {
                        affected_symbols.insert(self.semantic.declaration(declaration).symbol);
                        self.retire_declaration(declaration);
                        retired_declarations.insert(declaration);
                    }
                }
                retired_nodes.push(node);
            } else {
                pending.push((node, true));
                pending.extend(
                    self.semantic
                        .node_children(node)
                        .map(|child| (child, false)),
                );
            }
        }

        let active_symbols: Vec<_> = self
            .semantic
            .iter_symbols()
            .map(|entry| (entry.id, entry.symbol.scope))
            .collect();
        for (symbol, symbol_scope) in active_symbols {
            if retiring_scopes.contains(&symbol_scope) {
                let target = self
                    .semantic
                    .declarations(symbol)
                    .iter()
                    .find_map(|declaration| {
                        clone
                            .clone_of(declaration.node)
                            .filter(|node| reachable_clones.contains(node))
                            .map(|node| assigned_scopes[&node])
                    });
                if let Some(target) = target {
                    self.move_symbol(symbol, target, std::iter::empty());
                } else {
                    affected_symbols.insert(symbol);
                }
            }
        }
        for symbol in affected_symbols {
            let declarations_retired =
                self.semantic
                    .declarations(symbol)
                    .iter()
                    .all(|declaration| {
                        self.semantic
                            .declaration_of(declaration.node)
                            .is_some_and(|id| retired_declarations.contains(&id))
                    });
            let references_retired = self
                .semantic
                .uses(symbol)
                .iter()
                .all(|reference| retired_references.contains(reference));
            if declarations_retired && references_retired {
                self.retire_symbol(symbol);
            }
        }

        let mut retiring_scopes: Vec<_> = retiring_scopes.into_iter().collect();
        retiring_scopes.sort_by_key(|scope| std::cmp::Reverse(self.scope_depth(*scope)));
        for scope in retiring_scopes {
            self.retire_scope(scope);
        }
        for node in retired_nodes {
            self.retire_node(node);
        }
        self.replace_with_category(
            old_root,
            cloned_root,
            IdentityPolicy::Preserve,
            replacement_category,
        );
        DetachedScopeLayout {
            regions: regions.to_vec(),
            node_scopes: assigned_scopes,
        }
    }

    /// Creates generated scopes in AST preorder so pending parents always
    /// precede their children. Existing regions (for example a generated
    /// function scope) participate in inheritance but are not recreated.
    pub fn create_generated_scope_regions<N>(
        &mut self,
        root: N,
        root_scope: impl Into<ScopeHandle>,
        existing: &[DetachedScopeRegion],
        creators: &[DetachedScopeCreator],
    ) -> DetachedScopeLayout
    where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let root_scope = root_scope.into();
        let existing_scopes: HashMap<_, _> = existing
            .iter()
            .map(|region| (region.root, region.scope))
            .collect();
        let creators_by_owner: HashMap<_, _> = creators
            .iter()
            .map(|creator| (creator.owner, creator))
            .collect();
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_with(&mut collector);
        let mut scopes = HashMap::new();
        let mut child_scopes = HashMap::new();
        let mut scopes_by_entry = HashMap::new();
        let mut created = Vec::new();
        let mut created_scope_count = 0;
        for (node, parent) in collector.nodes {
            let mut inherited = parent
                .and_then(|parent| child_scopes.get(&parent).copied())
                .unwrap_or(root_scope);
            if let Some(scope) = scopes_by_entry.get(&node).copied() {
                inherited = scope;
            }
            let (scope, child_scope) = if let Some(scope) = existing_scopes.get(&node).copied() {
                (scope, scope)
            } else if let Some(creator) = creators_by_owner.get(&node).copied() {
                created_scope_count += 1;
                let pending =
                    self.create_scope(inherited, creator.owner, creator.kind, creator.flags);
                let scope = ScopeHandle::Pending(pending);
                for &entry in &creator.entries {
                    scopes_by_entry.insert(entry, scope);
                    created.push(DetachedScopeRegion { root: entry, scope });
                }
                let enters_at_owner = creator.entries.contains(&node);
                let node_scope = if creator.owner_in_scope || enters_at_owner {
                    scope
                } else {
                    inherited
                };
                let child_scope = if enters_at_owner { scope } else { inherited };
                (node_scope, child_scope)
            } else {
                (inherited, inherited)
            };
            scopes.insert(node, scope);
            child_scopes.insert(node, child_scope);
        }
        assert_eq!(
            created_scope_count,
            creators_by_owner.len(),
            "all scope creators are reachable"
        );
        let mut regions = Vec::with_capacity(existing.len() + created.len());
        regions.extend_from_slice(existing);
        regions.extend(created);
        DetachedScopeLayout {
            regions,
            node_scopes: scopes,
        }
    }

    /// Registers a fully generated subtree using a precomputed scope layout.
    /// Generated helpers use this path so their function and nested scopes
    /// exist before bindings and references are attached.
    pub fn register_generated_subtree_with_layout<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        layout: &DetachedScopeLayout,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_with(&mut collector);
        for (node, local_parent) in collector.nodes {
            self.register_node(node, local_parent.or(parent), layout.scope_of(node));
        }
    }

    /// Imports a detached generated program through the same pending identity
    /// graph used by handwritten semantic-aware builders.
    ///
    /// The generated program is analyzed in isolation first. Its root scope
    /// is mapped to `target_scope`; nested scopes, declarations, and
    /// references are recreated as pending transaction entries. The detached
    /// program wrapper itself is not published.
    pub fn import_generated_program(
        &mut self,
        generated: swc_next_ecma_ast::Program,
        parent: NodeId,
        target_scope: ScopeId,
    ) -> GeneratedProgramImport {
        assert!(self.semantic.is_scope_active(target_scope));
        let nodes = collect_subtree_nodes(self.ast, generated);
        for &node in &nodes {
            self.ast.set_node_span(node, DUMMY_SP);
        }

        let fragment = analyze(
            generated,
            self.ast,
            AnalyzeOptions {
                features: self.semantic.features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let root_scope = fragment.scope_of(generated.node_id());
        let target_strict = self.semantic.scope(target_scope).flags.strict;
        let mut scopes = HashMap::new();
        scopes.insert(root_scope, ScopeHandle::Existing(target_scope));
        for entry in fragment.iter_scopes() {
            if entry.id == root_scope || !fragment.scope_is_ancestor(root_scope, entry.id) {
                continue;
            }
            let source_parent = entry
                .scope
                .parent
                .expect("a generated fragment child scope must have a parent");
            let mapped_parent = scopes[&source_parent];
            let mut flags = entry.scope.flags;
            flags.strict |= target_strict;
            let pending =
                self.create_scope(mapped_parent, entry.scope.node, entry.scope.kind, flags);
            scopes.insert(entry.id, ScopeHandle::Pending(pending));
        }

        // Structural registration must be parent-first. NodeId allocation
        // order is generally child-first for parser-built trees, so the
        // fragment's identity iterator cannot provide this ordering.
        for node in nodes {
            if node == generated.node_id() {
                continue;
            }
            let source_parent = fragment.parent_of(node);
            let mapped_parent = match source_parent {
                Some(source_parent) if source_parent == generated.node_id() => Some(parent),
                Some(source_parent) => Some(source_parent),
                None => Some(parent),
            };
            self.register_node(node, mapped_parent, scopes[&fragment.scope_of(node)]);
        }

        let mut symbols = HashMap::new();
        for entry in fragment.iter_symbols() {
            let declarations = fragment.declarations(entry.id);
            assert_eq!(
                declarations.len(),
                1,
                "generated fragments do not support merged declarations yet"
            );
            let declaration = declarations[0];
            assert_eq!(
                declaration.flags, entry.symbol.flags,
                "generated fragment declaration flags must match its symbol"
            );
            let mapped_scope = scopes[&entry.symbol.scope];
            let pending = if let Some((index, pending)) = self
                .journal
                .pending_symbols
                .iter()
                .enumerate()
                .find(|(_, pending)| pending.node == declaration.node)
            {
                assert_eq!(pending.scope, mapped_scope);
                assert_eq!(pending.name, entry.symbol.name);
                assert_eq!(pending.flags, declaration.flags);
                PendingSymbolId(index)
            } else {
                self.declare(
                    mapped_scope,
                    declaration.node,
                    entry.symbol.name,
                    declaration.flags,
                )
            };
            symbols.insert(entry.id, SymbolHandle::Pending(pending));
        }
        for entry in fragment.iter_references() {
            let target = entry.reference.symbol.map(|symbol| symbols[&symbol]);
            self.reference(
                scopes[&entry.reference.scope],
                entry.reference.node,
                entry.reference.name,
                entry.reference.flags,
                target,
            );
        }

        GeneratedProgramImport {
            statements: generated
                .body(self.ast)
                .iter()
                .map(|id| self.ast.get_node_in_sub_range(id))
                .collect(),
            scopes,
            symbols,
        }
    }

    /// Replaces the payload stored at `old` while preserving `old`'s NodeId.
    ///
    /// `new` must be a detached node. Descendants introduced or removed by
    /// the replacement must be registered or retired in this transaction.
    pub fn replace(&mut self, old: NodeId, new: NodeId, identity: IdentityPolicy) {
        self.replace_with_category(old, new, identity, ReplacementCategory::Exact);
    }

    /// Replaces one expression variant with another while preserving the
    /// expression-union edge and, when requested, the original NodeId.
    pub fn replace_expression(&mut self, old: Expr, new: Expr, identity: IdentityPolicy) {
        self.replace_with_category(
            old.node_id(),
            new.node_id(),
            identity,
            ReplacementCategory::Expression,
        );
    }

    fn replace_with_category(
        &mut self,
        old: NodeId,
        new: NodeId,
        identity: IdentityPolicy,
        category: ReplacementCategory,
    ) {
        self.journal.ast_edits.push(AstEdit::Replace {
            old,
            new,
            identity,
            category,
        });
        self.mark_dirty(old);
    }

    /// Replaces one complete statement-list edge while preserving its owner.
    /// Matching semantic lifecycle edits must be part of this transaction.
    pub fn replace_statements(
        &mut self,
        owner: StatementListOwner,
        statements: impl IntoIterator<Item = Stmt>,
    ) {
        let statements = self.ast.add_typed_sub_range(statements);
        let span = self.ast.span(owner.node_id());
        let existing = self.journal.ast_edits.iter().position(|edit| {
            matches!(
                edit,
                AstEdit::Replace {
                    old,
                    identity: IdentityPolicy::Preserve,
                    ..
                } if *old == owner.node_id()
            )
        });
        let base = existing.map_or(owner.node_id(), |index| {
            match self.journal.ast_edits[index] {
                AstEdit::Replace { new, .. } => new,
            }
        });
        let replacement = match (owner, self.ast.kind_data(base)) {
            (StatementListOwner::Program(_), NodeKindData::Program(node)) => self
                .ast
                .program(
                    span,
                    node.source_type(self.ast),
                    node.directives(self.ast),
                    statements,
                    node.hashbang(self.ast),
                )
                .node_id(),
            (StatementListOwner::Block(_), NodeKindData::BlockStatement(_)) => {
                self.ast.block_statement(span, statements).node_id()
            }
            (StatementListOwner::FunctionBody(_), NodeKindData::FunctionBody(node)) => self
                .ast
                .function_body(span, node.directives(self.ast), statements)
                .node_id(),
            (StatementListOwner::StaticBlock(_), NodeKindData::StaticBlock(_)) => {
                self.ast.static_block(span, statements).node_id()
            }
            (StatementListOwner::SwitchCase(_), NodeKindData::SwitchCase(node)) => self
                .ast
                .switch_case(span, node.test(self.ast), statements)
                .node_id(),
            (StatementListOwner::TsModuleBlock(_), NodeKindData::TsModuleBlock(node)) => self
                .ast
                .ts_module_block(span, node.directives(self.ast), statements)
                .node_id(),
            _ => unreachable!("statement-list replacement preserves its owner kind"),
        };
        if let Some(index) = existing {
            self.journal.ast_edits[index] = AstEdit::Replace {
                old: owner.node_id(),
                new: replacement,
                identity: IdentityPolicy::Preserve,
                category: ReplacementCategory::Exact,
            };
            self.mark_dirty(owner.node_id());
        } else {
            self.replace(owner.node_id(), replacement, IdentityPolicy::Preserve);
        }
    }

    pub fn insert_before(
        &mut self,
        owner: StatementListOwner,
        anchor: NodeId,
        inserted: impl IntoIterator<Item = Stmt>,
    ) {
        let mut statements = self.statement_list(owner);
        let index = statements
            .iter()
            .position(|statement| statement.node_id() == anchor)
            .expect("statement anchor belongs to its list owner");
        statements.splice(index..index, inserted);
        self.replace_statements(owner, statements);
    }

    pub fn insert_after(
        &mut self,
        owner: StatementListOwner,
        anchor: NodeId,
        inserted: impl IntoIterator<Item = Stmt>,
    ) {
        let mut statements = self.statement_list(owner);
        let index = statements
            .iter()
            .position(|statement| statement.node_id() == anchor)
            .expect("statement anchor belongs to its list owner");
        statements.splice(index + 1..index + 1, inserted);
        self.replace_statements(owner, statements);
    }

    pub fn remove_statement(&mut self, owner: StatementListOwner, removed: NodeId) {
        self.replace_statement(owner, removed, std::iter::empty());
    }

    /// Removes a statement edge and retires every semantic identity wholly
    /// owned by that subtree. Bindings still used outside the subtree are
    /// rejected by preflight unless callers rebind those uses first.
    pub fn remove_subtree(&mut self, owner: StatementListOwner, root: NodeId) {
        self.remove_statement(owner, root);
        self.retire_subtree(root);
    }

    pub fn retire_subtree(&mut self, root: NodeId) {
        let mut post_order = Vec::new();
        let mut pending = vec![(root, false)];
        while let Some((node, visited)) = pending.pop() {
            if visited {
                post_order.push(node);
                continue;
            }
            pending.push((node, true));
            pending.extend(
                self.semantic
                    .node_children(node)
                    .map(|child| (child, false)),
            );
        }
        let nodes: HashSet<_> = post_order.iter().copied().collect();
        let mut symbols = HashSet::new();
        for node in &post_order {
            if let Some(reference) = self.semantic.reference_of(*node) {
                self.retire_reference(reference);
            }
            if let Some(declaration) = self.semantic.declaration_of(*node) {
                let symbol = self.semantic.declaration(declaration).symbol;
                self.retire_declaration(declaration);
                symbols.insert(symbol);
            }
        }
        for symbol in symbols {
            if self
                .semantic
                .declarations(symbol)
                .iter()
                .all(|declaration| nodes.contains(&declaration.node))
            {
                self.retire_symbol(symbol);
            }
        }

        let mut scopes: Vec<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| entry.id != ScopeId::ROOT && nodes.contains(&entry.scope.node))
            .map(|entry| entry.id)
            .collect();
        scopes.sort_by_key(|scope| std::cmp::Reverse(self.scope_depth(*scope)));
        for scope in scopes {
            self.retire_scope(scope);
        }
        for node in post_order {
            self.retire_node(node);
        }
    }

    /// Moves an attached subtree while preserving its node, scope, symbol,
    /// declaration, and reference identities.
    pub fn move_subtree(&mut self, root: NodeId, new_parent: NodeId, new_scope: ScopeId) {
        let mut nodes = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            nodes.push(node);
            pending.extend(self.semantic.node_children(node));
        }
        let node_set: HashSet<_> = nodes.iter().copied().collect();
        let owned_scopes: HashSet<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| node_set.contains(&entry.scope.node))
            .map(|entry| entry.id)
            .collect();
        let top_owned_scopes: Vec<_> = owned_scopes
            .iter()
            .copied()
            .filter(|scope| {
                self.semantic
                    .scope(*scope)
                    .parent
                    .is_none_or(|parent| !owned_scopes.contains(&parent))
            })
            .collect();

        let mut moved_symbols = HashSet::new();
        let mut rescoped_nodes = Vec::new();
        let mut rescoped_references = Vec::new();
        for node in &nodes {
            let scope = self.semantic.scope_of(*node);
            if !owned_scopes.contains(&scope) {
                rescoped_nodes.push(*node);
                if let Some(reference) = self.semantic.reference_of(*node) {
                    rescoped_references.push(reference);
                }
            }
            if let Some(declaration) = self.semantic.declaration_of(*node) {
                let symbol = self.semantic.declaration(declaration).symbol;
                if self
                    .semantic
                    .declarations(symbol)
                    .iter()
                    .all(|declaration| node_set.contains(&declaration.node))
                {
                    moved_symbols.insert(symbol);
                }
            }
        }

        self.reparent_node(root, Some(new_parent));
        for scope in top_owned_scopes {
            self.reparent_scope(scope, new_scope);
        }
        for node in rescoped_nodes {
            self.rescope_node(node, new_scope);
        }
        for reference in rescoped_references {
            self.rescope_reference(reference, new_scope);
        }
        for symbol in moved_symbols {
            let flags = self.semantic.symbol(symbol).flags;
            let target = if flags.is_hoisting_var() {
                self.semantic.scope(new_scope).hoist_target
            } else {
                new_scope
            };
            let mut hoisting_scopes = Vec::new();
            if flags.is_hoisting_var() {
                let mut current = new_scope;
                while current != target {
                    hoisting_scopes.push(ScopeHandle::Existing(current));
                    current = self
                        .semantic
                        .scope(current)
                        .parent
                        .expect("hoist target is an ancestor of the declaration scope");
                }
            }
            self.journal
                .resolution_edits
                .push(ResolutionEdit::MoveSymbol {
                    symbol,
                    scope: ScopeHandle::Existing(target),
                    hoisting_scopes,
                    closed_subtree: Some(nodes.clone()),
                });
        }
    }

    /// Rescopes an attached subtree that is adopted by a generated wrapper.
    /// The wrapper adoption owns the root-parent edit; this operation updates
    /// only scope membership, nested scope parents, and reference scopes.
    pub fn rescope_adopted_subtree(&mut self, root: NodeId, new_scope: impl Into<ScopeHandle>) {
        let new_scope = new_scope.into();
        let mut nodes = Vec::new();
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            nodes.push(node);
            pending.extend(self.semantic.node_children(node));
        }
        let node_set: HashSet<_> = nodes.iter().copied().collect();
        let owned_scopes: HashSet<_> = self
            .semantic
            .iter_scopes()
            .filter(|entry| node_set.contains(&entry.scope.node))
            .map(|entry| entry.id)
            .collect();
        let top_owned_scopes: Vec<_> = owned_scopes
            .iter()
            .copied()
            .filter(|scope| {
                self.semantic
                    .scope(*scope)
                    .parent
                    .is_none_or(|parent| !owned_scopes.contains(&parent))
            })
            .collect();
        for scope in top_owned_scopes {
            let already_reparented = self.journal.scope_edits.iter().any(|edit| {
                matches!(edit, ScopeEdit::Reparent { scope: edited, .. } if *edited == scope)
            });
            if !already_reparented {
                self.reparent_scope(scope, new_scope);
            }
        }
        for node in nodes {
            if !owned_scopes.contains(&self.semantic.scope_of(node)) {
                self.rescope_node(node, new_scope);
                if let Some(reference) = self.semantic.reference_of(node) {
                    self.rescope_reference(reference, new_scope);
                }
            }
        }
    }

    pub fn move_statement_before(
        &mut self,
        from: StatementListOwner,
        to: StatementListOwner,
        moved: NodeId,
        anchor: NodeId,
        new_scope: ScopeId,
    ) {
        assert_ne!(from, to, "same-list moves require one list rewrite");
        let statement = self
            .statement_list(from)
            .into_iter()
            .find(|statement| statement.node_id() == moved)
            .expect("moved statement belongs to its source owner");
        self.remove_statement(from, moved);
        self.insert_before(to, anchor, [statement]);
        self.move_subtree(moved, to.node_id(), new_scope);
    }

    pub fn move_statement_after(
        &mut self,
        from: StatementListOwner,
        to: StatementListOwner,
        moved: NodeId,
        anchor: NodeId,
        new_scope: ScopeId,
    ) {
        assert_ne!(from, to, "same-list moves require one list rewrite");
        let statement = self
            .statement_list(from)
            .into_iter()
            .find(|statement| statement.node_id() == moved)
            .expect("moved statement belongs to its source owner");
        self.remove_statement(from, moved);
        self.insert_after(to, anchor, [statement]);
        self.move_subtree(moved, to.node_id(), new_scope);
    }

    pub fn replace_statement(
        &mut self,
        owner: StatementListOwner,
        old: NodeId,
        replacement: impl IntoIterator<Item = Stmt>,
    ) {
        let mut statements = self.statement_list(owner);
        let index = statements
            .iter()
            .position(|statement| statement.node_id() == old)
            .expect("statement belongs to its list owner");
        statements.splice(index..=index, replacement);
        self.replace_statements(owner, statements);
    }

    fn statement_list(&self, owner: StatementListOwner) -> Vec<Stmt> {
        let range = match owner {
            StatementListOwner::Program(node) => node.body(self.ast),
            StatementListOwner::Block(node) => node.body(self.ast),
            StatementListOwner::FunctionBody(node) => node.body(self.ast),
            StatementListOwner::StaticBlock(node) => node.body(self.ast),
            StatementListOwner::SwitchCase(node) => node.consequent(self.ast),
            StatementListOwner::TsModuleBlock(node) => node.body(self.ast),
        };
        (0..range.len())
            .map(|index| range.get_node(self.ast, index).unwrap())
            .collect()
    }

    fn scope_depth(&self, mut scope: ScopeId) -> usize {
        let mut depth = 0;
        while let Some(parent) = self.semantic.scope(scope).parent {
            depth += 1;
            scope = parent;
        }
        depth
    }

    pub fn register_node(
        &mut self,
        node: NodeId,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) {
        self.journal
            .semantic_edits
            .push(SemanticEdit::RegisterNode {
                node,
                parent,
                scope: scope.into(),
            });
        self.mark_dirty(parent.unwrap_or(node));
    }

    pub(crate) fn register_subtree<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_with(&mut collector);
        let scope = scope.into();
        for (node, local_parent) in collector.nodes {
            self.register_node(node, local_parent.or(parent), scope);
        }
    }

    /// Registers the generated portion of a mixed detached/attached subtree.
    ///
    /// Existing nodes are adopted at their new edge while their descendants
    /// remain opaque and keep their established semantic identities. This is
    /// the common shape for wrappers around preserved function bodies, such
    /// as class method descriptors and helper calls.
    pub(crate) fn adopt_generated_subtree<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_with(&mut collector);
        let scope = scope.into();
        let mut opaque = HashSet::new();
        for (node, local_parent) in collector.nodes {
            if local_parent.is_some_and(|parent| opaque.contains(&parent)) {
                opaque.insert(node);
                continue;
            }
            let semantic_parent = local_parent.or(parent);
            match self.semantic.node_state(node) {
                SemanticNodeState::Unregistered => {
                    self.register_node(node, semantic_parent, scope);
                }
                SemanticNodeState::Active => {
                    self.reparent_node(node, semantic_parent);
                    opaque.insert(node);
                }
                SemanticNodeState::Inactive => {
                    panic!("inactive nodes cannot be adopted into generated subtrees")
                }
            }
        }
    }

    /// Registers generated nodes according to `layout` while adopting active
    /// subtree roots in place. Once an active node is reached its descendants
    /// remain opaque and keep their stable identities and scopes.
    pub(crate) fn adopt_generated_subtree_with_layout<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        layout: &DetachedScopeLayout,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_with(&mut collector);
        let mut opaque = HashSet::new();
        for (node, local_parent) in collector.nodes {
            if local_parent.is_some_and(|parent| opaque.contains(&parent)) {
                opaque.insert(node);
                continue;
            }
            let semantic_parent = local_parent.or(parent);
            match self.semantic.node_state(node) {
                SemanticNodeState::Unregistered => {
                    self.register_node(node, semantic_parent, layout.scope_of(node));
                }
                SemanticNodeState::Active => {
                    self.reparent_node(node, semantic_parent);
                    opaque.insert(node);
                }
                SemanticNodeState::Inactive => {
                    panic!("inactive nodes cannot be adopted into generated subtrees")
                }
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn register_subtree_children<N>(
        &mut self,
        root: N,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
    ) where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        let detached_root = root.node_id();
        let mut collector = DetachedSubtreeCollector {
            ast: self.ast,
            stack: Vec::new(),
            nodes: Vec::new(),
        };
        root.visit_children_with(&mut collector);
        let scope = scope.into();
        for (node, local_parent) in collector.nodes {
            if node == detached_root {
                continue;
            }
            let local_parent = local_parent
                .filter(|local_parent| *local_parent != detached_root)
                .unwrap_or(parent);
            self.register_node(node, Some(local_parent), scope);
        }
    }

    pub fn retire_node(&mut self, node: NodeId) {
        if self.semantic.is_node_active(node)
            && let Some(parent) = self.semantic.parent_of(node)
        {
            self.mark_dirty(parent);
        }
        self.journal
            .semantic_edits
            .push(SemanticEdit::RetireNode { node });
        self.mark_dirty(node);
    }

    pub fn retire_reference(&mut self, reference: ReferenceId) {
        self.journal.resolution_edits.retain(|edit| {
            !matches!(
                edit,
                ResolutionEdit::RetargetReferenceNode {
                    reference: edited,
                    ..
                } | ResolutionEdit::RenameReference {
                    reference: edited,
                    ..
                } | ResolutionEdit::RebindReference {
                    reference: edited,
                    ..
                } | ResolutionEdit::RescopeReference {
                    reference: edited,
                    ..
                } | ResolutionEdit::UpdateReferenceFlags {
                    reference: edited,
                    ..
                } if *edited == reference
            )
        });
        if self.reference_is_scheduled_for_retirement(reference) {
            return;
        }
        self.journal
            .identity_edits
            .push(IdentityEdit::RetireReference { reference });
    }

    pub fn retire_declaration(&mut self, declaration: DeclarationId) {
        self.journal
            .identity_edits
            .push(IdentityEdit::RetireDeclaration { declaration });
    }

    pub fn retire_symbol(&mut self, symbol: SymbolId) {
        self.journal
            .identity_edits
            .push(IdentityEdit::RetireSymbol { symbol });
    }

    pub fn update_symbol_flags(&mut self, symbol: SymbolId, flags: SymbolFlags) {
        self.journal
            .resolution_edits
            .push(ResolutionEdit::UpdateSymbolFlags { symbol, flags });
    }

    pub fn retarget_declaration_node(&mut self, declaration: DeclarationId, node: NodeId) {
        self.mark_dirty(self.semantic.declaration(declaration).node);
        self.mark_dirty(node);
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RetargetDeclarationNode { declaration, node });
    }

    pub fn retarget_reference_node(&mut self, reference: ReferenceId, node: NodeId) {
        self.mark_dirty(self.semantic.reference(reference).node);
        self.mark_dirty(node);
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RetargetReferenceNode { reference, node });
    }

    pub fn update_declaration_flags(&mut self, declaration: DeclarationId, flags: SymbolFlags) {
        self.journal
            .resolution_edits
            .push(ResolutionEdit::UpdateDeclarationFlags { declaration, flags });
    }

    pub fn move_symbol(
        &mut self,
        symbol: SymbolId,
        scope: impl Into<ScopeHandle>,
        hoisting_scopes: impl IntoIterator<Item = ScopeHandle>,
    ) {
        self.journal
            .resolution_edits
            .push(ResolutionEdit::MoveSymbol {
                symbol,
                scope: scope.into(),
                hoisting_scopes: hoisting_scopes.into_iter().collect(),
                closed_subtree: None,
            });
    }

    pub fn rename_symbol(&mut self, symbol: SymbolId, name: AstStr) {
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RenameSymbol { symbol, name });
    }

    pub fn rename_reference(&mut self, reference: ReferenceId, name: AstStr) {
        if self.reference_is_scheduled_for_retirement(reference) {
            return;
        }
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RenameReference { reference, name });
    }

    pub fn rebind_reference(
        &mut self,
        reference: ReferenceId,
        symbol: Option<impl Into<SymbolHandle>>,
    ) {
        if self.reference_is_scheduled_for_retirement(reference) {
            return;
        }
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RebindReference {
                reference,
                symbol: symbol.map(Into::into),
            });
    }

    pub fn rescope_reference(&mut self, reference: ReferenceId, scope: impl Into<ScopeHandle>) {
        if self.reference_is_scheduled_for_retirement(reference) {
            return;
        }
        self.journal
            .resolution_edits
            .push(ResolutionEdit::RescopeReference {
                reference,
                scope: scope.into(),
            });
    }

    pub fn update_reference_flags(&mut self, reference: ReferenceId, flags: ReferenceFlags) {
        if self.reference_is_scheduled_for_retirement(reference) {
            return;
        }
        self.journal
            .resolution_edits
            .push(ResolutionEdit::UpdateReferenceFlags { reference, flags });
    }

    fn reference_is_scheduled_for_retirement(&self, reference: ReferenceId) -> bool {
        self.journal.identity_edits.iter().any(|edit| {
            matches!(
                edit,
                IdentityEdit::RetireReference { reference: retired } if *retired == reference
            )
        })
    }

    pub fn reparent_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        if self.semantic.is_node_active(node)
            && let Some(old_parent) = self.semantic.parent_of(node)
        {
            self.mark_dirty(old_parent);
        }
        self.journal
            .semantic_edits
            .push(SemanticEdit::ReparentNode { node, parent });
        self.mark_dirty(node);
        if let Some(parent) = parent {
            self.mark_dirty(parent);
        }
    }

    pub fn rescope_node(&mut self, node: NodeId, scope: impl Into<ScopeHandle>) {
        let scope = scope.into();
        if let Some(index) = self.journal.semantic_edits.iter().position(
            |edit| matches!(edit, SemanticEdit::RegisterNode { node: candidate, .. } if *candidate == node),
        ) {
            let SemanticEdit::RegisterNode {
                scope: registered_scope,
                ..
            } = &mut self.journal.semantic_edits[index]
            else {
                unreachable!()
            };
            *registered_scope = scope;
            self.mark_dirty(node);
            return;
        }
        self.journal
            .semantic_edits
            .push(SemanticEdit::RescopeNode { node, scope });
        self.mark_dirty(node);
    }

    pub fn create_scope(
        &mut self,
        parent: impl Into<ScopeHandle>,
        owner: NodeId,
        kind: scope::ScopeKind,
        flags: scope::ScopeFlags,
    ) -> PendingScopeId {
        let id = PendingScopeId(self.journal.pending_scopes.len());
        let parent = parent.into();
        assert!(
            !matches!(parent, ScopeHandle::Pending(parent) if parent.0 >= id.0),
            "pending scope parents must be created before their children"
        );
        self.journal.pending_scopes.push(PendingScope {
            parent,
            owner,
            kind,
            flags,
        });
        self.mark_dirty(owner);
        id
    }

    pub fn retag_scope(
        &mut self,
        scope: ScopeId,
        kind: scope::ScopeKind,
        flags: scope::ScopeFlags,
    ) {
        if self.scope_is_scheduled_for_retirement(scope) {
            return;
        }
        if self.journal.scope_edits.iter().any(|edit| {
            matches!(
                edit,
                ScopeEdit::Retag {
                    scope: edited,
                    kind: edited_kind,
                    flags: edited_flags,
                } if *edited == scope && *edited_kind == kind && *edited_flags == flags
            )
        }) {
            return;
        }
        self.mark_dirty(self.semantic.scope(scope).node);
        self.journal
            .scope_edits
            .push(ScopeEdit::Retag { scope, kind, flags });
    }

    pub fn reowner_scope(&mut self, scope: ScopeId, owner: NodeId) {
        if self.scope_is_scheduled_for_retirement(scope) {
            return;
        }
        if self.journal.scope_edits.iter().any(|edit| {
            matches!(
                edit,
                ScopeEdit::Reowner {
                    scope: edited,
                    owner: edited_owner,
                } if *edited == scope && *edited_owner == owner
            )
        }) {
            return;
        }
        self.mark_dirty(self.semantic.scope(scope).node);
        self.mark_dirty(owner);
        self.journal
            .scope_edits
            .push(ScopeEdit::Reowner { scope, owner });
    }

    pub fn reparent_scope(&mut self, scope: ScopeId, parent: impl Into<ScopeHandle>) {
        if self.scope_is_scheduled_for_retirement(scope) {
            return;
        }
        let parent = parent.into();
        if self.journal.scope_edits.iter().any(|edit| {
            matches!(
                edit,
                ScopeEdit::Reparent {
                    scope: edited,
                    parent: edited_parent,
                } if *edited == scope && *edited_parent == parent
            )
        }) {
            return;
        }
        self.mark_dirty(self.semantic.scope(scope).node);
        if let Some(old_parent) = self.semantic.scope(scope).parent {
            self.mark_dirty(self.semantic.scope(old_parent).node);
        }
        let parent_owner = match parent {
            ScopeHandle::Existing(parent) => self.semantic.scope(parent).node,
            ScopeHandle::Pending(parent) => self.journal.pending_scopes[parent.0].owner,
        };
        self.mark_dirty(parent_owner);
        self.journal
            .scope_edits
            .push(ScopeEdit::Reparent { scope, parent });
    }

    pub fn retire_scope(&mut self, scope: ScopeId) {
        self.mark_dirty(self.semantic.scope(scope).node);
        if let Some(parent) = self.semantic.scope(scope).parent {
            self.mark_dirty(self.semantic.scope(parent).node);
        }
        self.journal.scope_edits.retain(|edit| {
            !matches!(
                edit,
                ScopeEdit::Retag {
                    scope: edited,
                    ..
                } | ScopeEdit::Reowner {
                    scope: edited,
                    ..
                } | ScopeEdit::Reparent {
                    scope: edited,
                    ..
                } if *edited == scope
            )
        });
        if self
            .journal
            .scope_edits
            .iter()
            .any(|edit| matches!(edit, ScopeEdit::Retire { scope: retired } if *retired == scope))
        {
            return;
        }
        let depth = self.scope_depth(scope);
        let insertion = self
            .journal
            .scope_edits
            .iter()
            .position(|edit| {
                matches!(
                    edit,
                    ScopeEdit::Retire { scope: retired }
                        if self.scope_depth(*retired) < depth
                )
            })
            .unwrap_or(self.journal.scope_edits.len());
        self.journal
            .scope_edits
            .insert(insertion, ScopeEdit::Retire { scope });
    }

    fn scope_is_scheduled_for_retirement(&self, scope: ScopeId) -> bool {
        self.journal
            .scope_edits
            .iter()
            .any(|edit| matches!(edit, ScopeEdit::Retire { scope: retired } if *retired == scope))
    }

    pub fn declare(
        &mut self,
        scope: impl Into<ScopeHandle>,
        node: NodeId,
        name: AstStr,
        flags: SymbolFlags,
    ) -> PendingSymbolId {
        let id = PendingSymbolId(self.journal.pending_symbols.len());
        self.journal.pending_symbols.push(PendingSymbol {
            scope: scope.into(),
            node,
            name,
            flags,
        });
        self.mark_dirty(node);
        id
    }

    pub fn retarget_default_export_local(
        &mut self,
        statement: NodeId,
        binding: NodeId,
        symbol: impl Into<SymbolHandle>,
    ) {
        self.journal
            .module_edits
            .push(ModuleEdit::RetargetDefaultLocal {
                statement,
                binding,
                symbol: symbol.into(),
            });
        self.mark_dirty(statement);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn replace_namespace_reexport(
        &mut self,
        old_statement: NodeId,
        statement: NodeId,
        specifier: NodeId,
        local: NodeId,
        exported: NodeId,
        symbol: impl Into<SymbolHandle>,
        reference: PendingReferenceId,
    ) {
        self.journal
            .module_edits
            .push(ModuleEdit::ReplaceNamespaceReexport {
                old_statement,
                statement,
                specifier,
                local,
                exported,
                symbol: symbol.into(),
                reference,
            });
        self.mark_dirty(old_statement);
        self.mark_dirty(statement);
    }

    pub fn reference(
        &mut self,
        scope: impl Into<ScopeHandle>,
        node: NodeId,
        name: AstStr,
        flags: ReferenceFlags,
        symbol: Option<impl Into<SymbolHandle>>,
    ) -> PendingReferenceId {
        let scope = scope.into();
        let symbol = symbol.map(Into::into);
        let flags = if flags.space == swc_next_ecma_semantic::ReferenceSpace::Value
            && self.reference_crosses_with(scope, symbol)
        {
            flags.dynamic()
        } else {
            flags
        };
        let id = PendingReferenceId(self.journal.pending_references.len());
        self.journal.pending_references.push(PendingReference {
            scope,
            node,
            name,
            flags,
            symbol,
        });
        self.mark_dirty(node);
        id
    }

    fn reference_crosses_with(&self, scope: ScopeHandle, symbol: Option<SymbolHandle>) -> bool {
        let target = symbol.map(|symbol| match symbol {
            SymbolHandle::Existing(symbol) => {
                ScopeHandle::Existing(self.semantic.symbol(symbol).scope)
            }
            SymbolHandle::Pending(symbol) => self.journal.pending_symbols[symbol.0].scope,
        });
        let mut current = Some(scope);
        while let Some(scope) = current {
            if Some(scope) == target {
                return false;
            }
            let (kind, parent) = match scope {
                ScopeHandle::Existing(scope) => {
                    let data = self.semantic.scope(scope);
                    (data.kind, data.parent.map(ScopeHandle::Existing))
                }
                ScopeHandle::Pending(scope) => {
                    let data = &self.journal.pending_scopes[scope.0];
                    (data.kind, Some(data.parent))
                }
            };
            if kind == scope::ScopeKind::With {
                return true;
            }
            current = parent;
        }
        false
    }

    pub fn declare_private(
        &mut self,
        scope: ScopeId,
        base: &str,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        self.declare_private_with_min_suffix(scope, base, 1, flags)
    }

    /// Declares one generated binding for a transaction-local logical key.
    ///
    /// Helper registries use this API so independent lowering sites can ask
    /// for the same helper without allocating duplicate declarations or
    /// symbols. Reusing a key with a different scope or flags is a caller bug:
    /// a logical helper has one declaration contract within a transaction.
    pub fn declare_shared_private(
        &mut self,
        scope: ScopeId,
        key: &str,
        base: &str,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        let key_text = self.ast.add_utf8(key);
        let key = self.ast.canonicalize_utf8(key_text);
        if let Some(shared) = self
            .journal
            .shared_private_bindings
            .iter()
            .find(|shared| shared.key == key)
        {
            assert_eq!(shared.scope, scope, "shared private binding scope mismatch");
            assert_eq!(shared.flags, flags, "shared private binding flags mismatch");
            return shared.binding;
        }

        let binding = self.declare_private(scope, base, flags);
        self.journal
            .shared_private_bindings
            .push(SharedPrivateBinding {
                key,
                scope,
                flags,
                binding,
            });
        binding
    }

    pub fn reserve_private_name(&mut self, scope: ScopeId, base: &str) -> ReservedPrivateName {
        self.reserve_private_name_in(scope, scope, base)
    }

    pub fn reserve_private_name_in(
        &mut self,
        collision_scope: ScopeId,
        reservation_scope: impl Into<ScopeHandle>,
        base: &str,
    ) -> ReservedPrivateName {
        let text = self.allocate_private_name(collision_scope, reservation_scope.into(), base, 1);
        let mark = self.ast.fresh_mark(Mark::root());
        let context = self.ast.apply_mark(SyntaxContext::empty(), mark);
        ReservedPrivateName { text, context }
    }

    pub fn reserve_generated_name_exact(&mut self, scope: ScopeId, name: AstStr) {
        if !self.journal.name_reservations.iter().any(|reservation| {
            reservation.scope == ScopeHandle::Existing(scope) && reservation.name == name
        }) {
            self.journal.name_reservations.push(PendingNameReservation {
                scope: ScopeHandle::Existing(scope),
                name,
            });
        }
    }

    pub fn declare_private_with_min_suffix(
        &mut self,
        scope: ScopeId,
        base: &str,
        min_suffix: u64,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        assert!(min_suffix > 0);
        let text =
            self.allocate_private_name(scope, ScopeHandle::Existing(scope), base, min_suffix);
        let name = self.ast.canonicalize_utf8(text);
        let mark = self.ast.fresh_mark(Mark::root());
        let context = self.ast.apply_mark(SyntaxContext::empty(), mark);
        let declaration =
            self.ast
                .binding_identifier(DUMMY_SP, text, TypedSubRange::empty(), None, false);
        self.ast.set_syntax_context(declaration.node_id(), context);
        let symbol = self.declare(scope, declaration.node_id(), name, flags);
        PrivateBinding {
            text,
            context,
            declaration,
            symbol,
        }
    }

    pub fn attach_private_declaration(
        &mut self,
        binding: PrivateBinding,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
    ) {
        self.register_node(binding.declaration.node_id(), Some(parent), scope);
    }

    pub fn private_reference(
        &mut self,
        binding: PrivateBinding,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
        flags: ReferenceFlags,
    ) -> IdentifierReference {
        let scope = scope.into();
        let reference = self.ast.identifier_reference(DUMMY_SP, binding.text);
        self.ast
            .set_syntax_context(reference.node_id(), binding.context);
        self.register_node(reference.node_id(), Some(parent), scope);
        self.reference(
            scope,
            reference.node_id(),
            self.ast.canonicalize_utf8(binding.text),
            flags,
            Some(binding.symbol),
        );
        reference
    }

    fn allocate_private_name(
        &mut self,
        collision_scope: ScopeId,
        reservation_scope: ScopeHandle,
        base: &str,
        min_suffix: u64,
    ) -> swc_next_ecma_ast::AstUtf8 {
        assert!(self.semantic.is_scope_active(collision_scope));
        let base_text = self.ast.add_utf8(base);
        let base_name = self.ast.canonicalize_utf8(base_text);
        let state = self
            .semantic
            .name_allocation_state(collision_scope, base_name);
        let locally_reserved = |name| {
            self.journal
                .name_reservations
                .iter()
                .any(|reservation| reservation.name == name)
        };

        let text = if state.occupancy_count == 0 && !state.reserved && !locally_reserved(base_name)
        {
            base_text
        } else {
            let mut suffix = state.next_suffix.max(min_suffix);
            loop {
                let candidate = self.ast.add_utf8(&format!("{base}{suffix}"));
                let name = self.ast.canonicalize_utf8(candidate);
                let candidate_state = self.semantic.name_allocation_state(collision_scope, name);
                if candidate_state.occupancy_count == 0
                    && !candidate_state.reserved
                    && !locally_reserved(name)
                {
                    break candidate;
                }
                suffix = suffix
                    .checked_add(1)
                    .expect("generated name suffix overflow");
            }
        };
        self.journal.name_reservations.push(PendingNameReservation {
            scope: reservation_scope,
            name: self.ast.canonicalize_utf8(text),
        });
        text
    }
}

#[doc(hidden)]
pub struct DetachedSubtreeCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    stack: Vec<NodeId>,
    nodes: Vec<(NodeId, Option<NodeId>)>,
}

fn collect_subtree_nodes<'ast, N>(ast: &Ast<'ast>, root: N) -> Vec<NodeId>
where
    N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
{
    let mut collector = DetachedSubtreeCollector {
        ast,
        stack: Vec::new(),
        nodes: Vec::new(),
    };
    root.visit_with(&mut collector);
    collector.nodes.into_iter().map(|(node, _)| node).collect()
}

impl<'ast> Visit<'ast> for DetachedSubtreeCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeId) {
        self.nodes.push((node, self.stack.last().copied()));
        self.stack.push(node);
    }

    fn leave_node(&mut self, node: NodeId) {
        assert_eq!(self.stack.pop(), Some(node));
    }
}
