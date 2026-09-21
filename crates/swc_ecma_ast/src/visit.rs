use crate::{
    Accessibility, AssignmentOperator, Ast, AstUtf8, AstWtf8, BinaryOperator, ClassType,
    FormalParameterKind, FunctionType, ImportOrExportKind, ImportPhase, LogicalOperator,
    MethodDefinitionKind, PropertyKind, SourceType, Span, TsMappedTypeModifier,
    TsMethodSignatureKind, TsModuleDeclarationKind, TsTypeOperatorKind, UnaryOperator,
    UpdateOperator, VariableKind, Visit, VisitMut, VisitMutWith, VisitWith,
};

/// Shared AST and mutable user state passed to every [`crate::Traverse`] hook.
///
/// AST mutation requires [`TraverseMutCtx`]:
pub struct TraverseCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse Ast<'ast>, state: State) -> Self {
        Self { state, ast }
    }
}

/// Mutable AST state passed to every [`crate::TraverseMut`] hook.
pub struct TraverseMutCtx<'traverse, 'ast, State = ()> {
    pub state: State,
    pub ast: &'traverse mut Ast<'ast>,
}

impl<'traverse, 'ast, State> TraverseMutCtx<'traverse, 'ast, State> {
    #[inline]
    pub fn new(ast: &'traverse mut Ast<'ast>, state: State) -> Self {
        Self { state, ast }
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

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::{TraverseCtx, TraverseMutCtx};
    use crate::{
        AssignmentTarget, Ast, DUMMY_SP, IdentifierReference, NodeKindData, SimpleAssignmentTarget,
        Stmt, SubRange, Traverse, TraverseMut, TraverseMutWith, TraverseWith,
    };

    #[derive(Default)]
    struct HookRecorder {
        events: Vec<&'static str>,
        entered: Vec<NodeKindData>,
        exited: Vec<NodeKindData>,
    }

    macro_rules! record_hooks {
        ($traverse:ident, $context:ident) => {
            impl<'a> $traverse<'a> for HookRecorder {
                fn enter_node(&mut self, node: NodeKindData, _ctx: &mut $context<'_, 'a>) {
                    self.events.push("enter_node");
                    self.entered.push(node);
                }

                fn exit_node(&mut self, node: NodeKindData, _ctx: &mut $context<'_, 'a>) {
                    self.events.push("exit_node");
                    self.exited.push(node);
                }

                fn enter_assignment_target(
                    &mut self,
                    _node: AssignmentTarget,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("enter_assignment_target");
                }

                fn exit_assignment_target(
                    &mut self,
                    _node: AssignmentTarget,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("exit_assignment_target");
                }

                fn enter_simple_assignment_target(
                    &mut self,
                    _node: SimpleAssignmentTarget,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("enter_simple_assignment_target");
                }

                fn exit_simple_assignment_target(
                    &mut self,
                    _node: SimpleAssignmentTarget,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("exit_simple_assignment_target");
                }

                fn enter_identifier_reference(
                    &mut self,
                    _node: IdentifierReference,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("enter_identifier_reference");
                }

                fn exit_identifier_reference(
                    &mut self,
                    _node: IdentifierReference,
                    _ctx: &mut $context<'_, 'a>,
                ) {
                    self.events.push("exit_identifier_reference");
                }

                fn enter_scope(&mut self, node: NodeKindData, ctx: &mut $context<'_, 'a>) {
                    assert_eq!(node, ctx.ast.kind_data(node.node_id()));
                    self.events.push("enter_scope");
                }

                fn leave_scope(&mut self, node: NodeKindData, ctx: &mut $context<'_, 'a>) {
                    assert_eq!(node, ctx.ast.kind_data(node.node_id()));
                    self.events.push("leave_scope");
                }
            }
        };
    }

    record_hooks!(Traverse, TraverseCtx);
    record_hooks!(TraverseMut, TraverseMutCtx);

    const NESTED_EVENTS: &[&str] = &[
        "enter_node",
        "enter_assignment_target",
        "enter_simple_assignment_target",
        "enter_identifier_reference",
        "exit_identifier_reference",
        "exit_simple_assignment_target",
        "exit_assignment_target",
        "exit_node",
    ];
    const DIRECT_EVENTS: &[&str] = &[
        "enter_node",
        "enter_identifier_reference",
        "exit_identifier_reference",
        "exit_node",
    ];
    const SCOPE_EVENTS: &[&str] = &["enter_node", "enter_scope", "leave_scope", "exit_node"];

    fn identifier(ast: &mut Ast<'_>) -> IdentifierReference {
        let name = ast.add_utf8("x");
        ast.identifier_reference(DUMMY_SP, name)
    }

    fn assert_hooks(recorder: HookRecorder, node: NodeKindData, events: &[&str]) {
        assert_eq!(recorder.events, events);
        assert_eq!(recorder.entered, [node]);
        assert_eq!(recorder.exited, [node]);
    }

    #[test]
    fn traversal_hooks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let node = identifier(&mut ast);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(node),
        );
        let mut recorder = HookRecorder::default();
        target.traverse_with(&mut recorder, &mut TraverseCtx::new(&ast, ()));
        assert_hooks(
            recorder,
            NodeKindData::IdentifierReference(node),
            NESTED_EVENTS,
        );

        let mut recorder = HookRecorder::default();
        node.traverse_with(&mut recorder, &mut TraverseCtx::new(&ast, ()));
        assert_hooks(
            recorder,
            NodeKindData::IdentifierReference(node),
            DIRECT_EVENTS,
        );

        let block = ast.block_statement(DUMMY_SP, SubRange::empty());
        let mut recorder = HookRecorder::default();
        Stmt::BlockStatement(block).traverse_with(&mut recorder, &mut TraverseCtx::new(&ast, ()));
        assert_hooks(recorder, NodeKindData::BlockStatement(block), SCOPE_EVENTS);
    }

    #[test]
    fn mutable_traversal_hooks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let node = identifier(&mut ast);
        let target = AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(node),
        );
        let mut recorder = HookRecorder::default();
        target.traverse_mut_with(&mut recorder, &mut TraverseMutCtx::new(&mut ast, ()));
        assert_hooks(
            recorder,
            NodeKindData::IdentifierReference(node),
            NESTED_EVENTS,
        );

        let mut recorder = HookRecorder::default();
        node.traverse_mut_with(&mut recorder, &mut TraverseMutCtx::new(&mut ast, ()));
        assert_hooks(
            recorder,
            NodeKindData::IdentifierReference(node),
            DIRECT_EVENTS,
        );

        let block = ast.block_statement(DUMMY_SP, SubRange::empty());
        let mut recorder = HookRecorder::default();
        Stmt::BlockStatement(block)
            .traverse_mut_with(&mut recorder, &mut TraverseMutCtx::new(&mut ast, ()));
        assert_hooks(recorder, NodeKindData::BlockStatement(block), SCOPE_EVENTS);
    }
}
