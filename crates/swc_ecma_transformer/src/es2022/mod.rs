use swc_next_ecma_ast::ClassId;
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

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

    pub(crate) fn exit_class<'a>(
        &mut self,
        class: ClassId,
        semantic: &mut Semantic<'a>,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        if self.static_blocks {
            static_blocks::StaticBlocks.exit_class(class, semantic, ctx);
        }
    }
}
