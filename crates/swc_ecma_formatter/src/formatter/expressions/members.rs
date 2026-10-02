//! Member receivers and source-owned lookup boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, IdentifierName, MemberExpression,
    PropertyKeyData, Span,
};

use super::{ExpressionContext, ExpressionPosition, Formatter, MemberLayout, TailOwner};
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum MemberKey {
    Static(IdentifierName),
    Computed(Expr),
}

#[derive(Clone, Copy)]
enum MemberSuppressionPosition {
    OwnLine,
    EndOfLine,
    Inline,
}

enum MemberContent {
    Verbatim,
    Formatted {
        receiver: ContentMode,
        key: ContentMode,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_member(
        &mut self,
        member: MemberExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if member.optional(self.ast) {
            return Err(self.unsupported("optional member", member));
        }
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
        let token = if member.computed(self.ast) { "[" } else { "." };
        let Some(operator) = self.find_token(gap, comments, token) else {
            return Err(self.unsupported("member operator boundary", member));
        };
        let content_comments = if self.has_suppression {
            self.member_receiver_comments(object, property_span.start, comments)
        } else {
            comments
        };
        let content = self.member_content(key, operator, property_span.start, content_comments);
        let MemberContent::Formatted {
            receiver,
            key: mut key_mode,
        } = content
        else {
            return self.format_expression(
                Expr::MemberExpression(member),
                context.with_content_mode(ContentMode::Verbatim),
                writer,
            );
        };
        let target = matches!(context.position, ExpressionPosition::MemberAssignmentObject);
        let position = if target {
            ExpressionPosition::MemberAssignmentObject
        } else {
            ExpressionPosition::MemberObject
        };
        let mut opening = LineConstraint::Unconstrained;
        if self.has_suppression && matches!(key, MemberKey::Computed(_)) {
            let after_opening =
                &comments[comments.partition_point(|comment| comment.span.end <= operator + 1)..];
            if self.content_mode(after_opening) == ContentMode::Verbatim {
                let first = after_opening
                    .first()
                    .map_or(property_span.start, |comment| comment.span.start);
                opening = match self.line_breaks(operator + 1, first) {
                    0 => LineConstraint::SameLine,
                    count => LineConstraint::NewLine(count),
                };
            }
            if key_mode != ContentMode::Verbatim {
                key_mode = self.trailing_content_mode(
                    Span::new(property_span.end, member.span(self.ast).end - 1),
                    BoundaryPunctuation::None,
                    true,
                );
            }
        }
        let key_gap = (self.has_suppression
            && matches!(key, MemberKey::Static(_))
            && self.content_mode(
                &comments[comments.partition_point(|comment| comment.span.end <= operator + 1)..],
            ) == ContentMode::Verbatim)
            .then_some(Span::new(operator + 1, property_span.start));
        let tail = self.format_expression(
            object,
            ExpressionContext::at(position)
                .with_statement_start(context.statement_start)
                .with_tail_owner(TailOwner::ParentOperation)
                .with_content_mode(receiver)
                .with_member_layout(context.member_layout),
            writer,
        )?;
        let source_end = tail
            .as_ref()
            .map_or(object_span.end, |tail| tail.source_end);
        let comments = self
            .comments
            .between(Span::new(source_end, property_span.start))?;
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let move_lines = !member.computed(self.ast)
            && comments[split..]
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
            && comments.iter().all(|&comment| {
                matches!(self.comment_semantics(comment), CommentSemantics::Ordinary)
            })
            && !self.diagnostic_range_is_sensitive(Span::new(source_end, property_span.start));
        let (before, after) = comments.split_at(if move_lines { comments.len() } else { split });
        self.comments.claim(before)?;
        self.format_comment_run(before, CommentRunLayout::SourceAfter(source_end), writer)?;
        let previous_end = before.last().map_or(source_end, |comment| comment.span.end);
        let mandatory_line = before
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line);
        let simple = matches!(
            self.expression_content(object).data,
            ExprData::IdentifierReference(_)
        ) && !matches!(context.position, ExpressionPosition::MemberObject);
        let inline = target
            || context.member_layout == MemberLayout::Tight
            || member.computed(self.ast)
            || simple;
        let breakable = !inline;
        let mut suffix = |writer: &mut Writer<'ast>| {
            if mandatory_line {
                writer.hard_line();
            } else {
                self.line_constraint(
                    previous_end,
                    if move_lines {
                        property_span.start
                    } else {
                        operator
                    },
                )
                .write(writer, FlatSeparator::Empty, |writer| {
                    if breakable {
                        writer.soft_line();
                    }
                });
            }
            match key {
                MemberKey::Computed(expression) => self.format_member_key(
                    expression,
                    Span::new(operator, member.span(self.ast).end),
                    key_mode,
                    opening,
                    writer,
                ),
                MemberKey::Static(identifier) => {
                    writer.text(".");
                    if let Some(gap) = key_gap {
                        self.format_member_source_gap(gap, writer)?;
                    } else {
                        self.comments.claim(after)?;
                        let next = after
                            .first()
                            .map_or(property_span.start, |comment| comment.span.start);
                        self.line_constraint(operator + 1, next).write(
                            writer,
                            FlatSeparator::Empty,
                            |_| {},
                        );
                        // The dot and key share one tight run; source-only breaks must
                        // not change a block comment's layout on the next pass.
                        self.format_comment_run(
                            after,
                            CommentRunLayout::FlatLeading(property_span.start),
                            writer,
                        )?;
                    }
                    if key_mode == ContentMode::Verbatim {
                        self.format_verbatim(property_span, writer)?;
                    } else {
                        writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                    }
                    Ok(())
                }
            }
        };
        if !breakable || mandatory_line && simple {
            suffix(writer)?;
        } else {
            writer.group(false, |writer| writer.indent(suffix))?;
        }
        Ok(None)
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
                MemberSuppressionPosition::OwnLine => {
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
                MemberSuppressionPosition::EndOfLine => receiver = ContentMode::Verbatim,
                MemberSuppressionPosition::Inline => {
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
    ) -> impl Iterator<Item = (&'a [Comment], MemberSuppressionPosition)> + 'a {
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
                    MemberSuppressionPosition::OwnLine
                } else if self.member_comment_run_ends_line(*last, following) {
                    MemberSuppressionPosition::EndOfLine
                } else {
                    MemberSuppressionPosition::Inline
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
