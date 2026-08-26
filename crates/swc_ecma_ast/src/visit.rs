use crate::{
    Accessibility, AssignmentOperator, Ast, AstId, AstNode, AstUtf8, AstWtf8, BinaryOperator,
    ClassType, Expr, FormalParameterKind, FunctionType, ImportOrExportKind, ImportPhase,
    LogicalOperator, MethodDefinitionKind, NodeKind, PropertyKind, SourceType, Span,
    TsMappedTypeModifier, TsMethodSignatureKind, TsModuleDeclarationKind, TsTypeOperatorKind,
    UnaryOperator, UpdateOperator, VariableKind, Visit, VisitMut, VisitMutWith, VisitWith,
};

/// Controls how traversal continues after an enter hook.
#[must_use]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TraverseAction<T: AstNode> {
    /// Visit this node's children and invoke its exit hooks.
    #[default]
    Proceed,
    /// Skip this node's children and exit hooks, then continue with its siblings.
    Skip,
    /// Stop the entire traversal immediately without invoking pending exit hooks.
    Stop,
    /// Replace this node in its parent with another node of the same AST type.
    /// The replacement is not traversed, and the replaced node's remaining
    /// hooks are not invoked. For a root node, the replacement is returned to
    /// the caller of [`crate::TraverseWith::traverse_with`].
    Replace(T),
}

/// Mutable AST state passed to every [`Traverse`] hook.
pub struct TraverseCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse mut Ast<'ast>,
    path: Vec<AstId>,
    expr_replacement: Option<(AstId, AstId)>,
}

impl<'traverse, 'ast, State> TraverseCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse mut Ast<'ast>, state: State) -> Self {
        Self {
            state,
            ast,
            path: Vec::new(),
            expr_replacement: None,
        }
    }

    /// Returns the concrete kind currently stored for `node_id`.
    #[inline]
    pub fn node_kind(&self, node_id: AstId) -> NodeKind {
        self.ast.node_kind(node_id)
    }

    /// Returns the path from the traversal root to the current node.
    #[inline]
    pub fn path(&self) -> &[AstId] {
        &self.path
    }

    /// Returns the current node, or `None` outside an enter/exit hook.
    #[inline]
    pub fn current_node(&self) -> Option<AstId> {
        self.ancestor(0)
    }

    /// Returns the current node's parent.
    #[inline]
    pub fn parent(&self) -> Option<AstId> {
        self.ancestor(1)
    }

    /// Returns an ancestor by distance, where zero is the current node.
    #[inline]
    pub fn ancestor(&self, distance: usize) -> Option<AstId> {
        distance
            .checked_add(1)
            .and_then(|offset| self.path.len().checked_sub(offset))
            .map(|index| self.path[index])
    }

    /// Iterates from the current node toward the traversal root.
    #[inline]
    pub fn ancestors(&self) -> impl DoubleEndedIterator<Item = AstId> + '_ {
        self.path.iter().rev().copied()
    }

    /// Returns zero for the root node, one for its direct children, and so on.
    #[inline]
    pub fn depth(&self) -> usize {
        self.path.len().saturating_sub(1)
    }

    /// Replaces the current expression through the parent field or sub-range
    /// slot that is already being traversed.
    ///
    /// The replacement is not traversed during the current pass.
    #[inline]
    pub fn replace_current_expr(&mut self, replacement: Expr) {
        let current = self
            .current_node()
            .expect("replace_current_expr must be called from a traversal hook");
        assert!(
            self.expr_replacement
                .replace((current, replacement.ast_id()))
                .is_none(),
            "the current node can only be replaced once"
        );
    }

    #[inline]
    pub(crate) fn take_expr_replacement(&mut self) -> Option<AstId> {
        self.expr_replacement
            .take()
            .map(|(_current, replacement)| replacement)
    }

    #[inline]
    pub(crate) fn push_node(&mut self, node_id: AstId) {
        self.path.push(node_id);
    }

    #[inline]
    pub(crate) fn pop_node(&mut self, expected: AstId) {
        let actual = self.path.pop();
        debug_assert_eq!(actual, Some(expected));
    }
}

macro_rules! dummy_visit_mut_impl {
    ($ty:ty) => {
        impl<'a, V: ?Sized + Visit<'a>> VisitWith<'a, V> for $ty {
            #[inline]
            fn visit_with(self, _visitor: &mut V) {}

            #[inline]
            fn visit_children_with(self, _visitor: &mut V) {}
        }

        impl<'a, V: ?Sized + VisitMut<'a>> VisitMutWith<'a, V> for $ty {
            #[inline]
            fn visit_mut_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }

            #[inline]
            fn visit_mut_children_with(self, _visitor: &mut V) -> Option<Self> {
                None
            }
        }
    };
}

dummy_visit_mut_impl!(bool);
dummy_visit_mut_impl!(f64);
dummy_visit_mut_impl!(Span);
dummy_visit_mut_impl!(AstUtf8);
dummy_visit_mut_impl!(AstWtf8);
dummy_visit_mut_impl!(ImportPhase);
dummy_visit_mut_impl!(SourceType);
dummy_visit_mut_impl!(MethodDefinitionKind);
dummy_visit_mut_impl!(Accessibility);
dummy_visit_mut_impl!(ClassType);
dummy_visit_mut_impl!(BinaryOperator);
dummy_visit_mut_impl!(LogicalOperator);
dummy_visit_mut_impl!(UnaryOperator);
dummy_visit_mut_impl!(UpdateOperator);
dummy_visit_mut_impl!(AssignmentOperator);
dummy_visit_mut_impl!(VariableKind);
dummy_visit_mut_impl!(FunctionType);
dummy_visit_mut_impl!(FormalParameterKind);
dummy_visit_mut_impl!(PropertyKind);
dummy_visit_mut_impl!(ImportOrExportKind);
dummy_visit_mut_impl!(TsTypeOperatorKind);
dummy_visit_mut_impl!(TsMethodSignatureKind);
dummy_visit_mut_impl!(TsModuleDeclarationKind);
dummy_visit_mut_impl!(TsMappedTypeModifier);
