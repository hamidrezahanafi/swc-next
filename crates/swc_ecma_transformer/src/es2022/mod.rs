use swc_next_ecma_ast::{BinaryExpression, Class};

use crate::state::TransformerCtx;

pub mod private_in_object;
pub mod static_blocks;

pub struct ES2022 {
    private_in_object: bool,
    static_blocks: bool,
}

#[derive(Default)]
pub struct ES2022Options {
    pub private_in_object: bool,
    pub static_blocks: bool,
}

impl ES2022 {
    pub fn new(options: ES2022Options) -> Self {
        Self {
            private_in_object: options.private_in_object,
            static_blocks: options.static_blocks,
        }
    }
}

impl ES2022 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.private_in_object || self.static_blocks
    }

    pub(crate) fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.private_in_object {
            private_in_object::PrivateInObject.exit_binary_expression(node, ctx);
        }
    }

    pub(crate) fn exit_class(&mut self, node: Class, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if self.static_blocks {
            static_blocks::StaticBlocks::new().exit_class(node, ctx);
        }
    }
}
