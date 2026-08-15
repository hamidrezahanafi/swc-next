use swc_next_ecma_ast::{
    ArrayExpression, AssignmentExpression, BinaryExpression, BlockStatement, CallExpression,
    MetaProperty, NewExpression, ObjectExpression, ObjectProperty, Program, RegExpLiteral,
    UnaryExpression, VariableDeclarator,
};
use swc_next_ecma_semantic::Semantic;

use crate::state::TransformerCtx;

pub mod arrow;
pub mod block_scoped_fn;
pub mod computed_props;
pub mod duplicate_keys;
pub mod for_of;
pub mod function_name;
mod generator;
pub mod instanceof;
pub mod new_target;
pub mod shorthand;
pub mod spread;
pub mod sticky_regex;
pub mod typeof_symbol;
pub(crate) mod util;

pub struct ES2015 {
    arrow: Option<arrow::ArrowFunctions>,
    block_scoped_fns: bool,
    computed_props: bool,
    duplicate_keys: bool,
    for_of: bool,
    generator: bool,
    function_name: bool,
    instanceof: bool,
    new_target: bool,
    new_target_state: Option<new_target::NewTarget>,
    shorthand_properties: bool,
    spread: bool,
    sticky_regex: bool,
    typeof_symbol: bool,
    typeof_symbol_state: Option<typeof_symbol::TypeofSymbol>,
}

#[derive(Default)]
pub struct ES2015Options {
    pub arrow_functions: Option<arrow::ArrowFunctionsOptions>,
    pub block_scoped_fns: bool,
    pub computed_props: bool,
    pub duplicate_keys: bool,
    pub for_of: bool,
    pub generator: bool,
    pub function_name: bool,
    pub instanceof: bool,
    pub new_target: bool,
    pub shorthand_properties: bool,
    pub spread: bool,
    pub sticky_regex: bool,
    pub typeof_symbol: bool,
}

impl ES2015 {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow: options.arrow_functions.map(arrow::ArrowFunctions::new),
            block_scoped_fns: options.block_scoped_fns,
            computed_props: options.computed_props,
            duplicate_keys: options.duplicate_keys,
            for_of: options.for_of,
            generator: options.generator,
            function_name: options.function_name,
            instanceof: options.instanceof,
            new_target: options.new_target,
            new_target_state: if options.new_target {
                Some(new_target::NewTarget::new())
            } else {
                None
            },
            shorthand_properties: options.shorthand_properties,
            spread: options.spread,
            sticky_regex: options.sticky_regex,
            typeof_symbol: options.typeof_symbol,
            typeof_symbol_state: if options.typeof_symbol {
                Some(typeof_symbol::TypeofSymbol::new())
            } else {
                None
            },
        }
    }
}

impl ES2015 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.block_scoped_fns
            || self.computed_props
            || self.duplicate_keys
            || self.function_name
            || self.instanceof
            || self.new_target
            || self.shorthand_properties
            || self.spread
            || self.sticky_regex
            || self.typeof_symbol
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
        if self.for_of {
            for_of::ForOf::new().transform(node, ctx.ast, semantic);
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
        if self.computed_props {
            computed_props::ComputedProps.exit_object_expression(node, ctx);
        }
    }

    pub(crate) fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.typeof_symbol {
            if let Some(ts) = &mut self.typeof_symbol_state {
                ts.enter_binary_expression(node, ctx);
            }
        }
    }

    pub(crate) fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.instanceof {
            instanceof::InstanceOf.exit_binary_expression(node, ctx);
        }
        if self.typeof_symbol {
            if let Some(ts) = &mut self.typeof_symbol_state {
                ts.exit_binary_expression(node, ctx);
            }
        }
    }

    pub(crate) fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.typeof_symbol {
            if let Some(ts) = &mut self.typeof_symbol_state {
                ts.exit_unary_expression(node, ctx);
            }
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

    pub(crate) fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.block_scoped_fns {
            block_scoped_fn::BlockScopedFns.exit_block_statement(node, ctx);
        }
    }

    pub(crate) fn exit_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.new_target {
            if let Some(nt) = &mut self.new_target_state {
                nt.exit_meta_property(node, ctx);
            }
        }
    }

    pub(crate) fn exit_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.spread {
            spread::Spread::new().exit_array_expression(node, ctx);
        }
    }

    pub(crate) fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.spread {
            spread::Spread::new().exit_call_expression(node, ctx);
        }
    }

    pub(crate) fn exit_new_expression(
        &mut self,
        node: NewExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.spread {
            spread::Spread::new().exit_new_expression(node, ctx);
        }
    }
}
