//! Expression dispatch and immediate grammar and statement-start contexts.

mod arrays;
mod binary;
mod parentheses;
mod unary;

use swc_next_ecma_ast::{
    Argument, ArgumentData, Expr, ExprData, GetSpan, Span, UnaryOperator, UpdateOperator,
};

use self::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use super::literals::StringContext;
use super::suppression::ContentMode;
use super::{
    Formatter,
    comments::{CommentRunLayout, ExpressionTail},
};
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum StatementStart {
    None,
    Pending,
    Guard,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HeadComments {
    Pending,
    Claimed,
}

/// Intermediate operands share their trailing comment owner with the flattened chain.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TailOwner {
    Expression,
    FlattenedChain,
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    statement_start: StatementStart,
    head_comments: HeadComments,
    content_mode: ContentMode,
    tail_owner: TailOwner,
}

impl ExpressionContext {
    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            statement_start: if matches!(position, ExpressionPosition::Statement) {
                StatementStart::Pending
            } else {
                StatementStart::None
            },
            head_comments: HeadComments::Pending,
            content_mode: ContentMode::Formatted,
            tail_owner: TailOwner::Expression,
        }
    }

    pub(super) fn with_statement_start(mut self, start: StatementStart) -> Self {
        self.statement_start = start;
        self
    }

    pub(super) fn with_content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }

    fn with_tail_owner(mut self, owner: TailOwner) -> Self {
        self.tail_owner = owner;
        self
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = expression.span(self.ast);
        if context.content_mode == ContentMode::Verbatim {
            if let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression)
                && !matches!(context.position, ExpressionPosition::Statement)
            {
                let child = parenthesized.expression(self.ast);
                let child_span = child.span(self.ast);
                let boundary = parentheses::SourceParentheses {
                    outer: span,
                    inner: child_span,
                };
                if self.parentheses_are_removable(boundary)
                    && self
                        .expression_needs_parentheses(self.ast.expr_data(child), context.position)
                        == Some(false)
                {
                    return self.format_source_parentheses(
                        parenthesized,
                        context.with_content_mode(ContentMode::Verbatim),
                        writer,
                    );
                }
            }
            self.format_verbatim_expression(expression, context, writer)?;
            return Ok(None);
        }
        if self.diagnostic_line_is_sensitive(span.start)
            && self.line_breaks(span.start, span.end) == 0
        {
            writer.flat(|writer| self.format_expression_content(expression, context, writer))
        } else {
            self.format_expression_content(expression, context, writer)
        }
    }

    fn format_verbatim_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let data = self.ast.expr_data(expression);
        let span = expression.span(self.ast);
        let source = self.source(span);
        let parenthesize = self.expression_needs_parentheses(data, context.position) == Some(true);
        let starts_update = matches!(data, ExprData::UpdateExpression(update) if update.prefix(self.ast))
            && (source.starts_with("++") || source.starts_with("--"));
        if self.options.semicolons == Semicolons::AsNeeded
            && context.statement_start != StatementStart::None
            && (parenthesize
                || !starts_update && source.starts_with(['(', '[', '/', '+', '-', '`']))
        {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
        }
        self.format_verbatim(span, writer)?;
        if parenthesize {
            writer.text(")");
        }
        Ok(())
    }

    fn format_expression_content(
        &mut self,
        expression: Expr,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let data = self.ast.expr_data(expression);
        if let ExprData::ParenthesizedExpression(parenthesized) = data {
            return self.format_source_parentheses(parenthesized, context, writer);
        }
        let binary = BinaryLike::from_data(data, self.ast);
        let parenthesize = self.expression_needs_parentheses(data, context.position) == Some(true);
        if self.options.semicolons != Semicolons::AsNeeded {
            context.statement_start = StatementStart::None;
        }
        if context.statement_start == StatementStart::Pending {
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
                let left = self.expression_content(binary.left).data;
                asi_guard |= self.expression_needs_parentheses(
                    left,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                ) == Some(true);
            }
            if asi_guard {
                context.statement_start = StatementStart::Guard;
            }
        }
        if context.statement_start == StatementStart::Guard && (binary.is_none() || parenthesize) {
            writer.text(";");
            context.statement_start = StatementStart::None;
        }
        if let Some(binary) = binary {
            if parenthesize && matches!(context.position, ExpressionPosition::UnaryArgument(_)) {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = self.format_binary_enclosed(binary, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(None)
                });
            }
            if parenthesize {
                writer.text("(");
            }
            let tail = self.format_binary(binary, context, writer)?;
            if parenthesize {
                writer.text(")");
            }
            return Ok(tail);
        }
        if parenthesize {
            writer.text("(");
        }
        let tail = match data {
            ExprData::StringLiteral(literal) => {
                self.format_string_literal(literal, StringContext::Expression, writer);
                Ok(None)
            }
            ExprData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(None)
            }
            ExprData::BooleanLiteral(literal) => {
                writer.text(if literal.value(self.ast) {
                    "true"
                } else {
                    "false"
                });
                Ok(None)
            }
            ExprData::NullLiteral(_) => {
                writer.text("null");
                Ok(None)
            }
            ExprData::NumericLiteral(literal) => {
                self.format_numeric_literal(literal, writer);
                Ok(None)
            }
            ExprData::BigIntLiteral(literal) => {
                self.format_bigint_literal(literal, writer);
                Ok(None)
            }
            ExprData::RegExpLiteral(literal) => {
                self.format_regexp_literal(literal, writer);
                Ok(None)
            }
            ExprData::ArrayExpression(array) => self.format_array(array, writer).map(|()| None),
            ExprData::UnaryExpression(unary) => self.format_unary(unary, writer),
            ExprData::UpdateExpression(update) => self.format_update(update, writer),
            _ => Err(self.unsupported("expression", expression)),
        }?;
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    /// Unknown grammar keeps source parentheses until its formatting rules are implemented.
    fn expression_needs_parentheses(
        &self,
        data: ExprData,
        position: ExpressionPosition,
    ) -> Option<bool> {
        match data {
            ExprData::SequenceExpression(_) => return Some(true),
            ExprData::IdentifierReference(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ArrayExpression(_)
            | ExprData::UnaryExpression(_)
            | ExprData::UpdateExpression(_)
            | ExprData::BinaryExpression(_)
            | ExprData::LogicalExpression(_)
            | ExprData::ParenthesizedExpression(_) => {}
            _ => return None,
        }
        Some(match (position, data) {
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
        })
    }

    fn format_argument(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expression) => self.format_expression(
                expression,
                ExpressionContext::at(ExpressionPosition::Nested).with_content_mode(mode),
                writer,
            ),
            ArgumentData::SpreadElement(spread) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(spread.span(self.ast), writer)?;
                    return Ok(None);
                }
                let argument = spread.argument(self.ast);
                writer.text("...");
                let comments = self.comments.between(Span::new(
                    spread.span(self.ast).start + 3,
                    argument.span(self.ast).start,
                ))?;
                let comments = self.comments.take(comments.len());
                let mode = self.content_mode(comments);
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
                if let Some(last) = comments.last() {
                    self.comment_gap(last.span.end, argument.span(self.ast).start, writer);
                }
                self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::SpreadArgument)
                        .with_content_mode(mode),
                    writer,
                )
            }
        }
    }
}
