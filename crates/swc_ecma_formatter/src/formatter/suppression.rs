//! Ignore targets resolved at their owning syntax boundaries.

use swc_next_ecma_ast::{Ast, Comment, Span};

use super::{Formatter, comments::CommentSemantics, source::is_source_whitespace};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentMode {
    Formatted,
    Verbatim,
}

#[derive(Clone, Copy)]
pub(super) enum SuppressionPosition {
    OwnLine,
    EndOfLine,
    Inline,
}

#[derive(Clone, Copy)]
pub(super) enum BoundaryPunctuation {
    None,
    Comma,
    Semicolon,
}

pub(super) fn is_suppression_comment(ast: &Ast<'_>, comment: Comment) -> bool {
    let value = ast.get_utf8(comment.value(ast.source().as_bytes()));
    value.trim_matches(|character| is_source_whitespace(character) && character != '\u{85}')
        == "prettier-ignore"
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn suppression_position(
        &self,
        comments: &[Comment],
        following_start: u32,
    ) -> Option<SuppressionPosition> {
        if !self.has_suppression {
            return None;
        }
        let index = comments.iter().position(|&comment| {
            matches!(
                self.comment_semantics(comment),
                CommentSemantics::Suppression
            )
        })?;
        let mut first = index;
        while first > 0
            && self.line_breaks(comments[first - 1].span.end, comments[first].span.start) == 0
        {
            first -= 1;
        }
        if self.source_position_starts_line(comments[first].span.start) {
            return Some(SuppressionPosition::OwnLine);
        }
        let mut last = index;
        while last + 1 < comments.len()
            && self.line_breaks(comments[last].span.end, comments[last + 1].span.start) == 0
        {
            last += 1;
        }
        let following = comments
            .get(last + 1)
            .map_or(following_start, |comment| comment.span.start);
        if self.line_breaks(comments[last].span.end, following) != 0 {
            Some(SuppressionPosition::EndOfLine)
        } else {
            Some(SuppressionPosition::Inline)
        }
    }

    pub(super) fn content_mode(&self, comments: &[Comment]) -> ContentMode {
        if self.has_suppression
            && comments
                .iter()
                .any(|&comment| is_suppression_comment(self.ast, comment))
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    /// A sibling owner supplies the complete boundary and its punctuation.
    /// An own-line directive targets the previous item only at a terminal boundary.
    pub(super) fn trailing_content_mode(
        &self,
        span: Span,
        punctuation: BoundaryPunctuation,
        terminal: bool,
    ) -> ContentMode {
        if !self.has_suppression {
            return ContentMode::Formatted;
        }
        let mut previous = span.start;
        for comment in self.comments.in_range(span) {
            let gap = self.source(Span::new(previous, comment.span.start));
            if !gap.chars().all(|character| {
                is_source_whitespace(character)
                    || matches!(
                        (punctuation, character),
                        (BoundaryPunctuation::Comma, ',') | (BoundaryPunctuation::Semicolon, ';')
                    )
            }) || !terminal && self.line_breaks(previous, comment.span.start) != 0
            {
                break;
            }
            if is_suppression_comment(self.ast, *comment) {
                return ContentMode::Verbatim;
            }
            previous = comment.span.end;
        }
        ContentMode::Formatted
    }

    pub(super) fn format_verbatim(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(span)?;
        self.comments.claim(comments)?;
        writer.literal_text(self.source(span));
        Ok(())
    }
}
