//! Prefix and postfix operators with explicitly owned operand boundaries.

use swc_next_ecma_ast::{
    CommentKind, GetSpan, SimpleAssignmentTargetData, Span, UnaryExpression, UnaryOperator,
    UpdateExpression,
};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
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
        let (argument, _) = self.unparenthesized_expression(unary.argument(self.ast));
        let span = unary.span(self.ast);
        let argument_span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(
            span.start + operator.as_str().len() as u32,
            argument_span.start,
        ))?;
        let trailing = self
            .comments
            .in_range(Span::new(argument_span.end, span.end));
        let position = ExpressionPosition::UnaryArgument(operator);
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
                    CommentRunLayout::GroupedLeading(argument_span.start),
                    writer,
                )?;
                let tail = self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::ParenthesizedUnaryArgument(operator)),
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
            let comments = self.comments.take(comments.len());
            self.format_comment_run(
                comments,
                CommentRunLayout::Leading(target_span.start),
                writer,
            )?;
            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
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
            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
            let boundary = Span::new(target_span.end, span.end - 2);
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
