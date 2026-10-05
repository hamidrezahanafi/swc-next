//! Assignment layouts and borrowed operator boundaries prepared before printing.

use swc_next_ecma_ast::{
    AssignmentExpression, Comment, CommentKind, Expr, ExprData, GetSpan, ParenthesizedExpression,
    Span,
};

use super::binary::BinaryLike;
use super::parentheses::ExpressionContent;
use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, SuppressionPosition, is_suppression_comment,
};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum AssignmentParent {
    Statement,
    Nested,
}

#[derive(Clone, Copy)]
enum AssignmentLayout {
    Fluid,
    BreakAfterOperator,
    OperatorLine,
    NeverBreakAfterOperator,
    Chain,
    ChainTail,
}

enum AssignmentRight {
    Expression(Expr),
    SourceParentheses(ParenthesizedExpression),
}

struct AssignmentSuppression {
    gap: Span,
    left: ContentMode,
    right: ContentMode,
    content: AssignmentRight,
}

pub(in crate::formatter) struct AssignmentBoundary<'ast> {
    pub(in crate::formatter) before: &'ast [Comment],
    pub(in crate::formatter) moved: &'ast [Comment],
    pub(in crate::formatter) operator_line: &'ast [Comment],
    pub(in crate::formatter) leading: &'ast [Comment],
    pub(in crate::formatter) left_end: u32,
    pub(in crate::formatter) operator_span: Span,
    pub(in crate::formatter) right_start: u32,
    pub(in crate::formatter) layout_start: u32,
    pub(in crate::formatter) before_operator: LineConstraint,
    pub(in crate::formatter) after_operator: LineConstraint,
    pub(in crate::formatter) own_line: bool,
    pub(in crate::formatter) mode: ContentMode,
}

impl<'ast> AssignmentBoundary<'ast> {
    pub(in crate::formatter) fn right_head(&self) -> NodeHead<'ast> {
        let source_gap = self.own_line
            || self
                .leading
                .iter()
                .any(|comment| comment.kind == CommentKind::Line);
        NodeHead {
            comments: self.leading,
            content_start: self.right_start,
            layout: if source_gap {
                CommentRunLayout::Source
            } else {
                CommentRunLayout::FlatLeading(self.layout_start)
            },
            gap_end: source_gap.then_some(self.layout_start),
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_assignment(
        &mut self,
        assignment: AssignmentExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let target = self.assignment_source_target(assignment)?;
        let right = assignment.right(self.ast);
        let suppression = if self.has_suppression {
            self.prepare_assignment_suppression(
                Span::new(target.span.end, right.span(self.ast).start),
                target.operator_span,
                right,
            )
        } else {
            None
        };
        if let Some(source) = suppression {
            return writer.group(false, |writer| {
                let leading = self.comments.between(Span::new(
                    assignment.span(self.ast).start,
                    target.span.start,
                ))?;
                self.comments.claim(leading)?;
                self.format_comment_run(
                    leading,
                    CommentRunLayout::Leading(target.span.start),
                    writer,
                )?;
                let tail = self.format_source_target(
                    &target,
                    ExpressionContext::at(ExpressionPosition::MemberAssignmentObject)
                        .with_expression_start(context.expression_start)
                        .with_content_mode(source.left),
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                let gap =
                    self.format_delimited_source_gap(source.gap, DelimitedFollowing::Node, writer)?;
                gap.following
                    .write(writer, FlatSeparator::Space, Writer::space);
                let parent = match context.position {
                    ExpressionPosition::Statement => AssignmentParent::Statement,
                    _ => AssignmentParent::Nested,
                };
                self.format_prepared_assignment_right(
                    source.content,
                    ExpressionContext::at(ExpressionPosition::AssignmentRight(parent))
                        .with_content_mode(source.right)
                        .with_tail_owner(context.tail_owner),
                    writer,
                )
            });
        }
        let content = self.expression_content(right);
        let data = content.data;
        let head = self.assignment_expression_head(content);
        let operator = target.operator();
        let boundary = self.assignment_boundary(target.span.end, &head, target.operator_span)?;
        let layout = self.assignment_layout(data, context.position, &boundary);
        let mut write = |writer: &mut Writer<'ast>| {
            let leading = self.comments.between(Span::new(
                assignment.span(self.ast).start,
                target.span.start,
            ))?;
            self.comments.claim(leading)?;
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(target.span.start),
                writer,
            )?;
            let tail = self.format_source_target(
                &target,
                ExpressionContext::at(ExpressionPosition::MemberAssignmentObject)
                    .with_expression_start(context.expression_start),
                writer,
            )?;
            self.format_expression_tail(tail, writer)?;
            self.comments.claim(boundary.before)?;
            self.format_comment_run(
                boundary.before,
                CommentRunLayout::SourceAfter(boundary.left_end),
                writer,
            )?;
            boundary
                .before_operator
                .write(writer, FlatSeparator::Space, Writer::space);
            writer.text(operator);
            self.comments.claim(boundary.operator_line)?;
            self.format_comment_run(
                boundary.operator_line,
                CommentRunLayout::SourceAfter(boundary.operator_span.end),
                writer,
            )?;
            let parent = match context.position {
                ExpressionPosition::Statement => AssignmentParent::Statement,
                _ => AssignmentParent::Nested,
            };
            let mut write_right = |writer: &mut Writer<'ast>| {
                self.comments.claim(boundary.moved)?;
                self.format_comment_run(boundary.moved, CommentRunLayout::Source, writer)?;
                if let Some(last) = boundary.moved.last() {
                    let next = boundary
                        .leading
                        .first()
                        .map_or(boundary.layout_start, |comment| comment.span.start);
                    let before = self.line_breaks(last.span.end, boundary.operator_span.start);
                    let after = self.line_breaks(boundary.operator_span.end, next);
                    // The displaced operator occupied a source line, not an empty line.
                    match before.max(after) {
                        0 => writer.space(),
                        1 => writer.hard_line(),
                        _ => {
                            writer.hard_line();
                            writer.hard_line();
                        }
                    }
                }
                self.comments.claim(boundary.leading)?;
                self.format_expression_with_leading_comments(
                    right,
                    boundary.right_head(),
                    ExpressionContext::at(ExpressionPosition::AssignmentRight(parent))
                        .with_content_mode(boundary.mode)
                        .with_tail_owner(context.tail_owner),
                    writer,
                )
            };
            let separator = |writer: &mut Writer<'ast>| {
                boundary
                    .after_operator
                    .write(writer, FlatSeparator::Space, |writer| {
                        if boundary.own_line {
                            writer.hard_line();
                        } else {
                            writer.line();
                        }
                    });
            };
            match layout {
                AssignmentLayout::Fluid => {
                    let group = writer.named_group(false, |writer| writer.indent(separator));
                    writer.indent_if_break(group, write_right)
                }
                AssignmentLayout::BreakAfterOperator => writer.group(false, |writer| {
                    writer.indent(|writer| {
                        separator(writer);
                        write_right(writer)
                    })
                }),
                AssignmentLayout::OperatorLine | AssignmentLayout::ChainTail => {
                    writer.indent(|writer| {
                        separator(writer);
                        write_right(writer)
                    })
                }
                AssignmentLayout::NeverBreakAfterOperator => {
                    writer.space();
                    write_right(writer)
                }
                AssignmentLayout::Chain => {
                    separator(writer);
                    write_right(writer)
                }
            }
        };
        match layout {
            AssignmentLayout::Chain | AssignmentLayout::ChainTail => write(writer),
            _ => writer.group(false, write),
        }
    }

    fn prepare_assignment_suppression(
        &self,
        gap: Span,
        operator: Span,
        right: Expr,
    ) -> Option<AssignmentSuppression> {
        let mut inner = right;
        while let ExprData::ParenthesizedExpression(node) = self.ast.expr_data(inner) {
            inner = node.expression(self.ast);
        }
        let header_end = inner.span(self.ast).start;
        let comments = self.comments.in_range(Span::new(gap.start, header_end));
        if self.content_mode(comments) != ContentMode::Verbatim {
            return None;
        }
        let complex = matches!(
            self.ast.expr_data(inner),
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
        );
        let parentheses = match self.ast.expr_data(right) {
            ExprData::ParenthesizedExpression(node)
                if comments.iter().any(|&comment| {
                    comment.span.start >= gap.end && is_suppression_comment(self.ast, comment)
                }) =>
            {
                AssignmentRight::SourceParentheses(node)
            }
            _ => AssignmentRight::Expression(right),
        };
        let mut source = AssignmentSuppression {
            gap,
            left: ContentMode::Formatted,
            right: ContentMode::Formatted,
            content: parentheses,
        };
        for (run, position) in self.suppressed_comment_runs(comments, header_end) {
            for &comment in run {
                if !is_suppression_comment(self.ast, comment) {
                    continue;
                }
                let left = match position {
                    SuppressionPosition::OwnLine => false,
                    SuppressionPosition::EndOfLine => comment.kind == CommentKind::Line && !complex,
                    SuppressionPosition::Inline => comment.span.end <= operator.start,
                };
                if left {
                    source.left = ContentMode::Verbatim;
                } else {
                    source.right = ContentMode::Verbatim;
                }
            }
        }
        Some(source)
    }

    fn format_prepared_assignment_right(
        &mut self,
        content: AssignmentRight,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let parenthesized = match content {
            AssignmentRight::Expression(expression) => {
                return self.format_expression(expression, context, writer);
            }
            AssignmentRight::SourceParentheses(node) => node,
        };
        let child = parenthesized.expression(self.ast);
        let content = match self.ast.expr_data(child) {
            ExprData::ParenthesizedExpression(node) => AssignmentRight::SourceParentheses(node),
            _ => AssignmentRight::Expression(child),
        };
        self.format_assignment_right_parentheses(
            parenthesized,
            context,
            writer,
            |formatter, context, writer| {
                formatter.format_prepared_assignment_right(content, context, writer)
            },
        )
    }

    fn assignment_expression_head(&self, mut content: ExpressionContent) -> ExpressionContent {
        loop {
            let expression = match content.data {
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                _ => return content,
            };
            content = self.expression_content(expression);
        }
    }

    pub(in crate::formatter) fn assignment_boundary(
        &self,
        left_end: u32,
        content: &ExpressionContent,
        operator_span: Span,
    ) -> Result<AssignmentBoundary<'ast>, FormatError> {
        let right_start = content.expression.span(self.ast).start;
        let span = Span::new(left_end, right_start);
        let comments = self.comments.in_range(span);
        let mut boundary = AssignmentBoundary {
            before: &[],
            moved: &[],
            operator_line: &[],
            leading: &[],
            left_end,
            operator_span,
            right_start,
            layout_start: right_start,
            before_operator: LineConstraint::Unconstrained,
            after_operator: LineConstraint::Unconstrained,
            own_line: false,
            mode: ContentMode::Formatted,
        };
        if let ExprData::ParenthesizedExpression(parenthesized) = content.data
            && let Some(child) =
                self.ignored_parentheses_child(parenthesized, ContentMode::Formatted)
            && self
                .expression_needs_parentheses(
                    self.ast.expr_data(child),
                    ExpressionPosition::Nested,
                    ContentMode::Verbatim,
                )
                .is_some()
        {
            let head = self.comments.in_range(Span::new(
                parenthesized.span(self.ast).start,
                child.span(self.ast).start,
            ));
            boundary.own_line = head.iter().any(|comment| comment.kind == CommentKind::Line);
        }
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(boundary);
        }
        let operator_start = operator_span.start;
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = comments.split_at(before_count);
        let hugs_comments = matches!(
            content.data,
            ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
        );
        let before_line = before
            .last()
            .is_some_and(|last| last.kind == CommentKind::Line);
        let promoted = !sensitive
            && before.first().is_some_and(|first| {
                hugs_comments && before_line
                    || self.line_breaks(left_end, first.span.start) != 0
                    || before.last().is_some_and(|last| {
                        self.line_breaks(last.span.start, last.span.end) != 0
                            && self.line_breaks(last.span.end, operator_start) != 0
                    })
            });
        if promoted {
            boundary.moved = before;
        } else {
            boundary.before = before;
        }
        let mut line_count = 0;
        if !promoted && !before_line && !hugs_comments {
            let mut end = boundary.operator_span.end;
            for (index, comment) in after.iter().enumerate() {
                if self.line_breaks(end, comment.span.start) != 0 {
                    break;
                }
                if comment.kind == CommentKind::Line {
                    line_count = index + 1;
                    break;
                }
                end = comment.span.end;
            }
        }
        let (operator_line, leading) = after.split_at(line_count);
        boundary.operator_line = operator_line;
        boundary.leading = leading;
        if let Some(parentheses) = content.parentheses
            && leading
                .last()
                .is_none_or(|comment| comment.span.end <= parentheses.outer.start)
        {
            // A delimiter before the head owns its inner source line breaks.
            boundary.layout_start = parentheses.outer.start;
        }
        boundary.mode = if promoted && self.content_mode(before) == ContentMode::Verbatim {
            ContentMode::Verbatim
        } else {
            self.content_mode(after)
        };
        boundary.own_line |= promoted;
        boundary.own_line |= match leading.first() {
            Some(&first) => {
                promoted
                    || before_line
                    || first.kind == CommentKind::Line
                    || self.line_breaks(boundary.operator_span.end, first.span.start) != 0
                    || self.comment_is_alignable(first)?
            }
            None => false,
        };
        if sensitive {
            let previous = before.last().map_or(left_end, |last| last.span.end);
            let next = after.first().map_or(right_start, |first| first.span.start);
            boundary.before_operator = self.line_constraint(previous, operator_start);
            boundary.after_operator = self.line_constraint(boundary.operator_span.end, next);
        }
        if let Some((last, previous)) = boundary.before.split_last()
            && last.kind == CommentKind::Line
            && self.line_breaks(
                previous.last().map_or(left_end, |comment| comment.span.end),
                last.span.start,
            ) != 0
            && boundary.before_operator == LineConstraint::Unconstrained
        {
            // Own-line comments are emitted directly, unlike deferred line suffixes.
            boundary.before_operator = LineConstraint::NewLine(1);
        }
        Ok(boundary)
    }

    fn assignment_layout(
        &self,
        data: ExprData,
        position: ExpressionPosition,
        boundary: &AssignmentBoundary<'ast>,
    ) -> AssignmentLayout {
        let tail = !matches!(data, ExprData::AssignmentExpression(_));
        if let ExpressionPosition::AssignmentRight(parent) = position
            && (!tail || matches!(parent, AssignmentParent::Nested))
        {
            return if tail {
                AssignmentLayout::ChainTail
            } else {
                AssignmentLayout::Chain
            };
        }
        if !boundary.operator_line.is_empty()
            || boundary
                .before
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
        {
            return AssignmentLayout::OperatorLine;
        }
        if boundary.own_line || self.assignment_right_breaks_after_operator(data) {
            return AssignmentLayout::BreakAfterOperator;
        }
        if matches!(
            data,
            ExprData::BooleanLiteral(_) | ExprData::NumericLiteral(_)
        ) {
            return AssignmentLayout::NeverBreakAfterOperator;
        }
        AssignmentLayout::Fluid
    }

    pub(in crate::formatter) fn assignment_right_breaks_after_operator(
        &self,
        mut data: ExprData,
    ) -> bool {
        match data {
            ExprData::AssignmentExpression(assignment) => {
                return matches!(
                    self.expression_content(assignment.right(self.ast)).data,
                    ExprData::AssignmentExpression(_)
                );
            }
            ExprData::ConditionalExpression(conditional) => {
                let test = self.expression_content(conditional.test(self.ast)).data;
                return BinaryLike::from_data(test, self.ast).is_some_and(|binary| {
                    !self.inline_logical_operand(binary.operator, binary.right)
                });
            }
            _ => {}
        }
        if let Some(binary) = BinaryLike::from_data(data, self.ast) {
            return !self.inline_logical_operand(binary.operator, binary.right);
        }
        while let ExprData::UnaryExpression(unary) = data {
            data = self.expression_content(unary.argument(self.ast)).data;
        }
        matches!(
            data,
            ExprData::StringLiteral(_) | ExprData::SequenceExpression(_)
        )
    }
}
