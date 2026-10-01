//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use crate::{FormatError, document::Writer};

pub(super) struct CommentCursor<'ast> {
    remaining: &'ast [Comment],
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
    pub(super) fn format_comment(
        &self,
        comment: Comment,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let source = &self.ast.source()[comment.span.start as usize..comment.span.end as usize];
        if source.contains(['\r', '\n']) {
            return Err(FormatError::unsupported(
                "multiline comment",
                None,
                comment.span,
            ));
        }
        if self
            .ast
            .get_utf8(comment.value(self.ast.source().as_bytes()))
            .trim()
            == "prettier-ignore"
        {
            return Err(FormatError::unsupported("suppression", None, comment.span));
        }
        writer.text(source.trim_end());
        Ok(())
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
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous = None;
        for &comment in comments {
            if let Some(end) = previous {
                self.comment_gap(end, comment.span.start, writer);
            }
            self.format_comment(comment, writer)?;
            previous = Some(comment.span.end);
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
            self.format_comment_run(comments, writer)?;
            Ok(last.span.end)
        } else {
            Ok(span.start)
        }
    }
}
