//! ES2021 transforms.

use swc_next_ecma_ast::AssignmentExpression;

use crate::state::TransformerCtx;

mod logical_assignment;

pub struct ES2021 {
    logical_assignment: bool,
}

#[derive(Default)]
pub struct ES2021Options {
    pub logical_assignment: bool,
}

impl ES2021 {
    pub fn new(options: ES2021Options) -> Self {
        Self {
            logical_assignment: options.logical_assignment,
        }
    }
}

impl ES2021 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.logical_assignment
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.logical_assignment {
            logical_assignment::LogicalAssignment.exit_assignment_expression(node, ctx);
        }
    }
}
