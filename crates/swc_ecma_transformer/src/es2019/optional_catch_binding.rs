use swc_next_ecma_ast::{CatchClause, DUMMY_SP, TypedSubRange};

use crate::state::TransformerCtx;

pub(super) struct OptionalCatchBinding;

impl OptionalCatchBinding {
    pub fn enter_catch_clause(&mut self, node: CatchClause, ctx: &mut TransformerCtx<'_, '_>) {
        if node.param(ctx.ast).is_some() {
            return;
        }

        let name = ctx.ast.add_utf8("unused");
        let param = ctx.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name,
            TypedSubRange::empty(),
            None,
            false,
        );
        node.set_param(ctx.ast, Some(param));
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, DUMMY_SP, TraverseCtx, TypedSubRange};

    use super::OptionalCatchBinding;
    use crate::{assumptions::Assumptions, state::TransformerState};

    fn transformer_state() -> TransformerState {
        TransformerState {
            assumptions: Assumptions::default(),
        }
    }

    #[test]
    fn adds_binding_to_optional_catch_clause() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let body = ast.block_statement(DUMMY_SP, TypedSubRange::empty());
        let catch_clause = ast.catch_clause(DUMMY_SP, None, body);

        OptionalCatchBinding.enter_catch_clause(
            catch_clause,
            &mut TraverseCtx::new(&mut ast, transformer_state()),
        );

        let binding = catch_clause
            .param(&ast)
            .and_then(|param| param.as_binding_identifier(&ast))
            .expect("optional catch binding is added");
        assert_eq!(ast.get_utf8(binding.name(&ast)), "unused");
    }

    #[test]
    fn preserves_existing_catch_binding() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let name = ast.add_utf8("error");
        let binding = ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name,
            TypedSubRange::empty(),
            None,
            false,
        );
        let body = ast.block_statement(DUMMY_SP, TypedSubRange::empty());
        let catch_clause = ast.catch_clause(DUMMY_SP, Some(binding), body);

        OptionalCatchBinding.enter_catch_clause(
            catch_clause,
            &mut TraverseCtx::new(&mut ast, transformer_state()),
        );

        let binding = catch_clause
            .param(&ast)
            .and_then(|param| param.as_binding_identifier(&ast))
            .expect("existing catch binding is preserved");
        assert_eq!(ast.get_utf8(binding.name(&ast)), "error");
    }
}
