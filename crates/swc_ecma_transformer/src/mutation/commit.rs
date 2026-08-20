use std::collections::{HashMap, HashSet};

use swc_next_ecma_ast::{AstStr, ClassType, NodeId, NodeKind, NodeKindData, ScopeId, SymbolId};
use swc_next_ecma_semantic::{ExportOrigin, SemanticNodeState};

use super::{
    IdentityPolicy, MutationCommit, MutationError, MutationTransaction, PendingScopeId,
    ScopeHandle, SymbolHandle,
    journal::{
        AstEdit, IdentityEdit, ModuleEdit, ReplacementCategory, ResolutionEdit, ScopeEdit,
        SemanticEdit,
    },
};

impl MutationTransaction<'_, '_> {
    pub fn commit(mut self) -> Result<MutationCommit, MutationError> {
        self.preflight()?;
        let dirty_roots = self.normalized_dirty_roots();
        let mut semantic = self.semantic.mutation();

        if self.journal.is_empty() {
            self.ast
                .truncate(self.ast_node_checkpoint, self.ast_extra_checkpoint);
            self.committed = true;
            return Ok(MutationCommit {
                epoch: self.ast.mutation_epoch(),
                dirty_roots,
                scopes: Vec::new(),
                symbols: Vec::new(),
                references: Vec::new(),
            });
        }

        // Retire semantic identities before their AST storage becomes
        // unreachable. Preflight guarantees these calls cannot fail.
        for edit in &self.journal.identity_edits {
            if let IdentityEdit::RetireReference { reference } = *edit {
                semantic.retire_reference(reference);
            }
        }
        for edit in &self.journal.identity_edits {
            if let IdentityEdit::RetireDeclaration { declaration } = *edit {
                semantic.retire_declaration(declaration);
            }
        }
        for edit in &self.journal.identity_edits {
            if let IdentityEdit::RetireSymbol { symbol } = *edit {
                semantic.retire_symbol(symbol);
            }
        }
        for edit in &self.journal.ast_edits {
            match *edit {
                AstEdit::Replace {
                    old,
                    new,
                    identity: IdentityPolicy::Preserve,
                    ..
                } => {
                    // SAFETY: preflight verifies equal concrete kinds or the
                    // explicit ClassExpression-to-Function/Call transitions.
                    // These retain a NodeId stored behind the expression union
                    // edge.
                    // The detached root is intentionally left unregistered;
                    // its payload may refer to earlier detached descendants.
                    //
                    // When the old payload was a class, generator, or async
                    // function, the post-order transform-fact entry is now
                    // stale — remove it.
                    // Note: we do NOT call add_to_post_order_lists here because
                    // set_data swaps the old/new payloads, meaning the old
                    // NodeId now holds the replacement data.  Post-order lists
                    // are populated from the initial semantic traversal and
                    // from RegisterNode entries for freshly-created nodes.
                    // The replacement node's data was already part of the
                    // initial traversal (it was a detached node) and any
                    // class/func/async entries it created during the initial
                    // registration are still in the post-order lists.
                    semantic.remove_from_post_order_lists(old);
                    unsafe { self.ast.set_data(old, new) };
                }
                AstEdit::Replace { .. } => unreachable!("preflight rejects unsupported identity"),
            }
        }

        let mut scopes = Vec::with_capacity(self.journal.pending_scopes.len());
        for pending in &self.journal.pending_scopes {
            let parent = resolve_scope(pending.parent, &scopes);
            scopes.push(semantic.create_scope(
                self.ast,
                parent,
                pending.owner,
                pending.kind,
                pending.flags,
            ));
        }

        for edit in &self.journal.scope_edits {
            match *edit {
                ScopeEdit::Retag { scope, kind, flags } => {
                    semantic.retag_scope(scope, kind, flags);
                }
                ScopeEdit::Reowner { .. } => {}
                ScopeEdit::Reparent { scope, parent } => {
                    semantic.reparent_scope(scope, resolve_scope(parent, &scopes));
                }
                ScopeEdit::Retire { .. } => {}
            }
        }

        for edit in &self.journal.semantic_edits {
            match *edit {
                SemanticEdit::RegisterNode {
                    node,
                    parent,
                    scope,
                } => {
                    semantic.register_node(node, parent, resolve_scope(scope, &scopes));
                }
                SemanticEdit::RetireNode { .. } => {}
                SemanticEdit::ReparentNode { node, parent } => {
                    semantic.reparent_node(node, parent);
                }
                SemanticEdit::RescopeNode { node, scope } => {
                    semantic.rescope_node(node, resolve_scope(scope, &scopes));
                }
            }
        }

        // After all nodes are registered, add class/generator/async nodes
        // to the post-order transform-fact lists.  This is deferred so that
        // method function scopes are available when the class fact is built
        // (add_to_post_order_lists reads node_scopes for each method).
        for edit in &self.journal.semantic_edits {
            if let SemanticEdit::RegisterNode { node, .. } = *edit {
                semantic.add_to_post_order_lists(self.ast, node);
            }
        }

        for edit in &self.journal.scope_edits {
            if let ScopeEdit::Reowner { scope, owner } = *edit {
                semantic.reowner_scope(scope, owner);
            }
        }

        for reservation in &self.journal.name_reservations {
            semantic.reserve_generated_name_exact(
                self.ast,
                resolve_scope(reservation.scope, &scopes),
                reservation.name,
            );
        }

        let mut symbols = Vec::with_capacity(self.journal.pending_symbols.len());
        for pending in &self.journal.pending_symbols {
            symbols.push(semantic.declare(
                self.ast,
                resolve_scope(pending.scope, &scopes),
                pending.node,
                pending.name,
                pending.flags,
            ));
        }

        for edit in &self.journal.module_edits {
            match *edit {
                ModuleEdit::RetargetNode { old, new } => {
                    semantic.retarget_module_node(old, new);
                }
                ModuleEdit::RetargetDefaultLocal {
                    statement,
                    binding,
                    symbol,
                } => {
                    let symbol = resolve_symbol(symbol, &symbols);
                    let declaration = semantic
                        .declaration_of(binding)
                        .expect("module target binding must have a declaration");
                    semantic.retarget_default_export_local(statement, symbol, declaration);
                }
                ModuleEdit::ReplaceNamespaceReexport { .. } => {}
            }
        }

        for edit in &self.journal.resolution_edits {
            match edit {
                ResolutionEdit::RetargetDeclarationNode { declaration, node } => {
                    semantic.retarget_declaration_node(*declaration, *node);
                }
                ResolutionEdit::RetargetReferenceNode { reference, node } => {
                    semantic.retarget_reference_node(*reference, *node);
                }
                ResolutionEdit::UpdateDeclarationFlags { declaration, flags } => {
                    semantic.update_declaration_flags(*declaration, *flags);
                }
                ResolutionEdit::UpdateSymbolFlags { symbol, flags } => {
                    semantic.update_symbol_flags(*symbol, *flags);
                }
                ResolutionEdit::MoveSymbol {
                    symbol,
                    scope,
                    hoisting_scopes,
                    ..
                } => {
                    let hoisting_scopes: Vec<_> = hoisting_scopes
                        .iter()
                        .map(|scope| resolve_scope(*scope, &scopes))
                        .collect();
                    semantic.move_symbol(
                        self.ast,
                        *symbol,
                        resolve_scope(*scope, &scopes),
                        &hoisting_scopes,
                    );
                }
                ResolutionEdit::RenameSymbol { symbol, name } => {
                    semantic.rename_symbol(self.ast, *symbol, *name);
                }
                ResolutionEdit::RenameReference { reference, name } => {
                    semantic.rename_reference(self.ast, *reference, *name);
                }
                ResolutionEdit::RebindReference { reference, symbol } => {
                    semantic.rebind_reference(
                        *reference,
                        symbol.map(|symbol| resolve_symbol(symbol, &symbols)),
                    );
                }
                ResolutionEdit::RescopeReference { reference, scope } => {
                    semantic.rescope_reference(
                        self.ast,
                        *reference,
                        resolve_scope(*scope, &scopes),
                    );
                }
                ResolutionEdit::UpdateReferenceFlags { reference, flags } => {
                    semantic.update_reference_flags(*reference, *flags);
                }
            }
        }

        let mut references = Vec::with_capacity(self.journal.pending_references.len());
        for pending in &self.journal.pending_references {
            let symbol = pending
                .symbol
                .map(|symbol| resolve_symbol(symbol, &symbols));
            references.push(semantic.reference_generated(
                self.ast,
                resolve_scope(pending.scope, &scopes),
                pending.node,
                pending.name,
                pending.flags,
                symbol,
            ));
        }

        for edit in &self.journal.module_edits {
            if let ModuleEdit::ReplaceNamespaceReexport {
                old_statement,
                statement,
                specifier,
                local,
                exported,
                symbol,
                reference,
            } = *edit
            {
                semantic.replace_namespace_reexport(
                    self.ast,
                    swc_next_ecma_semantic::NamespaceReexportReplacement {
                        old_statement,
                        statement,
                        specifier,
                        local,
                        exported,
                        symbol: resolve_symbol(symbol, &symbols),
                        reference: references[reference.0],
                    },
                );
            }
        }

        // Collect parent nodes whose aggregated counts need recomputation
        // after a child is retired or replaced.  RetireNode removes the child
        // from the parent's children list, and Replace changes the node kind
        // (e.g. Class → Function), so the per-node and ancestor counts
        // computed from the old kind are stale.
        let retired_parents: Vec<NodeId> = {
            let mut parents = Vec::new();
            for edit in &self.journal.semantic_edits {
                if let SemanticEdit::RetireNode { node } = *edit
                    && semantic.is_node_active(node)
                    && let Some(parent) = semantic.parent_of(node)
                {
                    parents.push(parent);
                }
            }
            for edit in &self.journal.ast_edits {
                let AstEdit::Replace { old, .. } = *edit;
                if semantic.is_node_active(old)
                    && let Some(parent) = semantic.parent_of(old)
                {
                    parents.push(parent);
                }
            }
            parents
        };
        for edit in &self.journal.semantic_edits {
            if let SemanticEdit::RetireNode { node } = *edit {
                semantic.retire_node(node);
            }
        }
        // Recompute effect/anchor counts for parents that lost retired children.
        for parent in &retired_parents {
            let mut current = *parent;
            loop {
                // parent_of asserts active; skip if the node itself was retired.
                if !semantic.is_node_active(current) {
                    break;
                }
                semantic.recompute_after_retire(self.ast, current);
                if let Some(p) = semantic.parent_of(current) {
                    current = p;
                } else {
                    break;
                }
            }
        }

        for edit in &self.journal.scope_edits {
            if let ScopeEdit::Retire { scope } = *edit {
                semantic.retire_scope(scope);
            }
        }

        let ast_epoch = self.ast.advance_mutation_epoch();
        let semantic_epoch = semantic.advance_mutation_epoch();
        assert_eq!(ast_epoch, semantic_epoch);
        self.committed = true;

        Ok(MutationCommit {
            epoch: ast_epoch,
            dirty_roots,
            scopes,
            symbols,
            references,
        })
    }

    fn preflight(&self) -> Result<(), MutationError> {
        if self.ast.mutation_epoch() != self.semantic.mutation_epoch() {
            return Err(MutationError::EpochMismatch {
                ast: self.ast.mutation_epoch(),
                semantic: self.semantic.mutation_epoch(),
            });
        }
        if !self.semantic.is_node_active(self.root) {
            return Err(MutationError::InactiveRoot(self.root));
        }

        let mut replaced = HashSet::new();
        for edit in &self.journal.ast_edits {
            match *edit {
                AstEdit::Replace {
                    old,
                    new,
                    identity,
                    category,
                } => {
                    if identity != IdentityPolicy::Preserve {
                        return Err(MutationError::UnsupportedIdentityPolicy(identity));
                    }
                    if !self.semantic.is_node_active(old) {
                        return Err(MutationError::InactiveNode(old));
                    }
                    if self.semantic.node_state(new) != SemanticNodeState::Unregistered {
                        return Err(MutationError::AttachedReplacement(new));
                    }
                    if old.index() >= self.ast.node_count() {
                        return Err(MutationError::InvalidNodeId(old));
                    }
                    if old == new || new.index() >= self.ast.node_count() {
                        return Err(MutationError::InvalidNodeId(new));
                    }
                    if new.index() < self.ast_node_checkpoint {
                        return Err(MutationError::ReplacementIsNotDetached(new));
                    }
                    let kind_compatible = matches!(
                        category,
                        ReplacementCategory::Expression | ReplacementCategory::Statement
                    ) || self.ast.node_kind(old) == self.ast.node_kind(new)
                        || matches!(
                            self.ast.kind_data(old),
                            NodeKindData::Class(class)
                                if class.class_type(self.ast) == ClassType::ClassExpression
                                    && matches!(
                                        self.ast.node_kind(new),
                                        NodeKind::Function | NodeKind::CallExpression
                                    )
                        );
                    if !kind_compatible {
                        return Err(MutationError::KindMismatch { old, new });
                    }
                    if !replaced.insert(old) {
                        return Err(MutationError::ConflictingNodeEdit(old));
                    }
                }
            }
        }

        let mut register_order = HashMap::new();
        for (index, edit) in self.journal.semantic_edits.iter().enumerate() {
            if let SemanticEdit::RegisterNode { node, .. } = *edit
                && register_order.insert(node, index).is_some()
            {
                return Err(MutationError::ConflictingNodeEdit(node));
            }
        }

        let mut retired_references = HashSet::new();
        let mut retired_declarations = HashSet::new();
        let mut retired_symbols = HashSet::new();
        for edit in &self.journal.identity_edits {
            match *edit {
                IdentityEdit::RetireReference { reference } => {
                    if !self.semantic.is_reference_active(reference) {
                        return Err(MutationError::InactiveReference(reference));
                    }
                    if !retired_references.insert(reference) {
                        return Err(MutationError::InactiveReference(reference));
                    }
                }
                IdentityEdit::RetireDeclaration { declaration } => {
                    if !self.semantic.is_declaration_active(declaration) {
                        return Err(MutationError::InactiveDeclaration(declaration));
                    }
                    if !retired_declarations.insert(declaration) {
                        return Err(MutationError::InactiveDeclaration(declaration));
                    }
                }
                IdentityEdit::RetireSymbol { symbol } => {
                    if !self.semantic.is_symbol_active(symbol) {
                        return Err(MutationError::InactiveSymbol(symbol));
                    }
                    if !retired_symbols.insert(symbol) {
                        return Err(MutationError::InactiveSymbol(symbol));
                    }
                }
            }
        }
        for symbol in &retired_symbols {
            let all_references_retired = self
                .semantic
                .uses(*symbol)
                .iter()
                .all(|reference| retired_references.contains(reference));
            let all_declarations_retired = self.semantic.declarations(*symbol).iter().all(|decl| {
                self.semantic
                    .declaration_of(decl.node)
                    .is_some_and(|id| retired_declarations.contains(&id))
            });
            if !all_references_retired || !all_declarations_retired {
                return Err(MutationError::SymbolStillInUse(*symbol));
            }
        }

        let relocated_declarations: HashSet<_> = self
            .journal
            .resolution_edits
            .iter()
            .filter_map(|edit| match *edit {
                ResolutionEdit::RetargetDeclarationNode { declaration, .. } => Some(declaration),
                _ => None,
            })
            .collect();
        let relocated_references: HashSet<_> = self
            .journal
            .resolution_edits
            .iter()
            .filter_map(|edit| match *edit {
                ResolutionEdit::RetargetReferenceNode { reference, .. } => Some(reference),
                _ => None,
            })
            .collect();

        let mut retired = HashSet::new();
        let node_retire_order: HashMap<_, _> = self
            .journal
            .semantic_edits
            .iter()
            .enumerate()
            .filter_map(|(index, edit)| match *edit {
                SemanticEdit::RetireNode { node } => Some((node, index)),
                _ => None,
            })
            .collect();
        let mut registered_nodes = HashSet::new();
        let mut reparented_nodes = HashMap::new();
        let mut rescoped_nodes = HashSet::new();
        for (index, edit) in self.journal.semantic_edits.iter().enumerate() {
            match *edit {
                SemanticEdit::RegisterNode {
                    node,
                    parent,
                    scope,
                } => {
                    if self.semantic.node_state(node) != SemanticNodeState::Unregistered {
                        return Err(MutationError::NodeAlreadyRegistered(node));
                    }
                    if node.index() >= self.ast.node_count() {
                        return Err(MutationError::InvalidNodeId(node));
                    }
                    self.validate_parent(parent, index, &register_order)?;
                    self.validate_scope(scope)?;
                    if !registered_nodes.insert(node) {
                        return Err(MutationError::ConflictingNodeEdit(node));
                    }
                }
                SemanticEdit::RetireNode { node } => {
                    self.validate_active_node(node)?;
                    if node == self.root {
                        return Err(MutationError::InactiveRoot(node));
                    }
                    if self.semantic.declaration_of(node).is_some_and(|id| {
                        !retired_declarations.contains(&id) && !relocated_declarations.contains(&id)
                    }) || self.semantic.reference_of(node).is_some_and(|id| {
                        !retired_references.contains(&id) && !relocated_references.contains(&id)
                    }) {
                        return Err(MutationError::NodeHasSemanticIdentity(node));
                    }
                    if !retired.insert(node) {
                        return Err(MutationError::ConflictingNodeEdit(node));
                    }
                }
                SemanticEdit::ReparentNode { node, parent } => {
                    self.validate_active_node(node)?;
                    self.validate_parent(parent, index, &register_order)?;
                    if reparented_nodes.insert(node, parent).is_some() {
                        return Err(MutationError::ConflictingNodeEdit(node));
                    }
                }
                SemanticEdit::RescopeNode { node, scope } => {
                    self.validate_active_node(node)?;
                    self.validate_scope(scope)?;
                    if !rescoped_nodes.insert(node) {
                        return Err(MutationError::ConflictingNodeEdit(node));
                    }
                }
            }
        }

        let mut module_statements = HashSet::new();
        let mut retargeted_module_nodes = HashSet::new();
        for edit in &self.journal.module_edits {
            match *edit {
                ModuleEdit::RetargetNode { old, new } => {
                    self.validate_active_node(old)?;
                    self.validate_available_node(new, &register_order, &retired)?;
                    if !retargeted_module_nodes.insert(old) {
                        return Err(MutationError::ConflictingModuleEdit(old));
                    }
                }
                ModuleEdit::RetargetDefaultLocal {
                    statement,
                    binding,
                    symbol,
                } => {
                    if !module_statements.insert(statement) {
                        return Err(MutationError::ConflictingModuleEdit(statement));
                    }
                    if !self.semantic.is_node_active(statement)
                        || !self.semantic.module_record().exports().iter().any(|entry| {
                            entry.statement == statement
                                && entry.origin == ExportOrigin::DefaultDeclaration
                        })
                    {
                        return Err(MutationError::MissingModuleExport(statement));
                    }
                    self.validate_available_node(binding, &register_order, &retired)?;
                    self.validate_symbol(symbol)?;
                    if matches!(symbol, SymbolHandle::Existing(symbol) if retired_symbols.contains(&symbol))
                    {
                        return Err(MutationError::InactiveSymbol(match symbol {
                            SymbolHandle::Existing(symbol) => symbol,
                            SymbolHandle::Pending(_) => unreachable!(),
                        }));
                    }
                }
                ModuleEdit::ReplaceNamespaceReexport {
                    old_statement,
                    statement,
                    specifier,
                    local,
                    exported,
                    symbol,
                    reference,
                } => {
                    if !module_statements.insert(old_statement)
                        || !self.semantic.module_record().exports().iter().any(|entry| {
                            entry.statement == old_statement
                                && entry.origin == ExportOrigin::NamespaceReExport
                        })
                    {
                        return Err(MutationError::MissingModuleExport(old_statement));
                    }
                    for node in [statement, specifier, local, exported] {
                        self.validate_available_node(node, &register_order, &retired)?;
                    }
                    self.validate_symbol(symbol)?;
                    if reference.0 >= self.journal.pending_references.len()
                        || self.journal.pending_references[reference.0].node != local
                    {
                        return Err(MutationError::ConflictingModuleEdit(old_statement));
                    }
                }
            }
        }
        if let Some(node) = registered_nodes.iter().chain(retired.iter()).find(|node| {
            (registered_nodes.contains(node) && retired.contains(node))
                || (registered_nodes.contains(node) && reparented_nodes.contains_key(node))
                || (registered_nodes.contains(node) && rescoped_nodes.contains(node))
                || (retired.contains(node) && reparented_nodes.contains_key(node))
                || (retired.contains(node) && rescoped_nodes.contains(node))
        }) {
            return Err(MutationError::ConflictingNodeEdit(*node));
        }

        let registered_parents: HashMap<_, _> = self
            .journal
            .semantic_edits
            .iter()
            .filter_map(|edit| match *edit {
                SemanticEdit::RegisterNode { node, parent, .. } => Some((node, parent)),
                _ => None,
            })
            .collect();
        for (node, parent) in &reparented_nodes {
            let mut current = *parent;
            let mut visited = HashSet::new();
            while let Some(candidate) = current {
                if candidate == *node || !visited.insert(candidate) {
                    return Err(MutationError::NodeCycle(*node));
                }
                current = if let Some(parent) = reparented_nodes.get(&candidate) {
                    *parent
                } else if let Some(parent) = registered_parents.get(&candidate) {
                    *parent
                } else if self.semantic.is_node_active(candidate) {
                    self.semantic.parent_of(candidate)
                } else {
                    None
                };
            }
        }

        for (retired_node, retire_index) in &node_retire_order {
            for child in self.semantic.node_children(*retired_node) {
                let child_retired_first = node_retire_order
                    .get(&child)
                    .is_some_and(|child_index| child_index < retire_index);
                let child_reparented = reparented_nodes
                    .get(&child)
                    .is_some_and(|parent| *parent != Some(*retired_node));
                if !child_retired_first && !child_reparented {
                    return Err(MutationError::ConflictingNodeEdit(*retired_node));
                }
            }
        }

        if replaced.iter().any(|node| retired.contains(node)) {
            return Err(MutationError::ConflictingNodeEdit(
                *replaced.iter().find(|node| retired.contains(node)).unwrap(),
            ));
        }

        for (index, pending) in self.journal.pending_scopes.iter().enumerate() {
            self.validate_scope(pending.parent)?;
            if matches!(pending.parent, ScopeHandle::Pending(parent) if parent.0 >= index) {
                return Err(MutationError::InvalidPendingScope(PendingScopeId(index)));
            }
            self.validate_available_node(pending.owner, &register_order, &retired)?;
        }

        let mut retagged_scopes = HashSet::new();
        let mut reowned_scopes = HashSet::new();
        let mut reparented_scopes = HashMap::new();
        let mut retired_scopes = HashMap::new();
        for (index, edit) in self.journal.scope_edits.iter().enumerate() {
            let scope = match *edit {
                ScopeEdit::Retag { scope, .. } => {
                    if !retagged_scopes.insert(scope) {
                        return Err(MutationError::ConflictingScopeEdit(scope));
                    }
                    scope
                }
                ScopeEdit::Reowner { scope, owner } => {
                    if !reowned_scopes.insert(scope) {
                        return Err(MutationError::ConflictingScopeEdit(scope));
                    }
                    self.validate_available_node(owner, &register_order, &retired)?;
                    scope
                }
                ScopeEdit::Reparent { scope, parent } => {
                    self.validate_scope(parent)?;
                    if reparented_scopes.insert(scope, parent).is_some() {
                        return Err(MutationError::ConflictingScopeEdit(scope));
                    }
                    scope
                }
                ScopeEdit::Retire { scope } => {
                    if retired_scopes.insert(scope, index).is_some() {
                        return Err(MutationError::ConflictingScopeEdit(scope));
                    }
                    scope
                }
            };
            if scope == ScopeId::ROOT {
                return Err(MutationError::RootScopeMutation);
            }
            if !self.semantic.is_scope_active(scope) {
                return Err(MutationError::InactiveScope(scope));
            }
        }
        for scope in retired_scopes.keys() {
            if retagged_scopes.contains(scope)
                || reowned_scopes.contains(scope)
                || reparented_scopes.contains_key(scope)
            {
                return Err(MutationError::ConflictingScopeEdit(*scope));
            }
        }
        for (scope, parent) in &reparented_scopes {
            self.validate_scope_reparent(*scope, *parent, &reparented_scopes)?;
        }
        for (scope, retire_index) in &retired_scopes {
            if self.semantic.scope_indexed_symbols(*scope).any(|symbol| {
                let moved_out = self.journal.resolution_edits.iter().any(|edit| {
                    matches!(
                        edit,
                        ResolutionEdit::MoveSymbol {
                            symbol: moved,
                            scope: target,
                            ..
                        } if *moved == symbol && *target != ScopeHandle::Existing(*scope)
                    )
                });
                !retired_symbols.contains(&symbol) && !moved_out
            }) {
                return Err(MutationError::ScopeNotEmpty(*scope));
            }
            for child in self.semantic.scope_children(*scope) {
                let child_reparented = reparented_scopes
                    .get(&child)
                    .is_some_and(|parent| *parent != ScopeHandle::Existing(*scope));
                let child_retired_first = retired_scopes
                    .get(&child)
                    .is_some_and(|child_index| child_index < retire_index);
                if !child_reparented && !child_retired_first {
                    return Err(MutationError::ScopeNotEmpty(*scope));
                }
            }
            for node in self.semantic.nodes_in_scope(*scope) {
                let moved = self.journal.semantic_edits.iter().any(|edit| {
                    matches!(
                        *edit,
                        SemanticEdit::RescopeNode { node: moved, scope: target }
                            if moved == node && target != ScopeHandle::Existing(*scope)
                    )
                });
                if !retired.contains(&node) && !moved {
                    return Err(MutationError::ScopeNotEmpty(*scope));
                }
            }
        }

        let mut final_symbols = HashMap::new();
        let mut retargeted_declarations = HashSet::new();
        let mut retargeted_references = HashSet::new();
        let mut retargeted_declaration_nodes = HashSet::new();
        let mut retargeted_reference_nodes = HashSet::new();
        for edit in &self.journal.resolution_edits {
            match edit {
                ResolutionEdit::RetargetDeclarationNode { declaration, node } => {
                    if !self.semantic.is_declaration_active(*declaration)
                        || retired_declarations.contains(declaration)
                    {
                        return Err(MutationError::InactiveDeclaration(*declaration));
                    }
                    self.validate_available_node(*node, &register_order, &retired)?;
                    if !retargeted_declarations.insert(*declaration)
                        || !retargeted_declaration_nodes.insert(*node)
                        || retargeted_reference_nodes.contains(node)
                    {
                        return Err(MutationError::ConflictingNodeEdit(*node));
                    }
                    if self.semantic.is_node_active(*node)
                        && self.semantic.declaration_of(*node).is_some()
                        && self.semantic.declaration(*declaration).node != *node
                    {
                        return Err(MutationError::NodeHasSemanticIdentity(*node));
                    }
                }
                ResolutionEdit::RetargetReferenceNode { reference, node } => {
                    if !self.semantic.is_reference_active(*reference)
                        || retired_references.contains(reference)
                    {
                        return Err(MutationError::InactiveReference(*reference));
                    }
                    self.validate_available_node(*node, &register_order, &retired)?;
                    if !retargeted_references.insert(*reference)
                        || !retargeted_reference_nodes.insert(*node)
                        || retargeted_declaration_nodes.contains(node)
                    {
                        return Err(MutationError::ConflictingNodeEdit(*node));
                    }
                    if self.semantic.is_node_active(*node)
                        && self.semantic.reference_of(*node).is_some()
                        && self.semantic.reference(*reference).node != *node
                    {
                        return Err(MutationError::NodeHasSemanticIdentity(*node));
                    }
                }
                ResolutionEdit::UpdateDeclarationFlags { declaration, .. } => {
                    if !self.semantic.is_declaration_active(*declaration)
                        || retired_declarations.contains(declaration)
                    {
                        return Err(MutationError::InactiveDeclaration(*declaration));
                    }
                }
                ResolutionEdit::UpdateSymbolFlags { symbol, .. }
                | ResolutionEdit::MoveSymbol { symbol, .. }
                | ResolutionEdit::RenameSymbol { symbol, .. } => {
                    if !self.semantic.is_symbol_active(*symbol) || retired_symbols.contains(symbol)
                    {
                        return Err(MutationError::InactiveSymbol(*symbol));
                    }
                    let current = self.semantic.symbol(*symbol);
                    let entry = final_symbols
                        .entry(*symbol)
                        .or_insert((ScopeHandle::Existing(current.scope), current.name));
                    match edit {
                        ResolutionEdit::MoveSymbol {
                            scope,
                            hoisting_scopes,
                            closed_subtree,
                            ..
                        } => {
                            self.validate_scope(*scope)?;
                            if matches!(*scope, ScopeHandle::Existing(scope) if retired_scopes.contains_key(&scope))
                            {
                                let ScopeHandle::Existing(scope) = *scope else {
                                    unreachable!()
                                };
                                return Err(MutationError::ScopeNotEmpty(scope));
                            }
                            for scope in hoisting_scopes {
                                self.validate_scope(*scope)?;
                            }
                            if let Some(nodes) = closed_subtree
                                && let Some(reference) = self
                                    .semantic
                                    .uses(*symbol)
                                    .iter()
                                    .copied()
                                    .find(|reference| {
                                        !nodes.contains(&self.semantic.reference(*reference).node)
                                    })
                            {
                                return Err(MutationError::ExternalSymbolUse {
                                    symbol: *symbol,
                                    reference,
                                });
                            }
                            entry.0 = *scope;
                        }
                        ResolutionEdit::RenameSymbol { name, .. } => entry.1 = *name,
                        ResolutionEdit::UpdateSymbolFlags { .. } => {}
                        ResolutionEdit::UpdateDeclarationFlags { .. } => unreachable!(),
                        _ => unreachable!(),
                    }
                }
                ResolutionEdit::RenameReference { reference, .. }
                | ResolutionEdit::RebindReference { reference, .. }
                | ResolutionEdit::RescopeReference { reference, .. }
                | ResolutionEdit::UpdateReferenceFlags { reference, .. } => {
                    if !self.semantic.is_reference_active(*reference)
                        || retired_references.contains(reference)
                    {
                        return Err(MutationError::InactiveReference(*reference));
                    }
                    if let ResolutionEdit::RebindReference {
                        symbol: Some(symbol),
                        ..
                    } = edit
                    {
                        self.validate_symbol(*symbol)?;
                        if matches!(*symbol, SymbolHandle::Existing(symbol) if retired_symbols.contains(&symbol))
                        {
                            let SymbolHandle::Existing(symbol) = *symbol else {
                                unreachable!()
                            };
                            return Err(MutationError::InactiveSymbol(symbol));
                        }
                    }
                    if let ResolutionEdit::RescopeReference { scope, .. } = edit {
                        self.validate_scope(*scope)?;
                        if matches!(*scope, ScopeHandle::Existing(scope) if retired_scopes.contains_key(&scope))
                        {
                            let ScopeHandle::Existing(scope) = *scope else {
                                unreachable!()
                            };
                            return Err(MutationError::ScopeNotEmpty(scope));
                        }
                    }
                }
            }
        }
        let mut final_bindings = HashSet::new();
        for (symbol, (scope, name)) in &final_symbols {
            if !final_bindings.insert((*scope, *name)) {
                return Err(MutationError::ConflictingSymbolEdit(*symbol));
            }
            if let ScopeHandle::Existing(scope) = *scope
                && let Some(existing) = self.semantic.binding(scope, *name)
                && existing != *symbol
                && !retired_symbols.contains(&existing)
                && final_symbols
                    .get(&existing)
                    .is_none_or(|target| *target == (ScopeHandle::Existing(scope), *name))
            {
                return Err(MutationError::BindingConflict {
                    scope: ScopeHandle::Existing(scope),
                    name: *name,
                });
            }
        }

        let targets_retired_scope = |scope: ScopeHandle| matches!(scope, ScopeHandle::Existing(scope) if retired_scopes.contains_key(&scope));
        for edit in &self.journal.semantic_edits {
            if matches!(
                *edit,
                SemanticEdit::RegisterNode { scope, .. }
                    | SemanticEdit::RescopeNode { scope, .. }
                    if targets_retired_scope(scope)
            ) {
                let scope = match *edit {
                    SemanticEdit::RegisterNode { scope, .. }
                    | SemanticEdit::RescopeNode { scope, .. } => scope,
                    _ => unreachable!(),
                };
                let ScopeHandle::Existing(scope) = scope else {
                    unreachable!()
                };
                return Err(MutationError::ScopeNotEmpty(scope));
            }
        }
        for pending in &self.journal.pending_scopes {
            if targets_retired_scope(pending.parent) {
                let ScopeHandle::Existing(scope) = pending.parent else {
                    unreachable!()
                };
                return Err(MutationError::ScopeNotEmpty(scope));
            }
        }

        for reservation in &self.journal.name_reservations {
            self.validate_scope(reservation.scope)?;
            if targets_retired_scope(reservation.scope) {
                let ScopeHandle::Existing(scope) = reservation.scope else {
                    unreachable!()
                };
                return Err(MutationError::ScopeNotEmpty(scope));
            }
        }

        let mut declaration_nodes = HashSet::new();
        let mut pending_bindings: HashSet<(ScopeHandle, AstStr)> = HashSet::new();
        for pending in &self.journal.pending_symbols {
            self.validate_scope(pending.scope)?;
            if targets_retired_scope(pending.scope) {
                let ScopeHandle::Existing(scope) = pending.scope else {
                    unreachable!()
                };
                return Err(MutationError::ScopeNotEmpty(scope));
            }
            self.validate_available_node(pending.node, &register_order, &retired)?;
            if !declaration_nodes.insert(pending.node)
                || retargeted_declaration_nodes.contains(&pending.node)
            {
                return Err(MutationError::ConflictingNodeEdit(pending.node));
            }
            if self.semantic.is_node_active(pending.node)
                && self
                    .semantic
                    .declaration_of(pending.node)
                    .is_some_and(|id| !retired_declarations.contains(&id))
            {
                return Err(MutationError::NodeHasSemanticIdentity(pending.node));
            }
            if !pending_bindings.insert((pending.scope, pending.name))
                || matches!(pending.scope, ScopeHandle::Existing(scope) if self.semantic.binding(scope, pending.name).is_some())
            {
                return Err(MutationError::BindingConflict {
                    scope: pending.scope,
                    name: pending.name,
                });
            }
        }

        let mut reference_nodes = HashSet::new();
        for pending in &self.journal.pending_references {
            self.validate_scope(pending.scope)?;
            if targets_retired_scope(pending.scope) {
                let ScopeHandle::Existing(scope) = pending.scope else {
                    unreachable!()
                };
                return Err(MutationError::ScopeNotEmpty(scope));
            }
            self.validate_available_node(pending.node, &register_order, &retired)?;
            if !reference_nodes.insert(pending.node)
                || retargeted_reference_nodes.contains(&pending.node)
            {
                return Err(MutationError::ConflictingNodeEdit(pending.node));
            }
            if self.semantic.is_node_active(pending.node)
                && self
                    .semantic
                    .reference_of(pending.node)
                    .is_some_and(|id| !retired_references.contains(&id))
            {
                return Err(MutationError::NodeHasSemanticIdentity(pending.node));
            }
            if let Some(symbol) = pending.symbol {
                self.validate_symbol(symbol)?;
                if matches!(symbol, SymbolHandle::Existing(symbol) if retired_symbols.contains(&symbol))
                {
                    return Err(MutationError::InactiveSymbol(match symbol {
                        SymbolHandle::Existing(symbol) => symbol,
                        SymbolHandle::Pending(_) => unreachable!(),
                    }));
                }
            }
        }

        Ok(())
    }

    fn validate_active_node(&self, node: NodeId) -> Result<(), MutationError> {
        if self.semantic.is_node_active(node) {
            Ok(())
        } else {
            Err(MutationError::InactiveNode(node))
        }
    }

    fn normalized_dirty_roots(&self) -> Vec<NodeId> {
        let mut roots = Vec::new();
        for dirty in &self.dirty_roots {
            let dirty = if self.semantic.is_node_active(*dirty) {
                *dirty
            } else {
                self.root
            };
            if !roots.contains(&dirty) {
                roots.push(dirty);
            }
        }
        let candidates = roots.clone();
        roots.retain(|candidate| {
            !candidates.iter().any(|other| {
                other != candidate
                    && self
                        .semantic
                        .ancestors(Some(*candidate))
                        .skip(1)
                        .any(|ancestor| ancestor == *other)
            })
        });
        roots
    }

    fn validate_available_node(
        &self,
        node: NodeId,
        register_order: &HashMap<NodeId, usize>,
        retired: &HashSet<NodeId>,
    ) -> Result<(), MutationError> {
        if !retired.contains(&node)
            && (self.semantic.is_node_active(node) || register_order.contains_key(&node))
        {
            Ok(())
        } else {
            Err(MutationError::InactiveNode(node))
        }
    }

    fn validate_parent(
        &self,
        parent: Option<NodeId>,
        edit_index: usize,
        register_order: &HashMap<NodeId, usize>,
    ) -> Result<(), MutationError> {
        if let Some(parent) = parent {
            if self.semantic.is_node_active(parent)
                || register_order
                    .get(&parent)
                    .is_some_and(|parent_index| *parent_index < edit_index)
            {
                return Ok(());
            }
            return Err(MutationError::InactiveNode(parent));
        }
        Ok(())
    }

    fn validate_scope(&self, scope: ScopeHandle) -> Result<(), MutationError> {
        match scope {
            ScopeHandle::Existing(scope) if self.semantic.is_scope_active(scope) => Ok(()),
            ScopeHandle::Existing(scope) => Err(MutationError::InactiveScope(scope)),
            ScopeHandle::Pending(scope) if scope.0 < self.journal.pending_scopes.len() => Ok(()),
            ScopeHandle::Pending(scope) => Err(MutationError::InvalidPendingScope(scope)),
        }
    }

    fn validate_scope_reparent(
        &self,
        scope: ScopeId,
        parent: ScopeHandle,
        reparented: &HashMap<ScopeId, ScopeHandle>,
    ) -> Result<(), MutationError> {
        let mut current = Some(parent);
        let mut visited = HashSet::new();
        while let Some(handle) = current {
            if !visited.insert(handle) {
                return Err(MutationError::ScopeCycle(scope));
            }
            current = match handle {
                ScopeHandle::Existing(candidate) => {
                    if candidate == scope {
                        return Err(MutationError::ScopeCycle(scope));
                    }
                    if let Some(parent) = reparented.get(&candidate) {
                        Some(*parent)
                    } else {
                        self.semantic
                            .scope(candidate)
                            .parent
                            .map(ScopeHandle::Existing)
                    }
                }
                ScopeHandle::Pending(pending) => {
                    Some(self.journal.pending_scopes[pending.0].parent)
                }
            };
        }
        Ok(())
    }

    fn validate_symbol(&self, symbol: SymbolHandle) -> Result<(), MutationError> {
        match symbol {
            SymbolHandle::Existing(symbol) if self.semantic.is_symbol_active(symbol) => Ok(()),
            SymbolHandle::Existing(symbol) => Err(MutationError::InactiveSymbol(symbol)),
            SymbolHandle::Pending(symbol) if symbol.0 < self.journal.pending_symbols.len() => {
                Ok(())
            }
            SymbolHandle::Pending(symbol) => Err(MutationError::InvalidPendingSymbol(symbol)),
        }
    }
}

fn resolve_scope(handle: ScopeHandle, pending: &[ScopeId]) -> ScopeId {
    match handle {
        ScopeHandle::Existing(scope) => scope,
        ScopeHandle::Pending(scope) => pending[scope.0],
    }
}

fn resolve_symbol(handle: SymbolHandle, pending: &[SymbolId]) -> SymbolId {
    match handle {
        SymbolHandle::Existing(symbol) => symbol,
        SymbolHandle::Pending(symbol) => pending[symbol.0],
    }
}
