use swc_next_ecma_ast::{AssignmentExpression, Ast, IdentifierReference, NodeId, TraverseCtx};

use crate::{ReferenceFlags, binder::SymbolTracker, scope::ScopeTracker};

pub type Cx<'t, 'a> = TraverseCtx<'t, 'a, State<'a>>;

pub struct State<'a> {
    pub(crate) path: Vec<NodeId>,
    pub(crate) scope: ScopeTracker<'a>,
    pub(crate) symbols: SymbolTracker<'a>,
}

impl<'t, 'a> State<'a> {
    pub fn new(ast: &'t Ast<'a>) -> Self {
        Self {
            path: Vec::with_capacity(64),
            scope: ScopeTracker::init(ast),
            symbols: SymbolTracker::init(ast),
        }
    }

    pub fn parent(&self) -> Option<NodeId> {
        self.path.iter().rev().nth(1).copied()
    }

    pub fn current_node(&self) -> Option<NodeId> {
        self.path.last().copied()
    }

    pub fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.path.iter().rev().copied()
    }

    pub fn record_ts_expando_assignment(&mut self, ast: &'t Ast<'a>, node: AssignmentExpression) {
        self.symbols.record_ts_expando_assignment(
            ast,
            node,
            &self.scope,
            self.path.iter().rev().copied(),
        );
    }

    #[inline]
    pub fn record_identifier_reference(
        &mut self,
        ast: &'t Ast<'a>,
        node: IdentifierReference,
        flags: ReferenceFlags,
    ) {
        self.symbols
            .add_identifier_reference(ast, node, &self.scope, flags);
    }
}
