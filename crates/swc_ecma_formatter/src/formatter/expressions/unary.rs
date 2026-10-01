//! Prefix and postfix operators with explicitly owned operand boundaries.

use swc_next_ecma_ast::{
    CommentKind, GetSpan, SimpleAssignmentTargetData, Span, UnaryExpression, UnaryOperator,
    UpdateExpression,
};

use super::{ExpressionPosition, Formatter};
use crate::formatter::comments::CommentRunLayout;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_unary(
        &mut self,
        unary: UnaryExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
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
            return self.format_expression(argument, position, writer);
        }
        // The operator owns one group around boundary comments and its operand.
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.comments.claim(leading)?;
                self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                if let Some(last) = leading.last() {
                    self.comment_gap(last.span.end, argument_span.start, writer);
                }
                self.format_expression(argument, position, writer)?;
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
            Ok(())
        })
    }

    pub(in crate::formatter) fn format_update(
        &mut self,
        update: UpdateExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
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
            self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
            if let Some(last) = comments.last() {
                self.comment_gap(last.span.end, target_span.start, writer);
            }
            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
        } else {
            writer.text(self.ast.get_utf8(identifier.name(self.ast)));
            let comments = self
                .comments
                .between(Span::new(target_span.end, span.end - 2))?;
            let comments = self.comments.take(comments.len());
            if !comments.is_empty() {
                writer.space();
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
            }
            writer.text(operator);
        }
        Ok(())
    }
}
