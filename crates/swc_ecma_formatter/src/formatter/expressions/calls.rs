//! Callees and source-owned argument delimiters.

use swc_next_ecma_ast::{ArgumentData, CallExpression, CommentKind, ExprData, GetSpan, Span};

use super::arguments::{ArgumentLayout, FollowingArgument, Separator};
use super::{ExpressionContext, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, TrailingCommas, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if call.optional(ast) || call.type_arguments(ast).is_some() {
            return Err(self.unsupported("call suffix", call));
        }
        let callee = call.callee(ast);
        let callee_span = callee.span(ast);
        let arguments = call.arguments(ast);
        let span = call.span(ast);
        let first = arguments
            .iter()
            .next()
            .map(|id| ast.get_node_in_sub_range(id));
        let gap = Span::new(
            callee_span.end,
            first.map_or(span.end - 1, |arg| arg.span(ast).start),
        );
        let boundary_comments = self.comments.in_range(gap);
        let Some(opening) = self.find_token(gap, boundary_comments, "(") else {
            return Err(self.unsupported("call opening boundary", call));
        };
        let prefix_span = Span::new(callee_span.end, opening);
        let prefix = self.comments.in_range(prefix_span);
        let prefix_belongs_to_arguments = first.is_some()
            && !prefix.is_empty()
            && prefix.iter().all(|&comment| {
                comment.kind == CommentKind::Block
                    && matches!(
                        self.comment_semantics(comment),
                        CommentSemantics::Ordinary | CommentSemantics::Suppression
                    )
            })
            && !self.diagnostic_range_is_sensitive(Span::new(callee_span.end, opening + 1));
        let first_mode = if !prefix_belongs_to_arguments && first.is_some() {
            self.first_argument_prefix_mode(prefix)
        } else {
            ContentMode::Formatted
        };
        let mode = if prefix_belongs_to_arguments {
            ContentMode::Formatted
        } else {
            self.trailing_content_mode(prefix_span, BoundaryPunctuation::None, false)
        };
        let tail = self.format_expression(
            callee,
            ExpressionContext::at(ExpressionPosition::CallCallee)
                .with_statement_start(context.statement_start)
                .with_tail_owner(TailOwner::ParentOperation)
                .with_content_mode(mode),
            writer,
        )?;
        let source_end = tail
            .as_ref()
            .map_or(callee_span.end, |tail| tail.source_end);
        let before = self.comments.between(Span::new(source_end, opening))?;
        let move_tail = first.is_some()
            && (prefix.is_empty() || prefix_belongs_to_arguments)
            && tail.as_ref().is_some_and(|tail| {
                tail.comments.iter().all(|&comment| {
                    comment.kind == CommentKind::Block
                        && matches!(self.comment_semantics(comment), CommentSemantics::Ordinary)
                })
            })
            && !self.diagnostic_range_is_sensitive(Span::new(source_end, opening + 1));
        let outside_count = if move_tail {
            0
        } else if prefix_belongs_to_arguments {
            tail.as_ref().map_or(0, |tail| tail.comments.len())
        } else {
            before.len()
        };
        let (outside, inside) = before.split_at(outside_count);
        self.comments.claim(outside)?;
        self.format_comment_run(outside, CommentRunLayout::SourceAfter(source_end), writer)?;
        let end = outside
            .last()
            .map_or(source_end, |comment| comment.span.end);
        if outside
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line)
        {
            writer.hard_line();
        } else {
            self.line_constraint(end, opening)
                .write(writer, FlatSeparator::Empty, |_| {});
        }
        let content_start = if inside.is_empty() {
            opening + 1
        } else if move_tail {
            source_end
        } else {
            callee_span.end
        };
        let content = Span::new(content_start, span.end - 1);
        if first.is_none()
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            self.format_verbatim(Span::new(opening, span.end), writer)?;
            return Ok(None);
        }
        let hugged = arguments.len() == 1
            && self.comments.in_range(content).is_empty()
            && !self.diagnostic_range_is_sensitive(content)
            && first.is_some_and(|argument| match ast.argument_data(argument) {
                ArgumentData::Expr(expression) => matches!(
                    self.expression_content(expression).data,
                    ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_)
                ),
                ArgumentData::SpreadElement(_) => false,
            });
        let expand = arguments
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .scan(content.start, |previous, argument| {
                let empty = self.next_line_is_empty(*previous, argument.span(ast).start);
                *previous = argument.span(ast).end;
                Some(empty)
            })
            .any(|empty| empty);
        writer.group(expand, |writer| {
            writer.text("(");
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hugged && (first.is_some() || !self.comments.in_range(content).is_empty()) {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut arguments = arguments
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_argument = false;
                while let Some(argument) = arguments.next() {
                    if has_argument {
                        let next_start = self
                            .comments
                            .between(Span::new(end, argument.span(ast).start))?
                            .first()
                            .map_or(argument.span(ast).start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next_start),
                            writer,
                        );
                    }
                    let following = arguments
                        .peek()
                        .copied()
                        .map_or(FollowingArgument::End, FollowingArgument::Argument);
                    let boundary = self.format_argument_item_with_mode(
                        argument,
                        following,
                        Span::new(end, content.end),
                        if has_argument {
                            ContentMode::Formatted
                        } else {
                            first_mode
                        },
                        !hugged && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    end = boundary.end;
                    separator = boundary.separator;
                    has_argument = true;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                self.comments.claim(dangling)?;
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            };
            let end = if hugged {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hugged && (first.is_some() || end != content.start) {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            writer.text(")");
            Ok(None)
        })
    }
}
