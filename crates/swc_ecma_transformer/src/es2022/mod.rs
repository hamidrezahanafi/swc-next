use swc_next_ecma_ast::{Class, ClassBody, NodeId};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

mod static_blocks;

pub struct ES2022 {
    static_blocks: Option<static_blocks::StaticBlocks>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ES2022Options {
    pub static_blocks: bool,
}

impl ES2022 {
    pub fn new(options: ES2022Options) -> Self {
        Self {
            static_blocks: options.static_blocks.then(static_blocks::StaticBlocks::new),
        }
    }

    pub(crate) fn enter_class_body<'a>(
        &mut self,
        body: ClassBody,
        semantic: &mut Semantic<'a>,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        if let Some(static_blocks) = &mut self.static_blocks {
            static_blocks.enter_class_body(body, semantic, ctx);
        }
    }

    pub(crate) fn exit_class<'a>(&mut self, class: Class, ctx: &mut TransformerCtx<'_, 'a>) {
        if let Some(static_blocks) = &mut self.static_blocks {
            static_blocks.exit_class(class, ctx);
        }
    }

    pub(crate) fn exit_node<'a>(&mut self, node: NodeId, ctx: &mut TransformerCtx<'_, 'a>) {
        if let Some(static_blocks) = &mut self.static_blocks {
            static_blocks.exit_node(node, ctx);
        }
    }
}
