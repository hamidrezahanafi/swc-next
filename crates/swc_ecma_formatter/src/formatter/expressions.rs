//! Expression dispatch and syntax context for statement boundaries.

mod arrays;
mod unary;

use swc_next_ecma_ast::{
    Argument, ArgumentData, Expr, ExprData, GetSpan, Span, UnaryOperator, UpdateOperator,
};

use super::literals::StringContext;
use super::{Formatter, comments::CommentRunLayout};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    Nested,
    UnaryArgument(UnaryOperator),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn unparenthesized_expression(&self, mut expression: Expr) -> (Expr, ExprData) {
        loop {
            let data = self.ast.expr_data(expression);
            match data {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    expression = parenthesized.expression(self.ast);
                }
                _ => return (expression, data),
            }
        }
    }

    pub(super) fn format_expression(
        &mut self,
        expression: Expr,
        position: ExpressionPosition,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let data = self.ast.expr_data(expression);
        if matches!(position, ExpressionPosition::Statement)
            && self.options.semicolons == Semicolons::AsNeeded
            && match data {
                ExprData::ArrayExpression(_)
                | ExprData::RegExpLiteral(_)
                | ExprData::StringLiteral(_) => true,
                ExprData::UnaryExpression(unary) => matches!(
                    unary.operator(self.ast),
                    UnaryOperator::Positive | UnaryOperator::Negate
                ),
                _ => false,
            }
        {
            writer.text(";");
        }
        let parenthesize = match (position, data) {
            (ExpressionPosition::UnaryArgument(parent), ExprData::UnaryExpression(unary)) => {
                matches!(parent, UnaryOperator::Positive | UnaryOperator::Negate)
                    && unary.operator(self.ast) == parent
            }
            (ExpressionPosition::UnaryArgument(parent), ExprData::UpdateExpression(update)) => {
                update.prefix(self.ast)
                    && matches!(
                        (parent, update.operator(self.ast)),
                        (UnaryOperator::Positive, UpdateOperator::Increment)
                            | (UnaryOperator::Negate, UpdateOperator::Decrement)
                    )
            }
            _ => false,
        };
        if parenthesize {
            // Adjacent signs must not merge into a different operator token.
            writer.text("(");
        }
        match data {
            ExprData::StringLiteral(literal) => {
                let statement = matches!(position, ExpressionPosition::Statement);
                if statement {
                    // An expression string must not become a directive after printing.
                    writer.text("(");
                }
                self.format_string_literal(literal, StringContext::Expression, writer);
                if statement {
                    writer.text(")");
                }
                Ok(())
            }
            ExprData::ParenthesizedExpression(parenthesized) => {
                let inner = parenthesized.expression(self.ast);
                let span = parenthesized.span(self.ast);
                let inner_span = inner.span(self.ast);
                if !self
                    .comments
                    .between(Span::new(span.start, inner_span.start))?
                    .is_empty()
                {
                    return Err(self.unsupported("parenthesized comments", parenthesized));
                }
                self.format_expression(inner, position, writer)?;
                if !self
                    .comments
                    .between(Span::new(inner_span.end, span.end))?
                    .is_empty()
                {
                    return Err(self.unsupported("parenthesized comments", parenthesized));
                }
                Ok(())
            }
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
            ExprData::UnaryExpression(unary) => self.format_unary(unary, writer),
            ExprData::UpdateExpression(update) => self.format_update(update, writer),
            _ => Err(self.unsupported("expression", expression)),
        }?;
        if parenthesize {
            writer.text(")");
        }
        Ok(())
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
