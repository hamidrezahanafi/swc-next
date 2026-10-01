//! Ignore targets resolved at their owning syntax boundaries.

use swc_next_ecma_ast::{Ast, Comment, Span};

use super::{Formatter, source::is_source_whitespace};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ContentMode {
    Formatted,
    Verbatim,
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
