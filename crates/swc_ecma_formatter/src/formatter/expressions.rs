//! Expression dispatch and immediate grammar and statement-start contexts.

mod arrays;
mod binary;
mod unary;

use swc_next_ecma_ast::{
    Argument, ArgumentData, Expr, ExprData, GetSpan, Span, UnaryOperator, UpdateOperator,
};

use self::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::literals::StringContext;
use super::{Formatter, comments::CommentRunLayout};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    Nested,
    UnaryArgument(UnaryOperator),
    ParenthesizedUnaryArgument(UnaryOperator),
    BinaryOperand {
        operator: BinaryOperatorKind,
        side: OperandSide,
    },
    SpreadArgument,
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    statement_start: bool,
}

impl ExpressionContext {
    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            statement_start: matches!(position, ExpressionPosition::Statement),
        }
    }
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
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let data = self.ast.expr_data(expression);
        // Removed source parentheses keep the same immediate grammar relation.
        if let ExprData::ParenthesizedExpression(parenthesized) = data {
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
            self.format_expression(inner, context, writer)?;
            if !self
                .comments
                .between(Span::new(inner_span.end, span.end))?
                .is_empty()
            {
                return Err(self.unsupported("parenthesized comments", parenthesized));
            }
            return Ok(());
        }
        let binary = BinaryLike::from_data(data, self.ast);
        let parenthesize = self.expression_needs_parentheses(data, context.position);
        context.statement_start &= self.options.semicolons == Semicolons::AsNeeded;
        if context.statement_start {
            let mut asi_guard = parenthesize
                || match data {
                    ExprData::ArrayExpression(_) | ExprData::RegExpLiteral(_) => true,
                    ExprData::UnaryExpression(unary) => matches!(
                        unary.operator(self.ast),
                        UnaryOperator::Positive | UnaryOperator::Negate
                    ),
                    _ => false,
                };
            if let Some(binary) = binary {
                let (_, left) = self.unparenthesized_expression(binary.left);
                asi_guard |= self.expression_needs_parentheses(
                    left,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                );
            }
            if asi_guard {
                writer.text(";");
                context.statement_start = false;
            }
        }
        if let Some(binary) = binary {
            if parenthesize && matches!(context.position, ExpressionPosition::UnaryArgument(_)) {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        self.format_binary_enclosed(binary, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(())
                });
            }
            if parenthesize {
                writer.text("(");
            }
            self.format_binary(binary, context, writer)?;
            if parenthesize {
                writer.text(")");
            }
            return Ok(());
        }
        if parenthesize {
            writer.text("(");
        }
        match data {
            ExprData::StringLiteral(literal) => {
                self.format_string_literal(literal, StringContext::Expression, writer);
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

    fn expression_needs_parentheses(&self, data: ExprData, position: ExpressionPosition) -> bool {
        match (position, data) {
            (
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent),
                ExprData::UnaryExpression(unary),
            ) => {
                matches!(parent, UnaryOperator::Positive | UnaryOperator::Negate)
                    && unary.operator(self.ast) == parent
            }
            (
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent),
                ExprData::UpdateExpression(update),
            ) => {
                update.prefix(self.ast)
                    && matches!(
                        (parent, update.operator(self.ast)),
                        (UnaryOperator::Positive, UpdateOperator::Increment)
                            | (UnaryOperator::Negate, UpdateOperator::Decrement)
                    )
            }
            (
                ExpressionPosition::UnaryArgument(_) | ExpressionPosition::SpreadArgument,
                ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_),
            ) => true,
            (ExpressionPosition::BinaryOperand { operator, side }, _) => {
                match BinaryOperatorKind::from_data(data, self.ast) {
                    Some(child) => operator.needs_parentheses(child, side),
                    None => matches!(
                        (side, operator, data),
                        (
                            OperandSide::Left,
                            BinaryOperatorKind::Binary(swc_next_ecma_ast::BinaryOperator::Exponent),
                            ExprData::UnaryExpression(_) | ExprData::UpdateExpression(_)
                        ) | (
                            OperandSide::Left,
                            BinaryOperatorKind::Binary(
                                swc_next_ecma_ast::BinaryOperator::In
                                    | swc_next_ecma_ast::BinaryOperator::Instanceof
                            ),
                            ExprData::UnaryExpression(_)
                        )
                    ),
                }
            }
            (ExpressionPosition::Statement, ExprData::StringLiteral(_)) => true,
            _ => false,
        }
    }

    fn format_argument(
        &mut self,
        argument: Argument,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expression) => self.format_expression(
                expression,
                ExpressionContext::at(ExpressionPosition::Nested),
                writer,
            ),
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
                self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::SpreadArgument),
                    writer,
                )
            }
        }
    }
}
