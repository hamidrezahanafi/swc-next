use std::collections::{HashMap, HashSet};

use swc_next_ecma_ast::{
    Ast, AstNode, Class, Function, IdentifierReference, MetaProperty, NodeId, NodeKind, Program,
    Visit, VisitWith, YieldExpression, run_visit,
};
use swc_next_ecma_semantic::{ReferenceSpace, Semantic};

/// Immutable facts about yields in the original tree.
///
/// A node is present when evaluating that node may reach a `yield` belonging
/// to the same function. Yields in nested function and arrow bodies stop at
/// their own function boundary and therefore do not taint the enclosing tree.
/// These facts are intentionally collected before any AST mutation and are
/// never queried with generated node IDs.
#[derive(Default)]
pub(super) struct YieldFacts {
    containing_yield: HashSet<NodeId>,
}

impl YieldFacts {
    fn collect<'a>(
        program: Program,
        ast: &'a Ast<'a>,
        semantic: &'a Semantic<'a>,
    ) -> GeneratorAnalysis {
        let mut collector = YieldCollector {
            ast,
            path: Vec::new(),
            facts: Self::default(),
            generators: Vec::new(),
            semantic,
            function_boundaries: Vec::new(),
            captures: HashMap::new(),
        };
        run_visit(program, &mut collector);
        GeneratorAnalysis {
            yields: collector.facts,
            generators: collector.generators,
            captures: collector.captures,
        }
    }

    #[inline]
    pub(super) fn contains<N: AstNode>(&self, node: N) -> bool {
        self.containing_yield.contains(&node.node_id())
    }
}

struct YieldCollector<'a> {
    ast: &'a Ast<'a>,
    path: Vec<NodeId>,
    facts: YieldFacts,
    generators: Vec<Function>,
    semantic: &'a Semantic<'a>,
    /// `None` is a class boundary. A class field must not capture the
    /// surrounding generator's function environment.
    function_boundaries: Vec<Option<Function>>,
    captures: HashMap<NodeId, CaptureFacts>,
}

impl<'a> Visit<'a> for YieldCollector<'a> {
    fn ast(&self) -> &Ast<'a> {
        self.ast
    }

    fn enter_node(&mut self, node: NodeId) {
        self.path.push(node);
    }

    fn leave_node(&mut self, node: NodeId) {
        let popped = self.path.pop();
        debug_assert_eq!(popped, Some(node));
    }

    fn visit_yield_expression(&mut self, node: YieldExpression) {
        self.enter_node(node.node_id());

        // Mark the yield and every containing subtree up to, and including,
        // its own function. Do not let a nested function's yield affect an
        // enclosing generator.
        for ancestor in self.path.iter().rev().copied() {
            self.facts.containing_yield.insert(ancestor);
            if matches!(
                self.ast.node_kind(ancestor),
                NodeKind::Function | NodeKind::ArrowFunctionExpression
            ) {
                break;
            }
        }

        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }

    fn visit_function(&mut self, node: Function) {
        self.function_boundaries.push(Some(node));
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.function_boundaries.pop();

        // Post-order is important: lowering an outer generator must see any
        // nested generator already converted, while its original yield facts
        // remain available by stable NodeId.
        if node.generator(self.ast) {
            self.generators.push(node);
        }
    }

    fn visit_class(&mut self, node: Class) {
        self.function_boundaries.push(None);
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
        self.function_boundaries.pop();
    }

    fn visit_identifier_reference(&mut self, node: IdentifierReference) {
        if self.ast.get_utf8(node.name(self.ast)) == "arguments" {
            let unresolved = self
                .semantic
                .reference_of(node.node_id())
                .map(|id| self.semantic.reference(id))
                .is_some_and(|reference| {
                    reference.flags.space == ReferenceSpace::Value && reference.symbol.is_none()
                });
            if unresolved
                && let Some(Some(function)) = self.function_boundaries.last().copied()
                && function.generator(self.ast)
            {
                self.captures
                    .entry(function.node_id())
                    .or_default()
                    .arguments
                    .push(node);
            }
        }
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }

    fn visit_meta_property(&mut self, node: MetaProperty) {
        let meta = node.meta(self.ast);
        let property = node.property(self.ast);
        if self.ast.get_utf8(meta.name(self.ast)) == "new"
            && self.ast.get_utf8(property.name(self.ast)) == "target"
            && let Some(Some(function)) = self.function_boundaries.last().copied()
            && function.generator(self.ast)
        {
            self.captures
                .entry(function.node_id())
                .or_default()
                .new_targets
                .push(node);
        }
        self.enter_node(node.node_id());
        node.visit_children_with(self);
        self.leave_node(node.node_id());
    }
}

pub(in crate::es2015) struct GeneratorAnalysis {
    pub(super) yields: YieldFacts,
    pub(super) generators: Vec<Function>,
    pub(super) captures: HashMap<NodeId, CaptureFacts>,
}

#[derive(Default)]
pub(super) struct CaptureFacts {
    pub(super) arguments: Vec<IdentifierReference>,
    pub(super) new_targets: Vec<MetaProperty>,
}

pub(super) fn analyze<'a>(
    program: Program,
    ast: &'a Ast<'a>,
    semantic: &'a Semantic<'a>,
) -> GeneratorAnalysis {
    YieldFacts::collect(program, ast, semantic)
}
