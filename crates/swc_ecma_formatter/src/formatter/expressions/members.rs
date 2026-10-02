//! Member receivers and source-owned lookup boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, IdentifierName, MemberExpression,
    PropertyKeyData, Span,
};

use super::{
    AccessKind, ExpressionContext, ExpressionPosition, Formatter, MemberLayout, TailOwner,
};
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum MemberKey {
    Static(IdentifierName),
    Computed(Expr),
}

enum MemberPunctuation {
    Static {
        key: IdentifierName,
        operator: Span,
        key_gap: Option<Span>,
    },
    Computed {
        key: Expr,
        optional: Option<Span>,
        brackets: Span,
        opening: LineConstraint,
    },
}

impl MemberPunctuation {
    fn operator(&self) -> Span {
        match *self {
            Self::Static { operator, .. } => operator,
            Self::Computed {
                optional: Some(operator),
                ..
            } => operator,
            Self::Computed {
                optional: None,
                brackets,
                ..
            } => Span::new(brackets.start, brackets.start + 1),
        }
    }
}

enum MemberContent {
    Verbatim,
    Formatted {
        receiver: ContentMode,
        key: ContentMode,
    },
}

pub(super) enum PreparedMember {
    Verbatim(MemberExpression),
    Formatted(SourceMember),
}

pub(super) struct SourceMember {
    pub(super) object: Expr,
    property_span: Span,
    punctuation: MemberPunctuation,
    pub(super) access: AccessKind,
    pub(super) receiver_mode: ContentMode,
    key_mode: ContentMode,
}

pub(super) struct MemberBoundary<'ast> {
    before: &'ast [Comment],
    after: &'ast [Comment],
    source_end: u32,
    pub(super) constraint: LineConstraint,
    pub(super) mandatory_line: bool,
    pub(super) empty_line: bool,
    pub(super) has_comments: bool,
}

impl MemberBoundary<'_> {
    pub(super) fn has_before_comments(&self) -> bool {
        !self.before.is_empty()
    }
}

#[derive(Clone, Copy)]
pub(super) enum MemberLookupOwner {
    InlineContext,
    Receiver,
    Expression,
}

impl MemberLookupOwner {
    pub(super) fn from_context(context: ExpressionContext) -> Self {
        if matches!(context.position, ExpressionPosition::MemberAssignmentObject)
            || context.member_layout == MemberLayout::Tight
        {
            Self::InlineContext
        } else if matches!(context.position, ExpressionPosition::MemberObject(_)) {
            Self::Receiver
        } else {
            Self::Expression
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum MemberLookupLayout {
    Inline,
    Grouped,
}

impl MemberLookupLayout {
    pub(super) fn write<'ast>(
        self,
        mandatory_line: bool,
        constraint: LineConstraint,
        writer: &mut Writer<'ast>,
        access: impl FnOnce(&mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        let write = |writer: &mut Writer<'ast>| {
            if mandatory_line {
                writer.hard_line();
            } else {
                constraint.write(writer, FlatSeparator::Empty, |writer| {
                    if matches!(self, Self::Grouped) {
                        writer.soft_line();
                    }
                });
            }
            access(writer)
        };
        match self {
            Self::Inline => write(writer),
            Self::Grouped => writer.group(false, |writer| writer.indent(write)),
        }
    }
}

impl SourceMember {
    pub(super) fn is_computed(&self) -> bool {
        matches!(self.punctuation, MemberPunctuation::Computed { .. })
    }

    pub(super) fn key(&self) -> Option<Expr> {
        match self.punctuation {
            MemberPunctuation::Computed { key, .. } => Some(key),
            MemberPunctuation::Static { .. } => None,
        }
    }

    pub(super) fn name(&self) -> Option<IdentifierName> {
        match self.punctuation {
            MemberPunctuation::Static { key, .. } => Some(key),
            MemberPunctuation::Computed { .. } => None,
        }
    }

    pub(super) fn receiver_context(&self, context: ExpressionContext) -> ExpressionContext {
        let position = if matches!(context.position, ExpressionPosition::MemberAssignmentObject) {
            ExpressionPosition::MemberAssignmentObject
        } else {
            ExpressionPosition::MemberObject(self.access)
        };
        ExpressionContext::at(position)
            .with_expression_start(context.expression_start)
            .with_tail_owner(TailOwner::ParentOperation)
            .with_content_mode(self.receiver_mode)
            .with_member_layout(context.member_layout)
            .with_member_chain_layout(context.member_chain_layout)
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_member(
        &mut self,
        member: MemberExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedMember::Formatted(member) = self.source_member(member)? else {
            return self.format_expression(
                Expr::MemberExpression(member),
                context.with_content_mode(ContentMode::Verbatim),
                writer,
            );
        };
        let tail =
            self.format_expression(member.object, member.receiver_context(context), writer)?;
        let boundary = self.format_member_boundary(&member, tail, writer)?;
        let layout = self.member_lookup_layout(&member, MemberLookupOwner::from_context(context));
        layout.write(
            boundary.mandatory_line,
            boundary.constraint,
            writer,
            |writer| self.format_member_access(&member, boundary, writer),
        )?;
        Ok(None)
    }

    pub(super) fn member_lookup_layout(
        &self,
        member: &SourceMember,
        owner: MemberLookupOwner,
    ) -> MemberLookupLayout {
        if matches!(owner, MemberLookupOwner::InlineContext) || member.is_computed() {
            return MemberLookupLayout::Inline;
        }
        let simple = matches!(owner, MemberLookupOwner::Expression)
            && matches!(
                self.expression_content(member.object).data,
                ExprData::IdentifierReference(_)
            );
        if simple {
            MemberLookupLayout::Inline
        } else {
            MemberLookupLayout::Grouped
        }
    }

    pub(super) fn source_member(
        &self,
        member: MemberExpression,
    ) -> Result<PreparedMember, FormatError> {
        let access = if member.optional(self.ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let property = member.property(self.ast);
        let key = match (
            member.computed(self.ast),
            self.ast.property_key_data(property),
        ) {
            (false, PropertyKeyData::IdentifierName(identifier)) => MemberKey::Static(identifier),
            (true, PropertyKeyData::Expr(expression)) => MemberKey::Computed(expression),
            _ => return Err(self.unsupported("member property", property)),
        };
        let object = member.object(self.ast);
        let object_span = object.span(self.ast);
        let property_span = property.span(self.ast);
        let gap = Span::new(object_span.end, property_span.start);
        let comments = self.comments.in_range(gap);
        let mut punctuation = match key {
            MemberKey::Computed(expression) => {
                let optional = if access == AccessKind::Optional {
                    let start = self
                        .find_token(gap, comments, "?.")
                        .ok_or_else(|| self.unsupported("member optional boundary", member))?;
                    Some(Span::new(start, start + 2))
                } else {
                    None
                };
                let opening_gap = Span::new(optional.map_or(gap.start, |span| span.end), gap.end);
                let opening = self
                    .find_token(opening_gap, comments, "[")
                    .ok_or_else(|| self.unsupported("member opening boundary", member))?;
                MemberPunctuation::Computed {
                    key: expression,
                    optional,
                    brackets: Span::new(opening, member.span(self.ast).end),
                    opening: LineConstraint::Unconstrained,
                }
            }
            MemberKey::Static(identifier) => {
                let token = if access == AccessKind::Optional {
                    "?."
                } else {
                    "."
                };
                let start = self
                    .find_token(gap, comments, token)
                    .ok_or_else(|| self.unsupported("member operator boundary", member))?;
                MemberPunctuation::Static {
                    key: identifier,
                    operator: Span::new(start, start + token.len() as u32),
                    key_gap: (self.has_suppression
                        && self.content_mode(
                            &comments[comments.partition_point(|comment| {
                                comment.span.end <= start + token.len() as u32
                            })..],
                        ) == ContentMode::Verbatim)
                        .then_some(Span::new(start + token.len() as u32, property_span.start)),
                }
            }
        };
        let key_boundary = match punctuation {
            MemberPunctuation::Computed { brackets, .. } => brackets.start,
            MemberPunctuation::Static { operator, .. } => operator.start,
        };
        let content_comments = if self.has_suppression {
            self.member_receiver_comments(object, property_span.start, comments)
        } else {
            comments
        };
        let content = self.member_content(key, key_boundary, property_span.start, content_comments);
        let MemberContent::Formatted { receiver, mut key } = content else {
            return Ok(PreparedMember::Verbatim(member));
        };
        if self.has_suppression
            && let MemberPunctuation::Computed {
                brackets, opening, ..
            } = &mut punctuation
        {
            let after_opening = &comments
                [comments.partition_point(|comment| comment.span.end <= brackets.start + 1)..];
            if self.content_mode(after_opening) == ContentMode::Verbatim {
                let first = after_opening
                    .first()
                    .map_or(property_span.start, |comment| comment.span.start);
                *opening = match self.line_breaks(brackets.start + 1, first) {
                    0 => LineConstraint::SameLine,
                    count => LineConstraint::NewLine(count),
                };
            }
            if key != ContentMode::Verbatim {
                key = self.trailing_content_mode(
                    Span::new(property_span.end, brackets.end - 1),
                    BoundaryPunctuation::None,
                    true,
                );
            }
        }
        Ok(PreparedMember::Formatted(SourceMember {
            object,
            property_span,
            punctuation,
            access,
            receiver_mode: receiver,
            key_mode: key,
        }))
    }

    pub(super) fn format_member_boundary(
        &mut self,
        member: &SourceMember,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<MemberBoundary<'ast>, FormatError> {
        let boundary = self.prepare_member_boundary(member, tail)?;
        self.format_member_trailing(&boundary, writer)?;
        Ok(boundary)
    }

    pub(super) fn prepare_member_boundary(
        &self,
        member: &SourceMember,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<MemberBoundary<'ast>, FormatError> {
        let operator = member.punctuation.operator();
        let computed = member.is_computed();
        let source_end = tail
            .as_ref()
            .map_or(member.object.span(self.ast).end, |tail| tail.source_end);
        let comments = self
            .comments
            .between(Span::new(source_end, member.property_span.start))?;
        let split = comments.partition_point(|comment| comment.span.end <= operator.start);
        let move_lines = !computed
            && comments[split..]
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
            && comments.iter().all(|&comment| {
                matches!(self.comment_semantics(comment), CommentSemantics::Ordinary)
            })
            && !self
                .diagnostic_range_is_sensitive(Span::new(source_end, member.property_span.start));
        let (before, after) = comments.split_at(if move_lines { comments.len() } else { split });
        let previous_end = before.last().map_or(source_end, |comment| comment.span.end);
        let mandatory_line = before
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line);
        let next_start = if move_lines {
            member.property_span.start
        } else {
            operator.start
        };
        Ok(MemberBoundary {
            before,
            after,
            source_end,
            constraint: self.line_constraint(previous_end, next_start),
            mandatory_line,
            empty_line: self.next_line_is_empty(previous_end, next_start),
            has_comments: !comments.is_empty(),
        })
    }

    pub(super) fn format_member_trailing(
        &mut self,
        boundary: &MemberBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(boundary.before)?;
        self.format_comment_run(
            boundary.before,
            CommentRunLayout::SourceAfter(boundary.source_end),
            writer,
        )
    }

    pub(super) fn format_member_access(
        &mut self,
        member: &SourceMember,
        boundary: MemberBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let after = boundary.after;
        match &member.punctuation {
            MemberPunctuation::Computed {
                key: expression,
                optional,
                brackets,
                opening,
            } => {
                if let Some(optional) = optional {
                    writer.text("?.");
                    let before_bracket_count =
                        after.partition_point(|comment| comment.span.end <= brackets.start);
                    let before_bracket = &after[..before_bracket_count];
                    self.comments.claim(before_bracket)?;
                    self.format_comment_run(
                        before_bracket,
                        CommentRunLayout::SourceAfter(optional.end),
                        writer,
                    )?;
                    let end = before_bracket
                        .last()
                        .map_or(optional.end, |comment| comment.span.end);
                    if before_bracket
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    } else {
                        let separator = if before_bracket.is_empty() {
                            FlatSeparator::Empty
                        } else {
                            FlatSeparator::Space
                        };
                        self.line_constraint(end, brackets.start).write(
                            writer,
                            separator,
                            |writer| {
                                if !before_bracket.is_empty() {
                                    writer.space();
                                }
                            },
                        );
                    }
                }
                self.format_member_key(*expression, *brackets, member.key_mode, *opening, writer)
            }
            MemberPunctuation::Static {
                key: identifier,
                operator,
                key_gap,
            } => {
                writer.text(if member.access == AccessKind::Optional {
                    "?."
                } else {
                    "."
                });
                if let Some(gap) = key_gap {
                    self.format_member_source_gap(*gap, writer)?;
                } else {
                    self.comments.claim(after)?;
                    let next = after
                        .first()
                        .map_or(member.property_span.start, |comment| comment.span.start);
                    self.line_constraint(operator.end, next).write(
                        writer,
                        FlatSeparator::Empty,
                        |_| {},
                    );
                    // The dot and key share one tight run; source-only breaks must
                    // not change a block comment's layout on the next pass.
                    self.format_comment_run(
                        after,
                        CommentRunLayout::FlatLeading(member.property_span.start),
                        writer,
                    )?;
                }
                if member.key_mode == ContentMode::Verbatim {
                    self.format_verbatim(member.property_span, writer)?;
                } else {
                    writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                }
                Ok(())
            }
        }
    }

    /// Parentheses do not make a new member comment owner. Include only trivia
    /// after the actual receiver operand, keeping its internal owners separate.
    fn member_receiver_comments(
        &self,
        object: Expr,
        end: u32,
        outside: &'ast [Comment],
    ) -> &'ast [Comment] {
        let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(object) else {
            return outside;
        };
        let mut operand = parenthesized.expression(self.ast);
        while let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(operand) {
            operand = parenthesized.expression(self.ast);
        }
        self.comments
            .in_range(Span::new(operand.span(self.ast).end, end))
    }

    fn member_content(
        &self,
        key: MemberKey,
        key_boundary: u32,
        key_start: u32,
        comments: &[Comment],
    ) -> MemberContent {
        let mut receiver = ContentMode::Formatted;
        let mut key_mode = ContentMode::Formatted;
        if !self.has_suppression {
            return MemberContent::Formatted {
                receiver,
                key: key_mode,
            };
        }
        for (run, position) in self.member_suppressed_comment_runs(comments, key_start) {
            match position {
                SuppressionPosition::OwnLine => {
                    let identifier = match key {
                        MemberKey::Static(_) => true,
                        MemberKey::Computed(expression) => {
                            self.member_key_is_identifier(expression)
                        }
                    };
                    if identifier {
                        return MemberContent::Verbatim;
                    }
                    key_mode = ContentMode::Verbatim;
                }
                SuppressionPosition::EndOfLine => receiver = ContentMode::Verbatim,
                SuppressionPosition::Inline => {
                    if run
                        .last()
                        .is_some_and(|comment| comment.span.end <= key_boundary)
                    {
                        receiver = ContentMode::Verbatim;
                    } else {
                        key_mode = ContentMode::Verbatim;
                    }
                }
            }
        }
        MemberContent::Formatted {
            receiver,
            key: key_mode,
        }
    }

    /// Physical runs stop at punctuation and line breaks and share source placement.
    /// Syntax owners resolve each marker's actual target, including Block/Line differences.
    fn member_suppressed_comment_runs<'a>(
        &'a self,
        comments: &'a [Comment],
        following_start: u32,
    ) -> impl Iterator<Item = (&'a [Comment], SuppressionPosition)> + 'a {
        let comments = if self.has_suppression { comments } else { &[] };
        let mut runs = comments
            .chunk_by(|left, right| {
                self.source(Span::new(left.span.end, right.span.start))
                    .chars()
                    .all(|character| {
                        is_source_whitespace(character)
                            && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    })
            })
            .peekable();
        std::iter::from_fn(move || {
            while let Some(run) = runs.next() {
                if self.content_mode(run) != ContentMode::Verbatim {
                    continue;
                }
                let first = run.first()?;
                let last = run.last()?;
                let following = runs
                    .peek()
                    .and_then(|next| next.first())
                    .map_or(following_start, |comment| comment.span.start);
                let position = if self.source_position_starts_line(first.span.start) {
                    SuppressionPosition::OwnLine
                } else if self.member_comment_run_ends_line(*last, following) {
                    SuppressionPosition::EndOfLine
                } else {
                    SuppressionPosition::Inline
                };
                return Some((run, position));
            }
            None
        })
    }

    fn member_comment_run_ends_line(&self, last: Comment, following: u32) -> bool {
        last.kind == CommentKind::Line
            || self
                .source(Span::new(last.span.end, following))
                .chars()
                .find(|&character| {
                    !is_source_whitespace(character)
                        || matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                })
                .is_some_and(|character| matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
    }

    // The shared delimited source-gap printer is introduced by functional
    // comments later in this history. This is its existing Node branch.
    fn format_member_source_gap(
        &mut self,
        gap: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.in_range(gap);
        let source_end = gap.start
            + self
                .source(gap)
                .trim_end_matches(is_source_whitespace)
                .len() as u32;
        let last = comments
            .last()
            .filter(|comment| comment.span.end == source_end);
        let following = if last.is_some() {
            match self.line_breaks(source_end, gap.end) {
                0 => LineConstraint::SameLine,
                count => LineConstraint::NewLine(count),
            }
        } else {
            self.line_constraint(source_end, gap.end)
        };
        self.format_verbatim(Span::new(gap.start, source_end), writer)?;
        following.write(writer, FlatSeparator::Empty, |_| {});
        Ok(())
    }

    fn member_key_is_identifier(&self, mut expression: Expr) -> bool {
        loop {
            let content = self.expression_content(expression);
            let ExprData::ParenthesizedExpression(parenthesized) = content.data else {
                return matches!(content.data, ExprData::IdentifierReference(_));
            };
            if self.parenthesis_has_type_cast(parenthesized.span(self.ast).start) {
                return false;
            }
            let Some(child) = self.ignored_parentheses_child(parenthesized, ContentMode::Formatted)
            else {
                return false;
            };
            expression = child;
        }
    }

    fn format_member_key(
        &mut self,
        expression: Expr,
        span: Span,
        mode: ContentMode,
        opening: LineConstraint,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let expression_span = expression.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start + 1, expression_span.start))?;
        let compact = matches!(self.ast.expr_data(expression), ExprData::NumericLiteral(_));
        writer.group(false, |writer| {
            writer.text("[");
            let (end, line_comment) = writer.indent(|writer| {
                let first = leading
                    .first()
                    .map_or(expression_span.start, |comment| comment.span.start);
                let opening = if opening == LineConstraint::Unconstrained {
                    self.line_constraint(span.start + 1, first)
                } else {
                    opening
                };
                opening.write(writer, FlatSeparator::Empty, |writer| {
                    if !compact {
                        writer.soft_line();
                    }
                });
                self.comments.claim(leading)?;
                self.format_comment_run(
                    leading,
                    CommentRunLayout::GroupedLeading(expression_span.start),
                    writer,
                )?;
                let tail = self.format_expression(
                    expression,
                    ExpressionContext::at(ExpressionPosition::Nested)
                        .with_tail_owner(TailOwner::ParentOperation)
                        .with_content_mode(mode),
                    writer,
                )?;
                let end = tail
                    .as_ref()
                    .map_or(expression_span.end, |tail| tail.source_end);
                let trailing = self.comments.between(Span::new(end, span.end - 1))?;
                self.comments.claim(trailing)?;
                self.format_comment_run(trailing, CommentRunLayout::SourceAfter(end), writer)?;
                Ok::<_, FormatError>((
                    trailing.last().map_or(end, |comment| comment.span.end),
                    trailing
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            })?;
            if line_comment {
                writer.hard_line();
            } else {
                self.line_constraint(end, span.end - 1).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if !compact {
                            writer.soft_line();
                        }
                    },
                );
            }
            writer.text("]");
            Ok(())
        })
    }
}
