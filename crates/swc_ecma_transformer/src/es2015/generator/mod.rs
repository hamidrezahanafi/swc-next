use swc_next_ecma_ast::Ast;
use swc_next_ecma_semantic::Semantic;

mod compiler;

pub(super) struct Generator;

impl Generator {
    pub(super) fn transform<'a>(&mut self, ast: &mut Ast<'a>, semantic: &mut Semantic<'a>) {
        compiler::transform(ast, semantic);
    }
}
