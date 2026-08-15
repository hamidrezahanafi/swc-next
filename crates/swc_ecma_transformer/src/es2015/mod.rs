use swc_next_ecma_ast::{
    AssignmentExpression, ObjectExpression, ObjectProperty, Program, RegExpLiteral,
    VariableDeclarator,
};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub mod arrow;
pub mod duplicate_keys;
pub mod function_name;
mod generator;
pub mod shorthand;
pub mod sticky_regex;
pub(crate) mod util;

pub struct ES2015 {
    arrow: Option<arrow::ArrowFunctions>,
    duplicate_keys: bool,
    generator: bool,
    function_name: bool,
    shorthand_properties: bool,
    sticky_regex: bool,
}

#[derive(Default)]
pub struct ES2015Options {
    pub arrow_functions: Option<arrow::ArrowFunctionsOptions>,
    pub duplicate_keys: bool,
    pub generator: bool,
    pub function_name: bool,
    pub shorthand_properties: bool,
    pub sticky_regex: bool,
}

impl ES2015 {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow: options.arrow_functions.map(arrow::ArrowFunctions::new),
            duplicate_keys: options.duplicate_keys,
            generator: options.generator,
            function_name: options.function_name,
            shorthand_properties: options.shorthand_properties,
            sticky_regex: options.sticky_regex,
        }
    }
}

impl ES2015 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.duplicate_keys
            || self.function_name
            || self.shorthand_properties
            || self.sticky_regex
    }

    pub(crate) fn transform<'a>(
        &mut self,
        node: Program,
        semantic: &mut Semantic<'a>,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        if let Some(arrow) = &mut self.arrow {
            arrow.transform(node, ctx.ast, semantic);
        }
        if self.generator {
            generator::Generator.transform(ctx.ast, semantic);
        }
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.function_name {
            function_name::FunctionName.exit_object_property(node, ctx);
        }
        if self.shorthand_properties {
            shorthand::Shorthand.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.duplicate_keys {
            duplicate_keys::DuplicateKeys.exit_object_expression(node, ctx);
        }
    }

    pub(crate) fn exit_variable_declarator(
        &mut self,
        node: VariableDeclarator,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.function_name {
            function_name::FunctionName.exit_variable_declarator(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.function_name {
            function_name::FunctionName.exit_assignment_expression(node, ctx);
        }
    }

    pub(crate) fn exit_reg_exp_literal(
        &mut self,
        node: RegExpLiteral,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.sticky_regex {
            sticky_regex::StickyRegex.exit_reg_exp_literal(node, ctx);
        }
    }
}
