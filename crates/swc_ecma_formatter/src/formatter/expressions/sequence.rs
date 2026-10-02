//! Typed comma-expression traversal and borrowed separator boundaries.

use swc_next_ecma_ast::{Comment, Expr, GetSpan, SequenceExpression, Span};

use super::{ExpressionContext, ExpressionPosition, Formatter, ParenthesesOwner, TailOwner};
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, Semicolons, document::Writer};

struct SequenceBoundary<'ast> {
    next: Expr,
    next_start: u32,
    source_end: u32,
    comments: &'ast [Comment],
    comma: Option<u32>,
    trailing: &'ast [Comment],
    leading: &'ast [Comment],
    previous_mode: ContentMode,
    before_comma: LineConstraint,
    after_comma: LineConstraint,
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
        let first_start = self.expression_content(first).expression.span(ast).start;
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
        if context.parentheses_owner == ParenthesesOwner::Expression
            && leading.iter().any(|&comment| {
                matches!(self.comment_semantics(comment), CommentSemantics::TypeCast)
            })
        {
            return self.format_parenthesized_expression(
                Expr::SequenceExpression(sequence),
                head,
                context,
                writer,
            );
        }
        let start = self.format_node_head(head, context.expression_start, writer)?;
        if self.options.semicolons == Semicolons::AsNeeded && start.is_statement() {
            writer.text(";");
        }
        let parenthesize = context.parentheses_owner == ParenthesesOwner::Expression
            && !matches!(context.position, ExpressionPosition::RestrictedArgument);
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
                    first,
                    &[],
                    first_start,
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
                        .after_comma
                        .write(writer, FlatSeparator::Space, Writer::line);
                    let following = expressions
                        .next()
                        .map(|next| self.sequence_boundary(boundary.next, next))
                        .transpose()?;
                    let tail = self.format_sequence_item(
                        boundary.next,
                        boundary.leading,
                        boundary.next_start,
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
        expression: Expr,
        leading: &'ast [Comment],
        next_start: u32,
        following: Option<&SequenceBoundary<'ast>>,
        tail_owner: TailOwner,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            following.map_or(ContentMode::Formatted, |boundary| boundary.previous_mode)
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
                content_start: next_start,
                layout: CommentRunLayout::Leading(next_start),
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
            previous_mode: self.trailing_content_mode(span, BoundaryPunctuation::Comma, false),
            before_comma: LineConstraint::Unconstrained,
            after_comma: LineConstraint::Unconstrained,
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
            self.split_separator_comments(comments, before_count, start, end)
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
            boundary.after_comma = self.line_constraint(comma + 1, next);
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
        self.prepare_sequence_comments(
            boundary,
            Span::new(boundary.source_end, boundary.next_start),
        )?;
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
