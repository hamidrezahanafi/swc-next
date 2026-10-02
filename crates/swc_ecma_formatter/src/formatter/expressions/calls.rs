//! Callees and source-owned argument delimiters.

use swc_next_ecma_ast::{CallExpression, GetSpan};

use super::{ExpressionContext, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::ExpressionTail;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if call.optional(ast) || call.type_arguments(ast).is_some() {
            return Err(self.unsupported("call suffix", call));
        }
        let callee = call.callee(ast);
        let arguments = self
            .source_arguments(
                call.arguments(ast),
                callee.span(ast).end,
                call.span(ast).end,
            )
            .ok_or_else(|| self.unsupported("call opening boundary", call))?;
        let tail = self.format_expression(
            callee,
            ExpressionContext::at(ExpressionPosition::CallCallee)
                .with_statement_start(context.statement_start)
                .with_tail_owner(TailOwner::ParentOperation)
                .with_content_mode(arguments.callee_mode),
            writer,
        )?;
        self.format_source_arguments(arguments, tail, writer)
    }
}
