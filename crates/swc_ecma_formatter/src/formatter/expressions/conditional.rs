//! Conditional positions, aligned branches, and prepared operator boundaries.

use swc_next_ecma_ast::{Comment, ConditionalExpression, Expr, ExprData, GetSpan, Span};

use super::parentheses::ExpressionContent;
use super::{ExpressionContext, ExpressionPosition, Formatter, OperandIndent, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, IndentStyle, document::Writer};

/// Continuation responsibility follows only an operation's actual head edge.
/// A direct root has no extra enclosure; a head borrows that root's scope.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionalEnvelope {
    Local,
    ContinuationRoot,
    OperationHead,
}

impl ConditionalEnvelope {
    pub(super) fn chain_head(self) -> Self {
        match self {
            Self::Local => Self::Local,
            Self::ContinuationRoot | Self::OperationHead => Self::OperationHead,
        }
    }

    pub(super) fn terminal_head(self) -> Self {
        match self {
            Self::ContinuationRoot => Self::OperationHead,
            Self::Local | Self::OperationHead => Self::Local,
        }
    }
}

/// The immediate suffix owns the closing pair's optional break.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionalClosing {
    Inline,
    MemberLine,
}

#[derive(Clone, Copy)]
enum ConditionalLayout {
    Root,
    Test,
    Consequent,
    Alternate,
}

struct ConditionalOperand {
    original: Expr,
    content: ExpressionContent,
    source_end: u32,
    content_start: u32,
}

struct PreparedConditional<'ast> {
    test: ConditionalOperand,
    consequent: ConditionalOperand,
    alternate: ConditionalOperand,
    question: ConditionalBoundary<'ast>,
    colon: ConditionalBoundary<'ast>,
}

struct ConditionalBoundary<'ast> {
    operator: &'static str,
    source_end: u32,
    comments: &'ast [Comment],
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    previous_mode: ContentMode,
    next_mode: ContentMode,
    before_operator: LineConstraint,
    after_operator: LineConstraint,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_conditional_enclosed(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let continuation = context.conditional_envelope == ConditionalEnvelope::OperationHead;
        let closing_line =
            continuation || context.conditional_closing == ConditionalClosing::MemberLine;
        writer.group(false, |writer| {
            writer.text("(");
            let mut write = |writer: &mut Writer<'ast>| {
                let context = if continuation {
                    context.with_operand_indent(OperandIndent::Parent)
                } else {
                    context
                };
                let tail = self.format_conditional(conditional, context, writer)?;
                self.format_expression_tail(tail, writer)
            };
            if continuation {
                writer.indent(|writer| {
                    writer.soft_line();
                    write(writer)
                })?;
            } else {
                write(writer)?;
            }
            if closing_line {
                writer.soft_line();
            }
            writer.text(")");
            Ok(None)
        })
    }

    pub(super) fn format_conditional(
        &mut self,
        conditional: ConditionalExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = match context.position {
            ExpressionPosition::ConditionalTest => ConditionalLayout::Test,
            ExpressionPosition::ConditionalConsequent => ConditionalLayout::Consequent,
            ExpressionPosition::ConditionalAlternate => ConditionalLayout::Alternate,
            _ => ConditionalLayout::Root,
        };
        let prepared = self.prepare_conditional(conditional)?;
        let write = |writer: &mut Writer<'ast>| {
            self.format_conditional_contents(prepared, context, layout, writer)
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.group(false, write),
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write(writer),
        }
    }

    fn prepare_conditional(
        &self,
        conditional: ConditionalExpression,
    ) -> Result<PreparedConditional<'ast>, FormatError> {
        let test = self.conditional_operand(conditional.test(self.ast));
        let consequent = self.conditional_operand(conditional.consequent(self.ast));
        let alternate = self.conditional_operand(conditional.alternate(self.ast));
        let question = self.conditional_boundary(&test, &consequent, "?")?;
        let colon = self.conditional_boundary(&consequent, &alternate, ":")?;
        Ok(PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        })
    }

    fn conditional_operand(&self, original: Expr) -> ConditionalOperand {
        let content = self.expression_content(original);
        ConditionalOperand {
            original,
            source_end: original.span(self.ast).end,
            content_start: content.expression.span(self.ast).start,
            content,
        }
    }

    fn format_conditional_contents(
        &mut self,
        prepared: PreparedConditional<'ast>,
        context: ExpressionContext,
        layout: ConditionalLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedConditional {
            test,
            consequent,
            alternate,
            question,
            colon,
        } = prepared;
        let write_test = |writer: &mut Writer<'ast>| {
            let tail = self.format_prepared_expression_with_leading_comments(
                test.original,
                test.content,
                NodeHead {
                    comments: &[],
                    content_start: test.content_start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                ExpressionContext::at(ExpressionPosition::ConditionalTest)
                    .with_expression_start(context.expression_start)
                    .with_operand_indent(context.operand_indent)
                    .with_content_mode(question.previous_mode),
                writer,
            )?;
            self.format_conditional_trailing_comments(&question, tail, writer)
        };
        if matches!(layout, ConditionalLayout::Alternate) {
            writer.align(2, write_test)?;
        } else {
            write_test(writer)?;
        }
        let options = self.options;
        let write_branches = |writer: &mut Writer<'ast>| {
            self.format_conditional_branch(
                consequent,
                ExpressionPosition::ConditionalConsequent,
                question,
                Some(&colon),
                writer,
            )?;
            self.format_conditional_branch(
                alternate,
                ExpressionPosition::ConditionalAlternate,
                colon,
                None,
                writer,
            )
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.indent(write_branches),
            ConditionalLayout::Consequent if options.indent_style == IndentStyle::Space => {
                let width = usize::from(options.indent_width.value()).saturating_sub(2);
                writer.align(width, write_branches)
            }
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write_branches(writer),
        }
    }

    fn conditional_boundary(
        &self,
        previous: &ConditionalOperand,
        next: &ConditionalOperand,
        operator: &'static str,
    ) -> Result<ConditionalBoundary<'ast>, FormatError> {
        let start = previous.source_end;
        let end = next.content_start;
        let span = Span::new(start, end);
        let comments = self.comments.in_range(span);
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(ConditionalBoundary {
                operator,
                source_end: start,
                comments,
                trailing: &[],
                leading: &[],
                previous_mode: ContentMode::Formatted,
                next_mode: ContentMode::Formatted,
                before_operator: LineConstraint::Unconstrained,
                after_operator: LineConstraint::Unconstrained,
            });
        }
        let Some(operator_start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "conditional operator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        if sensitive {
            let (before, after) = comments.split_at(before_count);
            let previous = before.last().map_or(start, |comment| comment.span.end);
            return Ok(ConditionalBoundary {
                operator,
                source_end: start,
                comments,
                trailing: before,
                leading: after,
                previous_mode: self.content_mode(before),
                next_mode: self.content_mode(after),
                before_operator: self.line_constraint(previous, operator_start),
                after_operator: self.line_constraint(operator_start + 1, end),
            });
        }
        let (trailing, leading) = self.split_separator_comments(comments, before_count, start, end);
        Ok(ConditionalBoundary {
            operator,
            source_end: start,
            comments,
            trailing,
            leading,
            previous_mode: self.content_mode(trailing),
            next_mode: self.content_mode(leading),
            before_operator: LineConstraint::Unconstrained,
            after_operator: LineConstraint::Unconstrained,
        })
    }

    fn format_conditional_trailing_comments(
        &mut self,
        boundary: &ConditionalBoundary<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_expression_tail(tail, writer)?;
        self.comments.claim(boundary.comments)?;
        self.format_comment_run(
            boundary.trailing,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )
    }

    fn format_conditional_branch(
        &mut self,
        operand: ConditionalOperand,
        position: ExpressionPosition,
        boundary: ConditionalBoundary<'ast>,
        following: Option<&ConditionalBoundary<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        boundary
            .before_operator
            .write(writer, FlatSeparator::Space, Writer::line);
        writer.text(boundary.operator);
        writer.space();
        let ConditionalOperand {
            original,
            content,
            content_start,
            ..
        } = operand;
        let mode = match following {
            Some(next) if next.previous_mode == ContentMode::Verbatim => ContentMode::Verbatim,
            _ => boundary.next_mode,
        };
        let nested_consequent = matches!(position, ExpressionPosition::ConditionalConsequent)
            && matches!(content.data, ExprData::ConditionalExpression(_))
            && mode == ContentMode::Formatted;
        if nested_consequent {
            writer.if_flat("(");
        }
        let indent_style = self.options.indent_style;
        let write = |writer: &mut Writer<'ast>| {
            if boundary.leading.is_empty() {
                boundary
                    .after_operator
                    .write(writer, FlatSeparator::Empty, |_| {});
            }
            let mut tail = self.format_prepared_expression_with_leading_comments(
                original,
                content,
                NodeHead {
                    comments: boundary.leading,
                    content_start,
                    layout: CommentRunLayout::Source,
                    gap_end: Some(content_start),
                },
                ExpressionContext::at(position)
                    .with_operand_indent(OperandIndent::Parent)
                    .with_content_mode(mode)
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )?;
            if nested_consequent {
                self.format_expression_tail(tail.take(), writer)?;
                writer.if_flat(")");
            }
            if let Some(next) = following {
                self.format_conditional_trailing_comments(next, tail, writer)?;
                Ok(None)
            } else {
                Ok(tail)
            }
        };
        match indent_style {
            IndentStyle::Space => writer.align(2, write),
            IndentStyle::Tab => writer.indent(write),
        }
    }
}
