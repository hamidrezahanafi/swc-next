//! Shared source-ordered argument slots and comma/comment boundaries.

use super::callbacks::{SelectedArgument, SelectedCallback};
use super::{AccessKind, ExpressionContext, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::{
    ArrayCommentOwner, CommentRunLayout, CommentSemantics, ExpressionTail,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, TrailingCommas, document::Writer};
use swc_next_ecma_ast::{
    Argument, ArgumentData, Comment, CommentKind, Expr, ExprData, GetSpan, Span, SubRange,
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
    has_leading_comments: bool,
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
    TrailingLine,
}

#[derive(Clone, Copy)]
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
            if matches!(separator, Separator::TrailingLine) {
                // Fill may choose a flat separator even in an expanded array.
                // A pending owned suffix must flush before the following item.
                writer.hard_line();
                return;
            }
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

    pub(super) fn format_argument_item_with_array_comments(
        &mut self,
        argument: Argument,
        following: FollowingArgument,
        span: Span,
        trailing_comma: bool,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let following = match following {
            FollowingArgument::Argument(argument) => FollowingItem::Node(argument.span(self.ast)),
            FollowingArgument::Elision(argument) => {
                FollowingItem::Elision(argument.map(|node| node.span(self.ast)))
            }
            FollowingArgument::End => FollowingItem::End,
        };
        let item = self.prepare_delimited_item_with_array_comments(
            argument.span(self.ast),
            following,
            span,
            trailing_comma,
            array_comments.as_deref_mut(),
            writer,
        )?;
        let tail = self.format_argument_with_array_comments(
            argument,
            item.mode,
            array_comments.as_deref_mut(),
            writer,
        )?;
        self.finish_delimited_item_with_array_comments(item, tail, array_comments, writer)
    }

    pub(in crate::formatter) fn prepare_delimited_item(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        self.prepare_delimited_item_with_array_comments(
            item_span,
            following,
            span,
            trailing_comma,
            None,
            writer,
        )
    }

    fn prepare_delimited_item_with_array_comments(
        &mut self,
        item_span: Span,
        following: FollowingItem,
        span: Span,
        trailing_comma: bool,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let following_span = following.node();
        let leading = self
            .comments
            .between(Span::new(span.start, item_span.start))?;
        let has_leading_comments = !leading.is_empty();
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
        self.format_comment_run_with_array_comments(
            leading,
            CommentRunLayout::Source,
            array_comments,
            writer,
        )?;
        if let Some(last) = leading.last() {
            self.comment_gap(last.span.end, item_span.start, writer);
        }
        Ok(PreparedItem {
            item_span,
            following,
            span,
            mode,
            has_leading_comments,
            trailing_comma,
        })
    }

    pub(in crate::formatter) fn finish_delimited_item(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        self.finish_delimited_item_with_array_comments(item, tail, None, writer)
    }

    fn finish_delimited_item_with_array_comments(
        &mut self,
        item: PreparedItem,
        tail: Option<ExpressionTail<'ast>>,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let PreparedItem {
            item_span,
            following,
            span,
            mode,
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
        let mut physical_break = false;
        let mut inline_line = false;
        for (index, comment) in comments.iter().enumerate() {
            if comment.span.start >= trailing_limit {
                break;
            }
            if self.line_breaks(inline_end, comment.span.start) != 0 {
                physical_break = true;
                break;
            }
            inline_end = comment.span.end;
            if comment.kind == CommentKind::Line {
                trailing_count = index + 1;
                inline_line = true;
                break;
            }
            if following_span.is_none()
                || comma.is_none_or(|position| comment.span.start < position)
            {
                trailing_count = index + 1;
            }
        }
        let prepare_run = (physical_break || inline_line)
            && mode == ContentMode::Formatted
            && !constrained
            && !matches!(following, FollowingItem::Elision(_));
        let prepared_run = if prepare_run {
            self.prepare_delimited_trailing_run(
                comments,
                content_end,
                trailing_limit,
                following_span.is_some(),
                array_comments.as_deref_mut(),
            )
        } else {
            None
        };
        let canonical_run = prepared_run.is_some();
        let boundary_count = match prepared_run.as_ref() {
            Some(run) => {
                debug_assert!(
                    run.len() >= trailing_count,
                    "a canonical trailing run must include the complete legacy inline prefix",
                );
                run.len()
            }
            None => protected_comma.map_or(trailing_count, |comma| {
                comments
                    .partition_point(|comment| comment.span.end <= comma)
                    .max(trailing_count)
            }),
        };
        let boundary = self.comments.take(boundary_count);
        let (trailing, before_comma) = if canonical_run {
            (&boundary[..0], &boundary[..0])
        } else {
            boundary.split_at(trailing_count)
        };
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_inline_trailing_comments_with_array_comments(
            trailing,
            array_comments.as_deref_mut(),
            writer,
        )?;
        if !before_comma.is_empty() {
            self.format_comment_run_with_array_comments(
                before_comma,
                CommentRunLayout::SourceAfter(trailing_end),
                array_comments.as_deref_mut(),
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
        if let Some(run) = prepared_run {
            // Keep comma and the entire ordinary Block/Line packet together.
            // The next real separator owns its mandatory suffix termination.
            self.format_prepared_trailing_run(run, array_comments.as_deref_mut(), writer)?;
            if !matches!(separator, Separator::EmptyLine) {
                separator = Separator::TrailingLine;
            }
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
                self.format_comment_run_with_array_comments(
                    own_line,
                    CommentRunLayout::Lines,
                    array_comments,
                    writer,
                )?;
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
    pub(super) callee: Expr,
    callee_span: Span,
    prefix: SourceArgumentPrefix<'ast>,
    pub(super) callee_mode: ContentMode,
    pub(super) test_callback: Option<SelectedCallback>,
}

#[derive(Clone, Copy)]
struct SourceFirstArgument {
    node: Argument,
    mode: ContentMode,
}

impl SourceArguments<'_> {
    pub(super) fn access(&self) -> AccessKind {
        match self.prefix {
            SourceArgumentPrefix::Direct(_) => AccessKind::Ordinary,
            SourceArgumentPrefix::Optional { .. } => AccessKind::Optional,
        }
    }

    pub(super) fn has_outside_prefix(&self) -> bool {
        match &self.prefix {
            SourceArgumentPrefix::Direct(prefix) => {
                !prefix.belongs_to_arguments() && !prefix.comments().is_empty()
            }
            SourceArgumentPrefix::Optional { before, after, .. } => {
                !before.is_empty() || !after.belongs_to_arguments() && !after.comments().is_empty()
            }
        }
    }

    pub(super) fn callee_context(&self, context: ExpressionContext) -> ExpressionContext {
        ExpressionContext::at(ExpressionPosition::CallCallee(self.access()))
            .with_expression_start(context.expression_start)
            .with_tail_owner(TailOwner::ParentOperation)
            .with_content_mode(self.callee_mode)
            .with_member_chain_layout(context.member_chain_layout)
    }
}

enum ArgumentPrefix<'ast> {
    Outside(&'ast [Comment]),
    Arguments(&'ast [Comment]),
}

impl<'ast> ArgumentPrefix<'ast> {
    fn comments(&self) -> &'ast [Comment] {
        match self {
            Self::Outside(comments) | Self::Arguments(comments) => comments,
        }
    }

    fn belongs_to_arguments(&self) -> bool {
        matches!(self, Self::Arguments(_))
    }
}

enum SourceArgumentPrefix<'ast> {
    Direct(ArgumentPrefix<'ast>),
    Optional {
        operator: Span,
        before: &'ast [Comment],
        after: ArgumentPrefix<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arguments(
        &self,
        arguments: SubRange<Argument>,
        callee: Expr,
        end: u32,
        access: AccessKind,
    ) -> Option<SourceArguments<'ast>> {
        let ast = self.ast;
        let callee_span = callee.span(ast);
        let mut first = arguments.iter().next().map(|id| SourceFirstArgument {
            node: ast.get_node_in_sub_range(id),
            mode: ContentMode::Formatted,
        });
        let gap = Span::new(
            callee_span.end,
            first.map_or(end - 1, |arg| arg.node.span(ast).start),
        );
        let boundary_comments = self.comments.in_range(gap);
        let (opening, prefix, callee_mode) = match access {
            AccessKind::Ordinary => {
                let opening = self.find_token(gap, boundary_comments, "(")?;
                let span = Span::new(callee_span.end, opening);
                let prefix = self.argument_prefix(first.as_mut(), span);
                let mode = if prefix.belongs_to_arguments() {
                    ContentMode::Formatted
                } else {
                    self.trailing_content_mode(span, BoundaryPunctuation::None, false)
                };
                (opening, SourceArgumentPrefix::Direct(prefix), mode)
            }
            AccessKind::Optional => {
                let operator_start = self.find_token(gap, boundary_comments, "?.")?;
                let operator = Span::new(operator_start, operator_start + 2);
                let opening_gap = Span::new(operator.end, gap.end);
                let opening = self.find_token(opening_gap, boundary_comments, "(")?;
                let before_span = Span::new(callee_span.end, operator.start);
                self.prepare_first_argument_suppression(
                    first.as_mut(),
                    self.comments.in_range(before_span),
                );
                let before = self.comments.in_range(before_span);
                let before_mode =
                    self.trailing_content_mode(before_span, BoundaryPunctuation::None, false);
                let after_span = Span::new(operator.end, opening);
                let after = self.argument_prefix(first.as_mut(), after_span);
                let after_mode = if after.belongs_to_arguments() {
                    ContentMode::Formatted
                } else {
                    self.trailing_content_mode(after_span, BoundaryPunctuation::None, false)
                };
                if !after.belongs_to_arguments()
                    && self.content_mode(after.comments()) == ContentMode::Verbatim
                    && (first.is_some() || after_mode != ContentMode::Verbatim)
                {
                    return None;
                }
                let mode = if before_mode == ContentMode::Verbatim
                    || after_mode == ContentMode::Verbatim
                {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                };
                let prefix = SourceArgumentPrefix::Optional {
                    operator,
                    before,
                    after,
                };
                (opening, prefix, mode)
            }
        };
        Some(SourceArguments {
            arguments,
            first,
            delimiters: Span::new(opening, end),
            callee,
            callee_span,
            prefix,
            callee_mode,
            test_callback: None,
        })
    }

    pub(super) fn format_argument_with_function_fact(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        missing_function: &mut Option<&mut bool>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if missing_function.is_none() {
            return self.format_argument(argument, mode, writer);
        }
        let data = self.ast.argument_data(argument);
        let has_function = match data {
            ArgumentData::Expr(expression) => matches!(
                self.expression_content(expression).data,
                ExprData::Function(_) | ExprData::ArrowFunctionExpression(_)
            ),
            ArgumentData::SpreadElement(_) => false,
        };
        if has_function && let Some(function) = missing_function.take() {
            *function = true;
        }
        self.format_argument_data(data, mode, writer)
    }

    fn argument_prefix(
        &self,
        first: Option<&mut SourceFirstArgument>,
        span: Span,
    ) -> ArgumentPrefix<'ast> {
        let comments = self.comments.in_range(span);
        let belongs_to_arguments = first.is_some()
            && !comments.is_empty()
            && comments.iter().all(|&comment| {
                comment.kind == CommentKind::Block
                    && matches!(
                        self.comment_semantics(comment),
                        CommentSemantics::Ordinary | CommentSemantics::Suppression
                    )
            })
            && !self.diagnostic_range_is_sensitive(Span::new(span.start, span.end + 1));
        let prefix = if belongs_to_arguments {
            ArgumentPrefix::Arguments(comments)
        } else {
            ArgumentPrefix::Outside(comments)
        };
        if !belongs_to_arguments {
            self.prepare_first_argument_suppression(first, comments);
        }
        prefix
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
        self.format_source_arguments_with_function_fact(list, tail, None, writer)
    }

    pub(super) fn format_source_arguments_with_function_fact(
        &mut self,
        list: SourceArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        mut missing_function: Option<&mut bool>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let SourceArguments {
            arguments,
            first,
            delimiters,
            callee_span,
            prefix,
            test_callback,
            ..
        } = list;
        let first_mode = first.map_or(ContentMode::Formatted, |argument| argument.mode);
        let opening = delimiters.start;
        let source_end = tail
            .as_ref()
            .map_or(callee_span.end, |tail| tail.source_end);
        let content_start = match prefix {
            SourceArgumentPrefix::Direct(prefix) => {
                let before = self.comments.between(Span::new(source_end, opening))?;
                let prefix_belongs_to_arguments = prefix.belongs_to_arguments();
                let move_tail = first.is_some()
                    && (prefix.comments().is_empty() || prefix_belongs_to_arguments)
                    && tail.as_ref().is_some_and(|tail| {
                        tail.comments.iter().all(|&comment| {
                            comment.kind == CommentKind::Block
                                && matches!(
                                    self.comment_semantics(comment),
                                    CommentSemantics::Ordinary
                                )
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
                self.format_comment_run(
                    outside,
                    CommentRunLayout::SourceAfter(source_end),
                    writer,
                )?;
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
                if inside.is_empty() {
                    opening + 1
                } else if move_tail {
                    source_end
                } else {
                    callee_span.end
                }
            }
            SourceArgumentPrefix::Optional {
                operator,
                before,
                after,
            } => {
                let before = if tail.is_some() {
                    self.comments
                        .between(Span::new(source_end, operator.start))?
                } else {
                    before
                };
                self.comments.claim(before)?;
                self.format_comment_run(before, CommentRunLayout::SourceAfter(source_end), writer)?;
                let end = before.last().map_or(source_end, |comment| comment.span.end);
                if before
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line();
                } else {
                    self.line_constraint(end, operator.start).write(
                        writer,
                        FlatSeparator::Empty,
                        |_| {},
                    );
                }
                writer.text("?.");
                if after.belongs_to_arguments() {
                    operator.end
                } else {
                    let after = after.comments();
                    self.comments.claim(after)?;
                    self.format_comment_run(
                        after,
                        CommentRunLayout::SourceAfter(operator.end),
                        writer,
                    )?;
                    let end = after
                        .last()
                        .map_or(operator.end, |comment| comment.span.end);
                    if after
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    } else {
                        self.line_constraint(end, opening).write(
                            writer,
                            FlatSeparator::Empty,
                            |_| {},
                        );
                    }
                    opening + 1
                }
            }
        };
        let content = Span::new(content_start, delimiters.end - 1);
        if first.is_none() {
            let comments = self.comments.in_range(content);
            if self.content_mode(comments) == ContentMode::Verbatim {
                self.format_verbatim(Span::new(opening, delimiters.end), writer)?;
                return Ok(None);
            }
            if comments.is_empty() {
                writer.text("(");
                self.format_argument_dangling(content, writer)?;
                writer.text(")");
                return Ok(None);
            }
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
        if let Some(selected) = test_callback
            .map(SelectedArgument::Callback)
            .or_else(|| self.select_argument_layout(arguments, !hugged))
        {
            return self.format_selected_arguments(
                arguments,
                (content, first_mode),
                expand,
                selected,
                missing_function,
                writer,
            );
        }
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
                let mut previous = None;
                while let Some(argument) = arguments.next() {
                    let mut item = self.prepare_argument_slot(
                        argument,
                        arguments.peek().copied(),
                        previous,
                        content,
                        !hugged && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if previous.is_none() && first_mode == ContentMode::Verbatim {
                        item.mode = first_mode;
                    }
                    let tail = self.format_argument_with_function_fact(
                        argument,
                        item.mode,
                        &mut missing_function,
                        writer,
                    )?;
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                let end = previous.map_or(content.start, |boundary| boundary.end);
                self.format_argument_dangling(Span::new(end, content.end), writer)
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
    pub(super) fn prepare_argument_slot(
        &mut self,
        argument: Argument,
        following: Option<Argument>,
        previous: Option<ItemBoundary>,
        content: Span,
        trailing_comma: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedItem, FormatError> {
        let span = argument.span(self.ast);
        let end = previous.map_or(content.start, |boundary| boundary.end);
        if let Some(previous) = previous {
            let next = self
                .comments
                .between(Span::new(end, span.start))?
                .first()
                .map_or(span.start, |comment| comment.span.start);
            ArgumentLayout::Lines.separator(
                previous.separator,
                self.line_constraint(end, next),
                writer,
            );
        }
        let following = following.map_or(FollowingItem::End, |argument| {
            FollowingItem::Node(argument.span(self.ast))
        });
        self.prepare_delimited_item(
            span,
            following,
            Span::new(end, content.end),
            trailing_comma,
            writer,
        )
    }

    /// Reads the unconsumed outside boundary of this actual argument slot.
    /// Object-internal comments remain owned by the value formatter.
    pub(super) fn argument_has_boundary_comments(&self, item: &PreparedItem) -> bool {
        if item.has_leading_comments {
            return true;
        }
        let end = item
            .following
            .node()
            .map_or(item.span.end, |span| span.start);
        !self
            .comments
            .in_range(Span::new(item.item_span.end, end))
            .is_empty()
    }

    pub(super) fn format_argument_dangling(
        &mut self,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let dangling = self.comments.between(span)?;
        self.comments.claim(dangling)?;
        if dangling
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            writer.break_parent();
        }
        self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
        Ok(dangling
            .last()
            .map_or(span.start, |comment| comment.span.end))
    }
}
