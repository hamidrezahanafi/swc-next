use std::collections::HashMap;

use swc_next_allocator::{hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
    ArrowFunctionExpression, Ast, Class, Directive, Function, FunctionBody, MethodDefinition,
    MethodDefinitionKind, NodeId, NodeKind, NodeKindData, Program, ScopeId,
};

use crate::{
    ClassLoweringFact, ClassMethodFact, EffectCounts, FunctionCaptureData, Semantic,
    SemanticFeatureFlags, SemanticTransformFacts,
    binder::{ReferenceFlags, ReferenceSpace, SymbolTracker},
    module_record::ModuleRecordBuilder,
    scope::ScopeTracker,
};

pub(crate) struct SemanticVisitCtx<'traverse, 'ast> {
    pub state: SemanticContext<'ast>,
    pub ast: &'traverse Ast<'ast>,
    current_node: Option<NodeId>,
}

impl<'traverse, 'ast> SemanticVisitCtx<'traverse, 'ast> {
    pub fn new(ast: &'traverse Ast<'ast>, state: SemanticContext<'ast>) -> Self {
        Self {
            state,
            ast,
            current_node: None,
        }
    }

    pub fn enter_node(&mut self, node: NodeId) -> ReferenceFlags {
        let parent = self.current_node;
        self.current_node = Some(node);
        self.state.enter_transform_node(self.ast, node);
        self.state.enter_node(node, parent)
    }

    pub fn leave_node(&mut self, node: NodeId) {
        debug_assert_eq!(self.current_node, Some(node));
        self.state.leave_transform_node(self.ast, node);
        self.current_node = self.state.node_parents[node.index()];
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.current_node
            .and_then(|node| self.state.node_parents[node.index()])
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.current_node
    }

    pub fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.current_node, |node| {
            self.state.node_parents[node.index()]
        })
    }

    pub fn finalize(self) -> Semantic<'ast> {
        self.state.finalize(self.ast)
    }
}

#[derive(Default)]
struct CaptureCandidates {
    arguments: Vec<NodeId>,
    new_targets: Vec<NodeId>,
}

/// Per-class data collected during the single traversal pass.
/// Finalize resolves candidates into the finished `ClassLoweringFact`.
struct ClassInfoBuilder {
    class: Class,
    /// Constructor method, if present.
    constructor: Option<Function>,
    /// Methods collected during traversal.
    methods: Vec<CollectedMethod>,
    /// Computed key node IDs.
    computed_keys: Vec<NodeId>,
    has_property: bool,
    has_private_element: bool,
    has_static_block: bool,
    has_decorator: bool,
    /// Rewrite-site raw node IDs.
    super_nodes: Vec<NodeId>,
    this_nodes: Vec<NodeId>,
    new_target_nodes: Vec<NodeId>,
    /// Identifier reference nodes for self-reference resolution in finalize.
    identifier_reference_nodes: Vec<NodeId>,
}

struct CollectedMethod {
    definition: MethodDefinition,
    function: Function,
    kind: MethodDefinitionKind,
    is_static: bool,
    computed: bool,
}

struct TransformFactsCollector<'a> {
    features: SemanticFeatureFlags,
    effects: Option<ArenaVec<'a, EffectCounts>>,
    effect_owners: Option<ArenaVec<'a, Option<NodeId>>>,
    generator_functions: ArenaVec<'a, Function>,
    async_functions: ArenaVec<'a, NodeId>,
    function_boundaries: ArenaVec<'a, NodeId>,
    capture_boundaries: Vec<Option<NodeId>>,
    capture_candidates: HashMap<NodeId, CaptureCandidates>,
    yield_nodes: HashMap<NodeId, Vec<NodeId>>,
    /// Class stack for single-pass collection.
    class_stack: Vec<ClassInfoBuilder>,
    /// Post-order class builders, popped in leave_node.
    class_infos_post_order: Vec<ClassInfoBuilder>,
}

impl<'a> TransformFactsCollector<'a> {
    fn new(tree: &Ast<'a>, features: SemanticFeatureFlags) -> Self {
        let allocator = tree.allocator();
        let collect_effects =
            features.intersects(SemanticFeatureFlags::YIELD | SemanticFeatureFlags::AWAIT);
        let mut effects =
            collect_effects.then(|| ArenaVec::with_capacity_in(tree.node_count(), allocator));
        let mut effect_owners =
            collect_effects.then(|| ArenaVec::with_capacity_in(tree.node_count(), allocator));
        if let Some(effects) = &mut effects {
            effects.resize(tree.node_count(), EffectCounts::default());
        }
        if let Some(owners) = &mut effect_owners {
            owners.resize(tree.node_count(), None);
        }
        Self {
            features,
            effects,
            effect_owners,
            generator_functions: ArenaVec::new_in(allocator),
            async_functions: ArenaVec::new_in(allocator),
            function_boundaries: ArenaVec::new_in(allocator),
            capture_boundaries: Vec::new(),
            capture_candidates: HashMap::new(),
            yield_nodes: HashMap::new(),
            class_stack: Vec::new(),
            class_infos_post_order: Vec::new(),
        }
    }

    fn enter_node(&mut self, tree: &Ast<'_>, node: NodeId) {
        match tree.node_kind(node) {
            NodeKind::Function => self.capture_boundaries.push(Some(node)),
            NodeKind::Class => self.capture_boundaries.push(None),
            _ => {}
        }
        if self
            .features
            .contains(SemanticFeatureFlags::FUNCTION_ENV_CAPTURE)
            && let Some(Some(owner)) = self.capture_boundaries.last().copied()
            && let NodeKindData::Function(owner_function) = tree.kind_data(owner)
            && owner_function.generator(tree)
        {
            match tree.kind_data(node) {
                NodeKindData::IdentifierReference(identifier)
                    if tree.get_utf8(identifier.name(tree)) == "arguments" =>
                {
                    self.capture_candidates
                        .entry(owner)
                        .or_default()
                        .arguments
                        .push(node);
                }
                NodeKindData::MetaProperty(meta)
                    if tree.get_utf8(meta.meta(tree).name(tree)) == "new"
                        && tree.get_utf8(meta.property(tree).name(tree)) == "target" =>
                {
                    self.capture_candidates
                        .entry(owner)
                        .or_default()
                        .new_targets
                        .push(node);
                }
                _ => {}
            }
        }

        if matches!(
            tree.node_kind(node),
            NodeKind::Function | NodeKind::ArrowFunctionExpression
        ) {
            self.function_boundaries.push(node);
        }
        if let Some(owners) = &mut self.effect_owners {
            owners[node.index()] = self.function_boundaries.last().copied();
        }
        let Some(effects) = &mut self.effects else {
            self.enter_class_node(tree, node);
            return;
        };
        match tree.node_kind(node) {
            NodeKind::YieldExpression if self.features.contains(SemanticFeatureFlags::YIELD) => {
                effects[node.index()].yields = 1;
                if let Some(owner) = self.function_boundaries.last().copied() {
                    self.yield_nodes.entry(owner).or_default().push(node);
                }
            }
            NodeKind::AwaitExpression if self.features.contains(SemanticFeatureFlags::AWAIT) => {
                effects[node.index()].awaits = 1;
            }
            _ => {}
        }
        self.enter_class_node(tree, node);
    }

    /// Records class-related nodes during traversal for single-pass collection.
    fn enter_class_node(&mut self, tree: &Ast<'_>, node: NodeId) {
        if !self.features.contains(SemanticFeatureFlags::CLASS_LOWERING) {
            return;
        }
        // Push a new class builder when entering a class node.
        if let NodeKindData::Class(class) = tree.kind_data(node) {
            self.class_stack.push(ClassInfoBuilder {
                class,
                constructor: None,
                methods: Vec::new(),
                computed_keys: Vec::new(),
                has_property: false,
                has_private_element: false,
                has_static_block: false,
                has_decorator: !class.decorators(tree).is_empty(),
                super_nodes: Vec::new(),
                this_nodes: Vec::new(),
                new_target_nodes: Vec::new(),
                identifier_reference_nodes: Vec::new(),
            });
            return;
        }
        // Only record into the current (topmost) class builder.
        let Some(builder) = self.class_stack.last_mut() else {
            return;
        };
        // Record class body elements.
        // Class body elements are visited as children of the class node.
        // We detect them by their NodeKindData.
        match tree.kind_data(node) {
            NodeKindData::MethodDefinition(method) => {
                let function = method.value(tree);
                let computed = method.computed(tree);
                if method.kind(tree) == MethodDefinitionKind::Constructor {
                    builder.constructor = Some(function);
                }
                if computed {
                    builder.computed_keys.push(method.key(tree).node_id());
                }
                if method.key(tree).is_private_identifier(tree) {
                    builder.has_private_element = true;
                }
                if !method.decorators(tree).is_empty() {
                    builder.has_decorator = true;
                }
                // Always push to methods, including constructor, matching
                // the old build_class_lowering_fact behavior.
                builder.methods.push(CollectedMethod {
                    definition: method,
                    function,
                    kind: method.kind(tree),
                    is_static: method.r#static(tree),
                    computed,
                });
            }
            NodeKindData::PropertyDefinition(property) => {
                builder.has_property = true;
                if property.key(tree).is_private_identifier(tree) {
                    builder.has_private_element = true;
                }
                if !property.decorators(tree).is_empty() {
                    builder.has_decorator = true;
                }
                if property.computed(tree) {
                    builder.computed_keys.push(property.key(tree).node_id());
                }
            }
            NodeKindData::StaticBlock(_) => {
                builder.has_static_block = true;
            }
            NodeKindData::Super(_) => {
                builder.super_nodes.push(node);
            }
            NodeKindData::ThisExpression(_) => {
                builder.this_nodes.push(node);
            }
            NodeKindData::MetaProperty(meta) => {
                if tree.get_utf8(meta.meta(tree).name(tree)) == "new"
                    && tree.get_utf8(meta.property(tree).name(tree)) == "target"
                {
                    builder.new_target_nodes.push(node);
                }
            }
            NodeKindData::IdentifierReference(_) => {
                builder.identifier_reference_nodes.push(node);
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, tree: &Ast<'_>, node: NodeId, parent: Option<NodeId>) {
        match tree.kind_data(node) {
            NodeKindData::Function(function) => {
                if self
                    .features
                    .contains(SemanticFeatureFlags::GENERATOR_FUNCTIONS)
                    && function.generator(tree)
                {
                    self.generator_functions.push(function);
                }
                if self
                    .features
                    .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                    && function.r#async(tree)
                {
                    self.async_functions.push(node);
                }
            }
            NodeKindData::Class(_)
                if self.features.contains(SemanticFeatureFlags::CLASS_LOWERING) =>
            {
                // Pop the class builder from the stack and push to post-order.
                if let Some(builder) = self.class_stack.pop() {
                    self.class_infos_post_order.push(builder);
                }
            }
            NodeKindData::ArrowFunctionExpression(arrow)
                if self
                    .features
                    .contains(SemanticFeatureFlags::ASYNC_FUNCTIONS)
                    && arrow.r#async(tree) =>
            {
                self.async_functions.push(node);
            }
            _ => {}
        }

        let is_boundary = matches!(
            tree.node_kind(node),
            NodeKind::Function | NodeKind::ArrowFunctionExpression
        );
        if !is_boundary && let (Some(parent), Some(effects)) = (parent, &mut self.effects) {
            let child = effects[node.index()];
            effects[parent.index()].yields = effects[parent.index()]
                .yields
                .checked_add(child.yields)
                .expect("yield effect count overflow");
            effects[parent.index()].awaits = effects[parent.index()]
                .awaits
                .checked_add(child.awaits)
                .expect("await effect count overflow");
        }
        if matches!(tree.node_kind(node), NodeKind::Function | NodeKind::Class) {
            let expected = matches!(tree.node_kind(node), NodeKind::Function).then_some(node);
            debug_assert_eq!(self.capture_boundaries.pop(), Some(expected));
        }
        if is_boundary {
            debug_assert_eq!(self.function_boundaries.pop(), Some(node));
        }
    }

    fn finish(self, semantic: &Semantic<'a>, tree: &Ast<'a>) -> SemanticTransformFacts<'a> {
        let allocator = tree.allocator();
        let mut generator_function_counts =
            ArenaVec::with_capacity_in(tree.node_count(), allocator);
        generator_function_counts.resize(tree.node_count(), 0u32);
        for function in &self.generator_functions {
            for node in semantic.ancestors(Some(function.node_id())) {
                generator_function_counts[node.index()] = generator_function_counts[node.index()]
                    .checked_add(1)
                    .expect("generator function count overflow");
            }
        }
        let mut async_function_counts = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        async_function_counts.resize(tree.node_count(), 0u32);
        for function in &self.async_functions {
            for node in semantic.ancestors(Some(*function)) {
                async_function_counts[node.index()] = async_function_counts[node.index()]
                    .checked_add(1)
                    .expect("async function count overflow");
            }
        }
        let mut function_captures = ArenaHashMap::new_in(allocator);
        for (function, candidates) in self.capture_candidates {
            let mut captures = FunctionCaptureData {
                arguments: ArenaVec::new_in(allocator),
                new_targets: ArenaVec::new_in(allocator),
            };
            for node in candidates.arguments {
                let Some(reference_id) = semantic.reference_of(node) else {
                    continue;
                };
                let reference = semantic.reference(reference_id);
                if reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none() {
                    captures.arguments.push(reference_id);
                }
            }
            captures.new_targets.extend(candidates.new_targets);
            if !captures.arguments.is_empty() || !captures.new_targets.is_empty() {
                function_captures.insert(function, captures);
            }
        }
        let mut yield_nodes = ArenaHashMap::new_in(allocator);
        for (function, nodes) in self.yield_nodes {
            let mut arena_nodes = ArenaVec::with_capacity_in(nodes.len(), allocator);
            arena_nodes.extend(nodes);
            yield_nodes.insert(function, arena_nodes);
        }

        // Resolve class builders into ClassLoweringFact using only candidate resolution.
        let mut class_lowering = ArenaVec::with_capacity_in(self.class_infos_post_order.len(), allocator);
        for builder in self.class_infos_post_order {
            let fact = resolve_class_lowering_fact(builder, semantic, tree, allocator);
            class_lowering.push(fact);
        }

        let mut class_counts = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        class_counts.resize(tree.node_count(), 0u32);
        for facts in &class_lowering {
            for node in semantic.ancestors(Some(facts.class.node_id())) {
                class_counts[node.index()] = class_counts[node.index()]
                    .checked_add(1)
                    .expect("class count overflow");
            }
        }
        SemanticTransformFacts {
            effects: self.effects,
            effect_owners: self.effect_owners,
            generator_functions: self.generator_functions,
            async_functions: self.async_functions,
            generator_function_counts,
            async_function_counts,
            class_counts,
            function_captures,
            yield_nodes,
            class_lowering,
        }
    }
}

/// Resolves a collected `ClassInfoBuilder` into a `ClassLoweringFact`.
/// Only resolves already-recorded candidates — does not re-scan AST or class body.
fn resolve_class_lowering_fact<'a>(
    builder: ClassInfoBuilder,
    semantic: &Semantic<'a>,
    tree: &Ast<'a>,
    allocator: &'a swc_next_allocator::Allocator,
) -> ClassLoweringFact<'a> {
    let class = builder.class;

    // Resolve name symbol from the class id binding (candidate resolution).
    let name_symbol = class
        .id(tree)
        .and_then(|id| semantic.declaration_of(id.node_id()))
        .map(|declaration| semantic.declaration(declaration).symbol);

    // Resolve methods with known function scopes.
    let mut methods = ArenaVec::with_capacity_in(builder.methods.len(), allocator);
    for m in builder.methods {
        methods.push(ClassMethodFact {
            definition: m.definition,
            function: m.function,
            function_scope: semantic.scope_of(m.function.node_id()),
            kind: m.kind,
            r#static: m.is_static,
            computed: m.computed,
        });
    }

    // Resolve super() calls and super.member accesses from collected super nodes.
    let mut super_calls = ArenaVec::new_in(allocator);
    let mut super_members = ArenaVec::new_in(allocator);
    for super_node in &builder.super_nodes {
        if let Some(parent) = semantic.parent_of(*super_node) {
            match tree.kind_data(parent) {
                NodeKindData::CallExpression(_) => super_calls.push(parent),
                NodeKindData::MemberExpression(_) => super_members.push(parent),
                _ => {}
            }
        }
    }

    // Resolve self-references from collected identifier reference nodes.
    let mut self_references = ArenaVec::new_in(allocator);
    for id_node in &builder.identifier_reference_nodes {
        if let Some(reference) = semantic.reference_of(*id_node)
            && semantic.reference(reference).symbol == name_symbol
        {
            self_references.push(reference);
        }
    }

    // Resolve this and new.target nodes.
    let mut this_nodes = ArenaVec::with_capacity_in(builder.this_nodes.len(), allocator);
    for node in &builder.this_nodes {
        this_nodes.push(*node);
    }
    let mut new_targets = ArenaVec::with_capacity_in(builder.new_target_nodes.len(), allocator);
    for node in &builder.new_target_nodes {
        new_targets.push(*node);
    }

    // Computed keys.
    let mut computed_keys = ArenaVec::with_capacity_in(builder.computed_keys.len(), allocator);
    for key in builder.computed_keys {
        computed_keys.push(key);
    }

    ClassLoweringFact {
        class,
        class_scope: semantic.scope_of(class.node_id()),
        name_symbol,
        constructor: builder.constructor,
        methods,
        heritage: class.super_class(tree).map(|heritage| heritage.node_id()),
        computed_keys,
        super_calls,
        super_members,
        self_references,
        this_nodes,
        new_targets,
        has_properties: builder.has_property,
        has_private_elements: builder.has_private_element,
        has_static_blocks: builder.has_static_block,
        has_decorators: builder.has_decorator,
    }
}

/// Builds a `ClassLoweringFact` from a dirty subtree. Used by
/// `refresh_function_lists` when `set_data` replaces a class payload with a
/// function payload at the same NodeId without changing `node_kind`.
pub(crate) fn build_class_lowering_fact<'a>(
    semantic: &Semantic<'a>,
    tree: &Ast<'a>,
    class: Class,
) -> ClassLoweringFact<'a> {
    let allocator = tree.allocator();
    let name_symbol = class
        .id(tree)
        .and_then(|id| semantic.declaration_of(id.node_id()))
        .map(|declaration| semantic.declaration(declaration).symbol);
    let mut constructor = None;
    let mut methods = ArenaVec::new_in(allocator);
    let mut computed_keys = ArenaVec::new_in(allocator);
    let mut has_properties = false;
    let mut has_private_elements = false;
    let mut has_static_blocks = false;
    let mut has_decorators = !class.decorators(tree).is_empty();
    let elements = class.body(tree).body(tree);
    for index in 0..elements.len() {
        let element = elements.get_node(tree, index).unwrap();
        if let Some(method) = element.as_method_definition(tree) {
            let function = method.value(tree);
            if method.kind(tree) == swc_next_ecma_ast::MethodDefinitionKind::Constructor {
                constructor = Some(function);
            }
            if method.computed(tree) {
                computed_keys.push(method.key(tree).node_id());
            }
            has_private_elements |= method.key(tree).is_private_identifier(tree);
            has_decorators |= !method.decorators(tree).is_empty();
            methods.push(ClassMethodFact {
                definition: method,
                function,
                function_scope: semantic.scope_of(function.node_id()),
                kind: method.kind(tree),
                r#static: method.r#static(tree),
                computed: method.computed(tree),
            });
        } else if let Some(property) = element.as_property_definition(tree) {
            has_properties = true;
            has_private_elements |= property.key(tree).is_private_identifier(tree);
            has_decorators |= !property.decorators(tree).is_empty();
            if property.computed(tree) {
                computed_keys.push(property.key(tree).node_id());
            }
        } else if element.is_static_block(tree) {
            has_static_blocks = true;
        }
    }

    let mut super_calls = ArenaVec::new_in(allocator);
    let mut super_members = ArenaVec::new_in(allocator);
    let mut self_references = ArenaVec::new_in(allocator);
    let mut this_nodes = ArenaVec::new_in(allocator);
    let mut new_targets = ArenaVec::new_in(allocator);
    let mut pending: Vec<_> = semantic.node_children(class.node_id()).collect();
    while let Some(node) = pending.pop() {
        if matches!(tree.kind_data(node), NodeKindData::Class(_)) {
            continue;
        }
        match tree.kind_data(node) {
            NodeKindData::Super(_) => {
                if let Some(parent) = semantic.parent_of(node) {
                    match tree.kind_data(parent) {
                        NodeKindData::CallExpression(_) => super_calls.push(parent),
                        NodeKindData::MemberExpression(_) => super_members.push(parent),
                        _ => {}
                    }
                }
            }
            NodeKindData::IdentifierReference(_) => {
                if let Some(reference) = semantic.reference_of(node)
                    && semantic.reference(reference).symbol == name_symbol
                {
                    self_references.push(reference);
                }
            }
            NodeKindData::ThisExpression(_) => this_nodes.push(node),
            NodeKindData::MetaProperty(meta)
                if tree.get_utf8(meta.meta(tree).name(tree)) == "new"
                    && tree.get_utf8(meta.property(tree).name(tree)) == "target" =>
            {
                new_targets.push(node);
            }
            _ => {}
        }
        pending.extend(semantic.node_children(node));
    }

    ClassLoweringFact {
        class,
        class_scope: semantic.scope_of(class.node_id()),
        name_symbol,
        constructor,
        methods,
        heritage: class.super_class(tree).map(|heritage| heritage.node_id()),
        computed_keys,
        super_calls,
        super_members,
        self_references,
        this_nodes,
        new_targets,
        has_properties,
        has_private_elements,
        has_static_blocks,
        has_decorators,
    }
}

/// Walk context combining the current node, scope tracker, symbol tracker,
/// and reference state. Visitor hooks use it to inspect surrounding source
/// structure or to query and contribute symbols.
pub(crate) struct SemanticContext<'a> {
    pub scope: ScopeTracker<'a>,
    pub symbols: SymbolTracker<'a>,
    pub module_record: ModuleRecordBuilder<'a>,
    // depth of TS type-only context. inspect via `in_type_position()`.
    type_position_depth: u32,
    ts_namespace_depth: u32,
    reference_flags: ReferenceFlags,
    // per-node tables carried into the final `Semantic`
    node_scopes: ArenaVec<'a, ScopeId>,
    node_parents: ArenaVec<'a, Option<NodeId>>,
    features: SemanticFeatureFlags,
    transform_facts: Option<TransformFactsCollector<'a>>,
}

impl<'a> SemanticContext<'a> {
    pub fn init(root: Program, tree: &Ast<'a>, features: SemanticFeatureFlags) -> Self {
        let allocator = tree.allocator();
        let mut node_scopes = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        let mut node_parents = ArenaVec::with_capacity_in(tree.node_count(), allocator);
        node_scopes.resize(tree.node_count(), ScopeId::ROOT);
        node_parents.resize(tree.node_count(), None);
        Self {
            scope: ScopeTracker::init(root, tree),
            symbols: SymbolTracker::init(tree),
            module_record: ModuleRecordBuilder::new(allocator),
            type_position_depth: 0,
            ts_namespace_depth: 0,
            reference_flags: ReferenceFlags::default(),
            node_scopes,
            node_parents,
            features,
            transform_facts: (!features.is_empty())
                .then(|| TransformFactsCollector::new(tree, features)),
        }
    }

    fn enter_transform_node(&mut self, tree: &Ast<'_>, node: NodeId) {
        if let Some(facts) = &mut self.transform_facts {
            facts.enter_node(tree, node);
        }
    }

    fn leave_transform_node(&mut self, tree: &Ast<'_>, node: NodeId) {
        if let Some(facts) = &mut self.transform_facts {
            facts.leave_node(tree, node, self.node_parents[node.index()]);
        }
    }

    /// True when the walker is currently inside a TS type-only subtree.
    pub fn in_type_position(&self) -> bool {
        self.type_position_depth > 0
    }

    /// True when the walker is currently inside a TS namespace body.
    pub fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    #[inline]
    pub fn enter_node(&mut self, node: NodeId, parent: Option<NodeId>) -> ReferenceFlags {
        self.node_scopes[node.index()] = self.scope.current;
        self.node_parents[node.index()] = parent;
        let flags = self.reference_flags;
        self.reference_flags = self.default_reference_flags();
        flags
    }

    #[inline]
    pub fn default_reference_flags(&self) -> ReferenceFlags {
        ReferenceFlags::default().with_space(if self.in_type_position() {
            ReferenceSpace::Type
        } else {
            ReferenceSpace::Value
        })
    }

    #[inline]
    pub fn replace_reference_flags(&mut self, flags: ReferenceFlags) -> ReferenceFlags {
        std::mem::replace(&mut self.reference_flags, flags)
    }

    #[inline]
    pub fn enter_type_position(&mut self) {
        self.type_position_depth += 1;
    }

    #[inline]
    pub fn exit_type_position(&mut self) {
        assert!(self.type_position_depth > 0);
        self.type_position_depth -= 1;
        self.reference_flags = self.default_reference_flags();
    }

    pub fn enter_directive(&mut self, tree: &Ast, directive: Directive) {
        self.scope.enter_directive(tree, directive);
    }

    pub fn enter_function(&mut self, tree: &Ast, function: Function, annex_b_if_body: bool) {
        self.scope.enter_function(tree, function, annex_b_if_body);
    }

    pub fn enter_ts_function(&mut self, node: NodeId) {
        self.scope.enter_ts_function(node);
    }

    pub fn enter_arrow_function_expression(&mut self, tree: &Ast, arrow: ArrowFunctionExpression) {
        self.scope.enter_arrow_function_expression(tree, arrow);
    }

    pub fn enter_function_body(&mut self, tree: &Ast, body: FunctionBody, parent: Option<NodeId>) {
        self.scope.enter_function_body(tree, body, parent);
    }

    pub fn enter_block_statement(&mut self, tree: &Ast, node: NodeId) {
        self.scope.enter_block_statement(tree, node);
    }

    pub fn enter_with_statement(&mut self, node: NodeId) {
        self.scope.enter_with_scope(node);
        // The object expression evaluates in the enclosing environment, so
        // the with scope is created only after visiting it. Keep the creator
        // node associated with the scope it creates.
        self.node_scopes[node.index()] = self.scope.current;
    }

    pub fn exit_with_statement(&mut self, node: NodeId) {
        self.scope.exit_with_scope(node);
    }

    pub fn enter_switch_case(&mut self, parent: Option<NodeId>) {
        self.scope.enter_switch_case(parent);
    }

    pub fn enter_ts_enum_body(&mut self, node: NodeId) {
        self.scope.enter_ts_enum_body(node);
    }

    fn enter_block_scope(&mut self, node: NodeId) {
        self.scope.enter_block_scope(node);
    }

    pub fn enter_for_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_in_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_for_of_statement(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_catch_clause(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_interface_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_type_alias_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_function_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_constructor_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_method_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_call_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_construct_signature_declaration(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_index_signature(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_mapped_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_conditional_type(&mut self, node: NodeId) {
        self.enter_block_scope(node);
    }

    pub fn enter_ts_module_block(&mut self, node: NodeId) {
        self.ts_namespace_depth += 1;
        self.scope.enter_ts_module_block(node);
    }

    pub fn exit_ts_module_block(&mut self, node: NodeId) {
        debug_assert!(self.ts_namespace_depth > 0);
        self.scope.exit_scope(node);
        self.ts_namespace_depth -= 1;
    }

    pub fn enter_class(&mut self, tree: &Ast, class: Class) {
        self.scope.enter_class(tree, class);
    }

    pub fn enter_static_block(&mut self, node: NodeId) {
        self.scope.enter_static_block(node);
    }

    pub fn enter_decorator(&mut self) {
        self.scope.enter_decorator();
    }

    pub fn exit_scope(&mut self, node: NodeId) {
        self.scope.exit_scope(node);
    }

    pub fn exit_decorator(&mut self) {
        self.scope.exit_decorator();
    }

    pub fn finalize(mut self, tree: &Ast<'a>) -> Semantic<'a> {
        let (scopes, has_with_statement) = self.scope.into_scope_tree();
        let mut semantic = self.symbols.finalize(
            tree,
            scopes,
            has_with_statement,
            self.features,
            self.node_scopes,
            self.node_parents,
            self.module_record,
        );
        semantic.features = self.features;
        if let Some(facts) = self.transform_facts.take() {
            semantic.transform_facts = Some(facts.finish(&semantic, tree));
        }
        semantic
    }
}
