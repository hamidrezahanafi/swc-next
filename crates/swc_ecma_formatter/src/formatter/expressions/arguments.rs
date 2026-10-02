//! Shared source-ordered argument slots and comma/comment boundaries.

use super::Formatter;
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, TrailingCommas, document::Writer};
use swc_next_ecma_ast::{
    Argument, ArgumentData, Comment, CommentKind, ExprData, GetSpan, Span, SubRange,
};

#[derive(Clone, Copy)]
pub(super) enum FollowingArgument {
    End,
    Argument(Argument),
    Elision(Option<Argument>),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum FollowingItem {
    End,
    Node(Span),
    Elision(Option<Span>),
}

impl FollowingItem {
    fn node(self) -> Option<Span> {
        match self {
            Self::End => None,
            Self::Node(span) => Some(span),
            Self::Elision(span) => span,
        }
    }
}

pub(in crate::formatter) struct PreparedItem {
    item_span: Span,
    following: FollowingItem,
    span: Span,
    pub(in crate::formatter) mode: ContentMode,
    trailing_comma: bool,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ArgumentLayout {
    Lines,
    Fill,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum Separator {
    Line,
    EmptyLine,
    OwnLineComment,
}

pub(in crate::formatter) struct ItemBoundary {
    pub(in crate::formatter) end: u32,
    pub(in crate::formatter) separator: Separator,
}

impl ArgumentLayout {
    pub(super) fn entry<'a, T>(
        self,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match self {
            Self::Lines => write(writer),
            Self::Fill => writer.fill_entry(write),
        }
    }

    pub(in crate::formatter) fn separator(
        self,
        separator: Separator,
        constraint: LineConstraint,
        writer: &mut Writer<'_>,
    ) {
        self.entry(writer, |writer| {
            constraint.write(writer, FlatSeparator::Space, |writer| {
                match (self, separator) {
                    (Self::Fill, Separator::EmptyLine) => {
                        writer.hard_line();
                        writer.hard_line();
                    }
                    (Self::Fill, Separator::OwnLineComment) => writer.hard_line(),
                    (_, Separator::EmptyLine) => {
                        writer.line();
                        writer.soft_line();
                    }
                    _ => writer.line(),
                }
            })
        });
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn first_argument_prefix_mode(&self, comments: &[Comment]) -> ContentMode {
        if !self.has_suppression {
            return ContentMode::Formatted;
        }
        let mut runs = comments.chunk_by(|left, right| {
            self.source(Span::new(left.span.end, right.span.start))
                .chars()
                .all(|character| {
                    is_source_whitespace(character)
                        && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                })
        });
        let own_line = runs.any(|run| {
            run.first()
                .is_some_and(|first| self.source_position_starts_line(first.span.start))
                && self.content_mode(run) == ContentMode::Verbatim
        });
        if own_line {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    pub(super) fn format_argument_item(
        &mut self,
        argument: Argument,
        following: FollowingArgument,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.format_argument_item_with_mode(
            argument,
            following,
            span,
            ContentMode::Formatted,
            trailing_comma,
            writer,
        )
    }

    pub(super) fn format_argument_item_with_mode(
        &mut self,
        argument: Argument,
        following: FollowingArgument,
        span: Span,
        leading_mode: ContentMode,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let following = match following {
            FollowingArgument::Argument(argument) => FollowingItem::Node(argument.span(self.ast)),
            FollowingArgument::Elision(argument) => {
                FollowingItem::Elision(argument.map(|node| node.span(self.ast)))
            }
            FollowingArgument::End => FollowingItem::End,
        };
        let mut item = self.prepare_delimited_item(
            argument.span(self.ast),
            following,
            span,
            trailing_comma,
            writer,
        )?;
        if leading_mode == ContentMode::Verbatim {
            item.mode = leading_mode;
        }
        let tail = self.format_argument(argument, item.mode, writer)?;
        self.finish_delimited_item(item, tail, writer)
    }

    pub(in crate::formatter) fn prepare_delimited_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let following_span = following.node();
        let leading = self
            .comments
            .between(Span::new(span.start, item_span.start))?;
        let leading = self.comments.take(leading.len());
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim || !self.has_suppression {
            leading_mode
        } else {
            self.trailing_content_mode(
                Span::new(
                    item_span.end,
                    following_span.map_or(span.end, |span| span.start),
                ),
                BoundaryPunctuation::Comma,
                following_span.is_none(),
            )
        };
        self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
        if let Some(last) = leading.last() {
            self.comment_gap(last.span.end, item_span.start, writer);
        }
        Ok(PreparedItem {
            item_span,
            following,
            span,
            mode,
            trailing_comma,
        })
    }

    pub(in crate::formatter) fn finish_delimited_item(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let PreparedItem {
            item_span,
            following,
            span,
            trailing_comma,
            ..
        } = item;
        let following_span = following.node();
        let has_more = !matches!(following, FollowingItem::End);
        let content_end = tail.as_ref().map_or(item_span.end, |tail| tail.source_end);
        let gap = Span::new(
            content_end,
            following_span.map_or(span.end, |span| span.start),
        );
        let comments = self.comments.between(gap)?;
        let comma = self.find_comma(gap, comments);
        let constrained = self.diagnostic_range_is_sensitive(span);
        // A directive-covered separator owns the comments on its source side.
        let protected_comma =
            comma.filter(|&position| constrained && self.diagnostic_line_is_sensitive(position));
        let trailing_limit = if constrained && matches!(following, FollowingItem::Elision(_)) {
            comma
                .and_then(|comma| self.find_comma(Span::new(comma + 1, gap.end), comments))
                .unwrap_or(gap.end)
        } else {
            gap.end
        };
        let mut inline_end = content_end;
        let mut trailing_count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if comment.span.start >= trailing_limit {
                break;
            }
            if self.line_breaks(inline_end, comment.span.start) != 0 {
                break;
            }
            inline_end = comment.span.end;
            if comment.kind == CommentKind::Line {
                trailing_count = index + 1;
                break;
            }
            if following_span.is_none()
                || comma.is_none_or(|position| comment.span.start < position)
            {
                trailing_count = index + 1;
            }
        }
        let boundary_count = protected_comma.map_or(trailing_count, |comma| {
            comments
                .partition_point(|comment| comment.span.end <= comma)
                .max(trailing_count)
        });
        let boundary = self.comments.take(boundary_count);
        let (trailing, before_comma) = boundary.split_at(trailing_count);
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_inline_trailing_comments(trailing, writer)?;
        if !before_comma.is_empty() {
            self.format_comment_run(
                before_comma,
                CommentRunLayout::SourceAfter(trailing_end),
                writer,
            )?;
        }
        let boundary_end = boundary
            .last()
            .map_or(content_end, |comment| comment.span.end);
        let mut separator = if comments[boundary_count..]
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            Separator::OwnLineComment
        } else {
            Separator::Line
        };
        let required_comma = has_more || protected_comma.is_some();
        if required_comma {
            if let Some(comma) = comma
                && let Some(last) = boundary.last()
                && last.span.end <= comma
            {
                self.line_constraint(last.span.end, comma).write(
                    writer,
                    FlatSeparator::Empty,
                    |_| {},
                );
            }
            writer.text(",");
            if let Some(comma) = comma {
                let after_separator = Span::new(comma + 1, gap.end);
                // The source gap after the comma is independent of comment ownership.
                let after_comma = comments.partition_point(|comment| comment.span.start <= comma);
                let next_start = comments
                    .get(boundary_count.max(after_comma))
                    .map_or(gap.end, |comment| comment.span.start);
                let start = after_separator.start.max(boundary_end);
                if start < next_start && self.next_line_is_empty(start, next_start) {
                    separator = Separator::EmptyLine;
                }
            }
        } else if trailing_comma {
            writer.if_break(",");
        }
        let mut end = boundary_end;
        if required_comma
            && let Some(comma) = comma
            && constrained
            && self
                .comments
                .first()
                .is_none_or(|comment| comment.span.start >= comma)
        {
            end = end.max(comma + 1);
        }
        if !has_more || !constrained && following_span.is_none() {
            let own_line = self.comments.between(Span::new(end, span.end))?;
            let own_line = self.comments.take(own_line.len());
            if let Some(first) = own_line.first() {
                writer.hard_line();
                if self.line_breaks(end, first.span.start) == 2 {
                    writer.hard_line();
                }
                self.format_comment_run(own_line, CommentRunLayout::Lines, writer)?;
            }
            if let Some(last) = own_line.last() {
                end = last.span.end;
            }
        }
        Ok(ItemBoundary { end, separator })
    }
}

/// An actual source argument list and its borrowed pre-opener comment owner.
pub(super) struct SourceArguments<'ast> {
    arguments: SubRange<Argument>,
    first: Option<SourceFirstArgument>,
    delimiters: Span,
    callee_end: u32,
    prefix: &'ast [Comment],
    prefix_belongs_to_arguments: bool,
    pub(super) callee_mode: ContentMode,
}

#[derive(Clone, Copy)]
struct SourceFirstArgument {
    node: Argument,
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arguments(
        &self,
        arguments: SubRange<Argument>,
        callee_end: u32,
        end: u32,
    ) -> Option<SourceArguments<'ast>> {
        let ast = self.ast;
        let mut first = arguments.iter().next().map(|id| SourceFirstArgument {
            node: ast.get_node_in_sub_range(id),
            mode: ContentMode::Formatted,
        });
        let gap = Span::new(
            callee_end,
            first.map_or(end - 1, |arg| arg.node.span(ast).start),
        );
        let boundary_comments = self.comments.in_range(gap);
        let opening = self.find_token(gap, boundary_comments, "(")?;
        let prefix_span = Span::new(callee_end, opening);
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
            && !self.diagnostic_range_is_sensitive(Span::new(callee_end, opening + 1));
        if !prefix_belongs_to_arguments {
            self.prepare_first_argument_suppression(first.as_mut(), prefix);
        }
        let mode = if prefix_belongs_to_arguments {
            ContentMode::Formatted
        } else {
            self.trailing_content_mode(prefix_span, BoundaryPunctuation::None, false)
        };
        Some(SourceArguments {
            arguments,
            first,
            delimiters: Span::new(opening, end),
            callee_end,
            prefix,
            prefix_belongs_to_arguments,
            callee_mode: mode,
        })
    }

    fn prepare_first_argument_suppression(
        &self,
        first: Option<&mut SourceFirstArgument>,
        comments: &[Comment],
    ) {
        if !self.has_suppression {
            return;
        }
        let Some(first) = first else {
            return;
        };
        if self.first_argument_prefix_mode(comments) == ContentMode::Verbatim {
            first.mode = ContentMode::Verbatim;
        }
    }

    pub(super) fn format_source_arguments(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let SourceArguments {
            arguments,
            first,
            delimiters,
            callee_end,
            prefix,
            prefix_belongs_to_arguments,
            ..
        } = list;
        let first_mode = first.map_or(ContentMode::Formatted, |argument| argument.mode);
        let opening = delimiters.start;
        let source_end = tail.as_ref().map_or(callee_end, |tail| tail.source_end);
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
            callee_end
        };
        let content = Span::new(content_start, delimiters.end - 1);
        if first.is_none()
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
        {
            self.format_verbatim(Span::new(opening, delimiters.end), writer)?;
            return Ok(None);
        }
        let hugged = arguments.len() == 1
            && self.comments.in_range(content).is_empty()
            && !self.diagnostic_range_is_sensitive(content)
            && first.is_some_and(|argument| match ast.argument_data(argument.node) {
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
