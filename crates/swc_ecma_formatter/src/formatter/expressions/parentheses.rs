//! Source delimiter ownership and semantic boundaries.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, ParenthesizedExpression, Span,
};

use super::{
    ExpressionContext, ExpressionPosition, ExpressionStart, Formatter, ParenthesesOwner, TailOwner,
};
use crate::formatter::comments::{
    ArrayCommentOwner, CommentBinding, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, is_suppression_comment};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) struct SourceParentheses {
    pub(in crate::formatter) outer: Span,
    pub(in crate::formatter) inner: Span,
    pub(in crate::formatter) inner_target: ParenthesisInner,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum ParenthesisInner {
    Expression(Expr),
    SourcePair,
    Unresolved,
}

pub(in crate::formatter) struct ExpressionContent {
    pub(in crate::formatter) expression: Expr,
    pub(in crate::formatter) data: ExprData,
    pub(super) parentheses: Option<SourceParentheses>,
}

/// Numeric selection retains the source decision and a separate pure reference view.
#[derive(Clone, Copy)]
pub(super) enum NumericContentView {
    Source,
    Reference,
    Both,
}

pub(super) struct NumericArgumentContent {
    pub(super) source: Option<ExprData>,
    pub(super) reference: Option<NumericReferenceRoot>,
}

pub(super) struct NumericReferenceRoot {
    pub(super) expression: Expr,
    pub(super) data: ExprData,
}

#[derive(Clone, Copy)]
enum ParenthesisCastPosition {
    Outer,
    Inner,
}

struct ExpressionHeadAnnotation {
    index: usize,
    content: Option<ExprData>,
    guard_prefix: Option<LineConstraint>,
}

enum SourceCastState {
    Pending,
    Known(Option<Comment>),
}

/// Move this child boundary into the next wrapper iteration instead of
/// reconstructing its comment prefix or an already computed source cast.
struct NumericCastBoundary<'ast> {
    start: u32,
    before: &'ast [Comment],
    source: SourceCastState,
}

enum RetainedParentheses {
    Cast,
    Annotation,
    Operand,
}

struct RetainedSourceParentheses {
    boundary: SourceParentheses,
    kind: RetainedParentheses,
    hug: bool,
    leading_mode: Option<ContentMode>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_expression_with_leading_comments(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_with_leading_comments_with_array_comments(
            expression, head, context, None, writer,
        )
    }

    fn format_expression_with_leading_comments_with_array_comments(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        mut context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if !head.comments.is_empty() {
            context.arrow_layout = context.arrow_layout.with_leading_comments();
        }
        let content = if !head.comments.is_empty()
            && context.parentheses_owner == ParenthesesOwner::Expression
            && self.has_type_casts
            && head
                .comments
                .iter()
                .any(|&comment| self.comment_is_type_cast(comment))
        {
            Some(self.expression_content(expression).data)
        } else {
            None
        };
        let enclose_comments = content.is_some_and(|data| {
            self.expression_needs_parentheses(data, context.position, context.content_mode)
                == Some(true)
        });
        if !enclose_comments {
            context.expression_start = self.format_expression_head_with_array_comments(
                expression,
                head,
                context,
                content,
                array_comments.as_deref_mut(),
                writer,
            )?;
            return self.format_expression_with_array_comments(
                expression,
                context,
                array_comments,
                writer,
            );
        }
        self.format_parenthesized_expression_with_array_comments(
            expression,
            head,
            context,
            array_comments,
            writer,
        )
    }

    pub(super) fn format_expression_head(
        &self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        self.format_expression_head_with_array_comments(
            expression, head, context, None, None, writer,
        )
    }

    fn format_expression_head_with_array_comments(
        &self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        content: Option<ExprData>,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if head.comments.is_empty() {
            return Ok(context.expression_start);
        }
        let ignored_head = self.has_suppression && context.content_mode == ContentMode::Verbatim;
        let protected_start = if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && (self.has_target_annotations || ignored_head)
        {
            head.comments.iter().position(|&comment| {
                self.has_target_annotations && self.comment_has_target_annotation(comment)
                    || ignored_head && is_suppression_comment(self.ast, comment)
            })
        } else {
            None
        };
        if let Some(index) = protected_start {
            let guard_prefix = ignored_head.then(|| {
                if self.source_position_starts_line(head.comments[index].span.start) {
                    LineConstraint::NewLine(1)
                } else {
                    LineConstraint::SameLine
                }
            });
            return self.format_annotated_expression_head(
                expression,
                head,
                context,
                ExpressionHeadAnnotation {
                    index,
                    content,
                    guard_prefix,
                },
                array_comments,
                writer,
            );
        }
        self.format_node_head_with_array_comments(
            head,
            context.expression_start,
            array_comments,
            writer,
        )
    }

    fn format_annotated_expression_head(
        &self,
        expression: Expr,
        mut head: NodeHead<'ast>,
        mut context: ExpressionContext,
        annotation: ExpressionHeadAnnotation,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if self.expression_needs_asi_guard(expression, context, annotation.content)? {
            let (prefix, protected) = head.comments.split_at(annotation.index);
            self.format_node_head_with_array_comments(
                NodeHead {
                    comments: prefix,
                    content_start: protected[0].span.start,
                    layout: head.layout,
                    gap_end: Some(protected[0].span.start),
                },
                ExpressionStart::None,
                array_comments.as_deref_mut(),
                writer,
            )?;
            writer.text(";");
            if let Some(prefix) = annotation.guard_prefix {
                prefix.write(writer, FlatSeparator::Empty, |_| {});
            } else if protected
                .iter()
                .any(|comment| comment.kind == CommentKind::Line)
            {
                writer.space();
            }
            context.expression_start = ExpressionStart::StatementGuarded;
            head.comments = protected;
        }
        self.format_node_head_with_array_comments(
            head,
            context.expression_start,
            array_comments,
            writer,
        )
    }

    pub(in crate::formatter) fn format_node_head(
        &self,
        head: NodeHead<'ast>,
        start: ExpressionStart,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        self.format_node_head_with_array_comments(head, start, None, writer)
    }

    fn format_node_head_with_array_comments(
        &self,
        head: NodeHead<'ast>,
        start: ExpressionStart,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        let start = self.format_expression_leading_comments_with_array_comments(
            head.comments,
            head.content_start,
            head.layout,
            start,
            array_comments,
            writer,
        )?;
        if let Some(end) = head.gap_end
            && let Some(last) = head.comments.last()
        {
            self.comment_gap(last.span.end, end, writer);
        }
        Ok(start)
    }

    pub(super) fn format_parenthesized_expression(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_parenthesized_expression_with_array_comments(
            expression, head, context, None, writer,
        )
    }

    fn format_parenthesized_expression_with_array_comments(
        &mut self,
        expression: Expr,
        head: NodeHead<'ast>,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        // Keep leading tags with their source targets inside the added parentheses.
        writer.group(false, |writer| {
            writer.text("(");
            writer.indent(|writer| {
                writer.soft_line();
                self.format_comment_run_with_array_comments(
                    head.comments,
                    CommentRunLayout::FlatLeading(head.content_start),
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                let tail = self.format_expression_with_array_comments(
                    expression,
                    ExpressionContext::at(ExpressionPosition::Nested)
                        .with_content_mode(context.content_mode)
                        .with_tail_owner(context.tail_owner)
                        .with_member_chain_layout(context.member_chain_layout)
                        .inside_parentheses(),
                    array_comments.as_deref_mut(),
                    writer,
                )?;
                self.format_expression_tail_with_array_comments(tail, array_comments, writer)
            })?;
            writer.soft_line();
            writer.text(")");
            Ok(None)
        })
    }

    /// Collection may cross only a source wrapper whose immediate consumer can
    /// remove it without changing scope, comments, directives, or content mode.
    pub(super) fn chain_parentheses_operand(
        &self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
    ) -> Option<Expr> {
        let child = parenthesized.expression(self.ast);
        let boundary = SourceParentheses {
            outer: parenthesized.span(self.ast),
            inner: child.span(self.ast),
            inner_target: ParenthesisInner::Expression(child),
        };
        if context.content_mode != ContentMode::Formatted
            || !self
                .comments
                .in_range(Span::new(boundary.outer.start, boundary.inner.start))
                .is_empty()
            || !self
                .comments
                .in_range(Span::new(boundary.inner.end, boundary.outer.end))
                .is_empty()
            || self.diagnostic_range_is_sensitive(boundary.outer)
            || !self.parentheses_are_removable(boundary)
            || self.expression_needs_parentheses(
                self.ast.expr_data(child),
                context.position,
                context.content_mode,
            ) != Some(false)
        {
            return None;
        }
        Some(child)
    }

    pub(in crate::formatter) fn expression_content(&self, expression: Expr) -> ExpressionContent {
        let mut inner = expression;
        let data = loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                break data;
            };
            let child = parenthesized.expression(self.ast);
            if self
                .expression_needs_parentheses(
                    self.ast.expr_data(child),
                    ExpressionPosition::Nested,
                    ContentMode::Formatted,
                )
                .is_none()
                || !self.parentheses_are_removable(SourceParentheses {
                    outer: parenthesized.span(self.ast),
                    inner: child.span(self.ast),
                    inner_target: ParenthesisInner::Expression(child),
                })
            {
                break data;
            }
            inner = child;
        };
        let boundary = (inner != expression).then(|| SourceParentheses {
            outer: expression.span(self.ast),
            inner: inner.span(self.ast),
            inner_target: ParenthesisInner::Expression(inner),
        });
        ExpressionContent {
            expression: inner,
            data,
            parentheses: boundary,
        }
    }

    /// The selected Multiple-array owner drops a view once its decision is false.
    /// A requested source view keeps the existing expression_content stopping rule.
    pub(super) fn numeric_argument_content(
        &self,
        expression: Expr,
        view: NumericContentView,
    ) -> NumericArgumentContent {
        let mut inner = expression;
        let mut source = None;
        let mut source_pending =
            matches!(view, NumericContentView::Source | NumericContentView::Both);
        let mut reference = matches!(
            view,
            NumericContentView::Reference | NumericContentView::Both
        );
        let mut pending_cast = None;
        loop {
            let data = self.ast.expr_data(inner);
            let ExprData::ParenthesizedExpression(parenthesized) = data else {
                if source_pending {
                    source = Some(data);
                }
                return NumericArgumentContent {
                    source,
                    reference: reference.then_some(NumericReferenceRoot {
                        expression: inner,
                        data,
                    }),
                };
            };
            let child = parenthesized.expression(self.ast);
            let boundary = SourceParentheses {
                outer: parenthesized.span(self.ast),
                inner: child.span(self.ast),
                inner_target: ParenthesisInner::Expression(child),
            };
            let mut outer_cast = pending_cast.take();
            let mut inner_cast = None;
            if source_pending
                && (self
                    .expression_needs_parentheses(
                        self.ast.expr_data(child),
                        ExpressionPosition::Nested,
                        ContentMode::Formatted,
                    )
                    .is_none()
                    || !self.parentheses_are_removable_with_casts(boundary, |position| {
                        let (start, slot) = match position {
                            ParenthesisCastPosition::Outer => {
                                (boundary.outer.start, &mut outer_cast)
                            }
                            ParenthesisCastPosition::Inner => {
                                (boundary.inner.start, &mut inner_cast)
                            }
                        };
                        // Preserve source_type_cast's guard before its prefix query.
                        if !self.ast.source()[start as usize..].starts_with('(') {
                            return None;
                        }
                        let cast = slot.get_or_insert_with(|| self.numeric_cast_boundary(start));
                        self.numeric_source_cast(cast)
                    }))
            {
                source = Some(data);
                source_pending = false;
            }
            if reference && !self.ast.comments().is_empty() {
                let cast = outer_cast
                    .get_or_insert_with(|| self.numeric_cast_boundary(boundary.outer.start));
                if let Some(&comment) = cast.before.last()
                    && self.reference_parenthesis_cast(comment, cast.start)
                {
                    reference = false;
                }
            }
            if !source_pending && !reference {
                return NumericArgumentContent {
                    source,
                    reference: None,
                };
            }
            pending_cast = inner_cast;
            inner = child;
        }
    }

    fn numeric_cast_boundary(&self, start: u32) -> NumericCastBoundary<'ast> {
        let comments = self.ast.comments();
        let count = comments.partition_point(|comment| comment.span.end <= start);
        NumericCastBoundary {
            start,
            before: &comments[..count],
            source: SourceCastState::Pending,
        }
    }

    fn numeric_source_cast(&self, boundary: &mut NumericCastBoundary<'ast>) -> Option<Comment> {
        match boundary.source {
            SourceCastState::Known(cast) => cast,
            SourceCastState::Pending => {
                let cast = self.source_type_cast_before(boundary.start, boundary.before);
                boundary.source = SourceCastState::Known(cast);
                cast
            }
        }
    }

    /// Pinned reference postprocess keeps only an immediately preceding Closure cast.
    /// Keep its immediate-comment and trim-gap owner while sharing tag recognition.
    fn reference_parenthesis_cast(&self, comment: Comment, start: u32) -> bool {
        if comment.kind != CommentKind::Block {
            return false;
        }
        let raw = self.source(comment.span);
        if !raw.starts_with("/**")
            || !self
                .source(Span::new(comment.span.end, start))
                .chars()
                .all(|character| {
                    matches!(character,
                    '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}'
                    | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
                    | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
                })
        {
            return false;
        }
        self.comment_is_type_cast(comment)
    }

    pub(in crate::formatter) fn parentheses_are_removable(
        &self,
        boundary: SourceParentheses,
    ) -> bool {
        self.parentheses_are_removable_with_casts(boundary, |position| {
            let start = match position {
                ParenthesisCastPosition::Outer => boundary.outer.start,
                ParenthesisCastPosition::Inner => boundary.inner.start,
            };
            self.source_type_cast(start)
        })
    }

    fn parentheses_are_removable_with_casts(
        &self,
        boundary: SourceParentheses,
        mut cast: impl FnMut(ParenthesisCastPosition) -> Option<Comment>,
    ) -> bool {
        let comments = self.ast.comments();
        if comments.is_empty() {
            return true;
        }
        if self.parentheses_separate_diagnostic_lines(boundary) {
            return false;
        }
        let range = |span: Span| {
            let first = comments.partition_point(|comment| comment.span.start < span.start);
            let inside = &comments[first..];
            let count = inside.partition_point(|comment| comment.span.end <= span.end);
            &inside[..count]
        };
        let leading = range(Span::new(boundary.outer.start, boundary.inner.start));
        let trailing = range(Span::new(boundary.inner.end, boundary.outer.end));
        let mut inner_binding = None;
        let leading_binding = self.comment_run_binding(leading.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.is_ordinary() {
                return CommentBinding::Unbound;
            }
            if !semantics.type_cast
                || semantics.suppression
                || semantics.diagnostic
                || semantics.file_directive
                || semantics.optimizer
            {
                return CommentBinding::SourceBound;
            }
            *inner_binding.get_or_insert_with(|| {
                if cast(ParenthesisCastPosition::Inner).is_none() {
                    return CommentBinding::Unbound;
                }
                self.parentheses_inner_cast_target(boundary)
                    .map_or(CommentBinding::SourceBound, CommentBinding::BoundTo)
            })
        });
        let inner_target = match inner_binding {
            Some(CommentBinding::BoundTo(target)) => Some(target),
            _ => None,
        };
        if !leading_binding.allows_relocation(inner_target) {
            return false;
        }
        let trailing_binding = self.comment_run_binding(trailing.iter().copied(), |comment| {
            if self.comment_semantics(comment).is_ordinary() {
                CommentBinding::Unbound
            } else {
                CommentBinding::SourceBound
            }
        });
        trailing_binding.allows_relocation(None) && cast(ParenthesisCastPosition::Outer).is_none()
    }

    fn parentheses_inner_cast_target(&self, boundary: SourceParentheses) -> Option<Span> {
        let mut expression = match boundary.inner_target {
            ParenthesisInner::Expression(expression) => expression,
            ParenthesisInner::SourcePair => return Some(boundary.inner),
            ParenthesisInner::Unresolved => return None,
        };
        loop {
            if expression.span(self.ast).start != boundary.inner.start {
                return None;
            }
            let data = self.ast.expr_data(expression);
            expression = match data {
                ExprData::ParenthesizedExpression(node) => return Some(node.span(self.ast)),
                ExprData::MemberExpression(node) => node.object(self.ast),
                ExprData::CallExpression(node) => node.callee(self.ast),
                ExprData::ChainExpression(node) => node.expression(self.ast),
                ExprData::ConditionalExpression(node) => node.test(self.ast),
                ExprData::SequenceExpression(node) => {
                    let first = node.expressions(self.ast).iter().next()?;
                    self.ast.get_node_in_sub_range(first)
                }
                _ => super::binary::BinaryLike::from_data(data, self.ast)?.left,
            };
        }
    }

    fn parentheses_separate_diagnostic_lines(&self, boundary: SourceParentheses) -> bool {
        if !self.diagnostic_range_is_sensitive(boundary.outer) {
            return false;
        }
        // Delimiters can separate covered tokens from the next source line.
        let opening = (self.diagnostic_line_is_sensitive(boundary.outer.start)
            || self.diagnostic_line_is_sensitive(boundary.inner.start))
            && self.line_breaks(boundary.outer.start, boundary.inner.start) != 0;
        if opening {
            return true;
        }
        self.diagnostic_line_is_sensitive(boundary.inner.end)
            && self.line_breaks(boundary.inner.end, boundary.outer.end) != 0
    }

    pub(in crate::formatter) fn format_expression_leading_comments(
        &self,
        comments: &[Comment],
        next_start: u32,
        layout: CommentRunLayout,
        start: ExpressionStart,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        self.format_expression_leading_comments_with_array_comments(
            comments, next_start, layout, start, None, writer,
        )
    }

    fn format_expression_leading_comments_with_array_comments(
        &self,
        comments: &[Comment],
        next_start: u32,
        layout: CommentRunLayout,
        mut start: ExpressionStart,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        if start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !comments.is_empty()
            && self.parenthesis_has_type_cast(next_start)
        {
            writer.text(";");
            start = ExpressionStart::None;
        }
        self.format_comment_run_with_array_comments(comments, layout, array_comments, writer)?;
        Ok(start)
    }

    pub(in crate::formatter) fn capture_expression_tail(
        &self,
        span: Span,
        tail: Option<ExpressionTail<'ast>>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source_end = tail.as_ref().map_or(span.start, |tail| tail.source_end);
        let comments = self.comments.between(Span::new(source_end, span.end))?;
        Ok((!comments.is_empty()).then_some(ExpressionTail {
            source_end,
            comments,
        }))
    }

    pub(super) fn ignored_parentheses_child(
        &self,
        parenthesized: ParenthesizedExpression,
        mode: ContentMode,
    ) -> Option<Expr> {
        if !self.has_suppression {
            return None;
        }
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let mode = self.parentheses_content_mode(
            SourceParentheses {
                outer: span,
                inner: child_span,
                inner_target: ParenthesisInner::Expression(child),
            },
            mode,
        );
        if mode != ContentMode::Verbatim {
            return None;
        }
        let head = self
            .comments
            .in_range(Span::new(span.start, child_span.start));
        let trailing = Span::new(child_span.end, span.end - 1);
        let tail = self.comments.in_range(trailing);
        head.iter()
            .chain(tail)
            .all(|&comment| {
                let semantics = self.comment_semantics(comment);
                semantics.is_ordinary() || semantics.suppression
            })
            .then_some(child)
    }

    fn parentheses_content_mode(
        &self,
        boundary: SourceParentheses,
        mode: ContentMode,
    ) -> ContentMode {
        if mode == ContentMode::Verbatim || !self.has_suppression {
            return mode;
        }
        let leading = self
            .comments
            .in_range(Span::new(boundary.outer.start, boundary.inner.start));
        if self.content_mode(leading) == ContentMode::Verbatim {
            return ContentMode::Verbatim;
        }
        let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
        self.trailing_content_mode(trailing, true)
    }

    pub(super) fn format_source_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_source_parentheses_with_array_comments(parenthesized, context, None, writer)
    }

    pub(super) fn format_source_parentheses_with_array_comments(
        &mut self,
        parenthesized: ParenthesizedExpression,
        mut context: ExpressionContext,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let child_span = child.span(self.ast);
        let trailing = Span::new(child_span.end, span.end - 1);
        let inline_mode = self.trailing_content_mode(trailing, false);
        if inline_mode == ContentMode::Verbatim
            && (matches!(context.position, ExpressionPosition::Statement)
                || context.tail_owner == TailOwner::ParentOperation
                || self.expression_needs_parentheses(
                    self.ast.expr_data(child),
                    context.position,
                    ContentMode::Verbatim,
                ) == Some(true))
        {
            self.format_verbatim_expression(
                Expr::ParenthesizedExpression(parenthesized),
                context,
                writer,
            )?;
            if let Some(owner) = array_comments {
                owner.finish_source(self, span.end);
            }
            return Ok(None);
        }
        context.content_mode = self.parentheses_content_mode(
            SourceParentheses {
                outer: span,
                inner: child_span,
                inner_target: ParenthesisInner::Expression(child),
            },
            context.content_mode,
        );
        if self.parenthesis_has_type_cast(span.start) {
            return self.format_retained_parentheses_with_array_comments(
                parenthesized,
                context,
                RetainedParentheses::Cast,
                array_comments,
                writer,
            );
        }
        if matches!(context.position, ExpressionPosition::ConditionalTest)
            && matches!(
                self.ast.expr_data(child),
                ExprData::ConditionalExpression(_)
            )
        {
            return self.format_retained_parentheses_with_array_comments(
                parenthesized,
                context,
                RetainedParentheses::Operand,
                array_comments,
                writer,
            );
        }
        let content = self.expression_content(Expr::ParenthesizedExpression(parenthesized));
        let ignored_child = if context.content_mode == ContentMode::Verbatim {
            self.ignored_parentheses_child(parenthesized, context.content_mode)
        } else {
            None
        };
        if let Some(ignored) = ignored_child
            && (self.ignored_operand_owns_operation_head(ignored, context.position)
                || matches!(
                    context.position,
                    ExpressionPosition::Statement
                        | ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::ArrowBody
                        | ExpressionPosition::RestrictedArgument
                        | ExpressionPosition::YieldArgument
                        | ExpressionPosition::AwaitArgument
                )
                || context.expression_start == ExpressionStart::ArrowBody
                || self.parentheses_separate_diagnostic_lines(SourceParentheses {
                    outer: span,
                    inner: child_span,
                    inner_target: ParenthesisInner::Expression(child),
                })
                || self
                    .expression_needs_parentheses(
                        self.ast.expr_data(child),
                        context.position,
                        ContentMode::Verbatim,
                    )
                    .is_none())
        {
            return self.format_retained_parentheses_with_array_comments(
                parenthesized,
                context.with_content_mode(ContentMode::Verbatim),
                RetainedParentheses::Operand,
                array_comments,
                writer,
            );
        }
        if ignored_child.is_none()
            && content.expression == Expr::ParenthesizedExpression(parenthesized)
        {
            return self.format_retained_parentheses_with_array_comments(
                parenthesized,
                context,
                RetainedParentheses::Annotation,
                array_comments,
                writer,
            );
        }
        let inner = ignored_child.unwrap_or(content.expression);
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        self.comments.claim(leading)?;
        let layout = if matches!(context.position, ExpressionPosition::Statement) {
            CommentRunLayout::Source
        } else {
            CommentRunLayout::Leading(inner_span.start)
        };
        context.content_mode = if ignored_child.is_some() {
            ContentMode::Verbatim
        } else {
            self.content_mode(leading)
        };
        let tail = self.format_expression_with_leading_comments_with_array_comments(
            inner,
            NodeHead {
                comments: leading,
                content_start: inner_span.start,
                layout,
                gap_end: matches!(context.position, ExpressionPosition::Statement)
                    .then_some(inner_span.start),
            },
            context,
            array_comments,
            writer,
        )?;
        self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)
    }

    /// A leading ignore must stay inside its source pair when removing the pair
    /// would attach it to the enclosing operation's lexical head instead.
    fn ignored_operand_owns_operation_head(
        &self,
        expression: Expr,
        position: ExpressionPosition,
    ) -> bool {
        match position {
            ExpressionPosition::BinaryOperand {
                operator,
                side: super::binary::OperandSide::Left,
            } => super::binary::BinaryLike::from_data(self.ast.expr_data(expression), self.ast)
                .is_none_or(|child| !operator.flattens(child.operator)),
            ExpressionPosition::ConditionalTest => true,
            _ => false,
        }
    }

    fn format_retained_parentheses_with_array_comments(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        kind: RetainedParentheses,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = parenthesized.span(self.ast);
        let child = parenthesized.expression(self.ast);
        let (inner, data) = match context.content_mode {
            ContentMode::Verbatim => (child, self.ast.expr_data(child)),
            ContentMode::Formatted => {
                let content = self.expression_content(child);
                (content.expression, content.data)
            }
        };
        let inner_span = inner.span(self.ast);
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = self.comments.in_range(Span::new(inner_span.end, span.end));
        let hug = match data {
            ExprData::ArrayExpression(array) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!array.elements(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            ExprData::ObjectExpression(object) => {
                leading.is_empty()
                    && trailing.is_empty()
                    && (!object.properties(self.ast).is_empty()
                        || self.comments.in_range(inner_span).is_empty())
            }
            _ => false,
        };
        self.format_retained_source_parentheses_with_array_comments(
            RetainedSourceParentheses {
                boundary: SourceParentheses {
                    outer: span,
                    inner: inner_span,
                    inner_target: ParenthesisInner::Expression(inner),
                },
                kind,
                hug,
                leading_mode: None,
            },
            context,
            array_comments,
            writer,
            |formatter, context, array_comments, writer| {
                formatter.format_expression_with_array_comments(
                    inner,
                    context,
                    array_comments,
                    writer,
                )
            },
        )
    }

    pub(super) fn format_assignment_right_parentheses(
        &mut self,
        parenthesized: ParenthesizedExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let child = parenthesized.expression(self.ast);
        let span = parenthesized.span(self.ast);
        let kind = if self.parenthesis_has_type_cast(span.start) {
            RetainedParentheses::Cast
        } else {
            RetainedParentheses::Operand
        };
        self.format_retained_source_parentheses_with_array_comments(
            RetainedSourceParentheses {
                boundary: SourceParentheses {
                    outer: span,
                    inner: child.span(self.ast),
                    inner_target: ParenthesisInner::Expression(child),
                },
                kind,
                hug: false,
                leading_mode: Some(context.content_mode),
            },
            context,
            None,
            writer,
            |formatter, context, _, writer| content(formatter, context, writer),
        )
    }

    pub(super) fn format_retained_target_parentheses(
        &mut self,
        boundary: SourceParentheses,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let kind = if self.parenthesis_has_type_cast(boundary.outer.start) {
            RetainedParentheses::Cast
        } else {
            RetainedParentheses::Annotation
        };
        self.format_retained_source_parentheses(boundary, context, kind, false, writer, content)
    }

    fn format_retained_source_parentheses(
        &mut self,
        boundary: SourceParentheses,
        context: ExpressionContext,
        kind: RetainedParentheses,
        hug: bool,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_retained_source_parentheses_with_array_comments(
            RetainedSourceParentheses {
                boundary,
                kind,
                hug,
                leading_mode: None,
            },
            context,
            None,
            writer,
            |formatter, context, _, writer| content(formatter, context, writer),
        )
    }

    fn format_retained_source_parentheses_with_array_comments(
        &mut self,
        source: RetainedSourceParentheses,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(
            &mut Self,
            ExpressionContext,
            Option<&mut ArrayCommentOwner<'ast>>,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RetainedSourceParentheses {
            boundary,
            kind,
            hug,
            leading_mode,
        } = source;
        let span = boundary.outer;
        let inner_span = boundary.inner;
        let leading = self
            .comments
            .between(Span::new(span.start, inner_span.start))?;
        let trailing = self.comments.in_range(Span::new(inner_span.end, span.end));
        let expand = matches!(kind, RetainedParentheses::Cast)
            && (leading
                .first()
                .is_some_and(|comment| self.line_breaks(span.start, comment.span.start) != 0)
                || trailing.first().is_some_and(|comment| {
                    self.line_breaks(inner_span.end, comment.span.start) != 0
                }));
        let leading_layout = match kind {
            RetainedParentheses::Cast | RetainedParentheses::Operand => {
                CommentRunLayout::GroupedLeading(inner_span.start)
            }
            RetainedParentheses::Annotation => CommentRunLayout::FlatLeading(inner_span.start),
        };
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
        {
            writer.text(";");
        }
        let first = leading
            .first()
            .map_or(inner_span.start, |comment| comment.span.start);
        let opening = if leading_mode.is_some() {
            self.source_line_constraint(span.start + 1, first)
        } else {
            self.line_constraint(span.start + 1, first)
        };
        let last = trailing
            .last()
            .map_or(inner_span.end, |comment| comment.span.end);
        let closing = self.line_constraint(last, span.end - 1);
        let write_contents = |writer: &mut Writer<'ast>| {
            self.comments.claim(leading)?;
            self.format_comment_run_with_array_comments(
                leading,
                leading_layout,
                array_comments.as_deref_mut(),
                writer,
            )?;
            let mode = match context.content_mode {
                ContentMode::Verbatim => ContentMode::Verbatim,
                ContentMode::Formatted => {
                    leading_mode.unwrap_or_else(|| self.content_mode(leading))
                }
            };
            let context = match (mode, context.position) {
                (ContentMode::Verbatim, ExpressionPosition::UnaryArgument(operator)) => {
                    ExpressionContext::at(ExpressionPosition::ParenthesizedUnaryArgument(operator))
                }
                (_, ExpressionPosition::MemberAssignmentObject) => {
                    ExpressionContext::at(ExpressionPosition::MemberAssignmentObject)
                        .inside_parentheses()
                }
                _ => ExpressionContext::at(ExpressionPosition::Nested).inside_parentheses(),
            }
            .with_content_mode(mode)
            .with_member_chain_layout(context.member_chain_layout);
            let tail = content(self, context, array_comments.as_deref_mut(), writer)?;
            let tail = self.capture_expression_tail(Span::new(inner_span.end, span.end), tail)?;
            self.format_expression_tail_with_array_comments(tail, array_comments, writer)
        };
        writer.group(expand, |writer| {
            writer.text("(");
            if hug
                && !matches!(opening, LineConstraint::NewLine(_))
                && !matches!(closing, LineConstraint::NewLine(_))
            {
                write_contents(writer)?;
            } else {
                writer.indent(|writer| {
                    opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                    write_contents(writer)
                })?;
                if trailing
                    .last()
                    .is_some_and(|comment| comment.kind == CommentKind::Line)
                {
                    writer.hard_line();
                } else {
                    closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
            }
            writer.text(")");
            Ok(None)
        })
    }

    pub(in crate::formatter) fn parenthesis_has_type_cast(&self, start: u32) -> bool {
        self.source_type_cast(start).is_some()
    }

    /// A removable source envelope can contain the cast that owns its head.
    /// Resolve that target from immutable source trivia before printing the head,
    /// even when a surrounding owner has already claimed the leading run.
    pub(super) fn source_cast_head(&self, mut expression: Expr) -> Option<ParenthesizedExpression> {
        if !self.has_type_casts {
            return None;
        }
        loop {
            expression = match self.ast.expr_data(expression) {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    if self.parenthesis_has_type_cast(parenthesized.span(self.ast).start) {
                        return Some(parenthesized);
                    }
                    parenthesized.expression(self.ast)
                }
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                _ => return None,
            };
        }
    }

    fn source_type_cast(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts || !self.ast.source()[start as usize..].starts_with('(') {
            return None;
        }
        self.source_cast_before(start)
    }

    pub(in crate::formatter) fn source_cast_before(&self, start: u32) -> Option<Comment> {
        if !self.has_type_casts {
            return None;
        }
        let comments = self.ast.comments();
        if comments.is_empty() {
            return None;
        }
        let count = comments.partition_point(|comment| comment.span.end <= start);
        self.source_type_cast_before(start, &comments[..count])
    }

    fn source_type_cast_before(&self, start: u32, comments: &[Comment]) -> Option<Comment> {
        let mut next_start = start;
        for &comment in comments.iter().rev() {
            if !self
                .source(Span::new(comment.span.end, next_start))
                .chars()
                .all(is_source_whitespace)
            {
                break;
            }
            // Ordinary trivia is transparent; a newer JSDoc replaces the older one.
            if self.source(comment.span).starts_with("/**") {
                return self.comment_is_type_cast(comment).then_some(comment);
            }
            next_start = comment.span.start;
        }
        None
    }
}
