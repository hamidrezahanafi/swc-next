use crate::state::TransformerCtx;

mod export_default_from;
mod import_attributes;

#[derive(Default)]
pub struct ProposalOptions {
    pub import_attributes: bool,
    pub export_default_from: bool,
}

pub(super) struct Proposal {
    #[allow(dead_code)]
    import_attributes: bool,
    #[allow(dead_code)]
    export_default_from: bool,
    import_attributes_transformer: Option<import_attributes::ImportAttributes>,
    export_default_from_transformer: Option<export_default_from::ExportDefaultFrom>,
}

impl Proposal {
    pub fn new(options: ProposalOptions) -> Self {
        Self {
            import_attributes: options.import_attributes,
            export_default_from: options.export_default_from,
            import_attributes_transformer: if options.import_attributes {
                Some(import_attributes::ImportAttributes::new())
            } else {
                None
            },
            export_default_from_transformer: if options.export_default_from {
                Some(export_default_from::ExportDefaultFrom::new())
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
        if let Some(transformer) = &mut self.import_attributes_transformer {
            transformer.transform_program(program, ctx);
        }
        if let Some(transformer) = &mut self.export_default_from_transformer {
            transformer.transform_program(program, ctx);
        }
    }
}