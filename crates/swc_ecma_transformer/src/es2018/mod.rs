//! ES2018 transforms.

use crate::state::TransformerCtx;

mod object_rest_spread;

#[derive(Default)]
pub struct ES2018Options {
    pub object_rest_spread: bool,
}

pub(super) struct ES2018 {
    #[allow(dead_code)]
    object_rest_spread: bool,
    object_rest_spread_transformer: Option<object_rest_spread::ObjectRestSpread>,
}

impl ES2018 {
    pub fn new(options: ES2018Options) -> Self {
        Self {
            object_rest_spread: options.object_rest_spread,
            object_rest_spread_transformer: if options.object_rest_spread {
                Some(object_rest_spread::ObjectRestSpread::new())
            } else {
                None
            },
        }
    }

    pub fn needs_node_traversal(&self) -> bool {
        false
    }

    pub fn transform_program<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        if let Some(transformer) = &mut self.object_rest_spread_transformer {
            transformer.transform_program(program, ctx);
        }
    }
}
