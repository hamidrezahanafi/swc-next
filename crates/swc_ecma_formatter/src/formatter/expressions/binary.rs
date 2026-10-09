//! Typed operator chains with one continuation indent per precedence group.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Comment, CommentKind, Expr, ExprData, GetSpan, LogicalOperator,
    Precedence, Span,
};

use super::parentheses::SourceParentheses;
use super::{
    ExpressionContext, ExpressionPosition, ExpressionStart, Formatter, HeadComments, TailOwner,
};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::formatter) enum BinaryOperatorKind {
    Binary(BinaryOperator),
    Logical(LogicalOperator),
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum OperandSide {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub(super) struct BinaryLike {
    pub(super) left: Expr,
    pub(super) right: Expr,
    pub(super) operator: BinaryOperatorKind,
}

impl BinaryLike {
    pub(super) fn from_data(data: ExprData, ast: &Ast<'_>) -> Option<Self> {
        match data {
            ExprData::BinaryExpression(node) => Some(Self {
                left: node.left(ast),
                right: node.right(ast),
                operator: BinaryOperatorKind::Binary(node.operator(ast)),
            }),
            ExprData::LogicalExpression(node) => Some(Self {
                left: node.left(ast),
                right: node.right(ast),
                operator: BinaryOperatorKind::Logical(node.operator(ast)),
            }),
            _ => None,
        }
    }
}

impl BinaryOperatorKind {
    pub(super) fn from_data(data: ExprData, ast: &Ast<'_>) -> Option<Self> {
        match data {
            ExprData::BinaryExpression(node) => Some(Self::Binary(node.operator(ast))),
            ExprData::LogicalExpression(node) => Some(Self::Logical(node.operator(ast))),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Binary(operator) => operator.as_str(),
            Self::Logical(operator) => operator.as_str(),
        }
    }

    fn precedence(self) -> Precedence {
        match self {
            Self::Binary(operator) => operator.precedence(),
            Self::Logical(operator) => operator.precedence(),
        }
    }

    fn same_kind(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Binary(_), Self::Binary(_)) | (Self::Logical(_), Self::Logical(_))
        )
    }

    pub(super) fn flattens(self, child: Self) -> bool {
        if self.precedence() != child.precedence() {
            return false;
        }
        match self.precedence() {
            Precedence::Exponent | Precedence::Equality | Precedence::Shift => false,
            Precedence::Multiply => self == child && self != Self::Binary(BinaryOperator::Modulo),
            _ => true,
        }
    }

    pub(super) fn needs_parentheses(self, child: Self, side: OperandSide) -> bool {
        if matches!((self, child), (Self::Logical(_), Self::Logical(_))) && self != child {
            return true;
        }
        let parent_precedence = self.precedence();
        let child_precedence = child.precedence();
        if parent_precedence > child_precedence {
            return true;
        }
        if parent_precedence == child_precedence
            && (matches!(side, OperandSide::Right) || !self.flattens(child))
        {
            return true;
        }
        matches!(
            parent_precedence,
            Precedence::Shift
                | Precedence::BitwiseOr
                | Precedence::BitwiseXor
                | Precedence::BitwiseAnd
        ) || (parent_precedence == Precedence::Add && child == Self::Binary(BinaryOperator::Modulo))
    }
}

struct BinaryChain {
    first: ChainParent,
    ancestors: Vec<ChainParent>,
}

#[derive(Clone, Copy)]
struct ChainParent {
    binary: BinaryLike,
    left_content_end: u32,
    suppression: Option<BinarySuppression>,
}

#[derive(Clone, Copy)]
struct BinarySuppression {
    operator: Span,
    left_mode: ContentMode,
    right_mode: ContentMode,
}

impl ChainParent {
    fn left_mode(self) -> ContentMode {
        self.suppression
            .map_or(ContentMode::Formatted, |source| source.left_mode)
    }

    fn right_mode(self) -> ContentMode {
        self.suppression
            .map_or(ContentMode::Formatted, |source| source.right_mode)
    }
}

#[derive(Clone, Copy)]
enum OperandLayout {
    Break,
    Inline,
    IndentedInline,
}

#[derive(Clone, Copy)]
enum BinaryContinuationLayout {
    Expression,
    ControlCondition,
}

enum OperandBoundary<'ast> {
    Break,
    Inline,
    Constrained {
        constraint: LineConstraint,
        inline: bool,
        mode: ContentMode,
    },
    Comments {
        before: &'ast [Comment],
        after: &'ast [Comment],
        operator_start: u32,
        right_start: u32,
        layout: OperandLayout,
        mode: ContentMode,
    },
}

impl OperandBoundary<'_> {
    fn indents(&self) -> bool {
        matches!(
            self,
            Self::Comments {
                layout: OperandLayout::IndentedInline,
                ..
            }
        )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn binary_content(&self, expression: Expr) -> Option<(BinaryLike, Option<SourceParentheses>)> {
        let content = self.expression_content(expression);
        if content
            .parentheses
            .is_some_and(|boundary| !self.parentheses_are_removable(boundary))
        {
            return None;
        }
        let binary = BinaryLike::from_data(content.data, self.ast)?;
        Some((binary, content.parentheses))
    }

    fn prepare_binary_entry(
        &self,
        binary: BinaryLike,
        left_content_end: u32,
    ) -> Result<ChainParent, FormatError> {
        let mut entry = ChainParent {
            binary,
            left_content_end,
            suppression: None,
        };
        if !self.has_suppression {
            return Ok(entry);
        }
        let right_start = binary.right.span(self.ast).start;
        let span = Span::new(binary.left.span(self.ast).end, right_start);
        let comments = self.comments.in_range(span);
        if self.content_mode(comments) != ContentMode::Verbatim {
            return Ok(entry);
        }
        let operator = binary.operator.as_str();
        let Some(start) = self.find_token(span, comments, operator) else {
            return Err(FormatError::unsupported(
                "binary operator boundary",
                None,
                span,
            ));
        };
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        for position in self.suppression_positions(before, start) {
            match position {
                SuppressionPosition::OwnLine => right_mode = ContentMode::Verbatim,
                SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                    left_mode = ContentMode::Verbatim;
                }
            }
        }
        for position in self.suppression_positions(after, right_start) {
            match position {
                SuppressionPosition::EndOfLine => left_mode = ContentMode::Verbatim,
                SuppressionPosition::OwnLine | SuppressionPosition::Inline => {
                    right_mode = ContentMode::Verbatim;
                }
            }
        }
        entry.suppression = Some(BinarySuppression {
            operator: Span::new(start, start + operator.len() as u32),
            left_mode,
            right_mode,
        });
        Ok(entry)
    }

    fn binary_chain(&self, first: BinaryLike) -> Result<BinaryChain, FormatError> {
        let mut first = self.prepare_binary_entry(first, first.left.span(self.ast).end)?;
        let mut ancestors = Vec::new();
        while first.left_mode() != ContentMode::Verbatim {
            let Some((left, left_parentheses)) = self.binary_content(first.binary.left) else {
                break;
            };
            if !first.binary.operator.flattens(left.operator) {
                break;
            }
            first.left_content_end = left_parentheses.map_or_else(
                || first.binary.left.span(self.ast).end,
                |boundary| boundary.inner.end,
            );
            ancestors.push(first);
            first = self.prepare_binary_entry(left, left.left.span(self.ast).end)?;
        }
        Ok(BinaryChain { first, ancestors })
    }

    fn format_binary_head_comments(
        &mut self,
        mut expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<ExpressionStart, FormatError> {
        let start = expression.span(self.ast).start;
        let operand = expression;
        loop {
            let content = self.expression_content(expression);
            if matches!(content.data, ExprData::ParenthesizedExpression(_)) {
                break;
            }
            if content
                .parentheses
                .is_some_and(|boundary| !self.parentheses_are_removable(boundary))
            {
                break;
            }
            expression = content.expression;
            match BinaryLike::from_data(content.data, self.ast) {
                Some(binary) => expression = binary.left,
                None => break,
            }
        }
        let end = expression.span(self.ast).start;
        let comments = self.comments.between(Span::new(start, end))?;
        let comments = self.comments.take(comments.len());
        if self.has_target_annotations
            && context.expression_start.needs_asi_guard()
            && self.options.semicolons == crate::Semicolons::AsNeeded
            && !comments.is_empty()
        {
            return self.format_expression_head(
                operand,
                NodeHead {
                    comments,
                    content_start: end,
                    layout: CommentRunLayout::Leading(end),
                    gap_end: None,
                },
                context,
                writer,
            );
        }
        self.format_comment_run(comments, CommentRunLayout::Leading(end), writer)?;
        Ok(context.expression_start)
    }

    pub(super) fn format_binary(
        &mut self,
        binary: BinaryLike,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if matches!(context.position, ExpressionPosition::ControlCondition) {
            return self.format_binary_enclosed_with_context(binary, context, writer);
        }
        let parent_indents = context.operand_indent == super::OperandIndent::Parent
            || match context.position {
                ExpressionPosition::ParenthesizedUnaryArgument(_) => true,
                ExpressionPosition::AssignmentRight(_) | ExpressionPosition::PropertyValue => {
                    let inline = self.inline_logical_operand(binary.operator, binary.right);
                    !inline
                        || !BinaryLike::from_data(
                            self.expression_content(binary.left).data,
                            self.ast,
                        )
                        .is_some_and(|left| binary.operator.flattens(left.operator))
                }
                _ => false,
            };
        if parent_indents {
            return writer.group(false, |writer| self.format_binary_enclosed(binary, writer));
        }
        if context.head_comments == HeadComments::Pending {
            context.expression_start = self.format_binary_head_comments(
                binary.left,
                ExpressionContext {
                    position: ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                    ..context
                },
                writer,
            )?;
            context.head_comments = HeadComments::Claimed;
        }
        let chain = self.binary_chain(binary)?;
        let inline = chain.ancestors.is_empty()
            && self.inline_logical_operand(binary.operator, binary.right);
        writer.group(false, |writer| {
            let tail = self.format_binary_head(chain.first, context.expression_start, writer)?;
            if inline || matches!(context.position, ExpressionPosition::ArrowBody) {
                self.format_binary_continuation(chain, context, tail, writer)
            } else {
                writer
                    .indent(|writer| self.format_binary_continuation(chain, context, tail, writer))
            }
        })
    }

    pub(super) fn format_binary_enclosed(
        &mut self,
        binary: BinaryLike,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_binary_enclosed_with_context(
            binary,
            ExpressionContext::at(ExpressionPosition::Nested),
            writer,
        )
    }

    fn format_binary_enclosed_with_context(
        &mut self,
        binary: BinaryLike,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let chain = self.binary_chain(binary)?;
        self.format_binary_head_comments(
            binary.left,
            ExpressionContext {
                position: ExpressionPosition::BinaryOperand {
                    operator: binary.operator,
                    side: OperandSide::Left,
                },
                ..context
            },
            writer,
        )?;
        let tail = self.format_binary_head(chain.first, context.expression_start, writer)?;
        self.format_binary_continuation(chain, context, tail, writer)
    }

    fn format_binary_head(
        &mut self,
        entry: ChainParent,
        expression_start: ExpressionStart,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let first = entry.binary;
        writer.group(false, |writer| {
            self.format_expression(
                first.left,
                ExpressionContext {
                    position: ExpressionPosition::BinaryOperand {
                        operator: first.operator,
                        side: OperandSide::Left,
                    },
                    expression_start,
                    head_comments: HeadComments::Claimed,
                    content_mode: entry.left_mode(),
                    tail_owner: TailOwner::Expression,
                    parentheses_owner: super::ParenthesesOwner::Expression,
                    member_layout: super::MemberLayout::Normal,
                    member_chain_layout: super::MemberChainLayout::Fluent,
                    arrow_layout: super::arrows::ArrowLayout::Argument,
                    operand_indent: super::OperandIndent::Expression,
                    conditional_envelope: super::ConditionalEnvelope::Local,
                    conditional_closing: super::ConditionalClosing::Inline,
                },
                writer,
            )
        })
    }

    fn format_binary_continuation(
        &mut self,
        chain: BinaryChain,
        context: ExpressionContext,
        mut tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let parent = match context.position {
            ExpressionPosition::BinaryOperand { operator, .. } => Some(operator),
            _ => None,
        };
        let layout = match context.position {
            ExpressionPosition::ControlCondition => BinaryContinuationLayout::ControlCondition,
            _ => BinaryContinuationLayout::Expression,
        };
        let first = chain.first;
        let mut nodes = std::iter::once(first)
            .chain(chain.ancestors.into_iter().rev())
            .peekable();
        while let Some(entry) = nodes.next() {
            let (parent, tail_owner) = match nodes.peek() {
                Some(next) => (Some(next.binary.operator), TailOwner::ParentOperation),
                None => (parent, context.tail_owner),
            };
            tail = self.format_binary_right(entry, parent, layout, tail_owner, tail, writer)?;
        }
        Ok(tail)
    }

    pub(super) fn inline_logical_operand(
        &self,
        operator: BinaryOperatorKind,
        operand: Expr,
    ) -> bool {
        if !matches!(operator, BinaryOperatorKind::Logical(_)) {
            return false;
        }
        let right = self.expression_content(operand).data;
        match right {
            ExprData::ArrayExpression(array) => !array.elements(self.ast).is_empty(),
            ExprData::ObjectExpression(object) => !object.properties(self.ast).is_empty(),
            _ => false,
        }
    }

    fn format_binary_right(
        &mut self,
        entry: ChainParent,
        mut parent: Option<BinaryOperatorKind>,
        layout: BinaryContinuationLayout,
        tail_owner: TailOwner,
        mut tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut entry = entry;
        let mut closing_end = None;
        // Same-operator logical operands keep their short-circuit order in one group.
        while let Some((right, boundary)) = self.binary_content(entry.binary.right) {
            if !matches!(entry.binary.operator, BinaryOperatorKind::Logical(_))
                || entry.binary.operator != right.operator
                || entry.right_mode() == ContentMode::Verbatim
            {
                break;
            }
            let mut next = self.prepare_binary_entry(right, right.left.span(self.ast).end)?;
            if let Some(boundary) = boundary {
                closing_end.get_or_insert(boundary.outer.end);
            }
            let boundary = self.format_binary_operator(
                entry,
                right.left,
                self.inline_logical_operand(entry.binary.operator, right.left),
                tail,
                writer,
            )?;
            let mode = self.format_binary_operand_boundary(boundary, writer)?;
            if next.left_mode() != ContentMode::Verbatim
                && let Some((left, boundary)) = self.binary_content(right.left)
                && left.operator == entry.binary.operator
            {
                let chain = self.binary_chain(left)?;
                self.format_binary_head_comments(
                    left.left,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: left.operator,
                        side: OperandSide::Left,
                    }),
                    writer,
                )?;
                let head = self.format_binary_head(chain.first, ExpressionStart::None, writer)?;
                tail = self.format_binary_continuation(
                    chain,
                    ExpressionContext::at(match layout {
                        BinaryContinuationLayout::ControlCondition => {
                            ExpressionPosition::ControlCondition
                        }
                        BinaryContinuationLayout::Expression => ExpressionPosition::BinaryOperand {
                            operator: entry.binary.operator,
                            side: OperandSide::Right,
                        },
                    })
                    .with_tail_owner(TailOwner::ParentOperation),
                    head,
                    writer,
                )?;
                next.left_content_end = boundary.map_or_else(
                    || right.left.span(self.ast).end,
                    |boundary| boundary.inner.end,
                );
            } else {
                let mode = if next.left_mode() == ContentMode::Verbatim {
                    next.left_mode()
                } else {
                    mode
                };
                tail = self.format_expression(
                    right.left,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: entry.binary.operator,
                        side: OperandSide::Left,
                    })
                    .with_content_mode(mode)
                    .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?;
            }
            parent = Some(entry.binary.operator);
            entry = next;
        }
        let binary = entry.binary;
        let same_kind = |expression| {
            let data = self.expression_content(expression).data;
            BinaryOperatorKind::from_data(data, self.ast)
                .is_some_and(|child| child.same_kind(binary.operator))
        };
        let group_right = !matches!(
            (layout, binary.operator),
            (
                BinaryContinuationLayout::ControlCondition,
                BinaryOperatorKind::Logical(_)
            )
        ) && !parent.is_some_and(|parent| parent.same_kind(binary.operator))
            && !same_kind(binary.left)
            && !same_kind(binary.right);
        let write = |writer: &mut Writer<'ast>| {
            let boundary = self.format_binary_operator(
                entry,
                binary.right,
                self.inline_logical_operand(binary.operator, binary.right),
                tail,
                writer,
            )?;
            let indents = boundary.indents();
            let write_argument = |writer: &mut Writer<'ast>| {
                let mode = self.format_binary_operand_boundary(boundary, writer)?;
                self.format_expression(
                    binary.right,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Right,
                    })
                    .with_content_mode(mode)
                    .with_tail_owner(tail_owner),
                    writer,
                )
            };
            if indents {
                writer.indent(write_argument)
            } else {
                write_argument(writer)
            }
        };
        let tail = if group_right {
            writer.group(false, write)?
        } else {
            write(writer)?
        };
        if let Some(end) = closing_end {
            self.capture_expression_tail(Span::new(binary.right.span(self.ast).end, end), tail)
        } else {
            Ok(tail)
        }
    }

    fn format_binary_operator(
        &mut self,
        entry: ChainParent,
        right: Expr,
        inline: bool,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<OperandBoundary<'ast>, FormatError> {
        let left_end = tail
            .as_ref()
            .map_or(entry.left_content_end, |tail| tail.source_end);
        let right_start = if entry.suppression.is_some() {
            entry.binary.right.span(self.ast).start
        } else {
            let content = self.expression_content(right);
            if content
                .parentheses
                .is_none_or(|boundary| self.parentheses_are_removable(boundary))
            {
                content.expression.span(self.ast).start
            } else {
                right.span(self.ast).start
            }
        };
        let boundary = Span::new(left_end, right_start);
        let comments = self.comments.between(boundary)?;
        let operator = entry.binary.operator.as_str();
        if entry.suppression.is_some()
            || self.diagnostic_line_is_sensitive(left_end)
            || self.diagnostic_line_is_sensitive(right_start)
            || comments
                .iter()
                .any(|&comment| self.comment_is_diagnostic_directive(comment))
        {
            let operator_span = entry.suppression.map(|source| source.operator).or_else(|| {
                let start = self.find_token(boundary, comments, operator)?;
                Some(Span::new(start, start + operator.len() as u32))
            });
            let Some(operator_span) = operator_span else {
                return Err(FormatError::unsupported(
                    "binary operator boundary",
                    None,
                    boundary,
                ));
            };
            let operator_start = operator_span.start;
            let comments = self.comments.take(comments.len());
            let split = comments.partition_point(|comment| comment.span.end <= operator_start);
            let (before, after) = comments.split_at(split);
            self.format_comment_run(before, CommentRunLayout::SourceAfter(left_end), writer)?;
            let previous = before.last().map_or(left_end, |comment| comment.span.end);
            let constraint = if entry.suppression.is_some() {
                self.source_line_constraint(previous, operator_start)
            } else {
                self.line_constraint(previous, operator_start)
            };
            constraint.write(writer, FlatSeparator::Space, Writer::space);
            writer.text(operator);
            let operator_end = operator_span.end;
            self.format_comment_run(after, CommentRunLayout::SourceAfter(operator_end), writer)?;
            let previous = after
                .last()
                .map_or(operator_end, |comment| comment.span.end);
            return Ok(OperandBoundary::Constrained {
                constraint: if entry.suppression.is_some() {
                    self.source_line_constraint(previous, right_start)
                } else {
                    self.line_constraint(previous, right_start)
                },
                inline,
                mode: entry
                    .suppression
                    .map_or_else(|| self.content_mode(after), |source| source.right_mode),
            });
        }
        if comments.is_empty() {
            writer.space();
            writer.text(operator);
            return Ok(if inline {
                OperandBoundary::Inline
            } else {
                OperandBoundary::Break
            });
        }
        let Some(operator_start) = self.find_token(boundary, comments, operator) else {
            return Err(FormatError::unsupported(
                "binary operator boundary",
                None,
                boundary,
            ));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        // Keep an active source cast with the right operand's head.
        let trailing_limit =
            if before_count < comments.len() && self.source_cast_head(right).is_some() {
                before_count
            } else {
                comments.len()
            };
        let mode = self.content_mode(&comments[before_count..]);
        let comments = self.comments.take(comments.len());
        let mut trailing_count = 0;
        let mut previous_end = left_end;
        for (index, comment) in comments[..trailing_limit].iter().enumerate() {
            if self.line_breaks(previous_end, comment.span.start) != 0 {
                break;
            }
            previous_end = comment.span.end;
            if index < before_count
                || comment.kind == CommentKind::Line
                || self.line_breaks(
                    comment.span.end,
                    comments
                        .get(index + 1)
                        .map_or(right_start, |next| next.span.start),
                ) != 0
            {
                trailing_count = index + 1;
            }
        }
        let (trailing, leading) = comments.split_at(trailing_count);
        self.format_comment_run(trailing, CommentRunLayout::SourceAfter(left_end), writer)?;
        writer.space();
        writer.text(operator);
        let leading_before = leading.partition_point(|comment| comment.span.end <= operator_start);
        let (before, after) = leading.split_at(leading_before);
        let layout = if !inline {
            OperandLayout::Break
        } else if self.leading_comments_break(before, operator_start)
            || self.leading_comments_break(after, right_start)
        {
            OperandLayout::IndentedInline
        } else {
            OperandLayout::Inline
        };
        Ok(OperandBoundary::Comments {
            before,
            after,
            operator_start,
            right_start,
            layout,
            mode,
        })
    }

    fn leading_comments_break(&self, comments: &[Comment], next_start: u32) -> bool {
        let mut comments = comments.iter().peekable();
        while let Some(comment) = comments.next() {
            let following = comments.peek().map_or(next_start, |next| next.span.start);
            if self.line_breaks(comment.span.end, following) != 0 {
                return true;
            }
        }
        false
    }

    fn format_binary_operand_boundary(
        &self,
        boundary: OperandBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<ContentMode, FormatError> {
        let (before, after, operator_start, right_start, layout, mode) = match boundary {
            OperandBoundary::Constrained {
                constraint,
                inline,
                mode,
            } => {
                constraint.write(writer, FlatSeparator::Space, |writer| {
                    if inline {
                        writer.space();
                    } else {
                        writer.line();
                    }
                });
                return Ok(mode);
            }
            OperandBoundary::Inline => {
                writer.space();
                return Ok(ContentMode::Formatted);
            }
            OperandBoundary::Break => {
                writer.line();
                return Ok(ContentMode::Formatted);
            }
            OperandBoundary::Comments {
                before,
                after,
                operator_start,
                right_start,
                layout,
                mode,
            } => (before, after, operator_start, right_start, layout, mode),
        };
        match layout {
            OperandLayout::Inline => writer.space(),
            OperandLayout::Break | OperandLayout::IndentedInline => writer.line(),
        }
        self.format_comment_run(before, CommentRunLayout::Source, writer)?;
        if let Some(last) = before.last() {
            self.comment_gap(last.span.end, operator_start, writer);
        }
        self.format_comment_run(after, CommentRunLayout::Source, writer)?;
        if let Some(last) = after.last() {
            self.comment_gap(last.span.end, right_start, writer);
        }
        Ok(mode)
    }
}
