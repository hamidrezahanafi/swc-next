//! Assignment layouts and borrowed operator boundaries prepared before printing.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentTargetData, Comment, CommentKind, ExprData, GetSpan,
    SimpleAssignmentTargetData, Span,
};

use super::binary::BinaryLike;
use super::parentheses::ExpressionContent;
use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
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

struct AssignmentBoundary<'ast> {
    before: &'ast [Comment],
    moved: &'ast [Comment],
    operator_line: &'ast [Comment],
    leading: &'ast [Comment],
    left_end: u32,
    operator_span: Span,
    right_start: u32,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
    own_line: bool,
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_assignment(
        &mut self,
        assignment: AssignmentExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let left = assignment.left(self.ast);
        let AssignmentTargetData::SimpleAssignmentTarget(left) =
            self.ast.assignment_target_data(left)
        else {
            return Err(self.unsupported("assignment target", left));
        };
        let SimpleAssignmentTargetData::IdentifierReference(identifier) =
            self.ast.simple_assignment_target_data(left)
        else {
            return Err(self.unsupported("assignment target", left));
        };
        let right = assignment.right(self.ast);
        let content = self.expression_content(right);
        let operator = assignment.operator(self.ast).as_str();
        let boundary = self.assignment_boundary(left.span(self.ast).end, &content, operator)?;
        let layout = self.assignment_layout(content.data, context.position, &boundary);
        let mut write = |writer: &mut Writer<'ast>| {
            let leading = self.comments.between(Span::new(
                assignment.span(self.ast).start,
                identifier.span(self.ast).start,
            ))?;
            self.comments.claim(leading)?;
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(identifier.span(self.ast).start),
                writer,
            )?;
            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
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
                        .map_or(boundary.right_start, |comment| comment.span.start);
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
                let source_gap = boundary.own_line
                    || boundary
                        .leading
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line);
                if source_gap {
                    self.format_comment_run(boundary.leading, CommentRunLayout::Source, writer)?;
                    if let Some(last) = boundary.leading.last() {
                        self.comment_gap(last.span.end, boundary.right_start, writer);
                    }
                } else {
                    self.format_comment_run(
                        boundary.leading,
                        CommentRunLayout::FlatLeading(boundary.right_start),
                        writer,
                    )?;
                }
                self.format_expression(
                    right,
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

    fn assignment_boundary(
        &self,
        left_end: u32,
        content: &ExpressionContent,
        operator: &'static str,
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
            operator_span: Span::new(left_end, left_end),
            right_start,
            before_operator: LineConstraint::Unconstrained,
            after_operator: LineConstraint::Unconstrained,
            own_line: false,
            mode: ContentMode::Formatted,
        };
        if let ExprData::ParenthesizedExpression(parenthesized) = content.data
            && let Some(child) =
                self.ignored_parentheses_child(parenthesized, ContentMode::Formatted)
            && self
                .expression_needs_parentheses(self.ast.expr_data(child), ExpressionPosition::Nested)
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
        let Some(operator_start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "assignment operator boundary",
                None,
                span,
            ));
        };
        boundary.operator_span = Span::new(operator_start, operator_start + operator.len() as u32);
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = comments.split_at(before_count);
        let hugs_comments = matches!(content.data, ExprData::ArrayExpression(_));
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

    fn assignment_right_breaks_after_operator(&self, mut data: ExprData) -> bool {
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
