use crate::state::TransformerCtx;

pub mod async_to_generator;

pub struct ES2017 {
    async_to_generator: bool,
    transformer: Option<async_to_generator::AsyncToGenerator>,
}

impl ES2017 {
    pub fn new(options: ES2017Options) -> Self {
        Self {
            async_to_generator: options.async_to_generator,
            transformer: if options.async_to_generator {
                Some(async_to_generator::AsyncToGenerator::new())
            } else {
                None
            },
        }
    }
}

#[derive(Default)]
pub struct ES2017Options {
    pub async_to_generator: bool,
}

impl ES2017 {
    pub(crate) fn transform_program<'a>(
        &mut self,
        node: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        if let Some(transformer) = &mut self.transformer {
            transformer.transform(node, ctx);
        }
    }

    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.async_to_generator
    }
}
