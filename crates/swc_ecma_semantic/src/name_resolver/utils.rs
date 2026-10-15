use swc_next_ecma_ast::{
    Ast, BindingIdentifier, BindingPattern, Expr, PropertyKey, Visit, VisitWith,
};

struct BindingIdVisitor<'op, 'ast, 'id, F> {
    ast: &'ast Ast<'ast, 'id>,
    op: &'op mut F,
}

pub fn for_each_binding_id<'id, 'ast>(
    ast: &'ast Ast<'ast, 'id>,
    pattern: BindingPattern<'id>,
    mut op: impl FnMut(BindingIdentifier<'id>),
) {
    pattern.visit_with(&mut BindingIdVisitor { ast, op: &mut op });
}

impl<'id, 'ast, F: FnMut(BindingIdentifier<'id>)> Visit<'ast, 'id>
    for BindingIdVisitor<'_, 'ast, 'id, F>
{
    fn ast(&self) -> &Ast<'ast, 'id> {
        self.ast
    }

    fn visit_binding_identifier(&mut self, identifier: BindingIdentifier<'id>) {
        (self.op)(identifier);
    }

    /// Initializers and computed expressions do not introduce bindings.
    fn visit_expr(&mut self, _: Expr<'id>) {}

    /// A non-computed property name does not introduce a binding either.
    fn visit_property_key(&mut self, _: PropertyKey<'id>) {}
}
