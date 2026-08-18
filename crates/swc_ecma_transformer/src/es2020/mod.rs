//! ES2020 transforms.

use swc_next_ecma_ast::{AssignmentExpression, Expr, LogicalExpression, Program};

use crate::state::TransformerCtx;

mod export_namespace_from;
mod nullish_coalescing;
mod optional_chaining;

pub struct ES2020 {
    export_namespace_from: bool,
    nullish_coalescing: bool,
    optional_chaining: bool,
}

#[derive(Default)]
pub struct ES2020Options {
    pub export_namespace_from: bool,
    pub nullish_coalescing: bool,
    pub optional_chaining: bool,
}

impl ES2020 {
    pub fn new(options: ES2020Options) -> Self {
        Self {
            export_namespace_from: options.export_namespace_from,
            nullish_coalescing: options.nullish_coalescing,
            optional_chaining: options.optional_chaining,
        }
    }
}

impl ES2020 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.nullish_coalescing || self.export_namespace_from || self.optional_chaining
    }

    pub(crate) fn transform_program(&self, node: Program, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if self.export_namespace_from {
            export_namespace_from::ExportNamespaceFrom::new().transform_program(node, ctx);
        }
    }

    pub(crate) fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.nullish_coalescing {
            nullish_coalescing::NullishCoalescing.exit_logical_expression(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.nullish_coalescing {
            nullish_coalescing::NullishCoalescing.exit_assignment_expression(node, ctx);
        }
    }

    pub(crate) fn exit_expression(&mut self, node: Expr, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if self.optional_chaining {
            optional_chaining::OptionalChaining::new().exit_expression(node, ctx);
        }
    }
}
