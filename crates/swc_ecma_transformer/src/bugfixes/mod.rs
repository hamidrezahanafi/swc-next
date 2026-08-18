use swc_next_ecma_ast::{BindingProperty, Expr, Program};

use crate::state::TransformerCtx;

mod async_arrows_in_class;
mod edge_default_param;
mod safari_id_destructuring_collision_in_function_expression;
mod template_literal_caching;

#[derive(Default)]
pub struct BugfixesOptions {
    pub async_arrows_in_class: bool,
    pub edge_default_param: bool,
    pub safari_id_destructuring_collision_in_function_expression: bool,
    pub template_literal_caching: bool,
}

pub(super) struct Bugfixes {
    pub async_arrows_in_class: bool,
    pub edge_default_param: bool,
    #[allow(dead_code)]
    pub safari_id_destructuring: bool,
    in_arrow_params: bool,
    async_arrows_in_class_state: async_arrows_in_class::AsyncArrowsInClass,
    template_literal_caching_transformer: Option<template_literal_caching::TemplateLiteralCaching>,
    safari_id_destructuring_transformer:
        Option<safari_id_destructuring_collision_in_function_expression::SafariIdDestructuring>,
}

impl Bugfixes {
    pub fn new(options: BugfixesOptions) -> Self {
        Self {
            async_arrows_in_class: options.async_arrows_in_class,
            edge_default_param: options.edge_default_param,
            safari_id_destructuring: options
                .safari_id_destructuring_collision_in_function_expression,
            in_arrow_params: false,
            async_arrows_in_class_state: async_arrows_in_class::AsyncArrowsInClass::new(),
            template_literal_caching_transformer: if options.template_literal_caching {
                Some(template_literal_caching::TemplateLiteralCaching::new())
            } else {
                None
            },
            safari_id_destructuring_transformer: if options
                .safari_id_destructuring_collision_in_function_expression
            {
                Some(safari_id_destructuring_collision_in_function_expression::SafariIdDestructuring::new())
            } else {
                None
            },
        }
    }

    pub fn needs_node_traversal(&self) -> bool {
        self.edge_default_param || self.async_arrows_in_class
    }

    pub fn transform_program<'a>(
        &mut self,
        program: Program,
        ctx: &mut TransformerCtx<'_, '_, 'a>,
    ) {
        if let Some(transformer) = &mut self.template_literal_caching_transformer {
            transformer.transform(program, ctx);
        }
        if let Some(transformer) = &mut self.safari_id_destructuring_transformer {
            transformer.transform_program(program, ctx);
        }
    }

    pub fn enter_arrow_function_expression(&mut self) {
        self.in_arrow_params = true;
    }

    pub fn exit_arrow_function_expression(&mut self) {
        self.in_arrow_params = false;
    }

    pub fn enter_method_definition(&mut self) {
        if self.async_arrows_in_class {
            self.async_arrows_in_class_state.enter_method_definition();
        }
    }

    pub fn exit_method_definition(&mut self) {
        if self.async_arrows_in_class {
            self.async_arrows_in_class_state.exit_method_definition();
        }
    }

    pub fn exit_expression(&mut self, node: Expr, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if self.async_arrows_in_class {
            self.async_arrows_in_class_state.exit_expression(node, ctx);
        }
    }

    pub fn exit_binding_property(
        &self,
        node: BindingProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.edge_default_param && self.in_arrow_params {
            edge_default_param::EdgeDefaultParam::new().exit_binding_property(node, ctx);
        }
    }
}
