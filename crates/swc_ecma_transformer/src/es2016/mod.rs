//! ES2016 transforms.

use swc_next_ecma_ast::{AssignmentExpression, BinaryExpression};

use crate::state::TransformerCtx;

mod exponentiation;

pub struct ES2016 {
    exponentiation: bool,
}

#[derive(Default)]
pub struct ES2016Options {
    pub exponentiation: bool,
}

impl ES2016 {
    pub fn new(options: ES2016Options) -> Self {
        Self {
            exponentiation: options.exponentiation,
        }
    }
}

impl ES2016 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.exponentiation
    }

    pub(crate) fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.exponentiation {
            exponentiation::Exponentiation.exit_binary_expression(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.exponentiation {
            exponentiation::Exponentiation.exit_assignment_expression(node, ctx);
        }
    }
}
