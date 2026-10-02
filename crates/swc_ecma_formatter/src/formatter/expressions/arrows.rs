//! Expression arrows with parameter, arrow-token and body boundary ownership.

use swc_next_ecma_ast::{
    ArrowFunctionBodyData, ArrowFunctionExpression, CommentKind, ExprData, GetSpan, Span,
};

use super::{ExpressionContext, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::{FormatError, Semicolons, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_arrow(
        &mut self,
        arrow: ArrowFunctionExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if arrow.r#async(self.ast)
            || arrow.type_parameters(self.ast).is_some()
            || arrow.return_type(self.ast).is_some()
        {
            return Err(self.unsupported("async or typed arrow", arrow));
        }
        let body = self.ast.arrow_function_body_data(arrow.body(self.ast));
        let params = arrow.params(self.ast);
        let parameter_span = params.span(self.ast);
        let body_span = arrow.body(self.ast).span(self.ast);
        let gap = Span::new(parameter_span.end, body_span.start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=>") else {
            return Err(self.unsupported("arrow token boundary", arrow));
        };
        let (before, leading) =
            comments.split_at(comments.partition_point(|comment| comment.span.end <= operator));
        let parameters = self.source_arrow_parameters(params, !before.is_empty())?;
        let mode = self.content_mode(leading);
        let (head_end, hugs_shape, leading_layout) = match body {
            ArrowFunctionBodyData::Expr(expression) => {
                let content = self.expression_content(expression);
                (
                    content.expression.span(self.ast).start,
                    matches!(
                        content.data,
                        ExprData::ArrayExpression(_)
                            | ExprData::ObjectExpression(_)
                            | ExprData::SequenceExpression(_)
                            | ExprData::ArrowFunctionExpression(_)
                    ),
                    CommentRunLayout::GroupedLeading(body_span.start),
                )
            }
            ArrowFunctionBodyData::FunctionBody(_) => (
                body_span.start,
                true,
                CommentRunLayout::FlatLeading(body_span.start),
            ),
        };
        let head_comments = self.comments.in_range(Span::new(operator + 2, head_end));
        let body_starts_line = head_comments.iter().enumerate().any(|(index, comment)| {
            let next_start = head_comments
                .get(index + 1)
                .map_or(head_end, |next| next.span.start);
            comment.kind == CommentKind::Line
                || (matches!(leading_layout, CommentRunLayout::GroupedLeading(_))
                    && self.source_position_starts_line(comment.span.start)
                    && self.line_breaks(comment.span.end, next_start) != 0)
        });
        let hug_body = hugs_shape && !body_starts_line;
        if context.expression_start.is_statement()
            && self.options.semicolons == Semicolons::AsNeeded
            && parameters.starts_with_parenthesis()
        {
            writer.text(";");
        }
        writer.group(false, |writer| {
            self.format_source_parameters(parameters, writer)?;
            self.comments.claim(before)?;
            self.format_comment_run(
                before,
                CommentRunLayout::SourceAfter(parameter_span.end),
                writer,
            )?;
            let before_end = before
                .last()
                .map_or(parameter_span.end, |comment| comment.span.end);
            self.line_constraint(before_end, operator).write(
                writer,
                FlatSeparator::Space,
                Writer::space,
            );
            writer.text("=>");
            let first = leading
                .first()
                .map_or(body_span.start, |comment| comment.span.start);
            let mut write_body = |writer: &mut Writer<'ast>| {
                self.line_constraint(operator + 2, first).write(
                    writer,
                    FlatSeparator::Space,
                    |writer| {
                        if body_starts_line {
                            writer.hard_line();
                        } else if hug_body {
                            writer.space();
                        } else {
                            writer.line();
                        }
                    },
                );
                self.comments.claim(leading)?;
                match body {
                    ArrowFunctionBodyData::Expr(expression) => self
                        .format_expression_with_leading_comments(
                            expression,
                            NodeHead {
                                comments: leading,
                                content_start: body_span.start,
                                layout: leading_layout,
                                gap_end: None,
                            },
                            ExpressionContext::at(ExpressionPosition::ArrowBody)
                                .with_content_mode(mode)
                                .with_tail_owner(TailOwner::ParentOperation),
                            writer,
                        ),
                    ArrowFunctionBodyData::FunctionBody(body) => {
                        self.format_comment_run(leading, leading_layout, writer)?;
                        self.format_function_body(body, mode, writer)?;
                        Ok(None)
                    }
                }
            };
            if hug_body {
                write_body(writer)
            } else {
                writer.group(false, |writer| writer.indent(write_body))
            }
        })
    }
}
