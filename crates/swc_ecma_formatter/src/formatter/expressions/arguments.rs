//! Shared source-ordered argument slots and comma/comment boundaries.

use super::Formatter;
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{Argument, Comment, CommentKind, GetSpan, Span};

#[derive(Clone, Copy)]
pub(super) enum FollowingArgument {
    End,
    Argument(Argument),
    Elision(Option<Argument>),
}

#[derive(Clone, Copy)]
pub(super) enum ArgumentLayout {
    Lines,
    Fill,
}

#[derive(Clone, Copy)]
pub(super) enum Separator {
    Line,
    EmptyLine,
    OwnLineComment,
}

pub(super) struct ItemBoundary {
    pub(super) end: u32,
    pub(super) separator: Separator,
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

    pub(super) fn separator(
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
        let ast = self.ast;
        let has_more = !matches!(following, FollowingArgument::End);
        let following_argument = match following {
            FollowingArgument::Argument(argument) => Some(argument),
            FollowingArgument::Elision(argument) => argument,
            FollowingArgument::End => None,
        };
        let argument_span = argument.span(ast);
        let leading = self
            .comments
            .between(Span::new(span.start, argument_span.start))?;
        let leading = self.comments.take(leading.len());
        let leading_mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            self.content_mode(leading)
        };
        let mode = if leading_mode == ContentMode::Verbatim || !self.has_suppression {
            leading_mode
        } else {
            self.trailing_content_mode(
                Span::new(
                    argument_span.end,
                    following_argument.map_or(span.end, |node| node.span(ast).start),
                ),
                BoundaryPunctuation::Comma,
                following_argument.is_none(),
            )
        };
        self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
        if let Some(last) = leading.last() {
            self.comment_gap(last.span.end, argument_span.start, writer);
        }
        let tail = self.format_argument(argument, mode, writer)?;
        let content_end = tail
            .as_ref()
            .map_or(argument_span.end, |tail| tail.source_end);
        let gap = Span::new(
            content_end,
            following_argument.map_or(span.end, |node| node.span(ast).start),
        );
        let comments = self.comments.between(gap)?;
        let comma = self.find_comma(gap, comments);
        let constrained = self.diagnostic_range_is_sensitive(span);
        // A directive-covered separator owns the comments on its source side.
        let protected_comma =
            comma.filter(|&position| constrained && self.diagnostic_line_is_sensitive(position));
        let trailing_limit = if constrained && matches!(following, FollowingArgument::Elision(_)) {
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
            if following_argument.is_none()
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
        if !has_more || !constrained && following_argument.is_none() {
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
