//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use oxc_index::Idx;
use swc_next_allocator::{
    hash_map::{HashMap as ArenaHashMap, Values},
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    Ast, AstStr, AstUtf8, Diagnostic, Function, NodeId, Program, ScopeId, SymbolId, VisitWith,
};

use crate::{binder::Range, checker::Checker, scope::ScopeTree};

mod binder;
mod checker;
mod context;
mod ecmascript;
mod module_record;
pub mod scope;

pub use binder::{Declaration, Reference, ReferenceFlags, ReferenceSpace, Symbol, SymbolFlags};
pub use module_record::{
    ExportEntry, ExportName, ExportOrigin, ExportTarget, ExternalExportName, ModuleName,
    ModuleRecord,
};
pub use swc_next_ecma_ast::ReferenceId;

oxc_index::define_nonmax_u32_index_type! {
    /// Identifier for a declaration in a completed [`Semantic`] model.
    pub struct DeclarationId;
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
    pub struct SemanticFeatureFlags: u16 {
        const YIELD = 1 << 0;
        const AWAIT = 1 << 1;
        const GENERATOR_FUNCTIONS = 1 << 2;
        const ASYNC_FUNCTIONS = 1 << 3;
        const FUNCTION_ENV_CAPTURE = 1 << 4;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubtreeEffect {
    Yield,
    Await,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FeatureNotEnabled {
    pub feature: SemanticFeatureFlags,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct EffectCounts {
    pub(crate) yields: u32,
    pub(crate) awaits: u32,
}

#[derive(Debug)]
pub(crate) struct FunctionCaptureData<'a> {
    pub(crate) arguments: ArenaVec<'a, ReferenceId>,
    pub(crate) new_targets: ArenaVec<'a, NodeId>,
}

#[derive(Debug, Clone, Copy)]
pub struct FunctionCaptures<'a> {
    pub arguments: &'a [ReferenceId],
    pub new_targets: &'a [NodeId],
}

#[derive(Debug)]
pub(crate) struct SemanticTransformFacts<'a> {
    pub(crate) effects: Option<ArenaVec<'a, EffectCounts>>,
    pub(crate) effect_owners: Option<ArenaVec<'a, Option<NodeId>>>,
    pub(crate) generator_functions: ArenaVec<'a, Function>,
    pub(crate) async_functions: ArenaVec<'a, NodeId>,
    pub(crate) function_captures: ArenaHashMap<'a, NodeId, FunctionCaptureData<'a>>,
    pub(crate) yield_nodes: ArenaHashMap<'a, NodeId, ArenaVec<'a, NodeId>>,
}

#[derive(Debug, Clone, Copy)]
pub struct UsedNameState {
    pub bare_used: bool,
    pub next_suffix: u64,
}

impl Default for UsedNameState {
    fn default() -> Self {
        Self {
            bare_used: false,
            next_suffix: 1,
        }
    }
}

type ScopeMap<'a> = ArenaHashMap<'a, AstStr, SymbolId>;
type UsedNameMap<'a> = ArenaHashMap<'a, AstStr, UsedNameState>;

/// The complete semantic model of a tree, with every scope, symbol,
/// and reference fully resolved and cross-indexed. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, parents, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaVec<'a, Symbol>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaVec<'a, Reference>,
    /// Every declaration, grouped by symbol and indexed by [`DeclarationId`].
    declarations: ArenaVec<'a, Declaration>,
    declaration_lists: ArenaVec<'a, ArenaVec<'a, Declaration>>,
    module_record: ModuleRecord<'a>,
    use_lists: ArenaVec<'a, ArenaVec<'a, ReferenceId>>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
    node_registered: ArenaVec<'a, bool>,
    node_declarations: ArenaVec<'a, Option<DeclarationId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,
    pub(crate) features: SemanticFeatureFlags,
    pub(crate) transform_facts: Option<SemanticTransformFacts<'a>>,
    used_names: ArenaVec<'a, UsedNameMap<'a>>,
    direct_symbol_names: ArenaVec<'a, UsedNameMap<'a>>,
}

impl<'a> Semantic<'a> {
    #[inline]
    pub fn features(&self) -> SemanticFeatureFlags {
        self.features
    }

    pub fn contains_effect(
        &self,
        node: NodeId,
        effect: SubtreeEffect,
    ) -> Result<bool, FeatureNotEnabled> {
        let feature = match effect {
            SubtreeEffect::Yield => SemanticFeatureFlags::YIELD,
            SubtreeEffect::Await => SemanticFeatureFlags::AWAIT,
        };
        if !self.features.contains(feature) {
            return Err(FeatureNotEnabled { feature });
        }
        let counts = self
            .transform_facts
            .as_ref()
            .and_then(|facts| facts.effects.as_ref())
            .expect("enabled effects table must exist");
        let count = counts[node.index()];
        Ok(match effect {
            SubtreeEffect::Yield => count.yields != 0,
            SubtreeEffect::Await => count.awaits != 0,
        })
    }

    /// Removes the original yield contribution for a lowered generator without
    /// rescanning its AST. Each recorded yield updates only its ancestor path.
    pub fn clear_generator_yield_effects(&mut self, function: NodeId) {
        assert!(self.features.contains(SemanticFeatureFlags::YIELD));
        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        let yields = facts
            .yield_nodes
            .remove(&function)
            .map_or_else(Vec::new, |nodes| nodes.iter().copied().collect());
        let effects = facts
            .effects
            .as_mut()
            .expect("enabled effects table must exist");
        for effect_node in yields {
            let mut current = Some(effect_node);
            while let Some(node) = current {
                effects[node.index()].yields = effects[node.index()]
                    .yields
                    .checked_sub(1)
                    .expect("yield effect count underflow");
                if node == function {
                    break;
                }
                current = self.node_parents[node.index()];
            }
        }
        facts.function_captures.remove(&function);
    }

    pub fn finish_generator_transform(&mut self) {
        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        facts.generator_functions.clear();
    }

    pub fn effect_owner(&self, node: NodeId) -> Result<Option<NodeId>, FeatureNotEnabled> {
        let effects = SemanticFeatureFlags::YIELD | SemanticFeatureFlags::AWAIT;
        if !self.features.intersects(effects) {
            return Err(FeatureNotEnabled { feature: effects });
        }
        Ok(self
            .transform_facts
            .as_ref()
            .and_then(|facts| facts.effect_owners.as_ref())
            .expect("enabled effect owners must exist")[node.index()])
    }

    pub fn generator_functions_post_order(&self) -> Result<&[Function], FeatureNotEnabled> {
        if !self
            .features
            .contains(SemanticFeatureFlags::GENERATOR_FUNCTIONS)
        {
            return Err(FeatureNotEnabled {
                feature: SemanticFeatureFlags::GENERATOR_FUNCTIONS,
            });
        }
        Ok(&self
            .transform_facts
            .as_ref()
            .expect("enabled transform facts must exist")
            .generator_functions)
    }

    pub fn function_captures(
        &self,
        function: NodeId,
    ) -> Result<FunctionCaptures<'_>, FeatureNotEnabled> {
        if !self
            .features
            .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE)
        {
            return Err(FeatureNotEnabled {
                feature: SemanticFeatureFlags::FUNCTION_ENV_CAPTURE,
            });
        }
        let facts = self
            .transform_facts
            .as_ref()
            .expect("enabled transform facts must exist");
        Ok(facts.function_captures.get(&function).map_or(
            FunctionCaptures {
                arguments: &[],
                new_targets: &[],
            },
            |captures| FunctionCaptures {
                arguments: &captures.arguments,
                new_targets: &captures.new_targets,
            },
        ))
    }

    pub fn retarget_function_capture_arguments(
        &mut self,
        function: NodeId,
        old: &[ReferenceId],
        new: ReferenceId,
    ) {
        let Some(captures) = self
            .transform_facts
            .as_mut()
            .and_then(|facts| facts.function_captures.get_mut(&function))
        else {
            return;
        };
        captures
            .arguments
            .retain(|reference| !old.contains(reference));
        captures.arguments.push(new);
    }

    pub fn retarget_function_capture_new_targets(
        &mut self,
        function: NodeId,
        old: &[NodeId],
        new: NodeId,
    ) {
        let Some(captures) = self
            .transform_facts
            .as_mut()
            .and_then(|facts| facts.function_captures.get_mut(&function))
        else {
            return;
        };
        captures.new_targets.retain(|node| !old.contains(node));
        captures.new_targets.push(new);
    }

    pub fn async_functions_post_order(&self) -> Result<&[NodeId], FeatureNotEnabled> {
        if !self
            .features
            .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
        {
            return Err(FeatureNotEnabled {
                feature: SemanticFeatureFlags::ASYNC_FUNCTIONS,
            });
        }
        Ok(&self
            .transform_facts
            .as_ref()
            .expect("enabled transform facts must exist")
            .async_functions)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        assert!(id.index() < self.symbols.len());
        self.symbols[id.index()]
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        assert!(id.index() < self.references.len());
        self.references[id.index()]
    }

    /// The declaration with the given id.
    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        assert!(id.index() < self.declarations.len());
        self.declarations[id.index()]
    }

    /// The source text module's export occurrences and resolved targets.
    pub fn module_record(&self) -> &ModuleRecord<'a> {
        &self.module_record
    }

    /// The scope with the given id.
    pub fn scope(&self, id: ScopeId) -> scope::Scope {
        self.scopes.get(id)
    }

    /// The symbol declared at `node`, or the lexical fallback symbol of the
    /// reference at `node`. `None` when the node neither declares nor
    /// references a binding. Inspect [`ReferenceFlags::is_dynamic`] before
    /// treating a reference's fallback as its definite runtime binding.
    ///
    /// ## Example
    /// ```ts
    /// let a = 1; a;
    /// //  ^ the declared symbol
    /// //         ^ the same symbol, through the reference
    /// ```
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        assert!(node.index() < self.node_declarations.len());
        self.node_declarations[node.index()]
            .map(|id| self.declaration(id).symbol)
            .or_else(|| self.node_references[node.index()].and_then(|id| self.reference(id).symbol))
    }

    /// The declaration recorded at `node`, or `None` when the node is not a
    /// declaration site.
    pub fn declaration_of(&self, node: NodeId) -> Option<DeclarationId> {
        assert!(node.index() < self.node_declarations.len());
        self.node_declarations[node.index()]
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        assert!(node.index() < self.node_references.len());
        self.node_references[node.index()]
    }

    /// The innermost lexical scope containing `node`. A scope-creating
    /// node (function, block, class, ...) maps to the scope it creates;
    /// that scope's parent is the enclosing scope.
    pub fn scope_of(&self, node: NodeId) -> ScopeId {
        assert!(node.index() < self.node_scopes.len());
        self.node_scopes[node.index()]
    }

    /// The structural parent of `node`, or `None` at the root.
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        assert!(node.index() < self.node_parents.len());
        self.node_parents[node.index()]
    }

    /// Walks from `node` up to the root, yielding `node` first.
    pub fn ancestors(&self, node: Option<NodeId>) -> SemanticAncestorIterator<'_> {
        assert!(node.is_none_or(|node| node.index() < self.node_parents.len()));
        SemanticAncestorIterator {
            node_parents: &self.node_parents,
            current: node,
        }
    }

    /// Every declaration of `id`, in source order. Usually a single element.
    /// Symbols whose declarations merge (`var` redeclarations, TS overloads,
    /// `class` + `interface`) have one entry per declaration.
    ///
    /// ## Example
    /// ```ts
    /// var a = 1; var a = 2;
    /// //  ^ declarations[0]
    /// //             ^ declarations[1], same symbol
    /// ```
    ///
    /// A conflicting redeclaration is recorded here too, aliased onto
    /// the existing symbol so tooling on broken code still maps the node.
    pub fn declarations(&self, id: SymbolId) -> &[Declaration] {
        assert!(id.index() < self.declaration_lists.len());
        &self.declaration_lists[id.index()]
    }

    /// Every use site of `id`, in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::declarations`]. A dynamic
    /// `with` reference is included under its potential lexical fallback.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        assert!(id.index() < self.use_lists.len());
        &self.use_lists[id.index()]
    }

    /// Registers a newly created ordinary AST node in an existing scope.
    pub fn register_node(&mut self, node: NodeId, parent: Option<NodeId>, scope: ScopeId) {
        assert!(scope.index() < self.scopes.list.len());
        assert!(parent.is_none_or(|parent| {
            parent.index() < self.node_parents.len() && self.node_registered[parent.index()]
        }));
        let required = node.index() + 1;
        if self.node_scopes.len() < required {
            self.node_scopes.resize(required, ScopeId::ROOT);
            self.node_parents.resize(required, None);
            self.node_registered.resize(required, false);
            self.node_declarations.resize(required, None);
            self.node_references.resize(required, None);
            if let Some(facts) = &mut self.transform_facts {
                if let Some(effects) = &mut facts.effects {
                    effects.resize(required, EffectCounts::default());
                }
                if let Some(owners) = &mut facts.effect_owners {
                    owners.resize(required, None);
                }
            }
        }
        assert!(!self.node_registered[node.index()], "node registered twice");
        self.node_registered[node.index()] = true;
        self.node_scopes[node.index()] = scope;
        self.node_parents[node.index()] = parent;
        if let Some(owners) = self
            .transform_facts
            .as_mut()
            .and_then(|facts| facts.effect_owners.as_mut())
        {
            owners[node.index()] = parent.and_then(|parent| owners[parent.index()]);
        }
    }

    pub fn reparent_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        assert!(self.node_registered[node.index()]);
        assert!(parent.is_none_or(|parent| self.node_registered[parent.index()]));
        self.node_parents[node.index()] = parent;
        if let Some(owners) = self
            .transform_facts
            .as_mut()
            .and_then(|facts| facts.effect_owners.as_mut())
        {
            owners[node.index()] = parent.and_then(|parent| owners[parent.index()]);
        }
    }

    /// Appends a generated binding without renumbering an existing semantic ID.
    pub fn declare(
        &mut self,
        ast: &Ast<'a>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: SymbolFlags,
    ) -> SymbolId {
        assert!(self.node_registered[node.index()]);
        assert!(self.node_declarations[node.index()].is_none());
        assert!(self.binding(scope, name).is_none());
        let symbol = SymbolId::from_usize(self.symbols.len());
        let declaration_start =
            u32::try_from(self.declarations.len()).expect("declaration index overflow");
        self.symbols.push(Symbol {
            name,
            flags,
            scope,
            declarations: Range {
                start: declaration_start,
                len: 1,
            },
        });
        self.declaration_lists
            .push(ArenaVec::new_in(ast.allocator()));
        self.use_lists.push(ArenaVec::new_in(ast.allocator()));
        self.scope_maps[scope.index()].insert(name, symbol);
        let declaration = Declaration {
            node,
            symbol,
            flags,
        };
        let declaration_id = DeclarationId::from_usize(self.declarations.len());
        self.declarations.push(declaration);
        self.declaration_lists[symbol.index()].push(declaration);
        self.node_declarations[node.index()] = Some(declaration_id);
        self.observe_generated_name(ast, scope, name);
        symbol
    }

    /// Appends a generated reference and updates its target symbol use list.
    pub fn reference_generated(
        &mut self,
        ast: &Ast<'_>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: ReferenceFlags,
        symbol: Option<SymbolId>,
    ) -> ReferenceId {
        assert!(self.node_registered[node.index()]);
        assert!(self.node_references[node.index()].is_none());
        assert!(symbol.is_none_or(|symbol| symbol.index() < self.symbols.len()));
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(Reference {
            name,
            scope,
            node,
            symbol,
            flags,
        });
        self.node_references[node.index()] = Some(id);
        if let Some(symbol) = symbol {
            self.use_lists[symbol.index()].push(id);
        }
        self.observe_generated_name(ast, scope, name);
        id
    }

    /// Rebinds a non-dynamic reference while preserving its stable ID.
    pub fn rebind_reference(&mut self, id: ReferenceId, symbol: Option<SymbolId>) {
        assert!(id.index() < self.references.len());
        assert!(symbol.is_none_or(|symbol| symbol.index() < self.symbols.len()));
        let old = self.references[id.index()].symbol;
        if old == symbol {
            return;
        }
        assert!(!self.references[id.index()].flags.is_dynamic());
        if let Some(old) = old {
            let uses = &mut self.use_lists[old.index()];
            let index = uses
                .iter()
                .position(|candidate| *candidate == id)
                .expect("old symbol use index must contain reference");
            uses.remove(index);
        }
        self.references[id.index()].symbol = symbol;
        if let Some(symbol) = symbol {
            let uses = &mut self.use_lists[symbol.index()];
            let index = uses
                .binary_search_by_key(&id.index(), |candidate| candidate.index())
                .unwrap_or_else(|index| index);
            uses.insert(index, id);
        }
    }

    /// Renames a reference while preserving its stable ID and binding.
    pub fn rename_reference(&mut self, ast: &Ast<'_>, id: ReferenceId, name: AstStr) {
        assert!(id.index() < self.references.len());
        let scope = self.references[id.index()].scope;
        self.references[id.index()].name = name;
        self.observe_generated_name(ast, scope, name);
    }

    /// Renames a symbol and all of its references while preserving IDs.
    pub fn rename_symbol(&mut self, ast: &Ast<'_>, id: SymbolId, name: AstStr) {
        assert!(id.index() < self.symbols.len());
        let symbol = self.symbols[id.index()];
        assert!(
            self.binding(symbol.scope, name)
                .is_none_or(|existing| existing == id)
        );
        self.scope_maps[symbol.scope.index()].remove(&symbol.name);
        self.scope_maps[symbol.scope.index()].insert(name, id);
        self.symbols[id.index()].name = name;
        let uses: Vec<_> = self.use_lists[id.index()].iter().copied().collect();
        for reference in uses {
            self.references[reference.index()].name = name;
        }
        self.observe_generated_name(ast, symbol.scope, name);
    }

    /// The binding of `name` at `scope` alone, including a hoisting
    /// `var` passing through on its way to its hoist target. This does
    /// not walk the scope chain; see [`Semantic::lookup`].
    ///
    /// ## Example
    /// ```ts
    /// function f() { { var a; } }
    /// //             ^ binding here finds `a`, whose symbol
    /// //               lives in the function scope
    /// ```
    pub fn binding(&self, scope_id: ScopeId, name: AstStr) -> Option<SymbolId> {
        assert!(scope_id.index() < self.scope_maps.len());
        self.scope_maps[scope_id.index()]
            .get(&name)
            .copied()
            .or_else(|| {
                self.hoisting_variables[scope_id.index()]
                    .get(&name)
                    .copied()
            })
    }

    fn observe_generated_name(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        if !self
            .features
            .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE)
        {
            return;
        }
        binder::observe_used_name(ast, &mut self.direct_symbol_names[scope.index()], name);
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::observe_used_name(ast, &mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
    }

    pub fn name_allocation_state(&self, scope: ScopeId, base: AstStr) -> UsedNameState {
        assert!(
            self.features
                .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE),
            "generated-name allocation requires FUNCTION_ENV_CAPTURE"
        );
        let mut state = self.used_names[scope.index()]
            .get(&base)
            .copied()
            .unwrap_or_default();
        let parent = self.scope(scope).parent;
        for ancestor in self.scopes.ancestors(parent) {
            if let Some(ancestor_state) = self.direct_symbol_names[ancestor.index()].get(&base) {
                state.bare_used |= ancestor_state.bare_used;
                state.next_suffix = state.next_suffix.max(ancestor_state.next_suffix);
            }
        }
        state
    }

    /// Reserves a generated name in `scope` and every overlapping name index.
    /// A later allocation, including one performed by another transform pass,
    /// observes the reservation immediately.
    pub fn reserve_generated_name(
        &mut self,
        ast: &mut Ast<'_>,
        scope: ScopeId,
        base: &str,
    ) -> AstUtf8 {
        self.reserve_generated_name_with_min_suffix(ast, scope, base, 1)
    }

    pub fn reserve_generated_name_with_min_suffix(
        &mut self,
        ast: &mut Ast<'_>,
        scope: ScopeId,
        base: &str,
        min_suffix: u64,
    ) -> AstUtf8 {
        assert!(min_suffix > 0);
        assert!(scope.index() < self.used_names.len());
        let base_text = ast.add_utf8(base);
        let base_name = ast.canonicalize_utf8(base_text);
        let state = self.name_allocation_state(scope, base_name);
        let text = if state.bare_used {
            let suffix = state.next_suffix.max(min_suffix);
            let mut candidate = base.to_owned();
            candidate.push_str(&suffix.to_string());
            ast.add_utf8(&candidate)
        } else {
            base_text
        };
        let name = ast.canonicalize_utf8(text);

        self.observe_generated_name(ast, scope, name);
        text
    }

    pub fn binding_text(&self, ast: &Ast<'_>, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        ast.canonical_utf8(name)
            .and_then(|name| self.binding(scope_id, name))
    }

    /// Every symbol declared directly in `scope`. A hoisting `var`
    /// appears in its hoist target, not in blocks it passes through.
    pub fn bindings(&self, scope_id: ScopeId) -> BindingIterator<'_> {
        assert!(scope_id.index() < self.scope_maps.len());
        BindingIterator {
            inner: self.scope_maps[scope_id.index()].values(),
        }
    }

    /// The nearest lexical fallback binding of `name` visible in `space`
    /// from `scope`. A binding outside the space does not shadow.
    /// [`ReferenceSpace::Any`] matches by name alone. This does not query
    /// runtime object properties when the path crosses [`scope::ScopeKind::With`].
    ///
    /// ## Example
    /// ```ts
    /// type T = string;
    /// function f() {
    ///   const T = 1;
    ///   // type space finds the outer alias, value space the local const
    /// }
    /// ```
    pub fn lookup(
        &self,
        scope_id: ScopeId,
        name: AstStr,
        space: ReferenceSpace,
    ) -> Option<SymbolId> {
        for ancestor in self.scopes.ancestors(Some(scope_id)) {
            let Some(&id) = self.scope_maps[ancestor.index()].get(&name) else {
                continue;
            };
            if self.symbol(id).flags.visible_in(space) {
                return Some(id);
            }
        }
        None
    }

    /// Iterates every `(id, scope)` pair in creation order.
    pub fn iter_scopes(&self) -> ScopeIterator<'_> {
        ScopeIterator {
            list: &self.scopes.list,
            index: 0,
        }
    }

    /// Iterates every `(id, symbol)` pair in declaration order.
    pub fn iter_symbols(&self) -> SymbolIterator<'_> {
        SymbolIterator {
            symbols: &self.symbols,
            index: 0,
        }
    }

    /// Iterates every `(id, reference)` pair in source order.
    pub fn iter_references(&self) -> ReferenceIterator<'_> {
        ReferenceIterator {
            references: &self.references,
            index: 0,
        }
    }

    /// Iterates every `(id, declaration)` pair, grouped by symbol and in
    /// source order within each symbol.
    pub fn iter_declarations(&self) -> DeclarationIterator<'_> {
        DeclarationIterator {
            declarations: &self.declarations,
            index: 0,
        }
    }
}

/// A `(id, scope)` pair yielded by [`Semantic::iter_scopes`].
pub struct ScopeEntry {
    pub id: ScopeId,
    pub scope: scope::Scope,
}

/// A `(id, symbol)` pair yielded by [`Semantic::iter_symbols`].
pub struct SymbolEntry {
    pub id: SymbolId,
    pub symbol: Symbol,
}

/// A `(id, reference)` pair yielded by [`Semantic::iter_references`].
pub struct ReferenceEntry {
    pub id: ReferenceId,
    pub reference: Reference,
}

/// A `(id, declaration)` pair yielded by [`Semantic::iter_declarations`].
pub struct DeclarationEntry {
    pub id: DeclarationId,
    pub declaration: Declaration,
}

pub struct ScopeIterator<'a> {
    list: &'a [scope::Scope],
    index: u32,
}

impl Iterator for ScopeIterator<'_> {
    type Item = ScopeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let scope = *self.list.get(index)?;
        self.index += 1;
        Some(ScopeEntry {
            id: ScopeId::from_usize(index),
            scope,
        })
    }
}

pub struct SemanticAncestorIterator<'a> {
    node_parents: &'a [Option<NodeId>],
    current: Option<NodeId>,
}

impl Iterator for SemanticAncestorIterator<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        assert!(node.index() < self.node_parents.len());
        self.current = self.node_parents[node.index()];
        Some(node)
    }
}

pub struct BindingIterator<'a> {
    inner: Values<'a, AstStr, SymbolId>,
}

impl Iterator for BindingIterator<'_> {
    type Item = SymbolId;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

pub struct SymbolIterator<'a> {
    symbols: &'a [Symbol],
    index: u32,
}

impl Iterator for SymbolIterator<'_> {
    type Item = SymbolEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let symbol = *self.symbols.get(index)?;
        self.index += 1;
        Some(SymbolEntry {
            id: SymbolId::from_usize(index),
            symbol,
        })
    }
}

pub struct ReferenceIterator<'a> {
    references: &'a [Reference],
    index: u32,
}

pub struct DeclarationIterator<'a> {
    declarations: &'a [Declaration],
    index: u32,
}

impl Iterator for ReferenceIterator<'_> {
    type Item = ReferenceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let reference = *self.references.get(index)?;
        self.index += 1;
        Some(ReferenceEntry {
            id: ReferenceId::from_usize(index),
            reference,
        })
    }
}

impl Iterator for DeclarationIterator<'_> {
    type Item = DeclarationEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index as usize;
        let declaration = *self.declarations.get(index)?;
        self.index += 1;
        Some(DeclarationEntry {
            id: DeclarationId::from_usize(index),
            declaration,
        })
    }
}

/// Semantic analysis output and any requested scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AnalyzeOptions {
    /// Whether to check and report scope-dependent syntax errors.
    ///
    /// Disabled by default.
    pub check_syntax: bool,
    pub features: SemanticFeatureFlags,
}

/// Runs semantic analysis on a tree and builds the complete [`Semantic`] model.
///
/// When [`AnalyzeOptions::check_syntax`] is enabled, scope-dependent early
/// errors are returned separately from parser diagnostics. Their messages use
/// the tree's arena, so the result is valid as long as the tree is alive.
pub fn analyze<'a>(root: Program, ast: &Ast<'a>, options: AnalyzeOptions) -> SemanticReturn<'a> {
    let mut visitor = Checker::new(root, ast, options.check_syntax, options.features);
    root.visit_with(&mut visitor);
    let (semantic, diagnostics) = visitor.finish();
    SemanticReturn {
        semantic,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, Lang, NodeKindData, ScopeId, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, SymbolFlags, analyze,
        scope::ScopeKind,
    };

    fn diagnostics(source: &str, lang: Lang) -> Vec<String> {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Script,
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.message.into_owned())
        .collect()
    }

    #[test]
    fn declarations_are_grouped_by_symbol_in_source_order() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("interface A {} interface B {} interface A {}", &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Script,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let a = semantic.binding_text(&ast, ScopeId::ROOT, "A").unwrap();
        let b = semantic.binding_text(&ast, ScopeId::ROOT, "B").unwrap();
        let a_declarations = semantic.declarations(a);
        assert_eq!(a_declarations.len(), 2);
        assert!(ast.span(a_declarations[0].node).start < ast.span(a_declarations[1].node).start);

        let symbols: Vec<_> = semantic
            .iter_declarations()
            .map(|entry| entry.declaration.symbol)
            .collect();
        assert_eq!(symbols, [a, a, b]);
        for entry in semantic.iter_declarations() {
            assert_eq!(
                semantic.declaration_of(entry.declaration.node),
                Some(entry.id)
            );
        }
    }

    #[test]
    fn annex_b() {
        // Sloppy JavaScript blocks allow duplicate ordinary functions and
        // isolate functions declared directly in if/else clauses.
        for source in [
            "{ function f() {} function f() {} }",
            "{ function f() {} function f() { 'use strict'; } }",
            "{ function f() { 'use strict'; } function f() {} }",
            "switch (0) { case 0: function f() {} case 1: function f() {} }",
            "function outer() { { let f; if (true) function f() {} else function f() {} } try {} catch (f) { if (true) function f() {} } }",
        ] {
            assert!(diagnostics(source, Lang::Js).is_empty(), "{source}");
        }

        // Strict blocks and collisions with other value declarations remain
        // errors.
        for source in [
            "'use strict'; { function f() {} function f() {} }",
            "{ var f; function f() {} }",
            "{ function f() {} function* f() {} }",
            "{ function f() {} async function f() {} }",
        ] {
            assert!(!diagnostics(source, Lang::Js).is_empty(), "{source}");
        }

        // TypeScript retains its var/function merge rules outside catch
        // scopes.
        for source in [
            "{ var f; function f() {} }",
            "switch (0) { case 0: var f; case 1: function f() {} }",
        ] {
            assert!(diagnostics(source, Lang::Ts).is_empty(), "{source}");
        }

        // A TypeScript catch scope still rejects the merge.
        let source = "try {} catch (e) { var f; function f() {} }";
        assert!(!diagnostics(source, Lang::Ts).is_empty());
    }

    #[test]
    fn validates_unicode_property_escapes() {
        for source in [
            r"/\p{ASCII}/u;",
            r"/\p{Script=Latin}/u;",
            r"/\p{Basic_Emoji}/v;",
        ] {
            assert!(diagnostics(source, Lang::Js).is_empty(), "{source}");
        }

        for source in [
            r"/\p{ASCII=Yes}/u;",
            r"/[\p{ASCII}-a]/u;",
            r"/\p{Basic_Emoji}/u;",
            r"/\P{Basic_Emoji}/v;",
            r"/[^\p{Basic_Emoji}]/v;",
        ] {
            assert_eq!(
                diagnostics(source, Lang::Js),
                ["Invalid regular expression literal"],
                "{source}"
            );
        }
    }

    #[test]
    fn arrow_initializer_preserves_export_context() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "export const f = parameter => { const hidden = 0; }, g = 1;",
            &allocator,
        );
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let exported = |name: &[u8]| {
            semantic
                .iter_symbols()
                .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == name)
                .expect("symbol")
                .symbol
                .flags
                .contains(SymbolFlags::EXPORTED)
        };

        assert!(exported(b"f"));
        assert!(exported(b"g"));
        assert!(!exported(b"parameter"));
        assert!(!exported(b"hidden"));
    }

    #[test]
    fn binds_qualified_namespace_heads() {
        let allocator = Allocator::new();
        let source = "namespace A.B {} let value: A.B;";
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Script,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let statement = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let declaration = statement.as_declaration(&ast).unwrap();
        let declaration = declaration.as_ts_module_declaration(&ast).unwrap();
        let NodeKindData::TsQualifiedName(name) = ast.kind_data(declaration.id(&ast).node_id())
        else {
            panic!("dotted namespace did not parse as a qualified name");
        };
        let head = name.left(&ast).node_id();

        let result = analyze(parse.program, &ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        let declaration = result.semantic.declaration_of(head).unwrap();
        let symbol = result.semantic.declaration(declaration).symbol;
        assert_eq!(result.semantic.symbol_of(head), Some(symbol));

        let reference = result
            .semantic
            .iter_references()
            .find(|entry| {
                std::str::from_utf8(ast.get_ast_str_bytes(entry.reference.name)).unwrap() == "A"
            })
            .unwrap()
            .reference;
        assert_eq!(reference.symbol, Some(symbol));
        assert_eq!(reference.flags.space, ReferenceSpace::Namespace);
    }

    #[test]
    fn binds_local_export_specifiers() {
        let allocator = Allocator::new();
        let source = "export { value }; export type { Type }; export { external as alias } from 'dep'; const value = 1; type Type = string;";
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let body = parse.program.body(&ast);
        let export_name = |index| {
            let statement = body.get_node(&ast, index).unwrap();
            let declaration = statement.as_export_named_declaration(&ast).unwrap();
            let specifier = declaration.specifiers(&ast).get_node(&ast, 0).unwrap();
            specifier.local(&ast).node_id()
        };
        let value_node = export_name(0);
        let type_node = export_name(1);
        let re_export_node = export_name(2);

        let result = analyze(parse.program, &ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);

        let value_reference = result.semantic.reference_of(value_node).unwrap();
        let value_reference = result.semantic.reference(value_reference);
        let value_symbol = result
            .semantic
            .lookup(
                value_reference.scope,
                ast.canonical_utf8("value").unwrap(),
                ReferenceSpace::Any,
            )
            .unwrap();
        assert_eq!(value_reference.symbol, Some(value_symbol));
        assert_eq!(value_reference.flags.space, ReferenceSpace::Any);

        let type_reference = result.semantic.reference_of(type_node).unwrap();
        let type_reference = result.semantic.reference(type_reference);
        let type_symbol = result
            .semantic
            .lookup(
                type_reference.scope,
                ast.canonical_utf8("Type").unwrap(),
                ReferenceSpace::Any,
            )
            .unwrap();
        assert_eq!(type_reference.symbol, Some(type_symbol));
        assert_eq!(type_reference.flags.space, ReferenceSpace::Any);

        assert_eq!(result.semantic.reference_of(re_export_node), None);
    }

    #[test]
    fn models_local_and_external_export_targets() {
        let allocator = Allocator::new();
        let source = "export { value }; export { external as alias } from 'dep'; const value = 1;";
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Js,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

        let body = parse.program.body(&ast);
        let specifier = |index| {
            let statement = body.get_node(&ast, index).unwrap();
            let declaration = statement.as_export_named_declaration(&ast).unwrap();
            declaration.specifiers(&ast).get_node(&ast, 0).unwrap()
        };
        let local_specifier = specifier(0);
        let re_export_specifier = specifier(1);
        let local_node = local_specifier.local(&ast).node_id();

        let result = analyze(parse.program, &ast, AnalyzeOptions::default());
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
        let local_reference = result.semantic.reference_of(local_node).unwrap();
        let local_symbol = result.semantic.reference(local_reference).symbol.unwrap();

        let local_export = result
            .semantic
            .module_record()
            .exports()
            .iter()
            .find(|entry| entry.node == local_specifier.node_id())
            .unwrap();
        assert_eq!(local_export.origin, ExportOrigin::LocalSpecifier);
        assert_eq!(
            local_export.target,
            ExportTarget::Local {
                symbol: local_symbol,
                declaration: None,
                reference: Some(local_reference),
            }
        );

        let re_export = result
            .semantic
            .module_record()
            .exports()
            .iter()
            .find(|entry| entry.node == re_export_specifier.node_id())
            .unwrap();
        assert_eq!(re_export.origin, ExportOrigin::ReExportSpecifier);
        assert!(matches!(re_export.target, ExportTarget::External { .. }));
    }

    #[test]
    fn type_only_import_visibility() {
        let source = "import type { TypeOnly, Merged } from 'mod'; import type Alias = TypeOnly; TypeOnly; type T = TypeOnly; type N = TypeOnly.Member; type Q = typeof TypeOnly; export default TypeOnly; export type { TypeOnly }; const Merged: Merged = 1; Merged;";
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let type_only = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"TypeOnly")
            .expect("type-only import symbol")
            .id;
        assert!(
            semantic
                .symbol(type_only)
                .flags
                .contains(SymbolFlags::TYPE_IMPORT)
        );
        let type_only_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"TypeOnly")
            .map(|entry| {
                (
                    entry.reference.flags.space,
                    entry.reference.symbol,
                    entry.reference.flags.is_read(),
                    entry.reference.flags.is_write(),
                )
            })
            .collect();
        assert_eq!(
            type_only_references,
            [
                (ReferenceSpace::Any, Some(type_only), false, false),
                (ReferenceSpace::Value, None, true, false),
                (ReferenceSpace::Type, Some(type_only), false, false),
                (ReferenceSpace::Namespace, Some(type_only), false, false,),
                (ReferenceSpace::Typeof, Some(type_only), false, false),
                (ReferenceSpace::Any, Some(type_only), false, false),
                (ReferenceSpace::Any, Some(type_only), false, false),
            ]
        );

        let merged = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"Merged")
            .expect("merged import and value symbol")
            .id;
        let merged_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"Merged")
            .map(|entry| {
                (
                    entry.reference.flags.space,
                    entry.reference.symbol,
                    entry.reference.flags.is_read(),
                    entry.reference.flags.is_write(),
                )
            })
            .collect();
        assert_eq!(
            merged_references,
            [
                (ReferenceSpace::Type, Some(merged), false, false),
                (ReferenceSpace::Value, Some(merged), true, false),
            ]
        );
    }

    #[test]
    fn default_and_export_assignment_reference_spaces() {
        fn export_reference(source: &str, name: &[u8]) -> (ReferenceSpace, bool, bool) {
            let allocator = Allocator::new();
            let mut ast = Ast::new(source, &allocator);
            let parse = Parser::init(
                &mut ast,
                Options {
                    source_type: SourceType::Module,
                    lang: Lang::Ts,
                    ..Options::default()
                },
                NoTokenParserConfig,
            )
            .parse();
            assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);

            let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
            let reference = semantic
                .iter_references()
                .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == name)
                .expect("export reference");
            (
                reference.reference.flags.space,
                reference.reference.symbol.is_some(),
                reference.reference.flags.is_read(),
            )
        }

        assert_eq!(
            export_reference("type T = {}; export default T;", b"T"),
            (ReferenceSpace::Any, true, false)
        );
        assert_eq!(
            export_reference("type T = {}; export default T || {};", b"T"),
            (ReferenceSpace::Value, false, true)
        );
        assert_eq!(
            export_reference("type T = {}; export = T;", b"T"),
            (ReferenceSpace::Any, true, false)
        );
        assert_eq!(
            export_reference(
                "namespace N { export interface T {} } export default N.T;",
                b"N",
            ),
            (ReferenceSpace::Value, false, true)
        );
        assert_eq!(
            export_reference("namespace N { export interface T {} } export = N.T;", b"N",),
            (ReferenceSpace::Value, false, true)
        );
        assert_eq!(
            export_reference(
                "namespace N { export interface T {} } export = N['T'];",
                b"N",
            ),
            (ReferenceSpace::Value, false, true)
        );
    }

    #[test]
    fn alias_access_follows_the_resolved_symbol() {
        let source = r#"
            type T = {};
            const V = 1;
            namespace N { export interface I {} }
            import { Imported } from "mod";
            import type { TypeImported } from "mod";
            namespace Q { export const Member = 1; }
            export import Qualified = Q.Member;
            import type TypeQualified = Q.Member;
            export { T, V, N, Imported, TypeImported, Missing };
            export type { V as ValueType };
            export default T;
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let references = |name: &[u8]| {
            semantic
                .iter_references()
                .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == name)
                .map(|entry| {
                    (
                        entry.reference.flags.space,
                        entry.reference.symbol.is_some(),
                        entry.reference.flags.is_read(),
                        entry.reference.flags.is_write(),
                    )
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(
            references(b"T"),
            [
                (ReferenceSpace::Any, true, false, false),
                (ReferenceSpace::Any, true, false, false),
            ]
        );
        assert_eq!(
            references(b"V"),
            [
                (ReferenceSpace::Any, true, true, false),
                (ReferenceSpace::Any, true, false, false),
            ]
        );
        assert_eq!(
            references(b"N"),
            [(ReferenceSpace::Any, true, false, false)]
        );
        assert_eq!(
            references(b"Imported"),
            [(ReferenceSpace::Any, true, true, false)]
        );
        assert_eq!(
            references(b"TypeImported"),
            [(ReferenceSpace::Any, true, false, false)]
        );
        assert_eq!(
            references(b"Missing"),
            [(ReferenceSpace::Any, false, true, false)]
        );
        assert_eq!(
            references(b"Q"),
            [
                (ReferenceSpace::Any, true, true, false),
                (ReferenceSpace::Any, true, false, false),
            ]
        );
    }

    #[test]
    fn reference_access_flags() {
        let source = r#"
            let read, write, update, object, key, deleted;
            read;
            write = 1;
            update++;
            object[key] = 1;
            delete deleted;
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let flags = |name: &[u8]| {
            semantic
                .iter_references()
                .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == name)
                .expect("reference")
                .reference
                .flags
        };

        assert!(flags(b"read").is_read_only());
        assert!(flags(b"write").is_write_only());
        assert!(flags(b"update").is_read_write());
        assert!(flags(b"object").is_read_only());
        assert!(flags(b"key").is_read_only());
        assert!(!flags(b"deleted").is_read());
        assert!(!flags(b"deleted").is_write());
    }

    #[test]
    fn erased_declarations_do_not_read_runtime_values() {
        let source = r#"
            import type { TypeOnlyKey } from "mod";
            declare const decorator: any;
            const Base = class {};
            const DecoratedKey = Symbol();
            const DecoratedMethodKey = Symbol();
            const OverloadKey = Symbol();
            const RuntimeKey = Symbol();
            declare class Ambient extends Base { [Base](): void }
            declare class AmbientEval extends eval("ambient") {}
            declare enum AmbientEnum { Member = Base }
            abstract class AbstractMember { abstract [TypeOnlyKey](): void }
            abstract class AbstractProperty { abstract [TypeOnlyKey]: unknown }
            class DeclaredMember { declare [TypeOnlyKey]: unknown }
            class DecoratedMember { @decorator declare [DecoratedKey]: unknown }
            class DecoratedOverload {
                @decorator
                [DecoratedMethodKey](): void;
                [DecoratedMethodKey]() {}
            }
            class OverloadedMember {
                [OverloadKey](): void;
                [OverloadKey]() {}
            }
            class RuntimeMember { [RuntimeKey]() {} }
            class Runtime extends Base { [Base]() {} }
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                source_type: SourceType::Module,
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let accesses = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"Base")
            .map(|entry| {
                (
                    entry.reference.flags.space,
                    entry.reference.symbol.is_some(),
                    entry.reference.flags.is_read(),
                    entry.reference.flags.is_write(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            accesses,
            [
                (ReferenceSpace::Value, true, false, false),
                (ReferenceSpace::Typeof, true, false, false),
                (ReferenceSpace::Value, true, false, false),
                (ReferenceSpace::Value, true, true, false),
                (ReferenceSpace::Value, true, true, false),
            ]
        );
        let accesses = |name: &[u8]| {
            semantic
                .iter_references()
                .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == name)
                .map(|entry| {
                    (
                        entry.reference.flags.space,
                        entry.reference.symbol.is_some(),
                        entry.reference.flags.is_read(),
                        entry.reference.flags.is_write(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            accesses(b"TypeOnlyKey"),
            [
                (ReferenceSpace::Typeof, true, false, false),
                (ReferenceSpace::Typeof, true, false, false),
                (ReferenceSpace::Typeof, true, false, false),
            ]
        );
        assert_eq!(
            accesses(b"DecoratedKey"),
            [(ReferenceSpace::Value, true, true, false)]
        );
        assert_eq!(
            accesses(b"DecoratedMethodKey"),
            [
                (ReferenceSpace::Value, true, true, false),
                (ReferenceSpace::Value, true, true, false),
            ]
        );
        assert_eq!(
            accesses(b"OverloadKey"),
            [
                (ReferenceSpace::Typeof, true, false, false),
                (ReferenceSpace::Value, true, true, false),
            ]
        );
        assert_eq!(
            accesses(b"RuntimeKey"),
            [(ReferenceSpace::Value, true, true, false)]
        );
        assert!(
            semantic
                .iter_scopes()
                .all(|entry| !entry.scope.flags.contains_direct_eval())
        );
    }

    #[test]
    fn with_scope_marks_dynamic_references() {
        let source = r#"
            let object, second, outer;
            with (object) {
                let local;
                local;
                outer;
                missing;
                function nested() {
                    arguments;
                    outer;
                    with (second) {
                        local;
                        arguments;
                    }
                }
            }
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        let outer = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"outer")
            .expect("outer binding")
            .id;
        let reference = |name: &[u8]| {
            semantic
                .iter_references()
                .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == name)
                .expect("reference")
                .reference
        };

        let object = reference(b"object");
        assert!(!object.flags.is_dynamic());
        assert!(object.symbol.is_some());

        let local_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"local")
            .collect();
        let [local, intercepted_local] = local_references.as_slice() else {
            panic!("two local references")
        };
        assert!(!local.reference.flags.is_dynamic());
        assert!(local.reference.symbol.is_some());
        assert!(intercepted_local.reference.flags.is_dynamic());
        assert_eq!(intercepted_local.reference.symbol, local.reference.symbol);

        let missing = reference(b"missing");
        assert!(missing.flags.is_dynamic());
        assert!(missing.symbol.is_none());

        let outer_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"outer")
            .collect();
        assert_eq!(outer_references.len(), 2);
        assert!(outer_references.iter().all(|entry| {
            entry.reference.flags.is_dynamic() && entry.reference.symbol == Some(outer)
        }));
        let outer_uses: Vec<_> = outer_references.iter().map(|entry| entry.id).collect();
        assert_eq!(semantic.uses(outer), outer_uses);

        let arguments_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"arguments")
            .collect();
        let [arguments, intercepted_arguments] = arguments_references.as_slice() else {
            panic!("two arguments references")
        };
        assert!(!arguments.reference.flags.is_dynamic());
        assert!(arguments.reference.symbol.is_none());
        assert!(intercepted_arguments.reference.flags.is_dynamic());
        assert!(intercepted_arguments.reference.symbol.is_none());

        let second = reference(b"second");
        assert!(second.flags.is_dynamic());
        assert!(second.symbol.is_some());

        let with_scopes: Vec<_> = semantic
            .iter_scopes()
            .filter(|entry| entry.scope.kind == ScopeKind::With)
            .collect();
        assert_eq!(with_scopes.len(), 2);
        for with_scope in &with_scopes {
            assert_eq!(semantic.scope_of(with_scope.scope.node), with_scope.id);
        }
        assert!(
            with_scopes
                .iter()
                .any(|entry| entry.scope.parent == Some(object.scope))
        );
        assert!(
            with_scopes
                .iter()
                .any(|entry| entry.scope.parent == Some(second.scope))
        );
    }

    #[test]
    fn with_scope_marks_var_binding_writes_dynamic() {
        let source = r#"
            var source, values;
            with ({}) {
                var untouched;
                var initialized = 1;
                var { value: destructured } = source;
                var [ordered = ordered] = source;
                var [defaulted = function () { var nested = 1; }] = source;
                for (var iterated of values) {}
                function boundary() { var local = 1; }
                let lexical = 1;
            }
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        let declaration = |name: &[u8]| {
            semantic
                .iter_declarations()
                .find(|entry| {
                    let symbol = semantic.symbol(entry.declaration.symbol);
                    ast.get_ast_str_bytes(symbol.name) == name
                })
                .expect("declaration")
        };
        for name in [
            b"initialized".as_slice(),
            b"destructured",
            b"ordered",
            b"defaulted",
            b"iterated",
        ] {
            let declaration = declaration(name);
            assert_eq!(
                semantic.declaration_of(declaration.declaration.node),
                Some(declaration.id)
            );
            let reference_id = semantic
                .reference_of(declaration.declaration.node)
                .expect("runtime write reference");
            let reference = semantic.reference(reference_id);
            assert!(reference.flags.is_write_only());
            assert!(reference.flags.is_dynamic());
            assert_eq!(reference.symbol, Some(declaration.declaration.symbol));
            assert!(
                semantic
                    .uses(declaration.declaration.symbol)
                    .contains(&reference_id)
            );
        }

        let ordered = declaration(b"ordered").declaration.symbol;
        let ordered_references: Vec<_> = semantic
            .iter_references()
            .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"ordered")
            .collect();
        let [write, read] = ordered_references.as_slice() else {
            panic!("ordered binding write and default read")
        };
        assert!(write.reference.flags.is_write_only());
        assert!(write.reference.flags.is_dynamic());
        assert!(read.reference.flags.is_read_only());
        assert!(read.reference.flags.is_dynamic());
        assert_eq!(semantic.uses(ordered), &[write.id, read.id]);

        for name in [b"untouched".as_slice(), b"nested", b"local", b"lexical"] {
            let declaration = declaration(name);
            assert_eq!(semantic.reference_of(declaration.declaration.node), None);
        }
    }

    #[test]
    fn direct_eval_marks_lexical_ancestor_scopes() {
        let source = r#"
            function parameter(value = eval("value")) { let body_local; }
            function shadowed(eval) { eval("shadowed"); }
            function indirect() {
                (0, eval)("comma");
                eval?.("optional");
            }
            function deleted() { delete eval("deleted"); }
            function parent() {
                function child() { (eval)("child"); }
            }
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        let function_scope = |name: &[u8]| {
            let entry = semantic
                .iter_scopes()
                .find(|entry| {
                    if entry.scope.kind != ScopeKind::Function {
                        return false;
                    }
                    let NodeKindData::Function(function) = ast.kind_data(entry.scope.node) else {
                        return false;
                    };
                    function
                        .id(&ast)
                        .is_some_and(|id| ast.get_utf8(id.name(&ast)).as_bytes() == name)
                })
                .expect("function scope");
            (entry.id, entry.scope)
        };

        let (parameter_id, parameter) = function_scope(b"parameter");
        assert!(parameter.flags.contains_direct_eval());
        let parameter_body = semantic
            .iter_scopes()
            .find(|entry| {
                entry.scope.kind == ScopeKind::FunctionBody
                    && entry.scope.parent == Some(parameter_id)
            })
            .expect("parameter function body scope");
        assert!(!parameter_body.scope.flags.contains_direct_eval());

        assert!(function_scope(b"shadowed").1.flags.contains_direct_eval());
        assert!(!function_scope(b"indirect").1.flags.contains_direct_eval());
        assert!(function_scope(b"deleted").1.flags.contains_direct_eval());
        assert!(function_scope(b"parent").1.flags.contains_direct_eval());
        assert!(function_scope(b"child").1.flags.contains_direct_eval());
        assert!(semantic.scope(ScopeId::ROOT).flags.contains_direct_eval());
    }

    #[test]
    fn direct_eval_uses_runtime_transparent_typescript_wrappers() {
        let source = r#"
            function direct() { (eval as any)("direct"); }
            function indirect() { (0, eval as any)("indirect"); }
        "#;
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(
            &mut ast,
            Options {
                lang: Lang::Ts,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let contains_direct_eval = |name: &[u8]| {
            semantic
                .iter_scopes()
                .find(|entry| {
                    if entry.scope.kind != ScopeKind::Function {
                        return false;
                    }
                    let NodeKindData::Function(function) = ast.kind_data(entry.scope.node) else {
                        return false;
                    };
                    function
                        .id(&ast)
                        .is_some_and(|id| ast.get_utf8(id.name(&ast)).as_bytes() == name)
                })
                .expect("function scope")
                .scope
                .flags
                .contains_direct_eval()
        };

        assert!(contains_direct_eval(b"direct"));
        assert!(!contains_direct_eval(b"indirect"));
    }
}
