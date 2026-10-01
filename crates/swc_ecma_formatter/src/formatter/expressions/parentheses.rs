//! Removed source delimiters retain their leading run and return their pending tail.

use swc_next_ecma_ast::{Expr, ExprData, GetSpan, ParenthesizedExpression, Span};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::{FormatError, document::Writer};

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

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn expression_content(&self, expression: Expr) -> ExpressionContent {
        let (inner, data) = self.unparenthesized_expression(expression);
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
        !leading
            .iter()
            .chain(trailing)
            .any(|&comment| self.comment_has_parenthesis_semantics(comment))
            && !self.parenthesis_has_type_cast(boundary.outer.start)
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
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (inner, _) = self.unparenthesized_expression(parenthesized.expression(self.ast));
        let span = parenthesized.span(self.ast);
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        if !self.parentheses_are_removable(SourceParentheses {
            outer: span,
            inner: inner_span,
        }) {
            return Err(self.unsupported("semantic parentheses", parenthesized));
        }
        self.comments.claim(leading)?;
        if matches!(context.position, ExpressionPosition::Statement) {
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                self.comment_gap(last.span.end, inner_span.start, writer);
            }
        } else {
            self.format_comment_run(leading, CommentRunLayout::Leading(inner_span.start), writer)?;
        }
        let tail = self.format_expression(inner, context, writer)?;
        self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)
    }

    fn parenthesis_has_type_cast(&self, start: u32) -> bool {
        let comments = self.ast.comments();
        let count = comments.partition_point(|comment| comment.span.end <= start);
        let Some(&comment) = comments[..count].last() else {
            return false;
        };
        self.comment_is_type_cast(comment)
            && self
                .source(Span::new(comment.span.end, start))
                .chars()
                .all(char::is_whitespace)
    }
}
