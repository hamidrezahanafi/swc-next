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
        let data = self.ast.expr_data(expression);
        if matches!(position, ExpressionPosition::Statement)
            && self.options.semicolons == Semicolons::AsNeeded
            && matches!(
                data,
                ExprData::ArrayExpression(_) | ExprData::RegExpLiteral(_)
            )
        {
            writer.text(";");
        }
        match data {
            ExprData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(())
            }
            ExprData::BooleanLiteral(literal) => {
                writer.text(if literal.value(self.ast) {
                    "true"
                } else {
                    "false"
                });
                Ok(())
            }
            ExprData::NullLiteral(_) => {
                writer.text("null");
                Ok(())
            }
            ExprData::NumericLiteral(literal) => {
                self.format_numeric_literal(literal, writer);
                Ok(())
            }
            ExprData::BigIntLiteral(literal) => {
                self.format_bigint_literal(literal, writer);
                Ok(())
            }
            ExprData::RegExpLiteral(literal) => {
                self.format_regexp_literal(literal, writer);
                Ok(())
            }
            ExprData::ArrayExpression(array) => self.format_array(array, writer),
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
