//! Prefix and postfix operators with explicitly owned operand boundaries.

use swc_next_ecma_ast::{
    CommentKind, GetSpan, SimpleAssignmentTargetData, Span, UnaryExpression, UnaryOperator,
    UpdateExpression,
};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_unary(
        &mut self,
        unary: UnaryExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let operator = unary.operator(self.ast);
        writer.text(operator.as_str());
        if matches!(
            operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            writer.space();
        }
        let content = self.expression_content(unary.argument(self.ast));
        let argument = content.expression;
        let span = unary.span(self.ast);
        let argument_span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(
            span.start + operator.as_str().len() as u32,
            argument_span.start,
        ))?;
        let trailing = self
            .comments
            .in_range(Span::new(argument_span.end, span.end));
        let mode = self.content_mode(leading);
        let leading_end = match content.parentheses {
            Some(boundary)
                if mode == ContentMode::Verbatim
                    && leading
                        .last()
                        .is_some_and(|comment| comment.span.end <= boundary.outer.start) =>
            {
                boundary.outer.start
            }
            _ => argument_span.start,
        };
        let position = ExpressionPosition::UnaryArgument(operator);
        if self.diagnostic_range_is_sensitive(span) {
            let operator_end = span.start + operator.as_str().len() as u32;
            self.comments.claim(leading)?;
            self.format_comment_run(leading, CommentRunLayout::SourceAfter(operator_end), writer)?;
            let previous = leading
                .last()
                .map_or(operator_end, |comment| comment.span.end);
            let flat = if leading.is_empty() {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            self.line_constraint(previous, argument_span.start)
                .write(writer, flat, |_| {});
            let tail = self.format_expression(
                argument,
                ExpressionContext::at(position).with_content_mode(mode),
                writer,
            )?;
            return self.capture_expression_tail(Span::new(argument_span.end, span.end), tail);
        }
        if leading.is_empty() && trailing.is_empty() {
            return self.format_expression(argument, ExpressionContext::at(position), writer);
        }
        // The operator owns one group around boundary comments and its operand.
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.comments.claim(leading)?;
                self.format_comment_run(
                    leading,
                    CommentRunLayout::GroupedLeading(leading_end),
                    writer,
                )?;
                let tail = self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::ParenthesizedUnaryArgument(operator))
                        .with_content_mode(mode),
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.comments.claim(trailing)?;
                self.format_comment_run(
                    trailing,
                    CommentRunLayout::SourceAfter(argument_span.end),
                    writer,
                )
            })?;
            if trailing
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                writer.hard_line();
            } else {
                writer.soft_line();
            }
            writer.text(")");
            Ok(None)
        })
    }

    pub(in crate::formatter) fn format_update(
        &mut self,
        update: UpdateExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let target = update.argument(self.ast);
        let SimpleAssignmentTargetData::IdentifierReference(identifier) =
            self.ast.simple_assignment_target_data(target)
        else {
            return Err(self.unsupported("update target", target));
        };
        let span = update.span(self.ast);
        let target_span = target.span(self.ast);
        let operator = update.operator(self.ast).as_str();
        if update.prefix(self.ast) {
            writer.text(operator);
            let comments = self
                .comments
                .between(Span::new(span.start + 2, target_span.start))?;
            let mode = self.content_mode(comments);
            let comments = self.comments.take(comments.len());
            self.format_comment_run(
                comments,
                CommentRunLayout::Leading(target_span.start),
                writer,
            )?;
            if mode == ContentMode::Verbatim {
                self.format_verbatim(target_span, writer)?;
            } else {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
            }
            self.capture_expression_tail(Span::new(target_span.end, span.end), None)
        } else {
            let leading = self
                .comments
                .between(Span::new(span.start, target_span.start))?;
            let leading = self.comments.take(leading.len());
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(target_span.start),
                writer,
            )?;
            let boundary = Span::new(target_span.end, span.end - 2);
            let mode = self.content_mode(self.comments.in_range(boundary));
            if mode == ContentMode::Verbatim || self.content_mode(leading) == ContentMode::Verbatim
            {
                self.format_verbatim(target_span, writer)?;
            } else {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
            }
            let comments = self.comments.between(boundary)?;
            let count = comments
                .iter()
                .take_while(|comment| {
                    comment.kind == CommentKind::Block
                        && self.line_breaks(target_span.end, comment.span.start) == 0
                        && self.line_breaks(comment.span.start, comment.span.end) == 0
                })
                .count();
            let inline = self.comments.take(count);
            self.format_inline_trailing_comments(inline, writer)?;
            let source_end = inline
                .last()
                .map_or(target_span.end, |comment| comment.span.end);
            writer.text(operator);
            self.capture_expression_tail(Span::new(source_end, boundary.end), None)
        }
    }
}
