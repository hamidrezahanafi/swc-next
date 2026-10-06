//! TypeScript expression operators keep real operand owners apart from token gaps.

use swc_next_ecma_ast::{
    BinaryOperator, Expr, ExprData, GetSpan, Span, TsNonNullExpression, TsType, TsTypeAssertion,
    TsTypeData, TsTypeParameterInstantiation,
};

use super::binary::{BinaryOperatorKind, OperandSide};
use super::{AccessKind, ExpressionContext, ExpressionPosition, TailOwner};
use crate::formatter::Formatter;
use crate::formatter::comments::{CommentBoundary, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::{PreparedType, TypeContext};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum TypeBinaryOperator {
    As,
    Satisfies,
}

impl TypeBinaryOperator {
    fn token(self) -> &'static str {
        match self {
            Self::As => "as",
            Self::Satisfies => "satisfies",
        }
    }
}

/// A real runtime angle-list suffix, prepared before its callee consumes comments.
pub(super) struct ExpressionTypeArguments<'ast> {
    pub(super) node: TsTypeParameterInstantiation,
    pub(super) callee_mode: ContentMode,
    pub(super) mode: ContentMode,
    pub(super) access: AccessKind,
    has_comments: bool,
    prefix: ExpressionTypeArgumentPrefix<'ast>,
}

impl ExpressionTypeArguments<'_> {
    pub(super) fn has_comments(&self) -> bool {
        self.has_comments
    }
}

enum ExpressionTypeArgumentPrefix<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        optional: Option<CommentBoundary<'ast>>,
    },
}

enum TypeExpressionGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

struct BinaryTypeBoundary<'ast> {
    operator: TypeBinaryOperator,
    left_mode: ContentMode,
    layout: BinaryTypeLayout<'ast>,
}

enum BinaryTypeLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_expression_type_arguments(
        &self,
        expression: Expr,
        node: TsTypeParameterInstantiation,
        access: AccessKind,
    ) -> Result<ExpressionTypeArguments<'ast>, FormatError> {
        let gap = Span::new(expression.span(self.ast).end, node.span(self.ast).start);
        let comments = self.comments.in_range(gap);
        let optional = if access == AccessKind::Optional {
            let Some(start) = self.find_token(gap, comments, "?.") else {
                return Err(FormatError::unsupported(
                    "optional type arguments token",
                    None,
                    gap,
                ));
            };
            Some(Span::new(start, start + 2))
        } else {
            None
        };
        let split = optional.map_or(0, |token| {
            comments.partition_point(|comment| comment.span.end <= token.start)
        });
        let (before, after) = comments.split_at(split);
        let mut callee_mode = ContentMode::Formatted;
        let mut mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    callee_mode = ContentMode::Verbatim;
                } else {
                    mode = ContentMode::Verbatim;
                }
            }
        }
        let prefix = if callee_mode == ContentMode::Verbatim || mode == ContentMode::Verbatim {
            ExpressionTypeArgumentPrefix::Source(gap)
        } else if let Some(token) = optional {
            ExpressionTypeArgumentPrefix::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, token.start),
                optional: Some(self.prepare_comment_boundary(after, token.end, gap.end)),
            }
        } else {
            ExpressionTypeArgumentPrefix::Formatted {
                before: self.prepare_comment_boundary(comments, gap.start, gap.end),
                optional: None,
            }
        };
        Ok(ExpressionTypeArguments {
            node,
            callee_mode,
            mode,
            access,
            has_comments: !comments.is_empty(),
            prefix,
        })
    }

    pub(super) fn format_expression_type_arguments(
        &mut self,
        arguments: ExpressionTypeArguments<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_expression_tail(tail, writer)?;
        match arguments.prefix {
            ExpressionTypeArgumentPrefix::Source(gap) => self.format_verbatim(gap, writer)?,
            ExpressionTypeArgumentPrefix::Formatted { before, optional } => {
                self.format_type_expression_gap(
                    TypeExpressionGap::Formatted(before),
                    FlatSeparator::Empty,
                    writer,
                )?;
                if let Some(after) = optional {
                    writer.text("?.");
                    self.format_type_expression_gap(
                        TypeExpressionGap::Formatted(after),
                        FlatSeparator::Empty,
                        writer,
                    )?;
                }
            }
        }
        self.format_type_arguments(arguments.node, arguments.mode, writer)
    }

    /// These identifier tokens could expose a declaration or reserved statement head.
    pub(super) fn type_binary_head_needs_parentheses(&self, data: ExprData) -> bool {
        let ExprData::IdentifierReference(identifier) = data else {
            return false;
        };
        matches!(
            self.ast.get_utf8(identifier.name(self.ast)),
            "await" | "interface" | "module" | "namespace" | "using" | "yield" | "let" | "type"
        )
    }

    pub(super) fn format_binary_type_expression(
        &mut self,
        expression: Expr,
        annotation: TsType,
        operator: TypeBinaryOperator,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let boundary = self.prepare_binary_type_boundary(expression, &mut type_node, operator)?;
        writer.group(false, |writer| {
            let tail = self.format_expression(
                expression,
                ExpressionContext::at(ExpressionPosition::TypeBinaryOperand)
                    .with_expression_start(context.expression_start)
                    .with_content_mode(boundary.left_mode)
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )?;
            self.format_expression_tail(tail, writer)?;
            self.format_binary_type_boundary(boundary, writer)?;
            self.format_prepared_type(type_node, TypeContext::Root, writer)
        })
    }

    fn prepare_binary_type_boundary(
        &self,
        expression: Expr,
        right: &mut PreparedType,
        operator: TypeBinaryOperator,
    ) -> Result<BinaryTypeBoundary<'ast>, FormatError> {
        let gap = Span::new(
            expression.span(self.ast).end,
            right.node.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let token = operator.token();
        let Some(start) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "type expression operator",
                None,
                gap,
            ));
        };
        let split = comments.partition_point(|comment| comment.span.end <= start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut source = false;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, start) {
                source = true;
                if matches!(position, SuppressionPosition::OwnLine) {
                    self.select_expression_type_head(right, position);
                } else {
                    left_mode = ContentMode::Verbatim;
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                source = true;
                if matches!(position, SuppressionPosition::EndOfLine) {
                    left_mode = ContentMode::Verbatim;
                } else {
                    self.select_expression_type_head(right, position);
                }
            }
        }
        let layout = if source {
            BinaryTypeLayout::Source(gap)
        } else {
            BinaryTypeLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, start),
                after: self.prepare_comment_boundary(after, start + token.len() as u32, gap.end),
            }
        };
        Ok(BinaryTypeBoundary {
            operator,
            left_mode,
            layout,
        })
    }

    fn format_binary_type_boundary(
        &mut self,
        boundary: BinaryTypeBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (before, after) = match boundary.layout {
            BinaryTypeLayout::Source(gap) => return self.format_verbatim(gap, writer),
            BinaryTypeLayout::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(&before, writer)?;
        before
            .gap()
            .write(writer, FlatSeparator::Space, Writer::space);
        writer.text(boundary.operator.token());
        self.format_comment_boundary(&after, writer)?;
        after
            .gap()
            .write(writer, FlatSeparator::Space, Writer::space);
        Ok(())
    }

    /// The expression caller's whole type owner excludes removable type parentheses.
    /// Own-line union comments are consumed by its real first member instead.
    fn select_expression_type_head(
        &self,
        prepared: &mut PreparedType,
        position: SuppressionPosition,
    ) {
        let mut node = prepared.node;
        let mut data = prepared.data;
        while let TsTypeData::TsParenthesizedType(parentheses) = data {
            node = parentheses.type_annotation(self.ast);
            data = self.ast.ts_type_data(node);
        }
        if prepared.source_owner == Some(node) {
            return;
        }
        let owner = if matches!(position, SuppressionPosition::OwnLine)
            && let TsTypeData::TsUnionType(union) = data
        {
            union
                .types(self.ast)
                .first()
                .map_or(node, |id| self.ast.get_node_in_sub_range(id))
        } else {
            node
        };
        prepared.source_owner = Some(owner);
    }

    pub(super) fn format_non_null(
        &mut self,
        node: TsNonNullExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let expression = node.expression(self.ast);
        let suffix = Span::new(expression.span(self.ast).end, node.span(self.ast).end);
        let comments = self.comments.in_range(suffix);
        let Some(operator) = self.find_token(suffix, comments, "!") else {
            return Err(self.unsupported("non-null assertion token", node));
        };
        let gap = Span::new(suffix.start, operator);
        let mode = self.content_mode(comments);
        let prefix = if mode == ContentMode::Verbatim {
            TypeExpressionGap::Source(gap)
        } else {
            TypeExpressionGap::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        };
        let tail = self.format_expression(
            expression,
            ExpressionContext::at(ExpressionPosition::NonNullOperand)
                .with_expression_start(context.expression_start)
                .with_tail_owner(TailOwner::ParentOperation)
                .with_content_mode(mode),
            writer,
        )?;
        self.format_expression_tail(tail, writer)?;
        self.format_type_expression_gap(prefix, FlatSeparator::Empty, writer)?;
        writer.text("!");
        Ok(None)
    }

    pub(super) fn format_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = node.span(self.ast);
        let expression = node.expression(self.ast);
        let expression_span = expression.span(self.ast);
        let annotation = node.type_annotation(self.ast);
        let annotation_span = annotation.span(self.ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let before = Span::new(span.start + 1, annotation_span.start);
        let leading = self.comments.in_range(before);
        if self.has_suppression && self.content_mode(leading) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(leading, before.end) {
                self.select_expression_type_head(&mut type_node, position);
            }
        }
        let before = self.prepare_type_expression_gap(before, type_node.source_owner.is_some());
        let suffix = Span::new(annotation_span.end, expression_span.start);
        let comments = self.comments.in_range(suffix);
        let Some(closing) = self.find_token(suffix, comments, ">") else {
            return Err(self.unsupported("type assertion closing token", node));
        };
        let split = comments.partition_point(|comment| comment.span.end <= closing);
        let (type_tail, operand_head) = comments.split_at(split);
        let type_source =
            self.has_suppression && self.content_mode(type_tail) == ContentMode::Verbatim;
        if type_source {
            self.select_expression_type_head(&mut type_node, SuppressionPosition::Inline);
        }
        let after_type =
            self.prepare_type_expression_gap(Span::new(suffix.start, closing), type_source);
        let mode = self.content_mode(operand_head);
        let after_angle = self.prepare_type_expression_gap(
            Span::new(closing + 1, expression_span.start),
            mode == ContentMode::Verbatim,
        );
        writer.group(false, |writer| {
            writer.text("<");
            writer.indent(|writer| {
                self.format_type_expression_gap(before, FlatSeparator::Empty, writer)?;
                let tail = self.format_prepared_type(type_node, TypeContext::Root, writer)?;
                self.format_expression_tail(tail, writer)?;
                self.format_type_expression_gap(after_type, FlatSeparator::Empty, writer)
            })?;
            writer.text(">");
            self.format_type_expression_gap(after_angle, FlatSeparator::Empty, writer)?;
            self.format_expression(
                expression,
                ExpressionContext::at(ExpressionPosition::TypeAssertionOperand)
                    .with_expression_start(context.expression_start)
                    .with_content_mode(mode)
                    .with_tail_owner(context.tail_owner),
                writer,
            )
        })
    }

    fn prepare_type_expression_gap(&self, gap: Span, source: bool) -> TypeExpressionGap<'ast> {
        if source {
            TypeExpressionGap::Source(gap)
        } else {
            let comments = self.comments.in_range(gap);
            TypeExpressionGap::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        }
    }

    fn format_type_expression_gap(
        &mut self,
        gap: TypeExpressionGap<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match gap {
            TypeExpressionGap::Source(span) => self.format_verbatim(span, writer),
            TypeExpressionGap::Formatted(boundary) => {
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, separator, |_| {});
                Ok(())
            }
        }
    }
}

pub(super) fn type_expression_needs_parentheses(
    data: ExprData,
    position: ExpressionPosition,
) -> Option<bool> {
    match data {
        ExprData::TsAsExpression(_) | ExprData::TsSatisfiesExpression(_) => Some(matches!(
            position,
            ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::ParenthesizedUnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::YieldArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::BinaryOperand { .. }
                | ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::ConditionalTest
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::NonNullOperand
        )),
        ExprData::TsTypeAssertion(_) => Some(matches!(
            position,
            ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::TypeBinaryOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::BinaryOperand {
                    operator: BinaryOperatorKind::Binary(BinaryOperator::Exponent),
                    side: OperandSide::Left,
                }
        )),
        _ => None,
    }
}
