use swc_next_ecma_ast::{Ast, Program};
use swc_next_ecma_semantic::Semantic;

use self::analysis::GeneratorAnalysis;

mod analysis;
mod compiler;

pub(super) struct Generator;

impl Generator {
    pub(super) fn analyze<'a>(
        program: Program,
        ast: &'a Ast<'a>,
        semantic: &'a Semantic<'a>,
    ) -> GeneratorAnalysis {
        analysis::analyze(program, ast, semantic)
    }

    pub(super) fn transform(
        &mut self,
        analysis: GeneratorAnalysis,
        ast: &mut Ast<'_>,
        semantic: &Semantic<'_>,
    ) {
        compiler::transform(analysis, ast, semantic);
    }
}
