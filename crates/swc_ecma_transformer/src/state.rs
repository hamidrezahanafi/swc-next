use crate::{
    assumptions::Assumptions,
    mutation::{MutationCommit, MutationError, MutationTransaction, ReservedPrivateName},
};
use swc_next_ecma_ast::{NodeId, ScopeId, TraverseCtx};
use swc_next_ecma_semantic::Semantic;

pub type TransformerCtx<'traverse, 'semantic, 'ast> =
    TraverseCtx<'traverse, 'ast, TransformerState<'semantic, 'ast>>;

pub struct TransformerState<'semantic, 'ast> {
    pub assumptions: Assumptions,
    pub semantic: &'semantic mut Semantic<'ast>,
}

pub(crate) trait TransformerCtxExt<'ast> {
    fn semantic(&self) -> &Semantic<'ast>;

    fn current_scope(&self) -> Option<ScopeId>;

    fn transaction<T>(
        &mut self,
        root: NodeId,
        build: impl FnOnce(&mut MutationTransaction<'_, 'ast>) -> T,
    ) -> Result<(T, MutationCommit), MutationError>;
}

impl<'traverse, 'semantic, 'ast> TransformerCtxExt<'ast>
    for TransformerCtx<'traverse, 'semantic, 'ast>
{
    fn semantic(&self) -> &Semantic<'ast> {
        self.state.semantic
    }

    fn current_scope(&self) -> Option<ScopeId> {
        self.current_node()
            .map(|node| self.state.semantic.scope_of(node))
    }

    fn transaction<T>(
        &mut self,
        root: NodeId,
        build: impl FnOnce(&mut MutationTransaction<'_, 'ast>) -> T,
    ) -> Result<(T, MutationCommit), MutationError> {
        let mut transaction = MutationTransaction::begin(self.ast, self.state.semantic, root)?;
        let value = build(&mut transaction);
        let commit = transaction.commit()?;
        Ok((value, commit))
    }
}

pub(crate) type PrivateName = ReservedPrivateName;
