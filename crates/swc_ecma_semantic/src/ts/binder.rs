use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use swc_next_allocator::{hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{Ast, MemberExpression, NodeId, ScopeId, SymbolId};

use crate::{
    ReferenceSpace, ScopeMap, Symbol,
    binder::Range,
    ts::binder::{member_facts::TsExpandoBindingSite, module_instance::TsModuleInstanceMemo},
};

mod aliases;
mod lookup;
mod member_facts;
mod module_instance;

oxc_index::define_nonmax_u32_index_type! {
    struct TsMemberEnvironmentId;
}

type TsMemberMap<'a> = ArenaHashMap<'a, &'a [u8], TsMemberEntry>;
type TsNamespaceExportAliasMap<'a> = ArenaHashMap<'a, &'a [u8], TsNamespaceExportAliasEntry<'a>>;

pub struct TsSymbolTracker<'a> {
    ts_type_parameter_owners: ArenaHashMap<'a, ScopeId, SymbolId>,
    ts_type_parameters: ArenaHashMap<'a, SymbolId, ScopeMap<'a>>,
    ts_source_exports: Option<TsSourceExports<'a>>,
    ts_local_exports: TsLocalExportMap<'a>,
    ts_member_environments: TsMemberEnvironments<'a>,
    ts_global_owner: Option<SymbolId>,
    ts_member_views: ArenaVec<'a, TsMemberView>,
    ts_scope_view_records: ArenaVec<'a, (ScopeId, Range)>,
    ts_scope_view_index: Option<TsScopeViewIndex<'a>>,
    pending_ts_namespace_export_aliases: ArenaVec<'a, PendingTsNamespaceExportAlias<'a>>,
    ts_import_alias_targets: ArenaVec<'a, TsImportAliasTarget>,
    ts_expando_assignments: ArenaVec<'a, (MemberExpression, TsExpandoBindingSite)>,
    ts_import_aliases: Option<TsImportAliasIndex<'a>>,
    ts_container_stack: Vec<TsContainerContext>,
    ts_module_instance_scratch: HashMap<NodeId, TsModuleInstanceMemo, FxBuildHasher>,
    ts_enum_owner_stack: Vec<SymbolId>,
}

impl<'a> TsSymbolTracker<'a> {
    pub fn init(tree: &Ast<'a>) -> Self {
        let allocator = tree.allocator();
        let mut tracker = Self {
            ts_type_parameter_owners: ArenaHashMap::new_in(allocator),
            ts_type_parameters: ArenaHashMap::new_in(allocator),
            ts_source_exports: None,
            ts_local_exports: ArenaHashMap::new_in(allocator),
            ts_member_environments: TsMemberEnvironments::new_in(allocator),
            ts_global_owner: None,
            ts_member_views: ArenaVec::new_in(allocator),
            ts_scope_view_records: ArenaVec::new_in(allocator),
            ts_scope_view_index: None,
            pending_ts_namespace_export_aliases: ArenaVec::new_in(allocator),
            ts_import_alias_targets: ArenaVec::new_in(allocator),
            ts_expando_assignments: ArenaVec::new_in(allocator),
            ts_import_aliases: None,
            ts_container_stack: Vec::new(),
            ts_module_instance_scratch: HashMap::with_hasher(FxBuildHasher),
            ts_enum_owner_stack: Vec::new(),
        };
        if tree.is_ts() && tree.is_module() {
            tracker.init_ts_source_exports(tree);
        }
        tracker
    }

    fn init_ts_source_exports(&mut self, tree: &Ast<'a>) {
        let program = tree.root_program();
        let owner = self.declare_symbol(
            tree,
            PendingDeclaration {
                name: tree.empty_wtf8(),
                node: program.node_id(),
                binding: PendingBinding {
                    flags: SymbolFlags::VALUE_MODULE | SymbolFlags::NAMESPACE_MODULE,
                    excludes: SymbolFlags::empty(),
                    scope: ScopeId::ROOT,
                },
            },
            None,
        );
        let environment = self.ts_member_environments.ensure(owner, self.allocator);
        let start = self.ts_member_views.len();
        self.ts_member_views.push(TsMemberView {
            environment,
            kind: TsMemberViewKind::SourceFile,
        });
        self.record_ts_scope_views(
            ScopeId::MODULE,
            Range {
                start: start as u32,
                len: 1,
            },
        );
        let implicit_exports = self.ambient
            && !program.body(tree).iter().any(|statement| {
                let statement = tree.get_node_in_sub_range(statement).node_id();
                ts_statement_is_export_declaration(tree, statement)
            });
        self.ts_source_exports = Some(TsSourceExports {
            environment,
            implicit_exports,
            default: None,
        });
    }
}

#[derive(Debug)]
struct TsMemberEnvironment<'a> {
    // Canonical declarations visible from other bodies merged into the same
    // namespace or enum.
    members: TsMemberMap<'a>,
    // Public aliases do not become lexical bindings in merged namespace
    // bodies. TypeScript likewise keeps container locals and exports apart.
    export_aliases: TsNamespaceExportAliasMap<'a>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsMemberKinds: u8 {
        const MODULE = 1 << 0;
        const ENUM = 1 << 1;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsMemberSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        const TYPEOF = 1 << 3;
    }
}

impl TsMemberSpaces {
    fn from_flags(flags: SymbolFlags) -> Self {
        let mut spaces = Self::empty();
        spaces.set(Self::VALUE, flags.visible_in(ReferenceSpace::Value));
        spaces.set(Self::TYPE, flags.visible_in(ReferenceSpace::Type));
        spaces.set(Self::NAMESPACE, flags.visible_in(ReferenceSpace::Namespace));
        spaces.set(Self::TYPEOF, flags.visible_in(ReferenceSpace::Typeof));
        spaces
    }

    fn visible_in(self, space: ReferenceSpace) -> bool {
        match space {
            ReferenceSpace::Value => self.contains(Self::VALUE),
            ReferenceSpace::Type => self.contains(Self::TYPE),
            ReferenceSpace::Namespace => self.contains(Self::NAMESPACE),
            ReferenceSpace::Typeof => self.contains(Self::TYPEOF),
            ReferenceSpace::Any => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TsMemberEntry {
    symbol: SymbolId,
    kinds: TsMemberKinds,
    module_spaces: TsMemberVisibility,
    enum_spaces: TsMemberSpaces,
    // Excludes enum-only and import-alias contributions to the shared symbol.
    module_declaration: bool,
}

#[derive(Debug, Clone, Copy)]
struct TsMemberVisibility {
    possible: TsMemberSpaces,
    certain: TsMemberSpaces,
}

impl TsMemberVisibility {
    fn certain(spaces: TsMemberSpaces) -> Self {
        Self {
            possible: spaces,
            certain: spaces,
        }
    }

    fn insert(&mut self, other: Self) {
        self.possible |= other.possible;
        self.certain |= other.certain;
    }

    fn in_space(self, space: ReferenceSpace) -> BindingVisibility {
        if self.certain.visible_in(space) {
            BindingVisibility::Present
        } else if self.possible.visible_in(space) {
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TsNamespaceExportAliasEntry<'a> {
    type_only: bool,
    resolution: TsExportAliasResolution<'a>,
    node: NodeId,
}

#[derive(Debug, Clone, Copy)]
enum TsExportAliasResolution<'a> {
    Resolved(TsResolvedExportAliasTarget<'a>),
    Unresolved,
    NonLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TsMemberViewKind {
    Module,
    AmbientModule,
    SourceFile,
    Enum,
}

impl TsMemberViewKind {
    fn accepts(self, kinds: TsMemberKinds) -> bool {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => {
                kinds.contains(TsMemberKinds::MODULE)
            }
            Self::Enum => kinds.contains(TsMemberKinds::ENUM),
        }
    }

    fn spaces(self, entry: TsMemberEntry) -> TsMemberVisibility {
        match self {
            Self::Module | Self::AmbientModule | Self::SourceFile => entry.module_spaces,
            Self::Enum => TsMemberVisibility::certain(entry.enum_spaces),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TsMemberView {
    environment: TsMemberEnvironmentId,
    kind: TsMemberViewKind,
}

impl TsMemberView {
    #[inline(always)]
    fn candidates<'lookup, 'a>(
        self,
        environments: &'lookup TsMemberEnvironments<'a>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberCandidates<'lookup, 'a>> {
        let environment = &environments.environments[self.environment.index()];
        let direct = environment
            .members
            .raw_entry()
            .from_hash(hash, |candidate| *candidate == name)
            .map(|(_, entry)| *entry)
            .filter(|entry| self.kind.accepts(entry.kinds));
        let alias = if matches!(self.kind, TsMemberViewKind::Enum)
            || environment.export_aliases.is_empty()
        {
            None
        } else {
            environment
                .export_aliases
                .raw_entry()
                .from_hash(hash, |candidate| *candidate == name)
                .map(|(_, entry)| entry)
        };
        // TS7 excludes pure export specifiers in source files and ambient module bodies.
        // A symbol with its own declaration meaning still exposes its alias
        // contribution, even when the direct declaration occupies another space.
        if matches!(
            self.kind,
            TsMemberViewKind::AmbientModule | TsMemberViewKind::SourceFile
        ) && alias.is_some()
        {
            let has_declaration = direct.is_some_and(|entry| entry.module_declaration);
            if !has_declaration {
                return None;
            }
        }
        Some(TsMemberCandidates { direct, alias })
    }

    #[inline(always)]
    fn lookup<'a>(
        self,
        environments: &TsMemberEnvironments<'a>,
        hash: u64,
        name: &[u8],
    ) -> Option<TsMemberLookup<'a>> {
        let TsMemberCandidates { direct, alias } = self.candidates(environments, hash, name)?;
        let target = TsResolvedImportTarget::from_members(direct.map(|entry| entry.symbol), alias)?;
        let direct_spaces = direct
            .map(|entry| self.kind.spaces(entry))
            .unwrap_or_else(|| TsMemberVisibility::certain(TsMemberSpaces::empty()));
        Some(TsMemberLookup {
            target,
            direct_spaces,
        })
    }
}

struct TsMemberCandidates<'lookup, 'a> {
    direct: Option<TsMemberEntry>,
    alias: Option<&'lookup TsNamespaceExportAliasEntry<'a>>,
}

struct TsMemberLookup<'a> {
    target: TsResolvedImportTarget<'a>,
    direct_spaces: TsMemberVisibility,
}

impl TsMemberLookup<'_> {
    fn visit_candidates<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbols: &[Symbol],
        import_alias_spaces: &[Option<TsResolvedAliasSpaces>],
        space: ReferenceSpace,
        mut accept: impl FnMut(SymbolId, BindingVisibility) -> bool,
    ) -> bool {
        if let Some(symbol) = self.target.direct
            && accept(symbol, self.direct_spaces.in_space(space))
        {
            return true;
        }
        let Some(alias) = self.target.alias else {
            return false;
        };
        for contribution in alias
            .target
            .public_contributions(self.target.type_only_export)
        {
            if contribution.type_only_export.is_some() && space == ReferenceSpace::Value {
                continue;
            }
            let visibility = if PRECISE_IMPORT_ALIASES
                && let Some(spaces) =
                    precise_import_alias_spaces(symbols, import_alias_spaces, contribution.symbol)
            {
                // Namespace-body lookup uses ModuleMember, excluding property
                // values even when a qualified alias could access them.
                spaces
                    .difference(
                        TsResolvedAliasSpaces::PROPERTY_VALUE
                            | TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
                    )
                    .visibility(space, symbols[contribution.symbol.index()].flags)
            } else {
                symbol_visibility::<PRECISE_IMPORT_ALIASES>(
                    symbols,
                    import_alias_spaces,
                    contribution.symbol,
                    space,
                )
            };
            if accept(contribution.symbol, visibility) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
struct TsMemberEnvironments<'a> {
    owners: ArenaHashMap<'a, SymbolId, TsMemberEnvironmentId>,
    environments: ArenaVec<'a, TsMemberEnvironment<'a>>,
}

impl<'a> TsMemberEnvironments<'a> {
    fn needs_value_projections(&self) -> bool {
        self.environments.iter().any(|environment| {
            environment.export_aliases.iter().any(|(name, alias)| {
                let direct = environment.members.get(name).map(|entry| entry.symbol);
                TsResolvedImportTarget::from_members(direct, Some(alias))
                    .is_some_and(TsResolvedImportTarget::needs_value_projection)
            })
        })
    }

    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            owners: ArenaHashMap::new_in(allocator),
            environments: ArenaVec::new_in(allocator),
        }
    }

    fn ensure(&mut self, owner: SymbolId, allocator: &'a Allocator) -> TsMemberEnvironmentId {
        *self.owners.entry(owner).or_insert_with(|| {
            let environment = TsMemberEnvironmentId::from_usize(self.environments.len());
            self.environments.push(TsMemberEnvironment {
                members: TsMemberMap::new_in(allocator),
                export_aliases: TsNamespaceExportAliasMap::new_in(allocator),
            });
            environment
        })
    }

    fn get(&self, owner: SymbolId) -> Option<&TsMemberEnvironment<'a>> {
        let environment = *self.owners.get(&owner)?;
        Some(&self.environments[environment.index()])
    }

    fn members(&self, owner: SymbolId) -> Option<&TsMemberMap<'a>> {
        self.get(owner).map(|environment| &environment.members)
    }

    fn environment(&self, environment: TsMemberEnvironmentId) -> &TsMemberMap<'a> {
        &self.environments[environment.index()].members
    }

    fn environment_mut(&mut self, environment: TsMemberEnvironmentId) -> &mut TsMemberMap<'a> {
        &mut self.environments[environment.index()].members
    }

    fn environment_export_aliases(
        &self,
        environment: TsMemberEnvironmentId,
    ) -> &TsNamespaceExportAliasMap<'a> {
        &self.environments[environment.index()].export_aliases
    }

    fn export_aliases_mut(
        &mut self,
        environment: TsMemberEnvironmentId,
    ) -> &mut TsNamespaceExportAliasMap<'a> {
        &mut self.environments[environment.index()].export_aliases
    }

    fn member(&self, environment: TsMemberEnvironmentId, name: &[u8]) -> Option<SymbolId> {
        self.environment(environment)
            .get(name)
            .map(|entry| entry.symbol)
    }
}

#[derive(Debug)]
struct TsImportAliasIndex<'a> {
    spaces: ArenaVec<'a, Option<TsResolvedAliasSpaces>>,
    targets: ArenaVec<'a, Option<SymbolId>>,
}

#[derive(Debug)]
pub(crate) struct TsMemberIndex<'a> {
    source_exports: Option<TsSourceExports<'a>>,
    local_exports: TsLocalExportMap<'a>,
    environments: TsMemberEnvironments<'a>,
    scope_views: Option<TsScopeViewIndex<'a>>,
    views: ArenaVec<'a, TsMemberView>,
    import_aliases: Option<TsImportAliasIndex<'a>>,
    reference_values: Option<ArenaBox<'a, TsReferenceValues<'a>>>,
}

impl TsMemberIndex<'_> {
    pub(crate) fn reference_values(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        self.reference_values.as_ref()?.get(reference)
    }

    pub(crate) fn import_alias_target(&self, alias: SymbolId) -> Option<SymbolId> {
        self.import_aliases
            .as_ref()?
            .targets
            .get(alias.index())
            .copied()
            .flatten()
    }

    pub(crate) fn value_target(
        &self,
        symbols: &[Symbol],
        declarations: &[Declaration],
        scope_maps: &[ScopeMap<'_>],
        resolution: NameResolution,
        name: &[u8],
        space: ReferenceSpace,
    ) -> Option<NameResolution> {
        let spaces = self
            .import_aliases
            .as_ref()
            .map(|aliases| aliases.spaces.as_slice())
            .unwrap_or_default();
        let symbol = resolution.candidate()?;
        let target = TsValueTargetLookup {
            locals: &self.local_exports,
            symbols,
            declarations,
            scope_maps,
            import_alias_spaces: spaces,
        }
        .target::<true>(symbol, name, space);
        let candidate = target.candidate()?;
        if matches!(resolution, NameResolution::Uncertain { .. }) {
            Some(NameResolution::Uncertain { candidate })
        } else {
            Some(target)
        }
    }

    pub(crate) fn source_binding(&self, name: &[u8]) -> Option<SymbolId> {
        if name == b"default" {
            return None;
        }
        let source = self.source_exports.as_ref()?;
        self.environments.member(source.environment, name)
    }

    pub(crate) fn source_bindings<'s>(
        &'s self,
        locals: &'s ScopeMap<'s>,
    ) -> Option<TsSourceBindingIterator<'s>> {
        let source = self.source_exports.as_ref()?;
        Some(TsSourceBindingIterator {
            inner: self.environments.environment(source.environment).iter(),
            locals,
        })
    }

    pub(crate) fn namespace_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Module)
    }

    pub(crate) fn namespace_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Module)
    }

    pub(crate) fn enum_members(&self, owner: SymbolId) -> impl Iterator<Item = SymbolId> + '_ {
        self.members(owner, TsMemberViewKind::Enum)
    }

    pub(crate) fn enum_member(&self, owner: SymbolId, name: &[u8]) -> Option<SymbolId> {
        self.member(owner, name, TsMemberViewKind::Enum)
    }

    fn members(
        &self,
        owner: SymbolId,
        kind: TsMemberViewKind,
    ) -> impl Iterator<Item = SymbolId> + '_ {
        self.environments
            .members(owner)
            .into_iter()
            .flat_map(|members| members.values())
            .filter(move |entry| kind.accepts(entry.kinds))
            .map(|entry| entry.symbol)
    }

    fn member(&self, owner: SymbolId, name: &[u8], kind: TsMemberViewKind) -> Option<SymbolId> {
        self.environments
            .members(owner)
            .and_then(|members| members.get(name))
            .filter(|entry| kind.accepts(entry.kinds))
            .map(|entry| entry.symbol)
    }

    pub(crate) fn resolve_name(
        &self,
        scopes: &ScopeTree<'_>,
        symbols: &[Symbol],
        scope_maps: &[ScopeMap<'_>],
        scope: ScopeId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        let query = NameLookupQuery { scope, name, space };
        let lexical = LexicalLookup {
            scopes: &scopes.list,
            symbols,
            scope_maps,
        };
        let ts = TsNameLookup {
            source_exports: self.source_exports.as_ref(),
            local_exports: &self.local_exports,
            environments: &self.environments,
            views: &self.views,
            import_alias_spaces: self
                .import_aliases
                .as_ref()
                .map(|aliases| aliases.spaces.as_slice())
                .unwrap_or_default(),
        };
        if self.import_aliases.is_some() {
            lookup_name_in_scopes::<true, true>(
                lexical,
                &ts,
                self.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        } else {
            lookup_name_in_scopes::<true, false>(
                lexical,
                &ts,
                self.scope_views.as_ref(),
                query,
                |_| {},
            )
            .resolution()
        }
    }
}

pub(crate) struct TsSourceBindingIterator<'a> {
    inner: swc_next_allocator::hash_map::Iter<'a, &'a [u8], TsMemberEntry>,
    locals: &'a ScopeMap<'a>,
}

impl Iterator for TsSourceBindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|(&name, member)| {
            (name != b"default" && !self.locals.contains_key(name)).then_some(member.symbol)
        })
    }
}

#[derive(Clone, Copy)]
enum TsContainerContext {
    UnsupportedExternalModule {
        body: Option<NodeId>,
    },
    Member {
        environment: TsMemberEnvironmentId,
        views: Range,
        projection_scope: Option<ScopeId>,
        body: Option<NodeId>,
        body_scope: Option<ScopeId>,
        implicit_exports: bool,
        public_exports: bool,
    },
}

impl TsContainerContext {
    fn body(self) -> Option<NodeId> {
        match self {
            Self::UnsupportedExternalModule { body } | Self::Member { body, .. } => body,
        }
    }
}

#[derive(Clone, Copy)]
struct PendingTsNamespaceExportAlias<'a> {
    type_only: bool,
    environment: TsMemberEnvironmentId,
    exported_name: &'a [u8],
    local_reference: ReferenceId,
    node: NodeId,
    previous: Option<NodeId>,
}

#[derive(Clone, Copy)]
struct TsImportAliasTarget {
    alias: SymbolId,
    declaration: TsImportEqualsDeclaration,
    root_reference: ReferenceId,
}

// Like TS7's merged export symbols, these nodes retain the public name at
// which alias resolution meets declaration meaning. Pure forwarding aliases
// share their target; only a direct-plus-alias combination allocates a node.
#[derive(Debug, Clone, Copy)]
enum TsExportTarget<'a> {
    // Public access may cross an exported-value bridge; conflict checks keep
    // the lexical declaration meaning. Both stay compact for simple targets.
    Symbols { lexical: SymbolId, public: SymbolId },
    Member(&'a TsExportMember<'a>),
}

#[derive(Debug)]
struct TsExportMember<'a> {
    direct: SymbolId,
    alias: TsResolvedExportAliasTarget<'a>,
}

#[derive(Debug, Clone, Copy)]
struct TsResolvedExportAliasTarget<'a> {
    target: TsExportTarget<'a>,
    type_only_export: Option<NodeId>,
}

impl<'a> TsResolvedExportAliasTarget<'a> {
    fn target(target: TsExportTarget<'a>) -> Self {
        Self {
            target,
            type_only_export: None,
        }
    }

    fn symbol(symbol: SymbolId) -> Self {
        Self::target(TsExportTarget::Symbols {
            lexical: symbol,
            public: symbol,
        })
    }
}

#[derive(Clone, Copy)]
struct TsExportContribution {
    symbol: SymbolId,
    type_only_export: Option<NodeId>,
}

struct TsExportContributions<'a> {
    target: Option<TsExportTarget<'a>>,
    conflict: bool,
    type_only_export: Option<NodeId>,
}

impl<'a> TsExportTarget<'a> {
    fn public_contributions(self, type_only_export: Option<NodeId>) -> TsExportContributions<'a> {
        TsExportContributions {
            target: Some(self),
            conflict: false,
            type_only_export,
        }
    }

    fn conflict_contributions(self) -> TsExportContributions<'a> {
        TsExportContributions {
            target: Some(self),
            conflict: true,
            type_only_export: None,
        }
    }
}

impl Iterator for TsExportContributions<'_> {
    type Item = TsExportContribution;

    fn next(&mut self) -> Option<Self::Item> {
        let target = self.target.take()?;
        let type_only_export = self.type_only_export;
        let symbol = match target {
            TsExportTarget::Symbols { lexical, public } => {
                if self.conflict {
                    lexical
                } else {
                    public
                }
            }
            TsExportTarget::Member(member) => {
                self.target = Some(member.alias.target);
                // The member is already a public export symbol. Its alias edge
                // crosses the exported-value bridge, even when the caller is
                // checking a new alias against the member's declaration spaces.
                self.conflict = false;
                self.type_only_export = self.type_only_export.or(member.alias.type_only_export);
                member.direct
            }
        };
        Some(TsExportContribution {
            symbol,
            type_only_export,
        })
    }
}

#[derive(Clone, Copy)]
struct TsResolvedImportTarget<'a> {
    direct: Option<SymbolId>,
    alias: Option<TsResolvedExportAliasTarget<'a>>,
    type_only_export: Option<NodeId>,
    // Includes implicit members such as prototype; no lexical binding or
    // explicit member AST node is required to preserve this value meaning.
    property_value: PropertyValue,
}

impl<'a> TsResolvedImportTarget<'a> {
    fn needs_value_projection(self) -> bool {
        self.alias.is_some_and(|alias| match alias.target {
            TsExportTarget::Member(_) => true,
            TsExportTarget::Symbols { public, .. } => match self.direct {
                Some(direct) => direct != public,
                None => self.type_only_export.is_some(),
            },
        })
    }

    fn from_members(
        direct: Option<SymbolId>,
        alias: Option<&TsNamespaceExportAliasEntry<'a>>,
    ) -> Option<Self> {
        let alias = alias.and_then(|entry| match entry.resolution {
            TsExportAliasResolution::Resolved(target) => Some(target),
            TsExportAliasResolution::Unresolved | TsExportAliasResolution::NonLocal => None,
        });
        if direct.is_none() && alias.is_none() {
            return None;
        }
        Some(Self {
            direct,
            alias,
            type_only_export: alias.and_then(|alias| alias.type_only_export),
            property_value: PropertyValue::Absent,
        })
    }

    fn symbol(symbol: SymbolId) -> Self {
        Self {
            direct: Some(symbol),
            alias: None,
            type_only_export: None,
            property_value: PropertyValue::Absent,
        }
    }

    fn property_value(value: PropertyValue) -> Self {
        Self {
            direct: None,
            alias: None,
            type_only_export: None,
            property_value: value,
        }
    }

    fn non_symbol_spaces(self) -> TsResolvedAliasSpaces {
        match self.property_value {
            PropertyValue::Absent => TsResolvedAliasSpaces::empty(),
            PropertyValue::Present => TsResolvedAliasSpaces::PROPERTY_VALUE,
            PropertyValue::Unknown => TsResolvedAliasSpaces::UNKNOWN_PROPERTY_VALUE,
        }
    }

    // Symbol contributions exclude static members represented by non_symbol_spaces.
    fn public_contributions(self) -> impl Iterator<Item = TsExportContribution> + 'a {
        let direct = self.direct.map(|symbol| TsExportContribution {
            symbol,
            type_only_export: None,
        });
        direct.into_iter().chain(
            self.alias
                .into_iter()
                .flat_map(move |alias| alias.target.public_contributions(self.type_only_export)),
        )
    }

    fn public_symbols(self) -> impl Iterator<Item = SymbolId> + 'a {
        self.public_contributions()
            .map(|contribution| contribution.symbol)
    }

    fn conflict_symbols(self) -> impl Iterator<Item = SymbolId> + 'a {
        // A pure alias still checks its immediate lexical target. A name with
        // declarations of its own is already a public symbol: subsequent alias
        // edges contribute their exported meanings, not private bridge sources.
        let immediate_lexical_target =
            self.direct.is_none() && self.property_value == PropertyValue::Absent;
        self.direct
            .into_iter()
            .chain(self.alias.into_iter().flat_map(move |alias| {
                let contributions = if immediate_lexical_target {
                    alias.target.conflict_contributions()
                } else {
                    alias.target.public_contributions(None)
                };
                contributions.map(|contribution| contribution.symbol)
            }))
    }

    fn with_type_only_export(mut self, node: Option<NodeId>) -> Self {
        self.type_only_export = self.type_only_export.or(node);
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) enum TsNamespaceExportAliasIssue {
    Missing { name: AstUtf8, node: NodeId },
    NonLocal { name: AstUtf8, node: NodeId },
    Duplicate { node: NodeId, previous: NodeId },
    Conflict { node: NodeId, previous: NodeId },
}

#[derive(Clone, Copy)]
pub(crate) struct TsImportAliasConflictIssue {
    pub(crate) name: AstWtf8,
    pub(crate) node: NodeId,
    pub(crate) previous: NodeId,
    pub(crate) previous_flags: SymbolFlags,
}

#[derive(Clone, Copy)]
pub(crate) enum TsImportAliasIssue {
    Conflict(TsImportAliasConflictIssue),
    TypeOnlyExport { node: NodeId, export: NodeId },
}

enum TsNamespaceExportAliasLookup<'a> {
    Resolved(TsResolvedExportAliasTarget<'a>),
    Missing,
    NonLocal,
}

bitflags::bitflags! {
    /// TypeScript checker declaration spaces used when validating that merged
    /// declarations are either all exported or all local. These are narrower
    /// than reference/export capabilities: in particular, an unresolved
    /// import does not claim every space merely because it can be referenced
    /// in every space.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsDeclarationSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
    }
}

bitflags::bitflags! {
    /// Value, type, and namespace spaces contributed by a resolved internal
    /// TypeScript alias, plus uncertainty from opaque dependencies. Known bits
    /// prove contributions; UNKNOWN only permits conservative reference access.
    /// These results must not be written back into [`SymbolFlags`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TsResolvedAliasSpaces: u8 {
        const VALUE = 1 << 0;
        const TYPE = 1 << 1;
        const NAMESPACE = 1 << 2;
        // Opaque dependencies may contribute any reference space, but do not
        // prove a declaration-space conflict. Propagates with the known bits.
        const UNKNOWN = 1 << 3;
        // Properties and methods have value meaning, but are excluded by
        // TypeScript's ModuleMember mask inside merged namespace bodies.
        const PROPERTY_VALUE = 1 << 4;
        // Unresolved property names may add value meaning, but do not prove
        // existence, a conflict, or type/namespace contributions.
        const UNKNOWN_PROPERTY_VALUE = 1 << 5;
    }
}

impl TsResolvedAliasSpaces {
    fn known(self) -> Self {
        self.difference(Self::UNKNOWN | Self::UNKNOWN_PROPERTY_VALUE)
    }

    fn visibility(self, space: ReferenceSpace, flags: SymbolFlags) -> BindingVisibility {
        if self.known().visible_in(space, flags) {
            BindingVisibility::Present
        } else if self.visible_in(space, flags) {
            BindingVisibility::Unknown
        } else {
            BindingVisibility::Absent
        }
    }

    fn may_have(self, space: Self) -> bool {
        self.intersects(space | Self::UNKNOWN)
            || (space.contains(Self::PROPERTY_VALUE) && self.contains(Self::UNKNOWN_PROPERTY_VALUE))
    }

    fn may_have_value(self) -> bool {
        self.may_have(Self::VALUE | Self::PROPERTY_VALUE)
    }

    fn declaration_spaces(mut self) -> Self {
        if self.contains(Self::PROPERTY_VALUE) {
            self.remove(Self::PROPERTY_VALUE);
            self.insert(Self::VALUE);
        }
        self.intersection(Self::VALUE | Self::TYPE | Self::NAMESPACE)
    }

    fn visible_in(self, space: ReferenceSpace, flags: SymbolFlags) -> bool {
        match space {
            ReferenceSpace::Value => {
                // Type-only restricts the import contribution, not an
                // independent value declaration merged into the same symbol.
                flags.in_value_space()
                    || (flags.contains(SymbolFlags::IMPORT) && self.may_have_value())
            }
            ReferenceSpace::Typeof => self.may_have_value(),
            ReferenceSpace::Type => self.may_have(Self::TYPE),
            ReferenceSpace::Namespace => self.may_have(Self::NAMESPACE),
            ReferenceSpace::Any => true,
        }
    }
}

/// Value projections for references that selected an alias-backed member.
/// Other references use their canonical candidates without additional storage.
#[derive(Debug)]
struct TsReferenceValues<'a> {
    ranges: ArenaHashMap<'a, ReferenceId, Range>,
    targets: ArenaVec<'a, NameResolution>,
}

impl<'a> TsReferenceValues<'a> {
    fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            ranges: ArenaHashMap::new_in(allocator),
            targets: ArenaVec::new_in(allocator),
        }
    }

    fn get(&self, reference: ReferenceId) -> Option<&[NameResolution]> {
        let range = self.ranges.get(&reference)?;
        let start = range.start as usize;
        Some(&self.targets[start..start + range.len as usize])
    }

    fn record(&mut self, reference: ReferenceId, targets: &[NameResolution]) {
        self.ranges.insert(
            reference,
            Range {
                start: self.targets.len() as u32,
                len: targets.len() as u32,
            },
        );
        self.targets.extend(targets.iter().copied());
    }
}

#[derive(Debug)]
struct TsScopeViewIndex<'a> {
    ranges: ArenaVec<'a, Range>,
    lookup: ArenaVec<'a, TsScopeLookup>,
}

#[derive(Debug, Default, Clone, Copy)]
struct TsScopeLookup {
    // The nearest member overlay also bounds the ordinary lexical prefix.
    members: Option<ScopeId>,
    // An ancestor signature or conditional-type view can restrict lexical lookup.
    restricted_lexical: bool,
}

/// Source-file locals and exports have different identities. Named exports
/// use the common member environment; default declarations share its `default`
/// entry even when their local names differ. Local export links preserve TS7's
/// ExportValue and LocalSymbol.ExportSymbol relationship without duplicating
/// declaration ownership.
#[derive(Debug)]
struct TsSourceExports<'a> {
    environment: TsMemberEnvironmentId,
    implicit_exports: bool,
    default: Option<TsDefaultExport<'a>>,
}

#[derive(Debug)]
struct TsDefaultExport<'a> {
    symbol: SymbolId,
    first_local: Option<&'a [u8]>,
    first_local_is_function: bool,
}

#[derive(Debug, Clone, Copy)]
struct TsExportLocal {
    target: SymbolId,
    target_is_function: bool,
    exported_value: bool,
    default_declarations: Option<TsDefaultLocal>,
}

#[derive(Debug, Clone, Copy)]
struct TsDefaultLocal {
    node: NodeId,
    flags: SymbolFlags,
}

/// TS7 local export links belong to a declaration body, even when its public
/// member environment is shared across several namespace bodies. Recording
/// them during binding prevents same-named members in other bodies from
/// acquiring a local ExportValue relationship.
type TsLocalExportMap<'a> = ArenaHashMap<'a, (ScopeId, &'a [u8]), TsExportLocal>;

impl TsExportLocal {
    fn exported_value(self) -> Option<SymbolId> {
        self.exported_value.then_some(self.target)
    }
}

fn ts_exported_value(
    locals: &TsLocalExportMap<'_>,
    symbols: &[Symbol],
    declarations: &[Declaration],
    scope_maps: &[ScopeMap<'_>],
    lexical: SymbolId,
    name: &[u8],
) -> Option<SymbolId> {
    let first = symbols[lexical.index()].declaration;
    let scope = declarations[first.index()].scope;
    let target = locals.get(&(scope, name))?.exported_value()?;
    (scope_maps[scope.index()].get(name).copied() == Some(lexical)).then_some(target)
}

struct TsValueTargetLookup<'lookup, 'arena> {
    locals: &'lookup TsLocalExportMap<'arena>,
    symbols: &'lookup [Symbol],
    declarations: &'lookup [Declaration],
    scope_maps: &'lookup [ScopeMap<'arena>],
    import_alias_spaces: &'lookup [Option<TsResolvedAliasSpaces>],
}

impl TsValueTargetLookup<'_, '_> {
    fn target<const PRECISE_IMPORT_ALIASES: bool>(
        &self,
        symbol: SymbolId,
        name: &[u8],
        space: ReferenceSpace,
    ) -> NameResolution {
        debug_assert!(matches!(
            space,
            ReferenceSpace::Value | ReferenceSpace::Typeof
        ));
        let symbol = ts_exported_value(
            self.locals,
            self.symbols,
            self.declarations,
            self.scope_maps,
            symbol,
            name,
        )
        .unwrap_or(symbol);
        match symbol_visibility::<PRECISE_IMPORT_ALIASES>(
            self.symbols,
            self.import_alias_spaces,
            symbol,
            space,
        ) {
            BindingVisibility::Absent => NameResolution::Unresolved,
            BindingVisibility::Present => NameResolution::Resolved(symbol),
            BindingVisibility::Unknown => NameResolution::Uncertain { candidate: symbol },
        }
    }
}

impl TsSourceExports<'_> {
    fn lookup(
        &self,
        name: &[u8],
        space: ReferenceSpace,
        symbols: &[Symbol],
        locals: &TsLocalExportMap<'_>,
    ) -> Option<SymbolId> {
        if matches!(space, ReferenceSpace::Value | ReferenceSpace::Typeof)
            && let Some(symbol) = locals
                .get(&(ScopeId::MODULE, name))
                .and_then(|local| local.exported_value())
            && symbols[symbol.index()].flags.visible_in(space)
        {
            return Some(symbol);
        }
        let default = self.default.as_ref()?;
        (default.first_local == Some(name)
            && symbols[default.symbol.index()].flags.visible_in(space))
        .then_some(default.symbol)
    }
}
