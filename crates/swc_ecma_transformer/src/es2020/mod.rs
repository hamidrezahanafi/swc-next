//! ES2020 transforms.

use swc_next_ecma_ast::{AssignmentExpression, LogicalExpression};

use crate::state::TransformerCtx;

mod nullish_coalescing;

pub struct ES2020 {
    nullish_coalescing: bool,
}

#[derive(Default)]
pub struct ES2020Options {
    pub nullish_coalescing: bool,
}

impl ES2020 {
    pub fn new(options: ES2020Options) -> Self {
        Self {
            nullish_coalescing: options.nullish_coalescing,
        }
    }
}

impl ES2020 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.nullish_coalescing
    }

    pub(crate) fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.nullish_coalescing {
            nullish_coalescing::NullishCoalescing.exit_logical_expression(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.nullish_coalescing {
            nullish_coalescing::NullishCoalescing.exit_assignment_expression(node, ctx);
        }
    }
}