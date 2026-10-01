//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::source::{lines, split_line};
use crate::{FormatError, document::Writer};

pub(super) enum CommentRunLayout {
    Source,
    Lines,
}

pub(super) struct CommentCursor<'ast> {
    remaining: &'ast [Comment],
}

enum CommentText<'ast> {
    Inline(&'ast str),
    Literal {
        first: &'ast str,
        rest: &'ast str,
    },
    Aligned {
        first: &'ast str,
        rest: &'ast str,
        jsdoc: bool,
    },
}

struct PreparedComment<'ast> {
    span: Span,
    text: CommentText<'ast>,
}

impl PreparedComment<'_> {
    fn is_alignable(&self) -> bool {
        matches!(self.text, CommentText::Aligned { .. })
    }
}

impl<'ast> CommentText<'ast> {
    fn write(&self, writer: &mut Writer<'ast>) {
        match *self {
            Self::Inline(text) => writer.text(text),
            Self::Literal { first, rest } => {
                writer.text(first);
                for line in lines(rest) {
                    writer.literal_line();
                    writer.text(line);
                }
            }
            Self::Aligned { first, rest, jsdoc } => {
                writer.text(first.trim_end());
                for line in lines(rest) {
                    writer.hard_line();
                    writer.space();
                    let trimmed = line.trim();
                    writer.text(trimmed);
                    if jsdoc && trimmed != "*" && line.ends_with("  ") {
                        // Markdown uses two trailing spaces as a hard line break.
                        writer.text("  ");
                    }
                }
            }
        }
    }
}

impl<'ast> CommentCursor<'ast> {
    pub(super) fn new(comments: &'ast [Comment]) -> Self {
        debug_assert!(
            comments
                .windows(2)
                .all(|pair| pair[0].span.end <= pair[1].span.start),
            "AST comments must remain in source order",
        );
        Self {
            remaining: comments,
        }
    }

    pub(super) fn first(&self) -> Option<Comment> {
        self.remaining.first().copied()
    }

    pub(super) fn between(&self, span: Span) -> Result<&'ast [Comment], FormatError> {
        if let Some(comment) = self.first()
            && comment.span.start < span.start
        {
            return Err(FormatError::unsupported(
                "comment placement",
                None,
                comment.span,
            ));
        }
        let count = self
            .remaining
            .iter()
            .take_while(|comment| comment.span.end <= span.end)
            .count();
        Ok(&self.remaining[..count])
    }

    pub(super) fn take(&mut self, count: usize) -> &'ast [Comment] {
        let (claimed, remaining) = self.remaining.split_at(count);
        self.remaining = remaining;
        claimed
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn prepare_comment(&self, comment: Comment) -> Result<PreparedComment<'ast>, FormatError> {
        let source = &self.ast.source()[comment.span.start as usize..comment.span.end as usize];
        if self
            .ast
            .get_utf8(comment.value(self.ast.source().as_bytes()))
            .trim()
            == "prettier-ignore"
        {
            return Err(FormatError::unsupported("suppression", None, comment.span));
        }
        let (first, rest) = split_line(source);
        let text = match rest {
            None => CommentText::Inline(source.trim_end()),
            Some(rest) if lines(rest).all(|line| line.trim_start().starts_with('*')) => {
                CommentText::Aligned {
                    first,
                    rest,
                    jsdoc: source.starts_with("/**") && !source.starts_with("/***"),
                }
            }
            Some(rest) => CommentText::Literal { first, rest },
        };
        Ok(PreparedComment {
            span: comment.span,
            text,
        })
    }

    pub(super) fn comment_gap(&self, start: u32, end: u32, writer: &mut Writer<'ast>) {
        match self.line_breaks(start, end) {
            0 => writer.space(),
            1 => writer.hard_line(),
            _ => {
                writer.hard_line();
                writer.hard_line();
            }
        }
    }

    pub(super) fn format_comment_run(
        &self,
        comments: &[Comment],
        layout: CommentRunLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            if let Some(previous) = previous {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if !nestled {
                    match layout {
                        CommentRunLayout::Source => {
                            self.comment_gap(previous.span.end, prepared.span.start, writer);
                        }
                        CommentRunLayout::Lines => {
                            writer.hard_line();
                            if self.line_breaks(previous.span.end, prepared.span.start) == 2 {
                                writer.hard_line();
                            }
                        }
                    }
                }
            }
            prepared.text.write(writer);
            previous = Some(prepared);
        }
        Ok(())
    }

    pub(super) fn format_trailing_comments(
        &mut self,
        span: Span,
        has_following: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let comments = self.comments.between(span)?;
        let mut end = span.start;
        let mut count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if self.line_breaks(end, comment.span.start) != 0 {
                break;
            }
            let following = comments
                .get(index + 1)
                .map_or(span.end, |next| next.span.start);
            if comment.kind == CommentKind::Line
                || self.line_breaks(comment.span.end, following) != 0
            {
                count = index + 1;
                break;
            }
            end = comment.span.end;
            if !has_following {
                count = index + 1;
            }
        }
        let comments = self.comments.take(count);
        if let Some(last) = comments.last() {
            writer.space();
            self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
            Ok(last.span.end)
        } else {
            Ok(span.start)
        }
    }
}
