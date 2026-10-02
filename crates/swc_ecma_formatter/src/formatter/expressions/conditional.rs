//! Conditional positions, aligned branches, and prepared operator boundaries.

use swc_next_ecma_ast::{Comment, ConditionalExpression, Expr, ExprData, GetSpan, Span};

use super::{ExpressionContext, ExpressionPosition, Formatter, StatementStart, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, IndentStyle, document::Writer};

#[derive(Clone, Copy)]
enum ConditionalLayout {
    Root,
    Test,
    Consequent,
    Alternate,
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
        let mut write = |writer: &mut Writer<'ast>| {
            self.format_conditional_contents(conditional, context.statement_start, layout, writer)
        };
        match layout {
            ConditionalLayout::Root | ConditionalLayout::Test => writer.group(false, write),
            ConditionalLayout::Consequent | ConditionalLayout::Alternate => write(writer),
        }
    }

    fn format_conditional_contents(
        &mut self,
        conditional: ConditionalExpression,
        statement_start: StatementStart,
        layout: ConditionalLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let test = conditional.test(self.ast);
        let consequent = conditional.consequent(self.ast);
        let alternate = conditional.alternate(self.ast);
        let question = self.conditional_boundary(test, consequent, "?")?;
        let colon = self.conditional_boundary(consequent, alternate, ":")?;
        let mut write_test = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(
                test,
                ExpressionContext::at(ExpressionPosition::ConditionalTest)
                    .with_statement_start(statement_start)
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
        previous: Expr,
        next: Expr,
        operator: &'static str,
    ) -> Result<ConditionalBoundary<'ast>, FormatError> {
        let start = previous.span(self.ast).end;
        let end = self
            .expression_content(next)
            .expression
            .span(self.ast)
            .start;
        let span = Span::new(start, end);
        let comments = self.comments.in_range(span);
        if comments.is_empty() && !self.diagnostic_range_is_sensitive(span) {
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
        if self.diagnostic_range_is_sensitive(span) {
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
        expression: Expr,
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
        let content = self.expression_content(expression);
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
            let mut tail = self.format_expression_with_leading_comments(
                expression,
                NodeHead {
                    comments: boundary.leading,
                    content_start: content.expression.span(self.ast).start,
                    layout: CommentRunLayout::Source,
                    gap_end: Some(content.expression.span(self.ast).start),
                },
                ExpressionContext::at(position)
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
