//! Expression dispatch and syntax context for statement boundaries.

mod arrays;

use swc_next_ecma_ast::{Argument, ArgumentData, Expr, ExprData, GetSpan, Span};

use super::{Formatter, comments::CommentRunLayout};
use crate::{FormatError, Semicolons, document::Writer};

pub(super) enum ExpressionPosition {
    Statement,
    Nested,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_expression(
        &mut self,
        expression: Expr,
        position: ExpressionPosition,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(())
            }
            ExprData::ArrayExpression(array) => {
                if matches!(position, ExpressionPosition::Statement)
                    && self.options.semicolons == Semicolons::AsNeeded
                {
                    writer.text(";");
                }
                self.format_array(array, writer)
            }
            _ => Err(self.unsupported("expression", expression)),
        }
    }

    fn format_argument(
        &mut self,
        argument: Argument,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expression) => {
                self.format_expression(expression, ExpressionPosition::Nested, writer)
            }
            ArgumentData::SpreadElement(spread) => {
                let argument = spread.argument(self.ast);
                writer.text("...");
                let comments = self.comments.between(Span::new(
                    spread.span(self.ast).start + 3,
                    argument.span(self.ast).start,
                ))?;
                let comments = self.comments.take(comments.len());
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
                if let Some(last) = comments.last() {
                    self.comment_gap(last.span.end, argument.span(self.ast).start, writer);
                }
                self.format_expression(argument, ExpressionPosition::Nested, writer)
            }
        }
    }
}
