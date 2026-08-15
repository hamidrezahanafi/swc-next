use swc_next_ecma_ast::BindingProperty;

use crate::state::TransformerCtx;

mod edge_default_param;

#[derive(Default)]
pub struct BugfixesOptions {
    pub edge_default_param: bool,
}

pub(super) struct Bugfixes {
    pub edge_default_param: bool,
    /// Whether we're currently inside an arrow function's parameter list.
    in_arrow_params: bool,
}

impl Bugfixes {
    pub fn new(options: BugfixesOptions) -> Self {
        Self {
            edge_default_param: options.edge_default_param,
            in_arrow_params: false,
        }
    }

    pub fn needs_node_traversal(&self) -> bool {
        self.edge_default_param
    }

    pub fn enter_arrow_function_expression(&mut self) {
        self.in_arrow_params = true;
    }

    pub fn exit_arrow_function_expression(&mut self) {
        self.in_arrow_params = false;
    }

    pub fn exit_binding_property(
        &self,
        node: BindingProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.edge_default_param && self.in_arrow_params {
            edge_default_param::EdgeDefaultParam::new()
                .exit_binding_property(node, ctx);
        }
    }
}