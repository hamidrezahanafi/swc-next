use crate::assumptions::Assumptions;
use crate::statement_injector::StatementInjector;
use swc_next_ecma_ast::TraverseCtx;

pub type TransformerCtx<'t, 'a> = TraverseCtx<'t, 'a, TransformerState>;

pub struct TransformerState {
    pub assumptions: Assumptions,
    pub(crate) statement_injector: StatementInjector,
}
