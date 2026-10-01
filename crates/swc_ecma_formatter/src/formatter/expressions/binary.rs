//! Typed operator chains with one continuation indent per precedence group.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Comment, CommentKind, Expr, ExprData, GetSpan, LogicalOperator,
    Precedence, Span,
};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::CommentRunLayout;
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
    right: Expr,
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

    fn flattens(self, child: Self) -> bool {
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
    first: BinaryLike,
    ancestors: Vec<BinaryLike>,
}

#[derive(Clone, Copy)]
enum OperandLayout {
    Break,
    Inline,
    IndentedInline,
}

enum OperandBoundary<'ast> {
    Break,
    Inline,
    Comments {
        before: &'ast [Comment],
        after: &'ast [Comment],
        operator_start: u32,
        right_start: u32,
        layout: OperandLayout,
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
    fn binary_without_boundary_comments(&self, expression: Expr) -> Option<BinaryLike> {
        let (inner, data) = self.unparenthesized_expression(expression);
        let binary = BinaryLike::from_data(data, self.ast)?;
        if inner == expression {
            return Some(binary);
        }
        let span = expression.span(self.ast);
        let inner_span = inner.span(self.ast);
        if self
            .comments
            .in_range(Span::new(span.start, inner_span.start))
            .is_empty()
            && self
                .comments
                .in_range(Span::new(inner_span.end, span.end))
                .is_empty()
        {
            Some(binary)
        } else {
            None
        }
    }

    fn binary_chain(&self, mut first: BinaryLike) -> BinaryChain {
        let mut ancestors = Vec::new();
        while let Some(left) = self.binary_without_boundary_comments(first.left) {
            if !first.operator.flattens(left.operator) {
                break;
            }
            ancestors.push(first);
            first = left;
        }
        BinaryChain { first, ancestors }
    }

    pub(super) fn format_binary(
        &mut self,
        binary: BinaryLike,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if matches!(
            context.position,
            ExpressionPosition::ParenthesizedUnaryArgument(_)
        ) {
            return writer.group(false, |writer| self.format_binary_enclosed(binary, writer));
        }
        let chain = self.binary_chain(binary);
        let inline = chain.ancestors.is_empty() && self.inline_logical(binary);
        writer.group(false, |writer| {
            self.format_binary_head(chain.first, context.statement_start, writer)?;
            if inline {
                self.format_binary_continuation(chain, context, writer)
            } else {
                writer.indent(|writer| self.format_binary_continuation(chain, context, writer))
            }
        })
    }

    pub(super) fn format_binary_enclosed(
        &mut self,
        binary: BinaryLike,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let chain = self.binary_chain(binary);
        self.format_binary_head(chain.first, false, writer)?;
        self.format_binary_continuation(
            chain,
            ExpressionContext::at(ExpressionPosition::Nested),
            writer,
        )
    }

    fn format_binary_head(
        &mut self,
        first: BinaryLike,
        statement_start: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.group(false, |writer| {
            self.format_expression(
                first.left,
                ExpressionContext {
                    position: ExpressionPosition::BinaryOperand {
                        operator: first.operator,
                        side: OperandSide::Left,
                    },
                    statement_start,
                },
                writer,
            )
        })
    }

    fn format_binary_continuation(
        &mut self,
        chain: BinaryChain,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let parent = match context.position {
            ExpressionPosition::BinaryOperand { operator, .. } => Some(operator),
            _ => None,
        };
        let mut nodes = std::iter::once(chain.first)
            .chain(chain.ancestors.into_iter().rev())
            .peekable();
        while let Some(binary) = nodes.next() {
            let parent = nodes.peek().map(|node| node.operator).or(parent);
            self.format_binary_right(binary, parent, writer)?;
        }
        Ok(())
    }

    fn inline_logical(&self, binary: BinaryLike) -> bool {
        if !matches!(binary.operator, BinaryOperatorKind::Logical(_)) {
            return false;
        }
        let (_, right) = self.unparenthesized_expression(binary.right);
        matches!(right, ExprData::ArrayExpression(array) if !array.elements(self.ast).is_empty())
    }

    fn format_binary_right(
        &mut self,
        mut binary: BinaryLike,
        mut parent: Option<BinaryOperatorKind>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        // Same-operator logical operands keep their short-circuit order in one group.
        while let Some(right) = self.binary_without_boundary_comments(binary.right) {
            if !matches!(binary.operator, BinaryOperatorKind::Logical(_))
                || binary.operator != right.operator
            {
                break;
            }
            let boundary = self.format_binary_operator(binary, right.left, false, writer)?;
            self.format_binary_operand_boundary(boundary, writer)?;
            if let Some(left) = self.binary_without_boundary_comments(right.left)
                && left.operator == binary.operator
            {
                let chain = self.binary_chain(left);
                self.format_binary_head(chain.first, false, writer)?;
                self.format_binary_continuation(
                    chain,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Right,
                    }),
                    writer,
                )?;
            } else {
                self.format_expression(
                    right.left,
                    ExpressionContext::at(ExpressionPosition::Nested),
                    writer,
                )?;
            }
            parent = Some(binary.operator);
            binary = right;
        }
        let same_kind = |expression| {
            let (_, data) = self.unparenthesized_expression(expression);
            BinaryOperatorKind::from_data(data, self.ast)
                .is_some_and(|child| child.same_kind(binary.operator))
        };
        let group_right = !parent.is_some_and(|parent| parent.same_kind(binary.operator))
            && !same_kind(binary.left)
            && !same_kind(binary.right);
        let mut write = |writer: &mut Writer<'ast>| {
            let boundary = self.format_binary_operator(
                binary,
                binary.right,
                self.inline_logical(binary),
                writer,
            )?;
            let indents = boundary.indents();
            let write_argument = |writer: &mut Writer<'ast>| {
                self.format_binary_operand_boundary(boundary, writer)?;
                self.format_expression(
                    binary.right,
                    ExpressionContext::at(ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Right,
                    }),
                    writer,
                )
            };
            if indents {
                writer.indent(write_argument)
            } else {
                write_argument(writer)
            }
        };
        if group_right {
            writer.group(false, write)
        } else {
            write(writer)
        }
    }

    fn format_binary_operator(
        &mut self,
        binary: BinaryLike,
        right: Expr,
        inline: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<OperandBoundary<'ast>, FormatError> {
        let left_end = binary.left.span(self.ast).end;
        let right_start = right.span(self.ast).start;
        let boundary = Span::new(left_end, right_start);
        let comments = self.comments.between(boundary)?;
        let operator = binary.operator.as_str();
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
        let comments = self.comments.take(comments.len());
        let before_count = comments.partition_point(|comment| comment.span.end <= operator_start);
        let mut trailing_count = 0;
        let mut previous_end = left_end;
        for (index, comment) in comments.iter().enumerate() {
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
    ) -> Result<(), FormatError> {
        let (before, after, operator_start, right_start, layout) = match boundary {
            OperandBoundary::Inline => {
                writer.space();
                return Ok(());
            }
            OperandBoundary::Break => {
                writer.line();
                return Ok(());
            }
            OperandBoundary::Comments {
                before,
                after,
                operator_start,
                right_start,
                layout,
            } => (before, after, operator_start, right_start, layout),
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
        Ok(())
    }
}
