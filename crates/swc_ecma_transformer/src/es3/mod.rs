//! ES3 transforms.

use swc_next_ecma_ast::{AssignmentExpression, MemberExpression, ObjectProperty};

use crate::state::TransformerCtx;

mod member_expr_lits;
mod property_literals;
mod reserved_words;

pub struct ES3 {
    member_expr_lits: bool,
    property_literals: bool,
    reserved_words: bool,
}

#[derive(Default)]
pub struct ES3Options {
    pub member_expr_lits: bool,
    pub property_literals: bool,
    pub reserved_words: bool,
}

impl ES3 {
    pub fn new(options: ES3Options) -> Self {
        Self {
            member_expr_lits: options.member_expr_lits,
            property_literals: options.property_literals,
            reserved_words: options.reserved_words,
        }
    }
}

impl ES3 {
    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.member_expr_lits || self.property_literals || self.reserved_words
    }

    pub(crate) fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.member_expr_lits {
            member_expr_lits::MemberExprLits.exit_member_expression(node, ctx);
        }
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.property_literals {
            property_literals::PropertyLiterals.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.reserved_words {
            reserved_words::ReservedWords.exit_assignment_expression(node, ctx);
        }
    }
}