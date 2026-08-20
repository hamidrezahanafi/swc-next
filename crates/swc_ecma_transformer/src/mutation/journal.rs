use swc_next_ecma_ast::SymbolId;
use swc_next_ecma_ast::{AstStr, NodeId, ScopeId};
use swc_next_ecma_semantic::{DeclarationId, ReferenceFlags, ReferenceId, SymbolFlags, scope};

use super::{IdentityPolicy, PendingReferenceId, PrivateBinding, ScopeHandle, SymbolHandle};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ReplacementCategory {
    Exact,
    Expression,
    Statement,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AstEdit {
    Replace {
        old: NodeId,
        new: NodeId,
        identity: IdentityPolicy,
        category: ReplacementCategory,
    },
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub(super) enum SemanticEdit {
    RegisterNode {
        node: NodeId,
        parent: Option<NodeId>,
        scope: ScopeHandle,
    },
    RetireNode {
        node: NodeId,
    },
    ReparentNode {
        node: NodeId,
        parent: Option<NodeId>,
    },
    RescopeNode {
        node: NodeId,
        scope: ScopeHandle,
    },
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub(super) enum IdentityEdit {
    RetireReference { reference: ReferenceId },
    RetireDeclaration { declaration: DeclarationId },
    RetireSymbol { symbol: SymbolId },
}

#[derive(Debug)]
pub(super) enum ResolutionEdit {
    RetargetDeclarationNode {
        declaration: DeclarationId,
        node: NodeId,
    },
    RetargetReferenceNode {
        reference: ReferenceId,
        node: NodeId,
    },
    UpdateDeclarationFlags {
        declaration: DeclarationId,
        flags: SymbolFlags,
    },
    UpdateSymbolFlags {
        symbol: SymbolId,
        flags: SymbolFlags,
    },
    MoveSymbol {
        symbol: SymbolId,
        scope: ScopeHandle,
        hoisting_scopes: Vec<ScopeHandle>,
        closed_subtree: Option<Vec<NodeId>>,
    },
    RenameSymbol {
        symbol: SymbolId,
        name: AstStr,
    },
    RenameReference {
        reference: ReferenceId,
        name: AstStr,
    },
    RebindReference {
        reference: ReferenceId,
        symbol: Option<SymbolHandle>,
    },
    RescopeReference {
        reference: ReferenceId,
        scope: ScopeHandle,
    },
    UpdateReferenceFlags {
        reference: ReferenceId,
        flags: ReferenceFlags,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ModuleEdit {
    RetargetNode {
        old: NodeId,
        new: NodeId,
    },
    RetargetDefaultLocal {
        statement: NodeId,
        binding: NodeId,
        symbol: SymbolHandle,
    },
    ReplaceNamespaceReexport {
        old_statement: NodeId,
        statement: NodeId,
        specifier: NodeId,
        local: NodeId,
        exported: NodeId,
        symbol: SymbolHandle,
        reference: PendingReferenceId,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ScopeEdit {
    Retag {
        scope: ScopeId,
        kind: scope::ScopeKind,
        flags: scope::ScopeFlags,
    },
    Reowner {
        scope: ScopeId,
        owner: NodeId,
    },
    Reparent {
        scope: ScopeId,
        parent: ScopeHandle,
    },
    Retire {
        scope: ScopeId,
    },
}

#[derive(Debug, Default)]
pub(super) struct MutationJournal {
    pub(super) ast_edits: Vec<AstEdit>,
    pub(super) semantic_edits: Vec<SemanticEdit>,
    pub(super) identity_edits: Vec<IdentityEdit>,
    pub(super) resolution_edits: Vec<ResolutionEdit>,
    pub(super) module_edits: Vec<ModuleEdit>,
    pub(super) scope_edits: Vec<ScopeEdit>,
    pub(super) pending_scopes: Vec<PendingScope>,
    pub(super) pending_symbols: Vec<PendingSymbol>,
    pub(super) pending_references: Vec<PendingReference>,
    pub(super) name_reservations: Vec<PendingNameReservation>,
    pub(super) shared_private_bindings: Vec<SharedPrivateBinding>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingScope {
    pub(super) parent: ScopeHandle,
    pub(super) owner: NodeId,
    pub(super) kind: scope::ScopeKind,
    pub(super) flags: scope::ScopeFlags,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingSymbol {
    pub(super) scope: ScopeHandle,
    pub(super) node: NodeId,
    pub(super) name: AstStr,
    pub(super) flags: SymbolFlags,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingReference {
    pub(super) scope: ScopeHandle,
    pub(super) node: NodeId,
    pub(super) name: AstStr,
    pub(super) flags: ReferenceFlags,
    pub(super) symbol: Option<SymbolHandle>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingNameReservation {
    pub(super) scope: ScopeHandle,
    pub(super) name: AstStr,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SharedPrivateBinding {
    pub(super) key: AstStr,
    pub(super) scope: ScopeId,
    pub(super) flags: SymbolFlags,
    pub(super) binding: PrivateBinding,
}

impl MutationJournal {
    pub(super) fn is_empty(&self) -> bool {
        self.ast_edits.is_empty()
            && self.semantic_edits.is_empty()
            && self.identity_edits.is_empty()
            && self.resolution_edits.is_empty()
            && self.module_edits.is_empty()
            && self.scope_edits.is_empty()
            && self.pending_scopes.is_empty()
            && self.pending_symbols.is_empty()
            && self.pending_references.is_empty()
            && self.name_reservations.is_empty()
    }
}
