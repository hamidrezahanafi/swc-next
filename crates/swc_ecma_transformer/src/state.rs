use crate::{
    assumptions::Assumptions,
    mutation::{
        builder::DetachedSubtreeCollector, DetachedClone, DetachedScopeCreator,
        DetachedScopeLayout, DetachedScopeRegion, GeneratedProgramImport, IdentityPolicy,
        MutationCommit, MutationError, MutationTransaction, PendingReferenceId, PendingScopeId,
        PendingSymbolId, PrivateBinding, ReservedPrivateName, ScopeHandle, StatementListOwner,
        SymbolHandle,
    },
};
use swc_next_ecma_ast::{
    Ast, AstNode, AstStr, CloneIn, Expr, IdentifierReference, NodeId, ScopeId, Stmt, SymbolId,
    TraverseCtx, VisitWith,
};
use swc_next_ecma_semantic::{
    DeclarationId, ReferenceFlags, ReferenceId, Semantic, SymbolFlags,
    scope::{ScopeFlags, ScopeKind},
};

pub type TransformerCtx<'traverse, 'semantic, 'ast> =
    TraverseCtx<'traverse, 'ast, TransformerState<'semantic, 'ast>>;

pub struct TransformerState<'semantic, 'ast> {
    pub assumptions: Assumptions,
    pub semantic: &'semantic mut Semantic<'ast>,
}

pub(crate) trait TransformerCtxExt<'ast> {
    fn semantic(&self) -> &Semantic<'ast>;

    fn current_scope(&self) -> Option<ScopeId>;

    fn transaction<T>(
        &mut self,
        root: NodeId,
        build: impl FnOnce(&mut AstContext<'_, '_, 'ast>) -> T,
    ) -> Result<(T, MutationCommit), MutationError>;
}

impl<'traverse, 'semantic, 'ast> TransformerCtxExt<'ast>
    for TransformerCtx<'traverse, 'semantic, 'ast>
{
    fn semantic(&self) -> &Semantic<'ast> {
        self.state.semantic
    }

    fn current_scope(&self) -> Option<ScopeId> {
        self.current_node()
            .map(|node| self.state.semantic.scope_of(node))
    }

    fn transaction<T>(
        &mut self,
        root: NodeId,
        build: impl FnOnce(&mut AstContext<'_, '_, 'ast>) -> T,
    ) -> Result<(T, MutationCommit), MutationError> {
        let mut transaction = MutationTransaction::begin(self.ast, self.state.semantic, root)?;
        let value = {
            let mut ctx = AstContext(&mut transaction);
            build(&mut ctx)
        };
        let commit = transaction.commit()?;
        Ok((value, commit))
    }
}

/// Semantic-aware mutation context for transform passes.
///
/// Wraps `MutationTransaction` to prevent direct access to bare `&mut Ast`
/// or semantic mutation primitives. All AST and semantic changes go through
/// typed builder and edit methods.
pub(crate) struct AstContext<'a, 'tx, 'ast>(&'a mut MutationTransaction<'tx, 'ast>);

#[allow(dead_code)]
impl<'a, 'tx, 'ast> AstContext<'a, 'tx, 'ast> {
    /// Temporary escape hatch for passes not yet migrated to the AstContext
    /// API. New code should not use this.
    pub(crate) fn inner(&mut self) -> &mut MutationTransaction<'tx, 'ast> {
        self.0
    }

    /// Read-only AST access.
    pub fn ast(&self) -> &Ast<'ast> {
        self.0.ast()
    }

    /// Read-only semantic access.
    pub fn semantic(&self) -> &Semantic<'ast> {
        self.0.semantic()
    }

    // ── Building and cloning ──

    /// Build detached nodes in the AST arena without attaching them.
    pub fn build_detached<T>(&mut self, build: impl FnOnce(&mut Ast<'ast>) -> T) -> T {
        self.0.build_detached(build)
    }

    /// Build with semantic context.
    pub fn build_detached_with_semantic<T>(
        &mut self,
        build: impl FnOnce(&mut Ast<'ast>, &Semantic<'ast>) -> T,
    ) -> T {
        self.0.build_detached_with_semantic(build)
    }

    /// Clone a subtree into detached storage.
    pub fn clone_detached<N>(&mut self, root: N) -> DetachedClone<N>
    where
        N: Copy + CloneIn<'ast, Cloned = N> + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.clone_detached(root)
    }

    // ── Replacement ──

    /// Replace an attached node preserving identity.
    pub fn replace(&mut self, old: NodeId, new: NodeId, identity: IdentityPolicy) {
        self.0.replace(old, new, identity)
    }

    /// Replace one expression variant with another.
    pub fn replace_expression(&mut self, old: Expr, new: Expr, identity: IdentityPolicy) {
        self.0.replace_expression(old, new, identity)
    }

    /// Replace a statement list.
    pub fn replace_statements(
        &mut self,
        owner: StatementListOwner,
        statements: Vec<Stmt>,
    ) {
        self.0.replace_statements(owner, statements)
    }

    /// Replace a single statement in its list.
    pub fn replace_statement(
        &mut self,
        owner: StatementListOwner,
        old: NodeId,
        replacement: impl IntoIterator<Item = Stmt>,
    ) {
        self.0.replace_statement(owner, old, replacement)
    }

    // ── Transformed clone publishing ──

    /// Publish a transformed clone, preserving the root node type.
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
        self.0.replace_from_transformed_clone(old, clone, root_scope, regions)
    }

    /// Publish a transformed expression clone, allowing the expression variant to change.
    pub fn replace_expression_from_transformed_clone(
        &mut self,
        old: Expr,
        clone: DetachedClone<Expr>,
        root_scope: impl Into<ScopeHandle>,
        regions: &[DetachedScopeRegion],
    ) -> DetachedScopeLayout {
        self.0.replace_expression_from_transformed_clone(old, clone, root_scope, regions)
    }

    /// Publish a transformed statement clone, allowing the statement variant to change.
    pub fn replace_statement_from_transformed_clone(
        &mut self,
        old: Stmt,
        clone: DetachedClone<Stmt>,
        root_scope: impl Into<ScopeHandle>,
        regions: &[DetachedScopeRegion],
    ) -> DetachedScopeLayout {
        self.0.replace_statement_from_transformed_clone(old, clone, root_scope, regions)
    }

    /// Replace a subtree with an exact detached clone (not from transformed).
    pub fn replace_from_detached_clone<N>(&mut self, old: N, clone: DetachedClone<N>)
    where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.replace_from_detached_clone(old, clone)
    }

    /// Duplicate semantics for additional copies of a published subtree.
    pub fn duplicate_semantics_from_detached_clone<N>(
        &mut self,
        copy: &DetachedClone<N>,
        layout: &DetachedScopeLayout,
    ) {
        self.0.duplicate_semantics_from_detached_clone(copy, layout)
    }

    // ── Generated scope regions ──

    /// Create generated scope regions for a subtree.
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
        self.0.create_generated_scope_regions(root, root_scope, existing, creators)
    }

    /// Register a generated subtree using a precomputed scope layout.
    pub fn register_generated_subtree_with_layout<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        layout: &DetachedScopeLayout,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.register_generated_subtree_with_layout(root, parent, layout)
    }

    /// Import a detached generated program.
    pub fn import_generated_program(
        &mut self,
        generated: swc_next_ecma_ast::Program,
        parent: NodeId,
        target_scope: ScopeId,
    ) -> GeneratedProgramImport {
        self.0.import_generated_program(generated, parent, target_scope)
    }

    // ── Statement list operations ──

    /// Insert statements before an anchor.
    pub fn insert_before(
        &mut self,
        owner: StatementListOwner,
        anchor: NodeId,
        inserted: impl IntoIterator<Item = Stmt>,
    ) {
        self.0.insert_before(owner, anchor, inserted)
    }

    /// Insert statements after an anchor.
    pub fn insert_after(
        &mut self,
        owner: StatementListOwner,
        anchor: NodeId,
        inserted: impl IntoIterator<Item = Stmt>,
    ) {
        self.0.insert_after(owner, anchor, inserted)
    }

    /// Remove a statement from its owner and retire its semantics.
    pub fn remove_statement(&mut self, owner: StatementListOwner, removed: NodeId) {
        self.0.remove_statement(owner, removed)
    }

    /// Remove a subtree and retire all its semantics.
    pub fn remove_subtree(&mut self, owner: StatementListOwner, root: NodeId) {
        self.0.remove_subtree(owner, root)
    }

    /// Retire all semantic identities under a root.
    pub fn retire_subtree(&mut self, root: NodeId) {
        self.0.retire_subtree(root)
    }

    /// Move a statement to a new owner before an anchor.
    pub fn move_statement_before(
        &mut self,
        from: StatementListOwner,
        to: StatementListOwner,
        moved: NodeId,
        anchor: NodeId,
        new_scope: ScopeId,
    ) {
        self.0.move_statement_before(from, to, moved, anchor, new_scope)
    }

    /// Move a statement to a new owner after an anchor.
    pub fn move_statement_after(
        &mut self,
        from: StatementListOwner,
        to: StatementListOwner,
        moved: NodeId,
        anchor: NodeId,
        new_scope: ScopeId,
    ) {
        self.0.move_statement_after(from, to, moved, anchor, new_scope)
    }

    /// Move a subtree to a new parent and scope.
    pub fn move_subtree(&mut self, root: NodeId, new_parent: NodeId, new_scope: ScopeId) {
        self.0.move_subtree(root, new_parent, new_scope)
    }

    // ── Node lifecycle ──

    /// Register a node in the semantic tree.
    pub fn register_node(
        &mut self,
        node: NodeId,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) {
        self.0.register_node(node, parent, scope)
    }

    /// Retire a node.
    pub fn retire_node(&mut self, node: NodeId) {
        self.0.retire_node(node)
    }

    /// Register a subtree of entirely new nodes.
    pub(crate) fn register_subtree<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.register_subtree(root, parent, scope)
    }

    /// Register children of a subtree root, skipping the root itself.
    pub(crate) fn register_subtree_children<N>(
        &mut self,
        root: N,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
    ) where
        N: Copy + AstNode + for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.register_subtree_children(root, parent, scope)
    }

    /// Adopt a mixed detached/attached subtree, registering generated nodes
    /// and reparenting existing active nodes.
    pub(crate) fn adopt_generated_subtree<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        scope: impl Into<ScopeHandle>,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.adopt_generated_subtree(root, parent, scope)
    }

    /// Adopt a generated subtree with a precomputed scope layout.
    pub(crate) fn adopt_generated_subtree_with_layout<N>(
        &mut self,
        root: N,
        parent: Option<NodeId>,
        layout: &DetachedScopeLayout,
    ) where
        N: for<'visit> VisitWith<'ast, DetachedSubtreeCollector<'visit, 'ast>>,
    {
        self.0.adopt_generated_subtree_with_layout(root, parent, layout)
    }

    /// Reparent a node.
    pub fn reparent_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        self.0.reparent_node(node, parent)
    }

    /// Rescope a node.
    pub fn rescope_node(&mut self, node: NodeId, scope: impl Into<ScopeHandle>) {
        self.0.rescope_node(node, scope)
    }

    /// Rescope an adopted subtree, reparenting owned scopes and rescoping
    /// non-owned nodes.
    pub fn rescope_adopted_subtree(&mut self, root: NodeId, new_scope: impl Into<ScopeHandle>) {
        self.0.rescope_adopted_subtree(root, new_scope)
    }

    // ── Scope lifecycle ──

    /// Create a new scope.
    pub fn create_scope(
        &mut self,
        parent: impl Into<ScopeHandle>,
        owner: NodeId,
        kind: ScopeKind,
        flags: ScopeFlags,
    ) -> PendingScopeId {
        self.0.create_scope(parent, owner, kind, flags)
    }

    /// Retire a scope.
    pub fn retire_scope(&mut self, scope: ScopeId) {
        self.0.retire_scope(scope)
    }

    /// Retag a scope's kind and flags.
    pub fn retag_scope(&mut self, scope: ScopeId, kind: ScopeKind, flags: ScopeFlags) {
        self.0.retag_scope(scope, kind, flags)
    }

    /// Reparent a scope.
    pub fn reparent_scope(&mut self, scope: ScopeId, parent: impl Into<ScopeHandle>) {
        self.0.reparent_scope(scope, parent)
    }

    /// Reowner a scope.
    pub fn reowner_scope(&mut self, scope: ScopeId, owner: NodeId) {
        self.0.reowner_scope(scope, owner)
    }

    // ── Symbol and declaration lifecycle ──

    /// Declare a symbol.
    pub fn declare(
        &mut self,
        scope: impl Into<ScopeHandle>,
        node: NodeId,
        name: AstStr,
        flags: SymbolFlags,
    ) -> PendingSymbolId {
        self.0.declare(scope, node, name, flags)
    }

    /// Update symbol flags.
    pub fn update_symbol_flags(&mut self, symbol: SymbolId, flags: SymbolFlags) {
        self.0.update_symbol_flags(symbol, flags)
    }

    /// Retire a symbol.
    pub fn retire_symbol(&mut self, symbol: SymbolId) {
        self.0.retire_symbol(symbol)
    }

    /// Move a symbol to a new scope.
    pub fn move_symbol(
        &mut self,
        symbol: SymbolId,
        scope: impl Into<ScopeHandle>,
        hoisting_scopes: impl IntoIterator<Item = ScopeHandle>,
    ) {
        self.0.move_symbol(symbol, scope, hoisting_scopes)
    }

    /// Rename a symbol.
    pub fn rename_symbol(&mut self, symbol: SymbolId, name: AstStr) {
        self.0.rename_symbol(symbol, name)
    }

    /// Retire a declaration.
    pub fn retire_declaration(&mut self, declaration: DeclarationId) {
        self.0.retire_declaration(declaration)
    }

    /// Retarget a declaration node.
    pub fn retarget_declaration_node(&mut self, declaration: DeclarationId, node: NodeId) {
        self.0.retarget_declaration_node(declaration, node)
    }

    /// Update declaration flags.
    pub fn update_declaration_flags(&mut self, declaration: DeclarationId, flags: SymbolFlags) {
        self.0.update_declaration_flags(declaration, flags)
    }

    // ── Reference lifecycle ──

    /// Create a reference.
    pub fn reference(
        &mut self,
        scope: impl Into<ScopeHandle>,
        node: NodeId,
        name: AstStr,
        flags: ReferenceFlags,
        symbol: Option<impl Into<SymbolHandle>>,
    ) -> PendingReferenceId {
        self.0.reference(scope, node, name, flags, symbol)
    }

    /// Retire a reference.
    pub fn retire_reference(&mut self, reference: ReferenceId) {
        self.0.retire_reference(reference)
    }

    /// Retarget a reference node.
    pub fn retarget_reference_node(&mut self, reference: ReferenceId, node: NodeId) {
        self.0.retarget_reference_node(reference, node)
    }

    /// Rename a reference.
    pub fn rename_reference(&mut self, reference: ReferenceId, name: AstStr) {
        self.0.rename_reference(reference, name)
    }

    /// Rebind a reference.
    pub fn rebind_reference(
        &mut self,
        reference: ReferenceId,
        symbol: Option<impl Into<SymbolHandle>>,
    ) {
        self.0.rebind_reference(reference, symbol)
    }

    /// Rescope a reference.
    pub fn rescope_reference(&mut self, reference: ReferenceId, scope: impl Into<ScopeHandle>) {
        self.0.rescope_reference(reference, scope)
    }

    /// Update reference flags.
    pub fn update_reference_flags(&mut self, reference: ReferenceId, flags: ReferenceFlags) {
        self.0.update_reference_flags(reference, flags)
    }

    // ── Module records ──

    /// Retarget default export local.
    pub fn retarget_default_export_local(
        &mut self,
        statement: NodeId,
        binding: NodeId,
        symbol: impl Into<SymbolHandle>,
    ) {
        self.0.retarget_default_export_local(statement, binding, symbol)
    }

    /// Replace a namespace reexport.
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
        self.0.replace_namespace_reexport(old_statement, statement, specifier, local, exported, symbol, reference)
    }

    // ── Private bindings ──

    /// Create a private binding.
    pub fn declare_private(
        &mut self,
        scope: ScopeId,
        base: &str,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        self.0.declare_private(scope, base, flags)
    }

    /// Create a private binding with a minimum suffix.
    pub fn declare_private_with_min_suffix(
        &mut self,
        scope: ScopeId,
        base: &str,
        min_suffix: u64,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        self.0.declare_private_with_min_suffix(scope, base, min_suffix, flags)
    }

    /// Create a shared private binding (deduplicated by key).
    pub fn declare_shared_private(
        &mut self,
        scope: ScopeId,
        key: &str,
        base: &str,
        flags: SymbolFlags,
    ) -> PrivateBinding {
        self.0.declare_shared_private(scope, key, base, flags)
    }

    /// Reserve a private name.
    pub fn reserve_private_name(&mut self, scope: ScopeId, base: &str) -> ReservedPrivateName {
        self.0.reserve_private_name(scope, base)
    }

    /// Reserve a private name in a specific collision scope.
    pub fn reserve_private_name_in(
        &mut self,
        collision_scope: ScopeId,
        reservation_scope: impl Into<ScopeHandle>,
        base: &str,
    ) -> ReservedPrivateName {
        self.0.reserve_private_name_in(collision_scope, reservation_scope, base)
    }

    /// Reserve a generated name exactly.
    pub fn reserve_generated_name_exact(&mut self, scope: ScopeId, name: AstStr) {
        self.0.reserve_generated_name_exact(scope, name)
    }

    /// Attach a private declaration.
    pub fn attach_private_declaration(
        &mut self,
        binding: PrivateBinding,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
    ) {
        self.0.attach_private_declaration(binding, parent, scope)
    }

    /// Create a reference to a private binding.
    pub fn private_reference(
        &mut self,
        binding: PrivateBinding,
        parent: NodeId,
        scope: impl Into<ScopeHandle>,
        flags: ReferenceFlags,
    ) -> IdentifierReference {
        self.0.private_reference(binding, parent, scope, flags)
    }
}

pub(crate) type PrivateName = ReservedPrivateName;