use crate::state::TransformerCtx;

mod jsx_self;

#[derive(Default)]
pub struct ReactOptions {
    pub jsx_self: bool,
}

pub(super) struct React {
    #[allow(dead_code)]
    jsx_self: bool,
    jsx_self_transformer: Option<jsx_self::JsxSelf>,
}

impl React {
    pub fn new(options: ReactOptions) -> Self {
        Self {
            jsx_self: options.jsx_self,
            jsx_self_transformer: if options.jsx_self {
                Some(jsx_self::JsxSelf::new())
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
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        if let Some(t) = &mut self.jsx_self_transformer {
            t.transform_program(program, ctx);
        }
    }
}