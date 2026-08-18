//! Scope, symbol, reference, module-record, and early-error analysis for the flat ECMAScript AST.
//!
//! This crate is a Rust port of the semantic-analysis design from
//! [yuku-toolchain/yuku](https://github.com/yuku-toolchain/yuku).

#![deny(clippy::all)]
#![deny(unused)]

use std::collections::HashSet;

use oxc_index::Idx;
use swc_next_allocator::{
    hash_map::{HashMap as ArenaHashMap, Values},
    vec::Vec as ArenaVec,
};
use swc_next_ecma_ast::{
    Ast, AstStr, AstUtf8, Class, Diagnostic, Function, MethodDefinition, MethodDefinitionKind,
    NodeId, Program, ScopeId, SymbolId, VisitWith,
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
        const GENERATED_NAMES = 1 << 5;
        const CLASS_LOWERING = 1 << 6;
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

/// The semantic identities created while lowering a namespace re-export into
/// an import namespace binding followed by a local export specifier.
#[derive(Debug, Clone, Copy)]
pub struct NamespaceReexportReplacement {
    pub old_statement: NodeId,
    pub statement: NodeId,
    pub specifier: NodeId,
    pub local: NodeId,
    pub exported: NodeId,
    pub symbol: SymbolId,
    pub reference: ReferenceId,
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
pub struct ClassLoweringFact<'a> {
    pub class: Class,
    pub class_scope: ScopeId,
    pub name_symbol: Option<SymbolId>,
    pub constructor: Option<Function>,
    pub methods: ArenaVec<'a, ClassMethodFact>,
    pub heritage: Option<NodeId>,
    pub computed_keys: ArenaVec<'a, NodeId>,
    pub super_calls: ArenaVec<'a, NodeId>,
    pub super_members: ArenaVec<'a, NodeId>,
    pub self_references: ArenaVec<'a, ReferenceId>,
    pub this_nodes: ArenaVec<'a, NodeId>,
    pub new_targets: ArenaVec<'a, NodeId>,
    pub has_properties: bool,
    pub has_private_elements: bool,
    pub has_static_blocks: bool,
    pub has_decorators: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ClassMethodFact {
    pub definition: MethodDefinition,
    pub function: Function,
    pub function_scope: ScopeId,
    pub kind: MethodDefinitionKind,
    pub r#static: bool,
    pub computed: bool,
}

#[doc(hidden)]
pub struct DirtyTransformFacts {
    old_ancestors: Vec<NodeId>,
    old_nodes: Vec<NodeId>,
    function_ranges: Vec<DirtyFunctionRange>,
    capture_functions: Vec<NodeId>,
}

#[derive(Clone, Copy)]
struct DirtyFunctionRange {
    root: NodeId,
    generator_start: usize,
    generator_count: usize,
    async_start: usize,
    async_count: usize,
    class_start: usize,
    class_count: usize,
}

#[derive(Debug)]
pub(crate) struct SemanticTransformFacts<'a> {
    pub(crate) effects: Option<ArenaVec<'a, EffectCounts>>,
    pub(crate) effect_owners: Option<ArenaVec<'a, Option<NodeId>>>,
    pub(crate) generator_functions: ArenaVec<'a, Function>,
    pub(crate) async_functions: ArenaVec<'a, NodeId>,
    pub(crate) generator_function_counts: ArenaVec<'a, u32>,
    pub(crate) async_function_counts: ArenaVec<'a, u32>,
    pub(crate) class_counts: ArenaVec<'a, u32>,
    pub(crate) function_captures: ArenaHashMap<'a, NodeId, FunctionCaptureData<'a>>,
    pub(crate) yield_nodes: ArenaHashMap<'a, NodeId, ArenaVec<'a, NodeId>>,
    pub(crate) class_lowering: ArenaVec<'a, ClassLoweringFact<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct UsedNameState {
    pub occupancy_count: u32,
    pub next_suffix: u64,
    pub reserved: bool,
}

impl Default for UsedNameState {
    fn default() -> Self {
        Self {
            occupancy_count: 0,
            next_suffix: 1,
            reserved: false,
        }
    }
}

type ScopeMap<'a> = ArenaHashMap<'a, AstStr, SymbolId>;
type UsedNameMap<'a> = ArenaHashMap<'a, AstStr, UsedNameState>;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum SemanticNodeState {
    #[default]
    Unregistered,
    Active,
    Inactive,
}

/// The complete semantic model of a tree, with every scope, symbol,
/// and reference fully resolved and cross-indexed. Backed by the
/// tree's arena and valid for the lifetime of the tree.
///
/// The query methods connect declaration nodes, reference nodes,
/// containing scopes, parents, uses, and name lookup without rebuilding
/// indexes outside the arena-backed model.
#[derive(Debug)]
pub struct Semantic<'a> {
    mutation_epoch: u64,
    /// Every scope, indexed by [`ScopeId`].
    scopes: ScopeTree<'a>,
    /// Every symbol, in declaration order, indexed by [`SymbolId`].
    symbols: ArenaVec<'a, Symbol>,
    symbol_active: ArenaVec<'a, bool>,
    /// Every reference, in source order, indexed by [`ReferenceId`].
    references: ArenaVec<'a, Reference>,
    reference_active: ArenaVec<'a, bool>,
    /// Every declaration, grouped by symbol and indexed by [`DeclarationId`].
    declarations: ArenaVec<'a, Declaration>,
    declaration_active: ArenaVec<'a, bool>,
    declaration_lists: ArenaVec<'a, ArenaVec<'a, Declaration>>,
    module_record: ModuleRecord<'a>,
    use_lists: ArenaVec<'a, ArenaVec<'a, ReferenceId>>,
    scope_maps: ArenaVec<'a, ScopeMap<'a>>,
    hoisting_variables: ArenaVec<'a, ScopeMap<'a>>,
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
    node_children: Vec<Vec<NodeId>>,
    node_states: ArenaVec<'a, SemanticNodeState>,
    node_declarations: ArenaVec<'a, Option<DeclarationId>>,
    node_references: ArenaVec<'a, Option<ReferenceId>>,
    pub(crate) features: SemanticFeatureFlags,
    pub(crate) transform_facts: Option<SemanticTransformFacts<'a>>,
    used_names: ArenaVec<'a, UsedNameMap<'a>>,
    direct_symbol_names: ArenaVec<'a, UsedNameMap<'a>>,
}

/// Exclusive write access to a [`Semantic`] graph during an atomic AST
/// mutation commit. Query APIs remain available through [`Deref`], while the
/// graph mutation primitives are deliberately absent from `Semantic`'s public
/// surface.
#[doc(hidden)]
pub struct SemanticMutation<'semantic, 'ast> {
    semantic: &'semantic mut Semantic<'ast>,
}

impl<'ast> std::ops::Deref for SemanticMutation<'_, 'ast> {
    type Target = Semantic<'ast>;

    fn deref(&self) -> &Self::Target {
        self.semantic
    }
}

impl<'a> Semantic<'a> {
    /// Opens the Semantic half of a transformer mutation commit.
    #[doc(hidden)]
    pub fn mutation(&mut self) -> SemanticMutation<'_, 'a> {
        SemanticMutation { semantic: self }
    }

    #[inline]
    fn tracks_names(&self) -> bool {
        self.features.intersects(
            SemanticFeatureFlags::FUNCTION_ENV_CAPTURE | SemanticFeatureFlags::GENERATED_NAMES,
        )
    }

    #[inline]
    pub fn mutation_epoch(&self) -> u64 {
        self.mutation_epoch
    }

    #[inline]
    fn advance_mutation_epoch(&mut self) -> u64 {
        self.mutation_epoch = self
            .mutation_epoch
            .checked_add(1)
            .expect("semantic mutation epoch overflow");
        self.mutation_epoch
    }

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

    fn prepare_dirty_transform_facts(
        &self,
        ast: &Ast<'_>,
        roots: &[NodeId],
    ) -> DirtyTransformFacts {
        let mut old_ancestors = Vec::new();
        let mut old_nodes = HashSet::new();
        let mut old_postorder = Vec::new();
        let mut function_ranges = Vec::new();
        let mut capture_functions = Vec::new();
        for root in roots {
            if !self.is_node_active(*root) {
                continue;
            }
            for ancestor in self.ancestors(self.parent_of(*root)) {
                if !old_ancestors.contains(&ancestor) {
                    old_ancestors.push(ancestor);
                }
            }
            self.collect_node_postorder(*root, &mut old_nodes, &mut old_postorder);
            if let Some(function) = self.nearest_capture_function(ast, *root)
                && !capture_functions.contains(&function)
            {
                capture_functions.push(function);
            }
            if let Some(facts) = &self.transform_facts {
                function_ranges.push(DirtyFunctionRange {
                    root: *root,
                    generator_start: self
                        .function_postorder_offset(*root, &facts.generator_function_counts),
                    generator_count: facts.generator_function_counts[root.index()] as usize,
                    async_start: self
                        .function_postorder_offset(*root, &facts.async_function_counts),
                    async_count: facts.async_function_counts[root.index()] as usize,
                    class_start: self.function_postorder_offset(*root, &facts.class_counts),
                    class_count: facts.class_counts[root.index()] as usize,
                });
            }
        }
        for node in &old_postorder {
            if is_generator_function(ast, *node) && !capture_functions.contains(node) {
                capture_functions.push(*node);
            }
        }
        DirtyTransformFacts {
            old_ancestors,
            old_nodes: old_postorder,
            function_ranges,
            capture_functions,
        }
    }

    fn refresh_dirty_transform_facts(
        &mut self,
        ast: &Ast<'a>,
        roots: &[NodeId],
        snapshot: DirtyTransformFacts,
    ) {
        if self.transform_facts.is_none() {
            return;
        }
        let refresh_effects = self
            .features
            .intersects(SemanticFeatureFlags::YIELD | SemanticFeatureFlags::AWAIT);

        let mut dirty_postorder = Vec::new();
        let mut dirty_nodes = HashSet::new();
        for root in roots {
            if self.is_node_active(*root) {
                self.collect_node_postorder(*root, &mut dirty_nodes, &mut dirty_postorder);
            }
        }

        let DirtyTransformFacts {
            old_ancestors,
            old_nodes,
            function_ranges,
            capture_functions,
        } = snapshot;
        let mut ancestors = old_ancestors;
        for root in roots {
            if !self.is_node_active(*root) {
                continue;
            }
            for ancestor in self.ancestors(self.parent_of(*root)) {
                if !ancestors.contains(&ancestor) {
                    ancestors.push(ancestor);
                }
            }
        }
        ancestors.retain(|node| self.is_node_active(*node) && !dirty_nodes.contains(node));
        ancestors.sort_by_key(|node| std::cmp::Reverse(self.ancestors(Some(*node)).count()));

        if refresh_effects {
            self.refresh_effect_owners(ast, roots);
        }
        for node in dirty_postorder.iter().chain(&ancestors).copied() {
            if refresh_effects {
                self.recompute_node_effects(ast, node);
            }
            self.recompute_node_function_counts(ast, node);
        }
        let removed_or_recomputed: HashSet<_> = old_nodes
            .into_iter()
            .chain(dirty_nodes.iter().copied())
            .collect();
        if refresh_effects {
            self.refresh_yield_index(ast, &dirty_nodes, &removed_or_recomputed);
        }
        self.refresh_function_lists(ast, function_ranges);
        self.refresh_function_captures(ast, roots, &dirty_nodes, capture_functions);
    }

    fn function_postorder_offset(&self, node: NodeId, counts: &[u32]) -> usize {
        let mut offset = 0usize;
        let mut current = node;
        while let Some(parent) = self.parent_of(current) {
            for sibling in self.node_children(parent) {
                if sibling == current {
                    break;
                }
                offset = offset
                    .checked_add(counts[sibling.index()] as usize)
                    .expect("function post-order offset overflow");
            }
            current = parent;
        }
        offset
    }

    fn collect_node_postorder(
        &self,
        root: NodeId,
        seen: &mut HashSet<NodeId>,
        postorder: &mut Vec<NodeId>,
    ) {
        if !seen.insert(root) {
            return;
        }
        for child in self.node_children(root) {
            self.collect_node_postorder(child, seen, postorder);
        }
        postorder.push(root);
    }

    fn refresh_effect_owners(&mut self, ast: &Ast<'_>, roots: &[NodeId]) {
        for root in roots {
            if !self.is_node_active(*root) {
                continue;
            }
            let inherited = self
                .ancestors(self.parent_of(*root))
                .find(|node| is_effect_boundary(ast, *node));
            let mut pending = vec![(*root, inherited)];
            while let Some((node, owner)) = pending.pop() {
                let owner = is_effect_boundary(ast, node).then_some(node).or(owner);
                if let Some(owners) = self
                    .transform_facts
                    .as_mut()
                    .and_then(|facts| facts.effect_owners.as_mut())
                {
                    owners[node.index()] = owner;
                }
                pending.extend(
                    self.node_children[node.index()]
                        .iter()
                        .copied()
                        .map(|child| (child, owner)),
                );
            }
        }
    }

    fn recompute_node_effects(&mut self, ast: &Ast<'_>, node: NodeId) {
        let mut counts = EffectCounts::default();
        match ast.node_kind(node) {
            swc_next_ecma_ast::NodeKind::YieldExpression
                if self.features.contains(SemanticFeatureFlags::YIELD) =>
            {
                counts.yields = 1;
            }
            swc_next_ecma_ast::NodeKind::AwaitExpression
                if self.features.contains(SemanticFeatureFlags::AWAIT) =>
            {
                counts.awaits = 1;
            }
            _ => {}
        }
        let children: Vec<_> = self.node_children[node.index()]
            .iter()
            .copied()
            .filter(|child| self.is_node_active(*child) && !is_effect_boundary(ast, *child))
            .collect();
        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        let effects = facts
            .effects
            .as_mut()
            .expect("enabled effects table must exist");
        for child in children {
            counts.yields = counts
                .yields
                .checked_add(effects[child.index()].yields)
                .expect("yield effect count overflow");
            counts.awaits = counts
                .awaits
                .checked_add(effects[child.index()].awaits)
                .expect("await effect count overflow");
        }
        effects[node.index()] = counts;
    }

    fn recompute_node_function_counts(&mut self, ast: &Ast<'_>, node: NodeId) {
        let children: Vec<_> = self.node_children[node.index()]
            .iter()
            .copied()
            .filter(|child| self.is_node_active(*child))
            .collect();
        let mut generator_count = 0u32;
        let mut async_count = 0u32;
        let mut class_count = 0u32;
        match ast.kind_data(node) {
            swc_next_ecma_ast::NodeKindData::Function(function) => {
                generator_count = u32::from(
                    self.features
                        .contains(SemanticFeatureFlags::GENERATOR_FUNCTIONS)
                        && function.generator(ast),
                );
                async_count = u32::from(
                    self.features
                        .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                        && function.r#async(ast),
                );
            }
            swc_next_ecma_ast::NodeKindData::ArrowFunctionExpression(arrow) => {
                async_count = u32::from(
                    self.features
                        .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                        && arrow.r#async(ast),
                );
            }
            _ => {}
        }
        if self.features.contains(SemanticFeatureFlags::CLASS_LOWERING)
            && matches!(
                ast.kind_data(node),
                swc_next_ecma_ast::NodeKindData::Class(_)
            )
        {
            class_count = 1;
        }
        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        for child in children {
            generator_count = generator_count
                .checked_add(facts.generator_function_counts[child.index()])
                .expect("generator function count overflow");
            async_count = async_count
                .checked_add(facts.async_function_counts[child.index()])
                .expect("async function count overflow");
            class_count = class_count
                .checked_add(facts.class_counts[child.index()])
                .expect("class count overflow");
        }
        facts.generator_function_counts[node.index()] = generator_count;
        facts.async_function_counts[node.index()] = async_count;
        facts.class_counts[node.index()] = class_count;
    }

    fn refresh_function_lists(&mut self, ast: &Ast<'a>, ranges: Vec<DirtyFunctionRange>) {
        struct Update<'a> {
            range: DirtyFunctionRange,
            generators: Vec<Function>,
            async_functions: Vec<NodeId>,
            classes: Vec<ClassLoweringFact<'a>>,
        }

        let mut updates = Vec::with_capacity(ranges.len());
        for range in ranges {
            let mut postorder = Vec::new();
            if self.is_node_active(range.root) {
                self.collect_node_postorder(range.root, &mut HashSet::new(), &mut postorder);
            }
            let generators = postorder
                .iter()
                .filter_map(|node| match ast.kind_data(*node) {
                    swc_next_ecma_ast::NodeKindData::Function(function)
                        if self
                            .features
                            .contains(SemanticFeatureFlags::GENERATOR_FUNCTIONS)
                            && function.generator(ast) =>
                    {
                        Some(function)
                    }
                    _ => None,
                })
                .collect();
            let async_functions = postorder
                .iter()
                .copied()
                .filter(|node| match ast.kind_data(*node) {
                    swc_next_ecma_ast::NodeKindData::Function(function) => {
                        self.features
                            .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                            && function.r#async(ast)
                    }
                    swc_next_ecma_ast::NodeKindData::ArrowFunctionExpression(arrow) => {
                        self.features
                            .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                            && arrow.r#async(ast)
                    }
                    _ => false,
                })
                .collect();
            let classes = postorder
                .iter()
                .filter_map(|node| match ast.kind_data(*node) {
                    swc_next_ecma_ast::NodeKindData::Class(class)
                        if self.features.contains(SemanticFeatureFlags::CLASS_LOWERING) =>
                    {
                        Some(context::build_class_lowering_fact(self, ast, class))
                    }
                    _ => None,
                })
                .collect();
            updates.push(Update {
                range,
                generators,
                async_functions,
                classes,
            });
        }

        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        updates.sort_by_key(|update| std::cmp::Reverse(update.range.generator_start));
        for update in &updates {
            for _ in 0..update.range.generator_count {
                facts
                    .generator_functions
                    .remove(update.range.generator_start);
            }
            for (offset, function) in update.generators.iter().copied().enumerate() {
                facts
                    .generator_functions
                    .insert(update.range.generator_start + offset, function);
            }
        }
        updates.sort_by_key(|update| std::cmp::Reverse(update.range.async_start));
        for update in &updates {
            for _ in 0..update.range.async_count {
                facts.async_functions.remove(update.range.async_start);
            }
            for (offset, function) in update.async_functions.iter().copied().enumerate() {
                facts
                    .async_functions
                    .insert(update.range.async_start + offset, function);
            }
        }
        updates.sort_by_key(|update| std::cmp::Reverse(update.range.class_start));
        for update in updates {
            for _ in 0..update.range.class_count {
                facts.class_lowering.remove(update.range.class_start);
            }
            for (offset, class) in update.classes.into_iter().enumerate() {
                facts
                    .class_lowering
                    .insert(update.range.class_start + offset, class);
            }
        }
    }

    fn nearest_capture_function(&self, ast: &Ast<'_>, node: NodeId) -> Option<NodeId> {
        for ancestor in self.ancestors(Some(node)) {
            match ast.kind_data(ancestor) {
                swc_next_ecma_ast::NodeKindData::Class(_) => return None,
                swc_next_ecma_ast::NodeKindData::Function(function) => {
                    return function.generator(ast).then_some(ancestor);
                }
                _ => {}
            }
        }
        None
    }

    fn refresh_function_captures(
        &mut self,
        ast: &Ast<'a>,
        roots: &[NodeId],
        dirty_nodes: &HashSet<NodeId>,
        mut affected: Vec<NodeId>,
    ) {
        if !self
            .features
            .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE)
        {
            return;
        }
        for root in roots {
            if self.is_node_active(*root)
                && let Some(function) = self.nearest_capture_function(ast, *root)
                && !affected.contains(&function)
            {
                affected.push(function);
            }
        }
        for node in dirty_nodes {
            if is_generator_function(ast, *node) && !affected.contains(node) {
                affected.push(*node);
            }
        }

        let mut replacements = Vec::new();
        for function in &affected {
            if !self.is_node_active(*function) || !is_generator_function(ast, *function) {
                continue;
            }
            let mut arguments = Vec::new();
            let mut new_targets = Vec::new();
            let mut pending: Vec<_> = self.node_children(*function).collect();
            while let Some(node) = pending.pop() {
                match ast.kind_data(node) {
                    swc_next_ecma_ast::NodeKindData::Function(_)
                    | swc_next_ecma_ast::NodeKindData::Class(_) => continue,
                    swc_next_ecma_ast::NodeKindData::IdentifierReference(identifier)
                        if ast.get_utf8(identifier.name(ast)) == "arguments" =>
                    {
                        if let Some(reference) = self.reference_of(node) {
                            let reference_data = self.reference(reference);
                            if reference_data.flags.space == ReferenceSpace::Value
                                && reference_data.symbol.is_none()
                            {
                                arguments.push(reference);
                            }
                        }
                    }
                    swc_next_ecma_ast::NodeKindData::MetaProperty(meta)
                        if ast.get_utf8(meta.meta(ast).name(ast)) == "new"
                            && ast.get_utf8(meta.property(ast).name(ast)) == "target" =>
                    {
                        new_targets.push(node);
                    }
                    _ => {}
                }
                pending.extend(self.node_children(node));
            }
            replacements.push((*function, arguments, new_targets));
        }

        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        for function in affected {
            facts.function_captures.remove(&function);
        }
        for (function, arguments, new_targets) in replacements {
            if arguments.is_empty() && new_targets.is_empty() {
                continue;
            }
            let mut captures = FunctionCaptureData {
                arguments: ArenaVec::with_capacity_in(arguments.len(), ast.allocator()),
                new_targets: ArenaVec::with_capacity_in(new_targets.len(), ast.allocator()),
            };
            captures.arguments.extend(arguments);
            captures.new_targets.extend(new_targets);
            facts.function_captures.insert(function, captures);
        }
    }

    fn refresh_yield_index(
        &mut self,
        ast: &Ast<'a>,
        dirty_nodes: &HashSet<NodeId>,
        removed_or_recomputed: &HashSet<NodeId>,
    ) {
        if !self.features.contains(SemanticFeatureFlags::YIELD) {
            return;
        }
        let facts = self
            .transform_facts
            .as_mut()
            .expect("enabled transform facts must exist");
        for yields in facts.yield_nodes.values_mut() {
            yields.retain(|node| !removed_or_recomputed.contains(node));
        }
        let owners = facts
            .effect_owners
            .as_ref()
            .expect("enabled effect owners must exist");
        for node in dirty_nodes {
            if ast.node_kind(*node) != swc_next_ecma_ast::NodeKind::YieldExpression {
                continue;
            }
            if let Some(owner) = owners[node.index()] {
                facts
                    .yield_nodes
                    .entry(owner)
                    .or_insert_with(|| ArenaVec::new_in(ast.allocator()))
                    .push(*node);
            }
        }
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

    pub fn classes_post_order(&self) -> Result<&[ClassLoweringFact<'a>], FeatureNotEnabled> {
        if !self.features.contains(SemanticFeatureFlags::CLASS_LOWERING) {
            return Err(FeatureNotEnabled {
                feature: SemanticFeatureFlags::CLASS_LOWERING,
            });
        }
        Ok(&self
            .transform_facts
            .as_ref()
            .expect("enabled transform facts must exist")
            .class_lowering)
    }

    /// The symbol with the given id.
    pub fn symbol(&self, id: SymbolId) -> Symbol {
        assert!(id.index() < self.symbols.len());
        assert!(self.symbol_active[id.index()], "symbol is inactive");
        self.symbols[id.index()]
    }

    /// The reference with the given id.
    pub fn reference(&self, id: ReferenceId) -> Reference {
        assert!(id.index() < self.references.len());
        assert!(self.reference_active[id.index()], "reference is inactive");
        self.references[id.index()]
    }

    /// The declaration with the given id.
    pub fn declaration(&self, id: DeclarationId) -> Declaration {
        assert!(id.index() < self.declarations.len());
        assert!(
            self.declaration_active[id.index()],
            "declaration is inactive"
        );
        self.declarations[id.index()]
    }

    /// The source text module's export occurrences and resolved targets.
    pub fn module_record(&self) -> &ModuleRecord<'a> {
        &self.module_record
    }

    /// Retargets an anonymous default declaration after a transform gives it
    /// a generated local binding.
    ///
    /// Structural transforms should call this only as part of the same
    /// transaction that creates `symbol` and `declaration`.
    fn retarget_default_export_local(
        &mut self,
        statement: NodeId,
        symbol: SymbolId,
        declaration: DeclarationId,
    ) {
        assert!(self.is_symbol_active(symbol));
        assert!(self.is_declaration_active(declaration));
        assert_eq!(self.declaration(declaration).symbol, symbol);
        assert!(
            self.module_record
                .retarget_default_local(statement, symbol, declaration),
            "default export entry must exist"
        );
    }

    /// Retargets every module-record occurrence of an AST node whose
    /// semantic identity was moved to `new` by a structural transaction.
    fn retarget_module_node(&mut self, old: NodeId, new: NodeId) {
        assert!(self.is_node_active(new));
        self.module_record.retarget_node(old, new);
    }

    /// Replaces an `export * as name from source` record after lowering it to
    /// an import namespace binding followed by a local export specifier.
    fn replace_namespace_reexport(
        &mut self,
        ast: &Ast<'_>,
        replacement: NamespaceReexportReplacement,
    ) {
        assert!(self.is_symbol_active(replacement.symbol));
        assert!(self.is_reference_active(replacement.reference));
        assert_eq!(
            self.reference(replacement.reference).symbol,
            Some(replacement.symbol)
        );
        assert!(
            self.module_record
                .replace_namespace_reexport(ast, replacement,),
            "namespace re-export entry must exist"
        );
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
        self.assert_node_active(node);
        self.node_declarations[node.index()]
            .map(|id| self.declaration(id).symbol)
            .or_else(|| self.node_references[node.index()].and_then(|id| self.reference(id).symbol))
    }

    /// The declaration recorded at `node`, or `None` when the node is not a
    /// declaration site.
    pub fn declaration_of(&self, node: NodeId) -> Option<DeclarationId> {
        self.assert_node_active(node);
        self.node_declarations[node.index()]
    }

    /// The reference recorded at `node`, or `None` when the node is
    /// not a reference site.
    pub fn reference_of(&self, node: NodeId) -> Option<ReferenceId> {
        self.assert_node_active(node);
        self.node_references[node.index()]
    }

    /// The innermost lexical scope containing `node`. A scope-creating
    /// node (function, block, class, ...) maps to the scope it creates;
    /// that scope's parent is the enclosing scope.
    pub fn scope_of(&self, node: NodeId) -> ScopeId {
        self.assert_node_active(node);
        self.node_scopes[node.index()]
    }

    /// The structural parent of `node`, or `None` at the root.
    pub fn parent_of(&self, node: NodeId) -> Option<NodeId> {
        self.assert_node_active(node);
        self.node_parents[node.index()]
    }

    /// Walks from `node` up to the root, yielding `node` first.
    pub fn ancestors(&self, node: Option<NodeId>) -> SemanticAncestorIterator<'_> {
        assert!(node.is_none_or(|node| self.is_node_active(node)));
        SemanticAncestorIterator {
            node_parents: &self.node_parents,
            node_states: &self.node_states,
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
        assert!(self.symbol_active[id.index()], "symbol is inactive");
        &self.declaration_lists[id.index()]
    }

    /// Every use site of `id`, in source order. Declaration sites are
    /// not uses; those are returned by [`Semantic::declarations`]. A dynamic
    /// `with` reference is included under its potential lexical fallback.
    pub fn uses(&self, id: SymbolId) -> &[ReferenceId] {
        assert!(id.index() < self.use_lists.len());
        assert!(self.symbol_active[id.index()], "symbol is inactive");
        &self.use_lists[id.index()]
    }

    #[inline]
    pub fn is_scope_active(&self, id: ScopeId) -> bool {
        self.scopes.is_active(id)
    }

    pub fn scope_children(&self, id: ScopeId) -> impl Iterator<Item = ScopeId> + '_ {
        self.scopes
            .children(id)
            .iter()
            .copied()
            .filter(|child| self.is_scope_active(*child))
    }

    pub fn scope_is_ancestor(&self, ancestor: ScopeId, descendant: ScopeId) -> bool {
        assert!(self.is_scope_active(ancestor));
        assert!(self.is_scope_active(descendant));
        let mut current = Some(descendant);
        while let Some(scope) = current {
            if scope == ancestor {
                return true;
            }
            current = self.scope(scope).parent;
        }
        false
    }

    pub fn scope_indexed_symbols(&self, id: ScopeId) -> impl Iterator<Item = SymbolId> + '_ {
        assert!(self.is_scope_active(id));
        self.scope_maps[id.index()]
            .values()
            .chain(self.hoisting_variables[id.index()].values())
            .copied()
    }

    pub fn nodes_in_scope(&self, id: ScopeId) -> impl Iterator<Item = NodeId> + '_ {
        assert!(self.is_scope_active(id));
        self.node_scopes
            .iter()
            .enumerate()
            .filter(move |(index, scope)| {
                **scope == id && self.node_states[*index] == SemanticNodeState::Active
            })
            .map(|(index, _)| NodeId::from_usize(index))
    }

    pub fn node_children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.assert_node_active(node);
        self.node_children[node.index()]
            .iter()
            .copied()
            .filter(|child| self.is_node_active(*child))
    }

    #[inline]
    pub fn is_symbol_active(&self, id: SymbolId) -> bool {
        self.symbol_active.get(id.index()).copied().unwrap_or(false)
    }

    #[inline]
    pub fn is_reference_active(&self, id: ReferenceId) -> bool {
        self.reference_active
            .get(id.index())
            .copied()
            .unwrap_or(false)
    }

    #[inline]
    pub fn is_declaration_active(&self, id: DeclarationId) -> bool {
        self.declaration_active
            .get(id.index())
            .copied()
            .unwrap_or(false)
    }

    fn create_scope(
        &mut self,
        ast: &Ast<'a>,
        parent: ScopeId,
        node: NodeId,
        kind: scope::ScopeKind,
        flags: scope::ScopeFlags,
    ) -> ScopeId {
        assert!(
            node.index() < ast.node_count(),
            "scope owner node is invalid"
        );
        assert_ne!(
            self.node_state(node),
            SemanticNodeState::Inactive,
            "scope owner node is inactive"
        );
        let id = self
            .scopes
            .create(ast.allocator(), parent, node, kind, flags);
        self.scope_maps.push(ScopeMap::new_in(ast.allocator()));
        self.hoisting_variables
            .push(ScopeMap::new_in(ast.allocator()));
        if self.tracks_names() {
            self.used_names.push(UsedNameMap::new_in(ast.allocator()));
            self.direct_symbol_names
                .push(UsedNameMap::new_in(ast.allocator()));
        }
        id
    }

    fn retag_scope(&mut self, id: ScopeId, kind: scope::ScopeKind, flags: scope::ScopeFlags) {
        self.scopes.retag(id, kind, flags);
    }

    fn reowner_scope(&mut self, id: ScopeId, node: NodeId) {
        self.assert_node_active(node);
        self.scopes.reowner(id, node);
    }

    fn reparent_scope(&mut self, id: ScopeId, parent: ScopeId) {
        let old_parent = self
            .scope(id)
            .parent
            .expect("non-root active scope has a parent");
        if old_parent == parent {
            return;
        }
        self.move_scope_name_aggregate(id, old_parent, parent);
        self.scopes.reparent(id, parent);
    }

    fn retire_scope(&mut self, id: ScopeId) {
        assert!(self.scope_maps[id.index()].is_empty());
        assert!(self.hoisting_variables[id.index()].is_empty());
        if let Some(names) = self.used_names.get_mut(id.index()) {
            names.clear();
        }
        if let Some(names) = self.direct_symbol_names.get_mut(id.index()) {
            names.clear();
        }
        self.scopes.retire(id);
    }

    /// Registers a newly created ordinary AST node in an existing scope.
    fn register_node(&mut self, node: NodeId, parent: Option<NodeId>, scope: ScopeId) {
        assert!(self.scopes.is_active(scope), "scope is inactive");
        assert!(parent.is_none_or(|parent| self.is_node_active(parent)));
        let required = node.index() + 1;
        if self.node_scopes.len() < required {
            self.node_scopes.resize(required, ScopeId::ROOT);
            self.node_parents.resize(required, None);
            self.node_children.resize_with(required, Vec::new);
            self.node_states
                .resize(required, SemanticNodeState::Unregistered);
            self.node_declarations.resize(required, None);
            self.node_references.resize(required, None);
            if let Some(facts) = &mut self.transform_facts {
                facts.generator_function_counts.resize(required, 0);
                facts.async_function_counts.resize(required, 0);
                facts.class_counts.resize(required, 0);
                if let Some(effects) = &mut facts.effects {
                    effects.resize(required, EffectCounts::default());
                }
                if let Some(owners) = &mut facts.effect_owners {
                    owners.resize(required, None);
                }
            }
        }
        assert_eq!(
            self.node_states[node.index()],
            SemanticNodeState::Unregistered,
            "node registered twice or restored without an identity policy"
        );
        self.node_states[node.index()] = SemanticNodeState::Active;
        self.node_scopes[node.index()] = scope;
        self.node_parents[node.index()] = parent;
        if let Some(parent) = parent {
            self.node_children[parent.index()].push(node);
        }
        if let Some(owners) = self
            .transform_facts
            .as_mut()
            .and_then(|facts| facts.effect_owners.as_mut())
        {
            owners[node.index()] = parent.and_then(|parent| owners[parent.index()]);
        }
    }

    fn retire_node(&mut self, node: NodeId) {
        self.assert_node_active(node);
        assert!(
            self.node_children[node.index()]
                .iter()
                .all(|child| !self.is_node_active(*child)),
            "retire child nodes before retiring their parent"
        );
        assert!(
            self.node_declarations[node.index()].is_none(),
            "retire a declaration before retiring its node"
        );
        assert!(
            self.node_references[node.index()].is_none(),
            "retire a reference before retiring its node"
        );
        self.node_states[node.index()] = SemanticNodeState::Inactive;
        if let Some(parent) = self.node_parents[node.index()] {
            let index = self.node_children[parent.index()]
                .iter()
                .position(|child| *child == node)
                .expect("active parent contains child node");
            self.node_children[parent.index()].remove(index);
        }
        self.node_parents[node.index()] = None;
        if let Some(facts) = &mut self.transform_facts {
            facts.generator_function_counts[node.index()] = 0;
            facts.async_function_counts[node.index()] = 0;
            facts.class_counts[node.index()] = 0;
            if let Some(effects) = &mut facts.effects {
                effects[node.index()] = EffectCounts::default();
            }
            if let Some(owners) = &mut facts.effect_owners {
                owners[node.index()] = None;
            }
        }
    }

    fn rescope_node(&mut self, node: NodeId, scope: ScopeId) {
        self.assert_node_active(node);
        assert!(self.scopes.is_active(scope), "scope is inactive");
        self.node_scopes[node.index()] = scope;
    }

    fn reparent_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        self.assert_node_active(node);
        assert!(parent.is_none_or(|parent| self.is_node_active(parent)));
        assert!(
            parent.is_none_or(|parent| !self.ancestors(Some(parent)).any(|item| item == node)),
            "node reparenting would create a cycle"
        );
        let old_parent = self.node_parents[node.index()];
        if old_parent == parent {
            return;
        }
        if let Some(old_parent) = old_parent {
            let index = self.node_children[old_parent.index()]
                .iter()
                .position(|child| *child == node)
                .expect("old parent contains child node");
            self.node_children[old_parent.index()].remove(index);
        }
        if let Some(parent) = parent {
            self.node_children[parent.index()].push(node);
        }
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
    fn declare(
        &mut self,
        ast: &Ast<'a>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: SymbolFlags,
    ) -> SymbolId {
        self.assert_node_active(node);
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
        self.symbol_active.push(true);
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
        self.declaration_active.push(true);
        self.declaration_lists[symbol.index()].push(declaration);
        self.node_declarations[node.index()] = Some(declaration_id);
        self.observe_symbol_name(ast, scope, name);
        symbol
    }

    /// Appends a generated reference and updates its target symbol use list.
    fn reference_generated(
        &mut self,
        ast: &Ast<'_>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: ReferenceFlags,
        symbol: Option<SymbolId>,
    ) -> ReferenceId {
        self.assert_node_active(node);
        assert!(self.node_references[node.index()].is_none());
        assert!(symbol.is_none_or(|symbol| self.is_symbol_active(symbol)));
        let id = ReferenceId::from_usize(self.references.len());
        self.references.push(Reference {
            name,
            scope,
            node,
            symbol,
            flags,
        });
        self.reference_active.push(true);
        self.node_references[node.index()] = Some(id);
        if let Some(symbol) = symbol {
            self.use_lists[symbol.index()].push(id);
        }
        self.observe_reference_name(ast, scope, name);
        id
    }

    fn retire_reference(&mut self, id: ReferenceId) {
        let reference = self.reference(id);
        self.unobserve_reference_name(reference.scope, reference.name);
        if let Some(symbol) = reference.symbol {
            let uses = &mut self.use_lists[symbol.index()];
            let index = uses
                .iter()
                .position(|candidate| *candidate == id)
                .expect("symbol use index contains active reference");
            uses.remove(index);
        }
        if self.node_references[reference.node.index()] == Some(id) {
            self.node_references[reference.node.index()] = None;
        }
        self.reference_active[id.index()] = false;
    }

    fn rescope_reference(&mut self, ast: &Ast<'_>, id: ReferenceId, scope: ScopeId) {
        assert!(self.is_reference_active(id), "reference is inactive");
        assert!(self.is_scope_active(scope), "scope is inactive");
        let reference = self.references[id.index()];
        self.unobserve_reference_name(reference.scope, reference.name);
        self.references[id.index()].scope = scope;
        self.observe_reference_name(ast, scope, reference.name);
    }

    /// Moves an existing reference identity to a replacement AST node.
    ///
    /// The reference ID, resolution, flags, scope, use-list position, and
    /// generated-name accounting remain stable. Both nodes must already have
    /// explicit lifecycle states; callers normally register `node` in the
    /// same mutation transaction before invoking this operation.
    fn retarget_reference_node(&mut self, id: ReferenceId, node: NodeId) {
        assert!(self.is_reference_active(id), "reference is inactive");
        self.assert_node_active(node);
        let old = self.references[id.index()].node;
        if old == node {
            return;
        }
        assert!(
            self.node_references[node.index()].is_none(),
            "target node already has a reference identity"
        );
        assert_eq!(self.node_references[old.index()], Some(id));
        self.node_references[old.index()] = None;
        self.node_references[node.index()] = Some(id);
        self.references[id.index()].node = node;
    }

    fn retire_declaration(&mut self, id: DeclarationId) {
        let declaration = self.declaration(id);
        if self.node_declarations[declaration.node.index()] == Some(id) {
            self.node_declarations[declaration.node.index()] = None;
        }
        self.declaration_active[id.index()] = false;
        let declarations = &mut self.declaration_lists[declaration.symbol.index()];
        let index = declarations
            .iter()
            .position(|candidate| {
                candidate.node == declaration.node && candidate.symbol == declaration.symbol
            })
            .expect("symbol declaration list contains active declaration");
        declarations.remove(index);
        let flags = declarations
            .iter()
            .fold(SymbolFlags::empty(), |flags, declaration| {
                flags | declaration.flags
            });
        let symbol = &mut self.symbols[declaration.symbol.index()];
        symbol.flags = flags;
        symbol.declarations.len =
            u32::try_from(declarations.len()).expect("declaration count overflow");
    }

    /// Moves an existing declaration identity to a replacement AST node while
    /// preserving its declaration and symbol IDs.
    fn retarget_declaration_node(&mut self, id: DeclarationId, node: NodeId) {
        assert!(self.is_declaration_active(id), "declaration is inactive");
        self.assert_node_active(node);
        let declaration = self.declarations[id.index()];
        let old = declaration.node;
        if old == node {
            return;
        }
        assert!(
            self.node_declarations[node.index()].is_none(),
            "target node already has a declaration identity"
        );
        assert_eq!(self.node_declarations[old.index()], Some(id));
        self.node_declarations[old.index()] = None;
        self.node_declarations[node.index()] = Some(id);
        self.declarations[id.index()].node = node;
        self.declaration_lists[declaration.symbol.index()]
            .iter_mut()
            .find(|candidate| candidate.node == old)
            .expect("active declaration is indexed by its symbol")
            .node = node;
    }

    fn update_symbol_flags(&mut self, id: SymbolId, flags: SymbolFlags) {
        assert!(self.is_symbol_active(id), "symbol is inactive");
        self.symbols[id.index()].flags = flags;
    }

    fn update_declaration_flags(&mut self, id: DeclarationId, flags: SymbolFlags) {
        assert!(self.is_declaration_active(id), "declaration is inactive");
        let declaration = &mut self.declarations[id.index()];
        declaration.flags = flags;
        let declarations = &mut self.declaration_lists[declaration.symbol.index()];
        declarations
            .iter_mut()
            .find(|candidate| candidate.node == declaration.node)
            .expect("active declaration is indexed by its symbol")
            .flags = flags;
    }

    fn move_symbol(
        &mut self,
        ast: &Ast<'_>,
        id: SymbolId,
        scope: ScopeId,
        hoisting_scopes: &[ScopeId],
    ) {
        let symbol = self.symbol(id);
        assert!(self.is_scope_active(scope), "scope is inactive");
        assert!(
            self.binding(scope, symbol.name)
                .is_none_or(|existing| existing == id),
            "target scope already contains the binding"
        );
        if symbol.scope == scope {
            return;
        }
        self.unobserve_symbol_name(symbol.scope, symbol.name);
        if self.scope_maps[symbol.scope.index()].get(&symbol.name) == Some(&id) {
            self.scope_maps[symbol.scope.index()].remove(&symbol.name);
        }
        for map in &mut self.hoisting_variables {
            let names: Vec<_> = map
                .iter()
                .filter_map(|(name, symbol)| (*symbol == id).then_some(*name))
                .collect();
            for name in names {
                map.remove(&name);
            }
        }
        self.scope_maps[scope.index()].insert(symbol.name, id);
        for &passed_scope in hoisting_scopes {
            assert!(self.is_scope_active(passed_scope), "scope is inactive");
            self.hoisting_variables[passed_scope.index()].insert(symbol.name, id);
        }
        self.symbols[id.index()].scope = scope;
        self.observe_symbol_name(ast, scope, symbol.name);
    }

    fn retire_symbol(&mut self, id: SymbolId) {
        let symbol = self.symbol(id);
        assert!(
            self.declaration_lists[id.index()].is_empty(),
            "retire all declarations before retiring a symbol"
        );
        assert!(
            self.use_lists[id.index()].is_empty(),
            "rebind or retire all references before retiring a symbol"
        );
        if self.scope_maps[symbol.scope.index()].get(&symbol.name) == Some(&id) {
            self.scope_maps[symbol.scope.index()].remove(&symbol.name);
        }
        for map in &mut self.hoisting_variables {
            let names: Vec<_> = map
                .iter()
                .filter_map(|(name, symbol)| (*symbol == id).then_some(*name))
                .collect();
            for name in names {
                map.remove(&name);
            }
        }
        self.unobserve_symbol_name(symbol.scope, symbol.name);
        self.symbol_active[id.index()] = false;
    }

    /// Rebinds a reference's lexical fallback while preserving its stable ID.
    /// Dynamic references may still be intercepted by a `with` object; this
    /// only changes the binding used when that interception does not occur.
    fn rebind_reference(&mut self, id: ReferenceId, symbol: Option<SymbolId>) {
        assert!(id.index() < self.references.len());
        assert!(self.reference_active[id.index()], "reference is inactive");
        assert!(symbol.is_none_or(|symbol| {
            symbol.index() < self.symbols.len() && self.symbol_active[symbol.index()]
        }));
        let old = self.references[id.index()].symbol;
        if old == symbol {
            return;
        }
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
    fn rename_reference(&mut self, ast: &Ast<'_>, id: ReferenceId, name: AstStr) {
        assert!(id.index() < self.references.len());
        assert!(self.reference_active[id.index()], "reference is inactive");
        let reference = self.references[id.index()];
        if reference.name == name {
            return;
        }
        self.unobserve_reference_name(reference.scope, reference.name);
        self.references[id.index()].name = name;
        self.observe_reference_name(ast, reference.scope, name);
    }

    /// Updates the access mode/namespace of a reference whose AST occurrence
    /// survives a structural rewrite.
    fn update_reference_flags(&mut self, id: ReferenceId, flags: ReferenceFlags) {
        assert!(id.index() < self.references.len());
        assert!(self.reference_active[id.index()], "reference is inactive");
        self.references[id.index()].flags = flags;
    }

    /// Renames a symbol and all of its references while preserving IDs.
    fn rename_symbol(&mut self, ast: &Ast<'_>, id: SymbolId, name: AstStr) {
        assert!(id.index() < self.symbols.len());
        assert!(self.symbol_active[id.index()], "symbol is inactive");
        let symbol = self.symbols[id.index()];
        assert!(
            self.binding(symbol.scope, name)
                .is_none_or(|existing| existing == id)
        );
        if symbol.name == name {
            return;
        }
        self.unobserve_symbol_name(symbol.scope, symbol.name);
        let uses: Vec<_> = self.use_lists[id.index()].iter().copied().collect();
        for reference in &uses {
            let reference = self.references[reference.index()];
            self.unobserve_reference_name(reference.scope, reference.name);
        }
        self.scope_maps[symbol.scope.index()].remove(&symbol.name);
        self.scope_maps[symbol.scope.index()].insert(name, id);
        self.symbols[id.index()].name = name;
        for reference in uses {
            self.references[reference.index()].name = name;
            let scope = self.references[reference.index()].scope;
            self.observe_reference_name(ast, scope, name);
        }
        self.observe_symbol_name(ast, symbol.scope, name);
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

    fn observe_symbol_name(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        if !self.tracks_names() {
            return;
        }
        binder::observe_used_name(ast, &mut self.direct_symbol_names[scope.index()], name);
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::observe_used_name(ast, &mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
    }

    fn unobserve_symbol_name(&mut self, scope: ScopeId, name: AstStr) {
        if !self.tracks_names() {
            return;
        }
        binder::unobserve_used_name(&mut self.direct_symbol_names[scope.index()], name);
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::unobserve_used_name(&mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
    }

    fn observe_reference_name(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        if !self.tracks_names() {
            return;
        }
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::observe_used_name(ast, &mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
    }

    fn unobserve_reference_name(&mut self, scope: ScopeId, name: AstStr) {
        if !self.tracks_names() {
            return;
        }
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::unobserve_used_name(&mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
    }

    fn move_scope_name_aggregate(
        &mut self,
        scope: ScopeId,
        old_parent: ScopeId,
        new_parent: ScopeId,
    ) {
        if !self.tracks_names() {
            return;
        }
        let old_ancestors: Vec<_> = self.scopes.ancestors(Some(old_parent)).collect();
        let new_ancestors: Vec<_> = self.scopes.ancestors(Some(new_parent)).collect();
        let common = old_ancestors
            .iter()
            .copied()
            .find(|old| new_ancestors.contains(old));
        let old_exclusive = old_ancestors
            .into_iter()
            .take_while(|candidate| Some(*candidate) != common)
            .collect::<Vec<_>>();
        let new_exclusive = new_ancestors
            .into_iter()
            .take_while(|candidate| Some(*candidate) != common)
            .collect::<Vec<_>>();
        let aggregate: Vec<_> = self.used_names[scope.index()]
            .iter()
            .map(|(name, state)| (*name, *state))
            .collect();

        for ancestor in old_exclusive {
            for (name, state) in &aggregate {
                let target = self.used_names[ancestor.index()]
                    .get_mut(name)
                    .expect("old ancestor contains moved name aggregate");
                target.occupancy_count = target
                    .occupancy_count
                    .checked_sub(state.occupancy_count)
                    .expect("name occupancy count underflow");
            }
        }
        for ancestor in new_exclusive {
            for (name, state) in &aggregate {
                let target = self.used_names[ancestor.index()].entry(*name).or_default();
                target.occupancy_count = target
                    .occupancy_count
                    .checked_add(state.occupancy_count)
                    .expect("name occupancy count overflow");
                target.next_suffix = target.next_suffix.max(state.next_suffix);
                target.reserved |= state.reserved;
            }
        }
    }

    pub fn name_allocation_state(&self, scope: ScopeId, base: AstStr) -> UsedNameState {
        assert!(
            self.features
                .contains(SemanticFeatureFlags::GENERATED_NAMES),
            "generated-name allocation requires GENERATED_NAMES"
        );
        let mut state = self.used_names[scope.index()]
            .get(&base)
            .copied()
            .unwrap_or_default();
        let parent = self.scope(scope).parent;
        for ancestor in self.scopes.ancestors(parent) {
            if let Some(ancestor_state) = self.direct_symbol_names[ancestor.index()].get(&base) {
                state.occupancy_count = state
                    .occupancy_count
                    .checked_add(ancestor_state.occupancy_count)
                    .expect("name occupancy count overflow");
                state.next_suffix = state.next_suffix.max(ancestor_state.next_suffix);
                state.reserved |= ancestor_state.reserved;
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
        let text = if state.occupancy_count != 0 || state.reserved {
            let suffix = state.next_suffix.max(min_suffix);
            let mut candidate = base.to_owned();
            candidate.push_str(&suffix.to_string());
            ast.add_utf8(&candidate)
        } else {
            base_text
        };
        let name = ast.canonicalize_utf8(text);

        self.reserve_name(ast, scope, name);
        text
    }

    #[doc(hidden)]
    fn reserve_generated_name_exact(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        assert!(
            self.features
                .contains(SemanticFeatureFlags::GENERATED_NAMES),
            "generated-name allocation requires GENERATED_NAMES"
        );
        self.reserve_name(ast, scope, name);
    }

    fn reserve_name(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        assert!(self.tracks_names());
        binder::reserve_used_name(ast, &mut self.direct_symbol_names[scope.index()], name);
        let mut current = Some(scope);
        while let Some(scope) = current {
            binder::reserve_used_name(ast, &mut self.used_names[scope.index()], name);
            current = self.scopes.get(scope).parent;
        }
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
            active: self.scopes.active(),
            index: 0,
        }
    }

    pub fn iter_nodes(&self) -> SemanticNodeIterator<'_> {
        SemanticNodeIterator {
            states: &self.node_states,
            index: 0,
        }
    }

    /// Iterates every `(id, symbol)` pair in declaration order.
    pub fn iter_symbols(&self) -> SymbolIterator<'_> {
        SymbolIterator {
            symbols: &self.symbols,
            active: &self.symbol_active,
            index: 0,
        }
    }

    /// Iterates every `(id, reference)` pair in source order.
    pub fn iter_references(&self) -> ReferenceIterator<'_> {
        ReferenceIterator {
            references: &self.references,
            active: &self.reference_active,
            index: 0,
        }
    }

    /// Iterates every `(id, declaration)` pair, grouped by symbol and in
    /// source order within each symbol.
    pub fn iter_declarations(&self) -> DeclarationIterator<'_> {
        DeclarationIterator {
            declarations: &self.declarations,
            active: &self.declaration_active,
            index: 0,
        }
    }

    #[inline]
    pub fn is_node_active(&self, node: NodeId) -> bool {
        self.node_states
            .get(node.index())
            .is_some_and(|state| *state == SemanticNodeState::Active)
    }

    #[inline]
    pub fn node_state(&self, node: NodeId) -> SemanticNodeState {
        self.node_states
            .get(node.index())
            .copied()
            .unwrap_or(SemanticNodeState::Unregistered)
    }

    #[inline]
    fn assert_node_active(&self, node: NodeId) {
        assert!(self.is_node_active(node), "node is not active");
    }
}

impl<'ast> SemanticMutation<'_, 'ast> {
    pub fn prepare_dirty_transform_facts(
        &self,
        ast: &Ast<'_>,
        roots: &[NodeId],
    ) -> DirtyTransformFacts {
        self.semantic.prepare_dirty_transform_facts(ast, roots)
    }

    pub fn refresh_dirty_transform_facts(
        &mut self,
        ast: &Ast<'ast>,
        roots: &[NodeId],
        snapshot: DirtyTransformFacts,
    ) {
        self.semantic
            .refresh_dirty_transform_facts(ast, roots, snapshot);
    }

    pub fn advance_mutation_epoch(&mut self) -> u64 {
        self.semantic.advance_mutation_epoch()
    }

    pub fn create_scope(
        &mut self,
        ast: &Ast<'ast>,
        parent: ScopeId,
        node: NodeId,
        kind: scope::ScopeKind,
        flags: scope::ScopeFlags,
    ) -> ScopeId {
        self.semantic.create_scope(ast, parent, node, kind, flags)
    }

    pub fn retag_scope(&mut self, id: ScopeId, kind: scope::ScopeKind, flags: scope::ScopeFlags) {
        self.semantic.retag_scope(id, kind, flags);
    }

    pub fn reowner_scope(&mut self, id: ScopeId, node: NodeId) {
        self.semantic.reowner_scope(id, node);
    }

    pub fn reparent_scope(&mut self, id: ScopeId, parent: ScopeId) {
        self.semantic.reparent_scope(id, parent);
    }

    pub fn retire_scope(&mut self, id: ScopeId) {
        self.semantic.retire_scope(id);
    }

    pub fn register_node(&mut self, node: NodeId, parent: Option<NodeId>, scope: ScopeId) {
        self.semantic.register_node(node, parent, scope);
    }

    pub fn retire_node(&mut self, node: NodeId) {
        self.semantic.retire_node(node);
    }

    pub fn rescope_node(&mut self, node: NodeId, scope: ScopeId) {
        self.semantic.rescope_node(node, scope);
    }

    pub fn reparent_node(&mut self, node: NodeId, parent: Option<NodeId>) {
        self.semantic.reparent_node(node, parent);
    }

    pub fn declare(
        &mut self,
        ast: &Ast<'ast>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: SymbolFlags,
    ) -> SymbolId {
        self.semantic.declare(ast, scope, node, name, flags)
    }

    pub fn reference_generated(
        &mut self,
        ast: &Ast<'_>,
        scope: ScopeId,
        node: NodeId,
        name: AstStr,
        flags: ReferenceFlags,
        symbol: Option<SymbolId>,
    ) -> ReferenceId {
        self.semantic
            .reference_generated(ast, scope, node, name, flags, symbol)
    }

    pub fn retire_reference(&mut self, id: ReferenceId) {
        self.semantic.retire_reference(id);
    }

    pub fn rescope_reference(&mut self, ast: &Ast<'_>, id: ReferenceId, scope: ScopeId) {
        self.semantic.rescope_reference(ast, id, scope);
    }

    pub fn retarget_reference_node(&mut self, id: ReferenceId, node: NodeId) {
        self.semantic.retarget_reference_node(id, node);
    }

    pub fn retire_declaration(&mut self, id: DeclarationId) {
        self.semantic.retire_declaration(id);
    }

    pub fn retarget_declaration_node(&mut self, id: DeclarationId, node: NodeId) {
        self.semantic.retarget_declaration_node(id, node);
    }

    pub fn update_symbol_flags(&mut self, id: SymbolId, flags: SymbolFlags) {
        self.semantic.update_symbol_flags(id, flags);
    }

    pub fn update_declaration_flags(&mut self, id: DeclarationId, flags: SymbolFlags) {
        self.semantic.update_declaration_flags(id, flags);
    }

    pub fn move_symbol(
        &mut self,
        ast: &Ast<'_>,
        id: SymbolId,
        scope: ScopeId,
        hoisting_scopes: &[ScopeId],
    ) {
        self.semantic.move_symbol(ast, id, scope, hoisting_scopes);
    }

    pub fn retire_symbol(&mut self, id: SymbolId) {
        self.semantic.retire_symbol(id);
    }

    pub fn rebind_reference(&mut self, id: ReferenceId, symbol: Option<SymbolId>) {
        self.semantic.rebind_reference(id, symbol);
    }

    pub fn rename_reference(&mut self, ast: &Ast<'_>, id: ReferenceId, name: AstStr) {
        self.semantic.rename_reference(ast, id, name);
    }

    pub fn update_reference_flags(&mut self, id: ReferenceId, flags: ReferenceFlags) {
        self.semantic.update_reference_flags(id, flags);
    }

    pub fn rename_symbol(&mut self, ast: &Ast<'_>, id: SymbolId, name: AstStr) {
        self.semantic.rename_symbol(ast, id, name);
    }

    pub fn reserve_generated_name_exact(&mut self, ast: &Ast<'_>, scope: ScopeId, name: AstStr) {
        self.semantic.reserve_generated_name_exact(ast, scope, name);
    }

    pub fn retarget_default_export_local(
        &mut self,
        statement: NodeId,
        symbol: SymbolId,
        declaration: DeclarationId,
    ) {
        self.semantic
            .retarget_default_export_local(statement, symbol, declaration);
    }

    pub fn retarget_module_node(&mut self, old: NodeId, new: NodeId) {
        self.semantic.retarget_module_node(old, new);
    }

    pub fn replace_namespace_reexport(
        &mut self,
        ast: &Ast<'_>,
        replacement: NamespaceReexportReplacement,
    ) {
        self.semantic.replace_namespace_reexport(ast, replacement);
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
    active: &'a [bool],
    index: u32,
}

impl Iterator for ScopeIterator<'_> {
    type Item = ScopeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&scope) = self.list.get(self.index as usize) {
            let index = self.index as usize;
            self.index += 1;
            if self.active[index] {
                return Some(ScopeEntry {
                    id: ScopeId::from_usize(index),
                    scope,
                });
            }
        }
        None
    }
}

pub struct SemanticAncestorIterator<'a> {
    node_parents: &'a [Option<NodeId>],
    node_states: &'a [SemanticNodeState],
    current: Option<NodeId>,
}

impl Iterator for SemanticAncestorIterator<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        assert!(node.index() < self.node_parents.len());
        assert_eq!(
            self.node_states[node.index()],
            SemanticNodeState::Active,
            "ancestor node is inactive"
        );
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
    active: &'a [bool],
    index: u32,
}

pub struct SemanticNodeIterator<'a> {
    states: &'a [SemanticNodeState],
    index: usize,
}

impl Iterator for SemanticNodeIterator<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.states.get(self.index) {
            let index = self.index;
            self.index += 1;
            if *state == SemanticNodeState::Active {
                return Some(NodeId::from_usize(index));
            }
        }
        None
    }
}

impl Iterator for SymbolIterator<'_> {
    type Item = SymbolEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&symbol) = self.symbols.get(self.index as usize) {
            let index = self.index as usize;
            self.index += 1;
            if self.active[index] {
                return Some(SymbolEntry {
                    id: SymbolId::from_usize(index),
                    symbol,
                });
            }
        }
        None
    }
}

pub struct ReferenceIterator<'a> {
    references: &'a [Reference],
    active: &'a [bool],
    index: u32,
}

pub struct DeclarationIterator<'a> {
    declarations: &'a [Declaration],
    active: &'a [bool],
    index: u32,
}

impl Iterator for ReferenceIterator<'_> {
    type Item = ReferenceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&reference) = self.references.get(self.index as usize) {
            let index = self.index as usize;
            self.index += 1;
            if self.active[index] {
                return Some(ReferenceEntry {
                    id: ReferenceId::from_usize(index),
                    reference,
                });
            }
        }
        None
    }
}

impl Iterator for DeclarationIterator<'_> {
    type Item = DeclarationEntry;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&declaration) = self.declarations.get(self.index as usize) {
            let index = self.index as usize;
            self.index += 1;
            if self.active[index] {
                return Some(DeclarationEntry {
                    id: DeclarationId::from_usize(index),
                    declaration,
                });
            }
        }
        None
    }
}

/// Semantic analysis output and any requested scope-dependent early errors.
pub struct SemanticReturn<'a> {
    pub semantic: Semantic<'a>,
    pub diagnostics: Vec<Diagnostic<'a>>,
}

fn is_effect_boundary(ast: &Ast<'_>, node: NodeId) -> bool {
    matches!(
        ast.node_kind(node),
        swc_next_ecma_ast::NodeKind::Function
            | swc_next_ecma_ast::NodeKind::ArrowFunctionExpression
    )
}

fn is_generator_function(ast: &Ast<'_>, node: NodeId) -> bool {
    matches!(
        ast.kind_data(node),
        swc_next_ecma_ast::NodeKindData::Function(function) if function.generator(ast)
    )
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
    use swc_next_ecma_ast::{Ast, DUMMY_SP, Lang, NodeKindData, ScopeId, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

    use super::{
        AnalyzeOptions, ExportOrigin, ExportTarget, ReferenceSpace, SemanticFeatureFlags,
        SemanticNodeState, SymbolFlags, analyze,
        scope::{ScopeFlags, ScopeKind},
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
    fn semantic_entries_have_stable_active_lifecycles() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("let value = 1; value;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;

        let symbol = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"value")
            .unwrap()
            .id;
        let symbol_scope = semantic.symbol(symbol).scope;
        let declaration = semantic
            .declaration_of(semantic.declarations(symbol)[0].node)
            .unwrap();
        let declaration_node = semantic.declaration(declaration).node;
        let reference = semantic.uses(symbol)[0];
        let reference_node = semantic.reference(reference).node;

        semantic.retire_reference(reference);
        assert!(!semantic.is_reference_active(reference));
        assert!(semantic.reference_of(reference_node).is_none());
        assert!(semantic.uses(symbol).is_empty());
        assert!(
            semantic
                .iter_references()
                .all(|entry| entry.id != reference)
        );
        semantic.retire_node(reference_node);
        assert!(!semantic.is_node_active(reference_node));

        semantic.retire_declaration(declaration);
        assert!(!semantic.is_declaration_active(declaration));
        assert!(semantic.declaration_of(declaration_node).is_none());
        assert!(semantic.declarations(symbol).is_empty());
        assert!(
            semantic
                .iter_declarations()
                .all(|entry| entry.id != declaration)
        );

        semantic.retire_symbol(symbol);
        assert!(!semantic.is_symbol_active(symbol));
        assert!(semantic.binding_text(&ast, symbol_scope, "value").is_none());
        assert!(semantic.iter_symbols().all(|entry| entry.id != symbol));
        semantic.retire_node(declaration_node);
        assert!(!semantic.is_node_active(declaration_node));
    }

    #[test]
    fn analysis_leaves_detached_ast_slots_unregistered() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("value;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let detached = ast.empty_statement(DUMMY_SP);

        let semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;

        assert_eq!(
            semantic.node_state(detached.node_id()),
            SemanticNodeState::Unregistered
        );
        assert!(!semantic.iter_nodes().any(|node| node == detached.node_id()));
    }

    #[test]
    fn scopes_can_be_created_retagged_reparented_and_retired() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(parse.program, &ast, AnalyzeOptions::default()).semantic;
        let original_scope_count = semantic.iter_scopes().count();
        let owner = semantic.scope(ScopeId::ROOT).node;

        let block = semantic.create_scope(
            &ast,
            ScopeId::ROOT,
            owner,
            ScopeKind::Block,
            ScopeFlags::default(),
        );
        let nested =
            semantic.create_scope(&ast, block, owner, ScopeKind::Block, ScopeFlags::default());
        assert_eq!(semantic.scope(block).hoist_target, ScopeId::ROOT);
        assert_eq!(semantic.scope(nested).hoist_target, ScopeId::ROOT);

        semantic.retag_scope(block, ScopeKind::Function, ScopeFlags::default());
        assert_eq!(semantic.scope(block).hoist_target, block);
        assert_eq!(semantic.scope(nested).hoist_target, block);

        semantic.reparent_scope(nested, ScopeId::ROOT);
        assert_eq!(semantic.scope(nested).parent, Some(ScopeId::ROOT));
        assert_eq!(semantic.scope(nested).hoist_target, ScopeId::ROOT);

        semantic.retire_scope(nested);
        semantic.retire_scope(block);
        assert!(!semantic.is_scope_active(nested));
        assert!(!semantic.is_scope_active(block));
        assert_eq!(semantic.iter_scopes().count(), original_scope_count);
    }

    #[test]
    fn name_occupancy_tracks_retirement_and_scope_migration() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "let value; function outer() { function inner() { free; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::GENERATED_NAMES,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        let free = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"free")
            .unwrap();
        let free_name = free.reference.name;
        let inner_scope = free.reference.scope;
        let outer_scope = semantic.scope(inner_scope).parent.unwrap();
        let first_generated = semantic.reserve_generated_name(&mut ast, outer_scope, "_temp");
        let generated_name = ast.canonicalize_utf8(first_generated);
        let reserved = semantic.name_allocation_state(outer_scope, generated_name);
        assert_eq!(reserved.occupancy_count, 0);
        assert!(reserved.reserved);
        let second_generated = semantic.reserve_generated_name(&mut ast, outer_scope, "_temp");
        assert_eq!(ast.get_utf8(first_generated), "_temp");
        assert_eq!(ast.get_utf8(second_generated), "_temp1");
        assert_eq!(
            semantic
                .name_allocation_state(outer_scope, free_name)
                .occupancy_count,
            1
        );
        semantic.reparent_scope(inner_scope, ScopeId::ROOT);
        assert_eq!(
            semantic
                .name_allocation_state(outer_scope, free_name)
                .occupancy_count,
            0
        );
        semantic.retire_reference(free.id);
        assert_eq!(
            semantic
                .name_allocation_state(inner_scope, free_name)
                .occupancy_count,
            0
        );

        let value = semantic
            .iter_symbols()
            .find(|entry| ast.get_ast_str_bytes(entry.symbol.name) == b"value")
            .unwrap();
        let old_scope = value.symbol.scope;
        semantic.move_symbol(&ast, value.id, inner_scope, &[]);
        assert!(semantic.binding(old_scope, value.symbol.name).is_none());
        assert_eq!(
            semantic.binding(inner_scope, value.symbol.name),
            Some(value.id)
        );
        assert_eq!(semantic.symbol(value.id).scope, inner_scope);
    }

    #[test]
    fn class_lowering_facts_preserve_evaluation_and_scope_inputs() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "class C extends B { x; constructor() { super(); this.x; new.target; C; } [key]() {} static {} }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::CLASS_LOWERING,
                ..AnalyzeOptions::default()
            },
        )
        .semantic;

        let classes = semantic.classes_post_order().unwrap();
        assert_eq!(classes.len(), 1);
        let facts = &classes[0];
        assert!(facts.name_symbol.is_some());
        assert!(facts.constructor.is_some());
        assert_eq!(facts.methods.len(), 2);
        assert!(facts.heritage.is_some());
        assert_eq!(facts.computed_keys.len(), 1);
        assert_eq!(facts.super_calls.len(), 1);
        assert_eq!(facts.self_references.len(), 1);
        assert_eq!(facts.this_nodes.len(), 1);
        assert_eq!(facts.new_targets.len(), 1);
        assert!(facts.has_properties);
        assert!(facts.has_static_blocks);
        assert!(!facts.has_private_elements);
        assert!(!facts.has_decorators);
        assert!(semantic.is_scope_active(facts.class_scope));
        assert!(
            facts
                .methods
                .iter()
                .all(|method| semantic.is_scope_active(method.function_scope))
        );
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
