//! Source delimiter ownership and semantic boundaries.

use swc_next_ecma_ast::{Comment, Expr, ExprData, GetSpan, ParenthesizedExpression, Span};

use super::{ExpressionContext, ExpressionPosition, Formatter, StatementStart, TailOwner};
use crate::formatter::comments::{
    CommentBinding, CommentRunLayout, CommentSemantics, ExpressionTail,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) struct SourceParentheses {
    pub(super) outer: Span,
    pub(super) inner: Span,
}

pub(super) struct ExpressionContent {
    pub(super) expression: Expr,
    pub(super) data: ExprData,
    pub(super) parentheses: Option<SourceParentheses>,
}

enum RetainedParentheses {
    Cast,
    Annotation,
    Operand,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn expression_content(&self, expression: Expr) -> ExpressionContent {
        let mut inner = expression;
        let data = loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                break data;
            };
            let child = parenthesized.expression(self.ast);
            if self
                .expression_needs_parentheses(self.ast.expr_data(child), ExpressionPosition::Nested)
                .is_none()
                || !self.parentheses_are_removable(SourceParentheses {
                    outer: parenthesized.span(self.ast),
                    inner: child.span(self.ast),
                })
            {
                break data;
            }
            inner = child;
        };
        let boundary = (inner != expression).then(|| SourceParentheses {
            outer: expression.span(self.ast),
            inner: inner.span(self.ast),
        });
        ExpressionContent {
            expression: inner,
            data,
            parentheses: boundary,
        }
    }

    pub(super) fn parentheses_are_removable(&self, boundary: SourceParentheses) -> bool {
        let comments = self.ast.comments();
        let range = |span: Span| {
            let first = comments.partition_point(|comment| comment.span.start < span.start);
            let inside = &comments[first..];
            let count = inside.partition_point(|comment| comment.span.end <= span.end);
            &inside[..count]
        };
        let leading = range(Span::new(boundary.outer.start, boundary.inner.start));
        let trailing = range(Span::new(boundary.inner.end, boundary.outer.end));
        let mut inner_cast = None;
        let leading_binding =
            self.comment_run_binding(leading.iter().copied(), |comment| {
                match self.comment_semantics(comment) {
                    CommentSemantics::Ordinary => CommentBinding::Unbound,
                    CommentSemantics::TypeCast => {
                        if inner_cast
                            .get_or_insert_with(|| self.source_type_cast(boundary.inner.start))
                            .is_some()
                        {
                            CommentBinding::BoundTo(boundary.inner)
                        } else {
                            CommentBinding::SourceBound
                        }
                    }
                    _ => CommentBinding::SourceBound,
                }
            });
        let trailing_binding = self.comment_run_binding(trailing.iter().copied(), |comment| {
            if matches!(self.comment_semantics(comment), CommentSemantics::Ordinary) {
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        leading_binding.allows_relocation(Some(boundary.inner))
            && trailing_binding.allows_relocation(None)
            && !self.parenthesis_has_type_cast(boundary.outer.start)
    }

    pub(in crate::formatter) fn format_expression_leading_comments(
        &self,
        comments: &[Comment],
        next_start: u32,
        layout: CommentRunLayout,
        mut start: StatementStart,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementStart, FormatError> {
        if start != StatementStart::None
            && self.options.semicolons == Semicolons::AsNeeded
            && !comments.is_empty()
            && self.parenthesis_has_type_cast(next_start)
        {
            writer.text(";");
            start = StatementStart::None;
        }
        self.format_comment_run(comments, layout, writer)?;
        Ok(start)
    }

    pub(super) fn capture_expression_tail(
        &self,
        span: Span,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source_end = tail.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        Ok((!comments.is_empty()).then_some(ExpressionTail {
            source_end,
            comments,
        }))
    }

    pub(super) fn format_source_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let trailing = Span::new(child_span.end, span.end - 1);
        let inline_mode = self.trailing_content_mode(trailing, BoundaryPunctuation::None, false);
        if inline_mode == ContentMode::Verbatim
            && (matches!(context.position, ExpressionPosition::Statement)
                || context.tail_owner == TailOwner::ParentOperation
                || self.expression_needs_parentheses(self.ast.expr_data(child), context.position)
                    == Some(true))
        {
            self.format_verbatim_expression(
                Expr::ParenthesizedExpression(parenthesized),
                context,
                writer,
            )?;
            return Ok(None);
        }
        if self.parenthesis_has_type_cast(span.start) {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Cast,
                writer,
            );
        }
        if matches!(context.position, ExpressionPosition::ConditionalTest)
            && matches!(
                self.ast.expr_data(child),
                ExprData::ConditionalExpression(_)
            )
        {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Operand,
                writer,
            );
        }
        let content = self.expression_content(Expr::ParenthesizedExpression(parenthesized));
        let head = self
            .comments
            .in_range(Span::new(span.start, child_span.start));
        let tail = self.comments.in_range(trailing);
        let terminal_mode = self.trailing_content_mode(trailing, BoundaryPunctuation::None, true);
        let ignored_child = (context.content_mode == ContentMode::Verbatim
            || inline_mode == ContentMode::Verbatim
            || self.content_mode(head) == ContentMode::Verbatim
            || terminal_mode == ContentMode::Verbatim)
            && head.iter().chain(tail).all(|&comment| {
                matches!(
                    self.comment_semantics(comment),
                    CommentSemantics::Ordinary | CommentSemantics::Suppression
                )
            });
        if ignored_child
            && (self.ignored_operand_owns_operation_head(child, context.position)
                || matches!(context.position, ExpressionPosition::UnaryArgument(_))
                || self
                    .expression_needs_parentheses(self.ast.expr_data(child), context.position)
                    .is_none())
        {
            return self.format_retained_parentheses(
                parenthesized,
                context.with_content_mode(ContentMode::Verbatim),
                RetainedParentheses::Operand,
                writer,
            );
        }
        if !ignored_child && content.expression == Expr::ParenthesizedExpression(parenthesized) {
            return self.format_retained_parentheses(
                parenthesized,
                context,
                RetainedParentheses::Annotation,
                writer,
            );
        }
        let inner = if ignored_child {
            child
        } else {
            content.expression
        };
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        self.comments.claim(leading)?;
        let layout = if matches!(context.position, ExpressionPosition::Statement) {
            CommentRunLayout::Source
        } else {
            CommentRunLayout::Leading(inner_span.start)
        };
        context.statement_start = self.format_expression_leading_comments(
            leading,
            inner_span.start,
            layout,
            context.statement_start,
            writer,
        )?;
        context.content_mode = if ignored_child {
            ContentMode::Verbatim
        } else {
            self.content_mode(leading)
        };
        if matches!(context.position, ExpressionPosition::Statement)
            && let Some(last) = leading.last()
        {
            self.comment_gap(last.span.end, inner_span.start, writer);
        }
        let tail = self.format_expression(inner, context, writer)?;
        self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)
    }

    /// A leading ignore must stay inside its source pair when removing the pair
    /// would attach it to the enclosing operation's lexical head instead.
    fn ignored_operand_owns_operation_head(
        &self,
        expression: Expr,
        position: ExpressionPosition,
    ) -> bool {
        match position {
            ExpressionPosition::BinaryOperand {
                operator,
                side: super::binary::OperandSide::Left,
            } => super::binary::BinaryLike::from_data(self.ast.expr_data(expression), self.ast)
                .is_none_or(|child| !operator.flattens(child.operator)),
            ExpressionPosition::ConditionalTest => true,
            _ => false,
        }
    }

    fn format_retained_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        kind: RetainedParentheses,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let content = self.expression_content(parenthesized.expression(self.ast));
        let inner_span = content.expression.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = self.comments.in_range(Span::new(inner_span.end, span.end));
        let hug = match content.data {
            ExprData::ArrayExpression(array) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!array.elements(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            _ => false,
        };
        let expand = matches!(kind, RetainedParentheses::Cast)
            && (leading
                .first()
                .is_some_and(|comment| self.line_breaks(span.start, comment.span.start) != 0)
                || trailing.first().is_some_and(|comment| {
                    self.line_breaks(inner_span.end, comment.span.start) != 0
                }));
        let leading_layout = match kind {
            RetainedParentheses::Cast | RetainedParentheses::Operand => {
                CommentRunLayout::GroupedLeading(inner_span.start)
            }
            RetainedParentheses::Annotation => CommentRunLayout::FlatLeading(inner_span.start),
        };
        if context.statement_start != StatementStart::None
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let first = leading
            .first()
            .map_or(inner_span.start, |comment| comment.span.start);
        let opening = self.line_constraint(span.start + 1, first);
        let last = trailing
            .last()
            .map_or(inner_span.end, |comment| comment.span.end);
        let closing = self.line_constraint(last, span.end - 1);
        let mut write_contents = |writer: &mut Writer<'ast>| {
            self.comments.claim(leading)?;
            self.format_comment_run(leading, leading_layout, writer)?;
            let mode = match context.content_mode {
                ContentMode::Verbatim => ContentMode::Verbatim,
                ContentMode::Formatted => self.content_mode(leading),
            };
            let position = match (mode, context.position) {
                (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(operator)) => {
                    ExpressionPosition::ParenthesizedUnaryArgument(operator)
                }
                _ => ExpressionPosition::Nested,
            };
            let tail = self.format_expression(
                content.expression,
                ExpressionContext::at(position).with_content_mode(mode),
                writer,
            )?;
            let tail = self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)?;
            self.format_expression_tail(tail, writer)
        };
        writer.group(expand, |writer| {
            writer.text("(");
            if hug {
                write_contents(writer)?;
            } else {
                writer.indent(|writer| {
                    opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    write_contents(writer)
                })?;
                closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
            writer.text(")");
            Ok(None)
        })
    }

    fn parenthesis_has_type_cast(&self, start: u32) -> bool {
        self.source_type_cast(start).is_some()
    }

    fn source_type_cast(&self, start: u32) -> Option<Comment> {
        if !self.ast.source()[start as usize..].starts_with('(') {
            return None;
        }
        let comments = self.ast.comments();
        if comments.is_empty() {
            return None;
        }
        let count = comments.partition_point(|comment| comment.span.end <= start);
        let mut next_start = start;
        for &comment in comments[..count].iter().rev() {
            if !self
                .source(Span::new(comment.span.end, next_start))
                .chars()
                .all(is_source_whitespace)
            {
                break;
            }
            // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
            if self.source(comment.span).starts_with("/**") {
                return self.comment_is_type_cast(comment).then_some(comment);
            }
            next_start = comment.span.start;
        }
        None
    }
}
