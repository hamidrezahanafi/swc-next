//! Expression dispatch and immediate grammar and statement-start contexts.

pub(super) mod arguments;
mod arrays;
pub(super) mod assignment;
mod binary;
mod calls;
mod conditional;
mod constructors;
mod members;
mod objects;
mod parentheses;
mod sequence;
mod targets;
mod unary;

use swc_next_ecma_ast::{
    Argument, ArgumentData, Expr, ExprData, GetSpan, Span, UnaryOperator, UpdateOperator,
};

use self::assignment::AssignmentParent;
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
    PropertyValue,
    MemberObject,
    CallCallee,
    NewCallee,
    MemberAssignmentObject,
    ConditionalTest,
    ConditionalConsequent,
    ConditionalAlternate,
    AssignmentRight(AssignmentParent),
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

/// A surrounding operation may own the operand's trailing comment boundary.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TailOwner {
    Expression,
    ParentOperation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParenthesesOwner {
    Expression,
    Parent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemberLayout {
    Normal,
    Tight,
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    statement_start: StatementStart,
    head_comments: HeadComments,
    content_mode: ContentMode,
    tail_owner: TailOwner,
    parentheses_owner: ParenthesesOwner,
    member_layout: MemberLayout,
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
            parentheses_owner: ParenthesesOwner::Expression,
            member_layout: MemberLayout::Normal,
        }
    }

    pub(super) fn with_statement_start(mut self, start: StatementStart) -> Self {
        self.statement_start = start;
        self
    }

    fn inside_parentheses(mut self) -> Self {
        self.parentheses_owner = ParenthesesOwner::Parent;
        self.statement_start = StatementStart::None;
        self
    }

    pub(super) fn with_content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }

    fn with_member_layout(mut self, layout: MemberLayout) -> Self {
        self.member_layout = layout;
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
                    && self.expression_needs_parentheses(
                        self.ast.expr_data(child),
                        context.position,
                        ContentMode::Verbatim,
                    ) == Some(false)
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
        let parenthesize = context.parentheses_owner == ParenthesesOwner::Expression
            && (self.expression_needs_parentheses(data, context.position, context.content_mode)
                == Some(true)
                || matches!(data, ExprData::ObjectExpression(_))
                    && context.statement_start != StatementStart::None);
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
        if let ExprData::SequenceExpression(sequence) = data {
            return self.format_sequence(sequence, context, writer);
        }
        let binary = BinaryLike::from_data(data, self.ast);
        let parenthesize = context.parentheses_owner == ParenthesesOwner::Expression
            && (self.expression_needs_parentheses(data, context.position, context.content_mode)
                == Some(true)
                || matches!(data, ExprData::ObjectExpression(_))
                    && context.statement_start != StatementStart::None);
        if context.statement_start == StatementStart::Pending
            && self.options.semicolons == Semicolons::AsNeeded
        {
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
                    ContentMode::Formatted,
                ) == Some(true);
            }
            if asi_guard {
                context.statement_start = StatementStart::Guard;
            }
        }
        let conditional = matches!(data, ExprData::ConditionalExpression(_));
        if context.statement_start == StatementStart::Guard
            && (binary.is_none() && !conditional || parenthesize)
        {
            writer.text(";");
            context.statement_start = StatementStart::None;
        }
        if let Some(binary) = binary {
            if parenthesize
                && matches!(
                    context.position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::MemberObject
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee
                        | ExpressionPosition::NewCallee
                )
            {
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
        if let ExprData::ConditionalExpression(conditional) = data
            && parenthesize
            && matches!(
                context.position,
                ExpressionPosition::MemberObject
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::CallCallee
                    | ExpressionPosition::NewCallee
            )
        {
            return writer.group(false, |writer| {
                writer.text("(");
                let tail = self.format_conditional(conditional, context, writer)?;
                self.format_expression_tail(tail, writer)?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && matches!(context.position, ExpressionPosition::ConditionalTest)
        {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = self.format_conditional(conditional, context, writer)?;
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
        let tail = match data {
            ExprData::StringLiteral(literal) => {
                self.format_string_literal(literal, StringContext::Expression, writer);
                Ok(None)
            }
            ExprData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(None)
            }
            ExprData::ThisExpression(_) => {
                writer.text("this");
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
            ExprData::ObjectExpression(object) => self.format_object(object, writer).map(|()| None),
            ExprData::MemberExpression(member) => self.format_member(member, context, writer),
            ExprData::CallExpression(call) => self.format_call(call, context, writer),
            ExprData::NewExpression(constructor) => self.format_new(constructor, writer),
            ExprData::UnaryExpression(unary) => self.format_unary(unary, writer),
            ExprData::UpdateExpression(update) => self.format_update(update, context, writer),
            ExprData::ConditionalExpression(conditional) => {
                self.format_conditional(conditional, context, writer)
            }
            ExprData::AssignmentExpression(assignment) => {
                self.format_assignment(assignment, context, writer)
            }
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
        mode: ContentMode,
    ) -> Option<bool> {
        match data {
            ExprData::SequenceExpression(_) => return Some(true),
            ExprData::NewExpression(constructor) => {
                if mode == ContentMode::Formatted
                    || !matches!(
                        position,
                        ExpressionPosition::CallCallee
                            | ExpressionPosition::MemberObject
                            | ExpressionPosition::MemberAssignmentObject
                            | ExpressionPosition::NewCallee
                    )
                {
                    return Some(false);
                }
                let suffix_end = constructor.type_arguments(self.ast).map_or_else(
                    || constructor.callee(self.ast).span(self.ast).end,
                    |arguments| arguments.span(self.ast).end,
                );
                return Some(constructor.span(self.ast).end == suffix_end);
            }
            ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ArrayExpression(_)
            | ExprData::ObjectExpression(_)
            | ExprData::MemberExpression(_)
            | ExprData::CallExpression(_)
            | ExprData::UnaryExpression(_)
            | ExprData::UpdateExpression(_)
            | ExprData::BinaryExpression(_)
            | ExprData::LogicalExpression(_)
            | ExprData::ConditionalExpression(_)
            | ExprData::AssignmentExpression(_)
            | ExprData::ParenthesizedExpression(_) => {}
            _ => return None,
        }
        if matches!(position, ExpressionPosition::NewCallee)
            && self.new_callee_contains_call(data, mode)
        {
            return Some(true);
        }
        Some(match (position, data) {
            (
                ExpressionPosition::MemberObject | ExpressionPosition::MemberAssignmentObject,
                ExprData::NumericLiteral(_)
                | ExprData::UnaryExpression(_)
                | ExprData::UpdateExpression(_)
                | ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ConditionalExpression(_),
            ) => true,
            (
                ExpressionPosition::CallCallee | ExpressionPosition::NewCallee,
                ExprData::UnaryExpression(_)
                | ExprData::UpdateExpression(_)
                | ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ConditionalExpression(_),
            ) => true,
            (position, ExprData::AssignmentExpression(_)) => !matches!(
                position,
                ExpressionPosition::Statement | ExpressionPosition::AssignmentRight(_)
            ),
            (
                ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::ParenthesizedUnaryArgument(_)
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::BinaryOperand { .. }
                | ExpressionPosition::ConditionalTest,
                ExprData::ConditionalExpression(_),
            ) => true,
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
            (
                ExpressionPosition::Statement,
                ExprData::StringLiteral(_) | ExprData::ObjectExpression(_),
            ) => true,
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
