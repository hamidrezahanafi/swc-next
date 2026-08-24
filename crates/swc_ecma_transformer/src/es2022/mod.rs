use swc_next_ecma_ast::{Ast, Program};
use swc_next_ecma_semantic::Semantic;

mod static_blocks;

pub struct ES2022 {
    static_blocks: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ES2022Options {
    pub static_blocks: bool,
}

impl ES2022 {
    pub fn new(options: ES2022Options) -> Self {
        Self {
            static_blocks: options.static_blocks,
        }
    }

    pub(crate) fn transform_program<'a>(
        &mut self,
        program: Program,
        semantic: &mut Semantic<'a>,
        ast: &mut Ast<'a>,
    ) {
        if self.static_blocks {
            static_blocks::transform_program(program, semantic, ast);
        }
    }
}
