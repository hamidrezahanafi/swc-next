use crate::state::TransformerCtx;

mod display_name;
mod jsx_self;
mod pure_annotations;

#[derive(Default)]
pub struct ReactOptions {
    pub jsx_self: bool,
    pub display_name: bool,
    pub pure_annotations: bool,
}

pub(super) struct React {
    #[allow(dead_code)]
    jsx_self: bool,
    jsx_self_transformer: Option<jsx_self::JsxSelf>,
    #[allow(dead_code)]
    display_name: bool,
    display_name_transformer: Option<display_name::DisplayName>,
    #[allow(dead_code)]
    pure_annotations: bool,
    pure_annotations_transformer: Option<pure_annotations::PureAnnotations>,
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
            display_name: options.display_name,
            display_name_transformer: if options.display_name {
                Some(display_name::DisplayName::new())
            } else {
                None
            },
            pure_annotations: options.pure_annotations,
            pure_annotations_transformer: if options.pure_annotations {
                Some(pure_annotations::PureAnnotations::new())
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
        if let Some(t) = &mut self.jsx_self_transformer {
            t.transform_program(program, ctx);
        }
        if let Some(t) = &mut self.display_name_transformer {
            t.transform_program(program, ctx);
        }
        if let Some(t) = &mut self.pure_annotations_transformer {
            t.transform_program(program, ctx);
        }
    }
}
