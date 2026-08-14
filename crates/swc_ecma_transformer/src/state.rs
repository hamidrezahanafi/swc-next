use crate::assumptions::Assumptions;
use swc_next_ecma_ast::{Ast, AstUtf8, Mark, ScopeId, SyntaxContext, TraverseCtx};
use swc_next_ecma_semantic::Semantic;

pub type TransformerCtx<'t, 'a> = TraverseCtx<'t, 'a, TransformerState>;

pub struct TransformerState {
    pub assumptions: Assumptions,
}

#[derive(Clone, Copy)]
pub(crate) struct PrivateName {
    pub(crate) text: AstUtf8,
    pub(crate) context: SyntaxContext,
}

pub(crate) fn reserve_private_name(
    ast: &mut Ast<'_>,
    semantic: &mut Semantic<'_>,
    scope: ScopeId,
    base: &str,
) -> PrivateName {
    let text = semantic.reserve_generated_name(ast, scope, base);
    let mark = ast.fresh_mark(Mark::root());
    let context = ast.apply_mark(SyntaxContext::empty(), mark);
    PrivateName { text, context }
}
