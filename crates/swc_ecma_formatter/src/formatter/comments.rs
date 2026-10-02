//! Bounded, source-order comment consumption and basic comment layout.

use swc_next_ecma_ast::{Comment, CommentKind, Span};

use super::Formatter;
use super::diagnostics::FlatSeparator;
use super::source::{lines, split_line};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum CommentRunLayout {
    Source,
    SourceAfter(u32),
    Leading(u32),
    GroupedLeading(u32),
    FlatLeading(u32),
    Lines,
}

/// A claimed leading run and the source gap owned by its node.
pub(super) struct NodeHead<'ast> {
    pub(super) comments: &'ast [Comment],
    pub(super) content_start: u32,
    pub(super) layout: CommentRunLayout,
    pub(super) gap_end: Option<u32>,
}

/// A pending run whose parent owns the following punctuation and layout scope.
/// The cursor stays at this run until that parent claims it.
pub(super) struct ExpressionTail<'ast> {
    pub(super) source_end: u32,
    pub(super) comments: &'ast [Comment],
}

pub(super) enum CommentSemantics {
    Ordinary,
    Suppression,
    Diagnostic,
    Optimizer,
    TypeCast,
}

/// A syntax owner resolves the source target before relocating comments.
/// Unknown annotation targets and diagnostic coverage retain the source boundary.
#[derive(Clone, Copy)]
pub(super) enum CommentBinding {
    Unbound,
    BoundTo(Span),
    SourceBound,
}

impl CommentBinding {
    pub(super) fn allows_relocation(&self, target: Option<Span>) -> bool {
        match self {
            Self::Unbound => true,
            Self::BoundTo(source) => target == Some(*source),
            Self::SourceBound => false,
        }
    }
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
    kind: CommentKind,
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

    /// Prepares a later boundary without consuming comments inside its child.
    pub(super) fn in_range(&self, span: Span) -> &'ast [Comment] {
        let first = self
            .remaining
            .partition_point(|comment| comment.span.start < span.start);
        let comments = &self.remaining[first..];
        let count = comments.partition_point(|comment| comment.span.end <= span.end);
        &comments[..count]
    }

    /// A prepared boundary becomes claimable after its preceding child is printed.
    pub(super) fn claim(&mut self, comments: &'ast [Comment]) -> Result<(), FormatError> {
        match (self.first(), comments.first()) {
            (_, None) => Ok(()),
            (Some(pending), Some(first)) if pending.span == first.span => {
                self.take(comments.len());
                Ok(())
            }
            (Some(pending), Some(_)) => Err(FormatError::unsupported(
                "comment placement",
                None,
                pending.span,
            )),
            (None, Some(_)) => {
                unreachable!("prepared comment boundary must remain unconsumed")
            }
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn split_separator_comments(
        &self,
        comments: &'ast [Comment],
        before_separator: usize,
        source_end: u32,
        content_start: u32,
    ) -> (&'ast [Comment], &'ast [Comment]) {
        let mut trailing_count = 0;
        let mut previous_end = source_end;
        for (index, comment) in comments.iter().enumerate() {
            if index >= before_separator
                && matches!(self.comment_semantics(*comment), CommentSemantics::TypeCast)
            {
                break;
            }
            if self.line_breaks(previous_end, comment.span.start) != 0 {
                break;
            }
            previous_end = comment.span.end;
            let next_start = comments
                .get(index + 1)
                .map_or(content_start, |next| next.span.start);
            if index < before_separator {
                trailing_count = index + 1;
            }
            if comment.kind == CommentKind::Line
                || self.line_breaks(comment.span.end, next_start) != 0
            {
                trailing_count = index + 1;
                break;
            }
        }
        comments.split_at(trailing_count)
    }

    pub(super) fn comment_semantics(&self, comment: Comment) -> CommentSemantics {
        let raw = self.source(comment.span);
        if super::suppression::is_suppression_comment(self.ast, comment) {
            return CommentSemantics::Suppression;
        }
        if self.comment_is_diagnostic_directive(comment) {
            return CommentSemantics::Diagnostic;
        }
        for (offset, marker) in raw.char_indices() {
            if matches!(marker, '@' | '#') {
                let tag = &raw[offset + 1..];
                if tag.starts_with("__PURE__") || tag.starts_with("__NO_SIDE_EFFECTS__") {
                    return CommentSemantics::Optimizer;
                }
            }
        }
        if self.comment_is_type_cast(comment) {
            CommentSemantics::TypeCast
        } else {
            CommentSemantics::Ordinary
        }
    }

    pub(super) fn comment_run_binding(
        &self,
        comments: impl Iterator<Item = Comment>,
        mut resolve: impl FnMut(Comment) -> CommentBinding,
    ) -> CommentBinding {
        let mut source_target = None;
        for comment in comments {
            let target = match resolve(comment) {
                CommentBinding::Unbound => continue,
                CommentBinding::SourceBound => return CommentBinding::SourceBound,
                CommentBinding::BoundTo(target) => target,
            };
            if source_target.is_some_and(|source| source != target) {
                return CommentBinding::SourceBound;
            }
            source_target = Some(target);
        }
        match source_target {
            Some(target) => CommentBinding::BoundTo(target),
            None => CommentBinding::Unbound,
        }
    }

    pub(super) fn comment_is_type_cast(&self, comment: Comment) -> bool {
        let raw = self.source(comment.span);
        if !raw.starts_with("/**") {
            return false;
        }
        for (offset, marker) in raw.char_indices() {
            if marker != '@' {
                continue;
            }
            let tag = &raw[offset..];
            let after = tag
                .strip_prefix("@type")
                .or_else(|| tag.strip_prefix("@satisfies"));
            if after.is_some_and(|after| {
                after
                    .chars()
                    .next()
                    .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
            }) {
                return true;
            }
        }
        false
    }

    pub(super) fn format_expression_tail(
        &mut self,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some(tail) = tail {
            self.comments.claim(tail.comments)?;
            self.format_comment_run(
                tail.comments,
                CommentRunLayout::SourceAfter(tail.source_end),
                writer,
            )?;
        }
        Ok(())
    }

    fn prepare_comment(&self, comment: Comment) -> Result<PreparedComment<'ast>, FormatError> {
        let source = self.source(comment.span);
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
            kind: comment.kind,
            text,
        })
    }

    pub(super) fn comment_is_alignable(&self, comment: Comment) -> Result<bool, FormatError> {
        Ok(self.prepare_comment(comment)?.is_alignable())
    }

    pub(super) fn format_inline_trailing_comments(
        &self,
        comments: &[Comment],
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let mut previous: Option<PreparedComment<'ast>> = None;
        for &comment in comments {
            let prepared = self.prepare_comment(comment)?;
            match comment.kind {
                CommentKind::Line => writer.line_suffix(self.source(comment.span).trim_end()),
                CommentKind::Block => {
                    let nestled = previous.as_ref().is_some_and(|previous| {
                        previous.span.end == prepared.span.start
                            && previous.is_alignable()
                            && prepared.is_alignable()
                    });
                    if !nestled {
                        writer.space();
                    }
                    prepared.text.write(writer);
                }
            }
            previous = Some(prepared);
        }
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

    fn leading_comment_gap(
        &self,
        comment: &PreparedComment<'ast>,
        next_start: u32,
        grouped: bool,
        writer: &mut Writer<'ast>,
    ) {
        self.line_constraint(comment.span.end, next_start).write(
            writer,
            FlatSeparator::Space,
            |writer| match self.line_breaks(comment.span.end, next_start) {
                0 if comment.kind == CommentKind::Block => writer.space(),
                1 if comment.kind == CommentKind::Block
                    && !self.source_position_starts_line(comment.span.start) =>
                {
                    if grouped {
                        writer.line();
                    } else {
                        writer.space();
                    }
                }
                0 | 1 => writer.hard_line(),
                _ => {
                    writer.hard_line();
                    writer.hard_line();
                }
            },
        );
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
            let line_suffix = match (layout, comment.kind) {
                (CommentRunLayout::SourceAfter(start), CommentKind::Line) => {
                    let end = previous
                        .as_ref()
                        .map_or(start, |previous| previous.span.end);
                    self.line_breaks(end, comment.span.start) == 0
                }
                _ => false,
            };
            if let Some(previous) = previous {
                let nestled = previous.span.end == prepared.span.start
                    && previous.is_alignable()
                    && prepared.is_alignable();
                if !nestled && !line_suffix {
                    match layout {
                        CommentRunLayout::Source | CommentRunLayout::SourceAfter(_) => {
                            self.comment_gap(previous.span.end, prepared.span.start, writer);
                        }
                        CommentRunLayout::Leading(_) | CommentRunLayout::GroupedLeading(_) => {
                            self.leading_comment_gap(
                                &previous,
                                prepared.span.start,
                                matches!(layout, CommentRunLayout::GroupedLeading(_)),
                                writer,
                            );
                        }
                        CommentRunLayout::FlatLeading(_) => self
                            .line_constraint(previous.span.end, prepared.span.start)
                            .write(writer, FlatSeparator::Space, |writer| match previous.kind {
                                CommentKind::Line => writer.hard_line(),
                                CommentKind::Block => writer.space(),
                            }),
                        CommentRunLayout::Lines => {
                            writer.hard_line();
                            if self.line_breaks(previous.span.end, prepared.span.start) == 2 {
                                writer.hard_line();
                            }
                        }
                    }
                }
            } else if let CommentRunLayout::SourceAfter(start) = layout
                && !line_suffix
            {
                self.comment_gap(start, prepared.span.start, writer);
            }
            if line_suffix {
                writer.line_suffix(self.source(comment.span).trim_end());
            } else {
                prepared.text.write(writer);
            }
            previous = Some(prepared);
        }
        match (layout, previous) {
            (CommentRunLayout::Leading(next_start), Some(previous)) => {
                self.leading_comment_gap(&previous, next_start, false, writer)
            }
            (CommentRunLayout::GroupedLeading(next_start), Some(previous)) => {
                self.leading_comment_gap(&previous, next_start, true, writer)
            }
            (CommentRunLayout::FlatLeading(next_start), Some(previous)) => self
                .line_constraint(previous.span.end, next_start)
                .write(writer, FlatSeparator::Space, |writer| match previous.kind {
                    CommentKind::Line => writer.hard_line(),
                    CommentKind::Block => writer.space(),
                }),
            _ => {}
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
            self.format_inline_trailing_comments(comments, writer)?;
            Ok(last.span.end)
        } else {
            Ok(span.start)
        }
    }
}
