//! Typed comma-expression traversal and borrowed separator boundaries.

use swc_next_ecma_ast::{Comment, Expr, ExprData, GetSpan, SequenceExpression, Span};

use super::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::{ExpressionContext, ExpressionPosition, Formatter, ParenthesesOwner, TailOwner};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, DelimitedSuppression};
use crate::{FormatError, Semicolons, document::Writer};

struct SequenceBoundary<'ast> {
    next: Expr,
    next_start: u32,
    source_end: u32,
    comments: &'ast [Comment],
    comma: Option<u32>,
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    suppression: Option<DelimitedSuppression>,
    before_comma: LineConstraint,
    separator: LineConstraint,
}

struct SequenceItem<'ast> {
    expression: Expr,
    leading: &'ast [Comment],
    start: u32,
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_sequence(
        &mut self,
        sequence: SequenceExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let mut expressions = sequence
            .expressions(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = expressions.next() else {
            return Err(self.unsupported("empty sequence expression", sequence));
        };
        let first_content = self.expression_content(first);
        let first_start = first_content.expression.span(ast).start;
        let leading = self
            .comments
            .between(Span::new(sequence.span(ast).start, first_start))?;
        self.comments.claim(leading)?;
        let head = NodeHead {
            comments: leading,
            content_start: first_start,
            layout: CommentRunLayout::Leading(first_start),
            gap_end: None,
        };
        let parenthesize = context.parentheses_owner == ParenthesesOwner::Expression
            && !matches!(
                context.position,
                ExpressionPosition::RestrictedArgument
                    | ExpressionPosition::ForHead
                    | ExpressionPosition::TemplateInterpolation
                    | ExpressionPosition::ForInitializer { parentheses: false }
            );
        if parenthesize && !leading.is_empty() {
            let binding = self.expression_head_binding(leading, first);
            let binding = first_content
                .parentheses
                .map_or(binding, |removed| binding.combine(removed.head.binding));
            if !matches!(binding, CommentBinding::Unbound) {
                return self.format_prepared_expression_with_resolved_head(
                    Expr::SequenceExpression(sequence),
                    ExpressionContent {
                        expression: Expr::SequenceExpression(sequence),
                        data: ExprData::SequenceExpression(sequence),
                        parentheses: None,
                    },
                    ResolvedExpressionHead { head, binding },
                    context,
                    writer,
                );
            }
        }
        let generated = if parenthesize && self.has_type_casts && !leading.is_empty() {
            let content = ExpressionContent {
                expression: Expr::SequenceExpression(sequence),
                data: ExprData::SequenceExpression(sequence),
                parentheses: None,
            };
            self.generated_enclosure_owner(
                Expr::SequenceExpression(sequence),
                &content,
                &head,
                CommentBinding::Unbound,
                context,
            )
            .map(|owner| super::parentheses::PreparedGeneratedEnclosure { owner, content })
        } else {
            None
        };
        let start = self.format_node_head(head, context.expression_start, writer)?;
        if let Some(enclosure) = generated {
            return self.format_prepared_generated_enclosure(
                enclosure,
                context.with_expression_start(start),
                writer,
            );
        }
        if self.options.semicolons == Semicolons::AsNeeded && start.needs_asi_guard() {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
        }
        let arrow_body = matches!(context.position, ExpressionPosition::ArrowBody) && parenthesize;
        let tail = writer.group(false, |writer| {
            let mut write_items = |writer: &mut Writer<'ast>| {
                if arrow_body {
                    writer.soft_line();
                }
                let boundary = expressions
                    .next()
                    .map(|next| self.sequence_boundary(first, next))
                    .transpose()?;
                let tail = self.format_sequence_item(
                    SequenceItem {
                        expression: first,
                        leading: &[],
                        start: first_start,
                        mode: ContentMode::Formatted,
                    },
                    boundary.as_ref(),
                    context.tail_owner,
                    writer,
                )?;
                let Some(mut boundary) = boundary else {
                    return Ok(tail);
                };
                self.format_sequence_trailing(&mut boundary, tail, writer)?;
                let mut write_rest = |writer: &mut Writer<'ast>| loop {
                    boundary
                        .separator
                        .write(writer, FlatSeparator::Space, Writer::line);
                    let following = expressions
                        .next()
                        .map(|next| self.sequence_boundary(boundary.next, next))
                        .transpose()?;
                    let tail = self.format_sequence_item(
                        SequenceItem {
                            expression: boundary.next,
                            leading: boundary.leading,
                            start: boundary.next_start,
                            mode: boundary.suppression.as_ref().map_or_else(
                                || self.content_mode(boundary.leading),
                                |source| source.following,
                            ),
                        },
                        following.as_ref(),
                        context.tail_owner,
                        writer,
                    )?;
                    let Some(mut following) = following else {
                        break Ok(tail);
                    };
                    self.format_sequence_trailing(&mut following, tail, writer)?;
                    boundary = following;
                };
                if matches!(context.position, ExpressionPosition::Statement) {
                    writer.indent(write_rest)
                } else {
                    write_rest(writer)
                }
            };
            let tail = if arrow_body {
                writer.indent(write_items)?
            } else {
                write_items(writer)?
            };
            if arrow_body {
                writer.soft_line();
            }
            Ok(tail)
        })?;
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    fn format_sequence_item(
        &mut self,
        item: SequenceItem<'ast>,
        following: Option<&SequenceBoundary<'ast>>,
        tail_owner: TailOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let SequenceItem {
            expression,
            leading,
            start,
            mode,
        } = item;
        let mode = if mode == ContentMode::Verbatim {
            mode
        } else {
            following.map_or(ContentMode::Formatted, |boundary| {
                if boundary
                    .suppression
                    .as_ref()
                    .and_then(|source| source.preceding)
                    .is_some_and(|target| target == expression.span(self.ast))
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                }
            })
        };
        let tail_owner = if following.is_some() {
            TailOwner::ParentOperation
        } else {
            tail_owner
        };
        self.comments.claim(leading)?;
        self.format_expression_with_leading_comments(
            expression,
            NodeHead {
                comments: leading,
                content_start: start,
                layout: CommentRunLayout::Leading(start),
                gap_end: None,
            },
            ExpressionContext::at(ExpressionPosition::Nested)
                .with_content_mode(mode)
                .with_tail_owner(tail_owner),
            writer,
        )
    }

    fn sequence_boundary(
        &self,
        previous: Expr,
        next: Expr,
    ) -> Result<SequenceBoundary<'ast>, FormatError> {
        let start = previous.span(self.ast).end;
        let end = self
            .expression_content(next)
            .expression
            .span(self.ast)
            .start;
        let span = Span::new(start, end);
        // Retain the borrowed run while the previous operand consumes its own
        // comments. Its returned tail determines the remaining suffix.
        let comments = self
            .comments
            .in_range(Span::new(previous.span(self.ast).start, end));
        let comma = if comments.is_empty() && !self.diagnostic_range_is_sensitive(span) {
            None
        } else {
            let outside = comments.partition_point(|comment| comment.span.end <= start);
            let Some(comma) = self.find_comma(span, &comments[outside..]) else {
                return Err(FormatError::unsupported(
                    "sequence separator boundary",
                    None,
                    span,
                ));
            };
            Some(comma)
        };
        Ok(SequenceBoundary {
            next,
            next_start: end,
            source_end: start,
            comments,
            comma,
            trailing: &[],
            leading: &[],
            suppression: self.delimited_suppression(
                previous.span(self.ast),
                end,
                DelimitedFollowing::Node,
            ),
            before_comma: LineConstraint::Unconstrained,
            separator: LineConstraint::Unconstrained,
        })
    }

    fn prepare_sequence_comments(
        &self,
        boundary: &mut SequenceBoundary<'ast>,
        span: Span,
    ) -> Result<(), FormatError> {
        let count = boundary
            .comments
            .partition_point(|comment| comment.span.end <= span.start);
        let comments = &boundary.comments[count..];
        let start = span.start;
        let end = span.end;
        let sensitive = self.diagnostic_range_is_sensitive(span);
        if comments.is_empty() && !sensitive {
            return Ok(());
        }
        let Some(comma) = boundary.comma else {
            return Err(FormatError::unsupported(
                "sequence separator boundary",
                None,
                span,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= comma);
        (boundary.trailing, boundary.leading) = if sensitive {
            comments.split_at(before_count)
        } else {
            self.split_separator_comments(comments, before_count, start, Some(end))
        };
        if sensitive {
            let last = boundary
                .trailing
                .last()
                .map_or(start, |comment| comment.span.end);
            let next = boundary
                .leading
                .first()
                .map_or(end, |comment| comment.span.start);
            boundary.before_comma = self.line_constraint(last, comma);
            boundary.separator = self.line_constraint(comma + 1, next);
        }
        Ok(())
    }

    fn format_sequence_trailing(
        &mut self,
        boundary: &mut SequenceBoundary<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(tail) = tail {
            boundary.source_end = tail.source_end;
        }
        let gap = Span::new(boundary.source_end, boundary.next_start);
        if boundary
            .suppression
            .as_ref()
            .is_some_and(|source| source.preceding.is_some())
        {
            let source = self.format_delimited_source_gap(gap, DelimitedFollowing::Node, writer)?;
            boundary.separator = source.following;
            return Ok(());
        }
        self.prepare_sequence_comments(boundary, gap)?;
        self.comments.claim(boundary.trailing)?;
        self.format_comment_run(
            boundary.trailing,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )?;
        boundary
            .before_comma
            .write(writer, FlatSeparator::Empty, |_| {});
        writer.text(",");
        Ok(())
    }
}
