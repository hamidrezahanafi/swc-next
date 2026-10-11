//! TypeScript expression operators keep real operand owners apart from token gaps.

use swc_next_ecma_ast::{
    BinaryOperator, Comment, Expr, ExprData, GetSpan, Span, TsInstantiationExpression,
    TsNonNullExpression, TsType, TsTypeAssertion, TsTypeData, TsTypeParameterInstantiation,
};

use super::binary::{BinaryOperatorKind, OperandSide};
use super::parentheses::{ExpressionContent, ResolvedExpressionHead};
use super::{AccessKind, ExpressionContext, ExpressionPosition, TailOwner};
use crate::formatter::Formatter;
use crate::formatter::comments::{
    CommentBinding, CommentBoundary, CommentRunLayout, ExpressionTail, NodeHead,
};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::{PreparedDelimitedType, PreparedType, TypeContext, TypeLayout};
use crate::{
    FormatError,
    document::{GroupRequest, Writer},
};

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
pub(in crate::formatter) struct ExpressionTypeArguments<'ast> {
    pub(super) node: TsTypeParameterInstantiation,
    pub(in crate::formatter) callee_mode: ContentMode,
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

struct TypeAssertionHead<'ast> {
    body: PreparedDelimitedType<'ast>,
    before: TypeExpressionGap<'ast>,
    closing: TypeExpressionGap<'ast>,
}

enum AssertionOperandLayout {
    Direct,
    Enclosable,
}

/// A protected token gap stays source-owned; a relocatable head is prepared
/// through every removed source envelope before either layout candidate prints it.
enum TypeAssertionOperandHead<'ast> {
    Boundary {
        before: TypeExpressionGap<'ast>,
        content: Option<ExpressionContent>,
    },
    Prepared {
        head: ResolvedExpressionHead<'ast>,
        content: ExpressionContent,
    },
}

struct TypeAssertionOperand<'ast> {
    expression: Expr,
    context: ExpressionContext,
    head: TypeAssertionOperandHead<'ast>,
    layout: AssertionOperandLayout,
}

impl TypeAssertionOperandHead<'_> {
    fn has_leading_separator(&self) -> bool {
        matches!(self, Self::Prepared { head, .. } if !head.head.comments.is_empty())
    }
}

struct PreparedTypeAssertion<'ast> {
    head: TypeAssertionHead<'ast>,
    operand: TypeAssertionOperand<'ast>,
}

/// One assertion's source-owned operand and prepared token boundary.
pub(super) struct SourceNonNull<'ast> {
    pub(super) expression: Expr,
    mode: ContentMode,
    prefix: TypeExpressionGap<'ast>,
    has_comments: bool,
}

impl SourceNonNull<'_> {
    pub(super) fn operand_context(&self, context: ExpressionContext) -> ExpressionContext {
        context
            .type_operand(
                ExpressionPosition::NonNullOperand,
                self.mode,
                TailOwner::ParentOperation,
            )
            .with_conditional_envelope(context.conditional_envelope.chain_head())
    }

    pub(super) fn has_comments(&self) -> bool {
        self.has_comments
    }

    pub(super) fn source_break(&self) -> bool {
        self.has_comments
            || matches!(&self.prefix, TypeExpressionGap::Formatted(boundary) if boundary.gap().requires_line())
    }
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
    /// Postfix chain wrappers retain their source owners while sharing the operand's shape.
    pub(super) fn chain_element_data(&self, mut data: ExprData) -> ExprData {
        loop {
            let expression = match data {
                ExprData::ChainExpression(chain) => chain.expression(self.ast),
                ExprData::TsNonNullExpression(assertion) => assertion.expression(self.ast),
                _ => return data,
            };
            data = self.expression_content(expression).data;
        }
    }

    /// Collection arguments can keep their call delimiters around a type wrapper.
    pub(super) fn call_argument_is_collection(&self, mut expression: Expr) -> bool {
        loop {
            let content = self.expression_content(expression);
            expression = match content.data {
                ExprData::ArrayExpression(_) | ExprData::ObjectExpression(_) => return true,
                ExprData::TsAsExpression(assertion) => assertion.expression(self.ast),
                ExprData::TsSatisfiesExpression(assertion) => assertion.expression(self.ast),
                ExprData::TsTypeAssertion(assertion) => assertion.expression(self.ast),
                _ => return false,
            };
        }
    }

    /// A short cast neighbor has an atomic type after bounded array/reference peeling.
    pub(super) fn short_argument_type(&self, mut annotation: TsType) -> bool {
        let ast = self.ast;
        for _ in 0..2 {
            let TsTypeData::TsArrayType(array) = ast.ts_type_data(annotation) else {
                break;
            };
            annotation = array.element_type(ast);
        }
        if let TsTypeData::TsTypeReference(reference) = ast.ts_type_data(annotation)
            && let Some(arguments) = reference.type_arguments(ast)
        {
            let parameters = arguments.params(ast);
            if parameters.len() == 1
                && let Some(parameter) = parameters.first()
            {
                annotation = ast.get_node_in_sub_range(parameter);
            }
        }
        match ast.ts_type_data(annotation) {
            TsTypeData::TsAnyKeyword(_)
            | TsTypeData::TsUnknownKeyword(_)
            | TsTypeData::TsNeverKeyword(_)
            | TsTypeData::TsVoidKeyword(_)
            | TsTypeData::TsNullKeyword(_)
            | TsTypeData::TsUndefinedKeyword(_)
            | TsTypeData::TsStringKeyword(_)
            | TsTypeData::TsNumberKeyword(_)
            | TsTypeData::TsBigIntKeyword(_)
            | TsTypeData::TsBooleanKeyword(_)
            | TsTypeData::TsSymbolKeyword(_)
            | TsTypeData::TsObjectKeyword(_)
            | TsTypeData::TsIntrinsicKeyword(_)
            | TsTypeData::TsThisType(_)
            | TsTypeData::TsLiteralType(_)
            | TsTypeData::TsTemplateLiteralType(_) => true,
            TsTypeData::TsTypeReference(reference) => reference.type_arguments(ast).is_none(),
            _ => false,
        }
    }

    pub(in crate::formatter) fn prepare_expression_type_arguments(
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

    pub(in crate::formatter) fn format_expression_type_arguments(
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

    pub(super) fn format_instantiation(
        &mut self,
        node: TsInstantiationExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let expression = node.expression(self.ast);
        let arguments = self.prepare_expression_type_arguments(
            expression,
            node.type_arguments(self.ast),
            AccessKind::Ordinary,
        )?;
        let tail = self.format_expression(
            expression,
            context.type_operand(
                ExpressionPosition::InstantiationOperand,
                arguments.callee_mode,
                TailOwner::ParentOperation,
            ),
            writer,
        )?;
        self.format_expression_type_arguments(arguments, tail, writer)?;
        Ok(None)
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
                context
                    .type_operand(
                        ExpressionPosition::TypeBinaryOperand,
                        boundary.left_mode,
                        TailOwner::ParentOperation,
                    )
                    .with_conditional_envelope(context.conditional_envelope.terminal_head()),
                writer,
            )?;
            self.format_expression_tail(tail, writer)?;
            self.format_binary_type_right(boundary, type_node, writer)
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

    fn format_binary_type_right(
        &mut self,
        boundary: BinaryTypeBoundary<'ast>,
        node: PreparedType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match boundary.layout {
            BinaryTypeLayout::Source(gap) => {
                self.format_verbatim(gap, writer)?;
                self.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    TypeLayout::OpeningProvided,
                    writer,
                )
            }
            BinaryTypeLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::reserved_space);
                writer.text(boundary.operator.token());
                writer.reserved_space();
                self.format_prepared_type_after_comments(
                    node,
                    TypeContext::Root,
                    &after,
                    TypeLayout::Root,
                    writer,
                )
            }
        }
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

    pub(super) fn source_non_null(
        &self,
        node: TsNonNullExpression,
    ) -> Result<SourceNonNull<'ast>, FormatError> {
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
        Ok(SourceNonNull {
            expression,
            mode,
            prefix,
            has_comments: !comments.is_empty(),
        })
    }

    pub(super) fn format_source_non_null(
        &mut self,
        source: SourceNonNull<'ast>,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_expression_tail(tail, writer)?;
        self.format_type_expression_gap(source.prefix, FlatSeparator::Empty, writer)?;
        writer.text("!");
        Ok(())
    }

    pub(super) fn format_non_null(
        &mut self,
        node: TsNonNullExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let source = self.source_non_null(node)?;
        let tail =
            self.format_expression(source.expression, source.operand_context(context), writer)?;
        self.format_source_non_null(source, tail, writer)?;
        Ok(None)
    }

    pub(super) fn format_type_assertion(
        &mut self,
        node: TsTypeAssertion,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedTypeAssertion { head, operand } = self.prepare_type_assertion(node, context)?;
        writer.group(false, |writer| {
            if matches!(operand.layout, AssertionOperandLayout::Direct) {
                writer.group(false, |writer| {
                    self.format_type_assertion_head(head, writer)
                })?;
                if operand.head.has_leading_separator() {
                    writer.space();
                }
                return self.format_type_assertion_operand(operand, writer);
            }
            let (head, ()) = writer
                .record_group_with_value(|writer| self.format_type_assertion_head(head, writer))?;
            let leading_separator = operand.head.has_leading_separator();
            let (operand, tail) = writer
                .record_with_value(|writer| self.format_type_assertion_operand(operand, writer))?;
            let direct = writer.join_fragments(head.grouped(), leading_separator, operand);
            let enclosed = writer.conditional_group_fragment(
                head.grouped(),
                GroupRequest::Expanded,
                "(",
                operand,
                ")",
            );
            writer.write_alternatives(direct, &[direct, enclosed], direct);
            // Ordinary pending comments retain their enclosing expression owner.
            // Each candidate finishes its operand before returning the same lease.
            Ok(tail)
        })
    }

    fn prepare_type_assertion(
        &self,
        node: TsTypeAssertion,
        context: ExpressionContext,
    ) -> Result<PreparedTypeAssertion<'ast>, FormatError> {
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
        let before =
            self.prepare_type_expression_gap(before, leading, type_node.source_owner.is_some());
        let body = self.prepare_delimited_type(type_node)?;
        let after_type = self.prepare_type_expression_gap(
            Span::new(suffix.start, closing),
            type_tail,
            type_source,
        );
        let mode = self.content_mode(operand_head);
        let after_angle = self.prepare_type_expression_gap(
            Span::new(closing + 1, expression_span.start),
            operand_head,
            mode == ContentMode::Verbatim,
        );
        // Keep protected operands direct before preparing optional document variants.
        let content = (mode == ContentMode::Formatted).then(|| self.expression_content(expression));
        let direct_body = content.as_ref().is_some_and(|content| {
            matches!(
                content.data,
                ExprData::ArrayExpression(_)
                    | ExprData::ObjectExpression(_)
                    // The prepared peel stops at this actual protected source pair.
                    | ExprData::ParenthesizedExpression(_)
            )
        });
        let head = match (after_angle, content) {
            (TypeExpressionGap::Formatted(boundary), Some(content)) => {
                let binding = self.expression_head_binding(operand_head, expression);
                let binding = content
                    .parentheses
                    .map_or(binding, |removed| binding.combine(removed.head.binding));
                if matches!(binding, CommentBinding::SourceBound)
                    || self.diagnostic_range_is_sensitive(span)
                {
                    TypeAssertionOperandHead::Boundary {
                        before: TypeExpressionGap::Formatted(boundary),
                        content: Some(content),
                    }
                } else {
                    let start = content.expression.span(self.ast).start;
                    let comments = self.comments.in_range(Span::new(closing + 1, start));
                    TypeAssertionOperandHead::Prepared {
                        head: ResolvedExpressionHead {
                            head: NodeHead {
                                comments,
                                content_start: start,
                                layout: CommentRunLayout::FlatLeading(start),
                                gap_end: None,
                            },
                            binding,
                        },
                        content,
                    }
                }
            }
            (before, content) => TypeAssertionOperandHead::Boundary { before, content },
        };
        let layout = if mode == ContentMode::Verbatim
            || direct_body
            || self.diagnostic_range_is_sensitive(span)
            || matches!(head, TypeAssertionOperandHead::Boundary { .. })
        {
            AssertionOperandLayout::Direct
        } else {
            let movable = match &head {
                TypeAssertionOperandHead::Prepared { head, content } => head
                    .binding
                    .allows_relocation(Some(content.expression.span(self.ast))),
                TypeAssertionOperandHead::Boundary { .. } => false,
            };
            if movable {
                AssertionOperandLayout::Enclosable
            } else {
                AssertionOperandLayout::Direct
            }
        };
        Ok(PreparedTypeAssertion {
            head: TypeAssertionHead {
                body,
                before,
                closing: after_type,
            },
            operand: TypeAssertionOperand {
                expression,
                context: context.type_operand(
                    ExpressionPosition::TypeAssertionOperand,
                    mode,
                    context.tail_owner,
                ),
                head,
                layout,
            },
        })
    }

    /// The actual angle caller owns the opening gap and the type body's indentation.
    fn format_type_assertion_head(
        &mut self,
        head: TypeAssertionHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("<");
        let closing = writer.indent(|writer| {
            match head.before {
                TypeExpressionGap::Source(span) => self.format_verbatim(span, writer)?,
                TypeExpressionGap::Formatted(boundary) => {
                    self.format_comment_boundary(&boundary, writer)?;
                    boundary
                        .gap()
                        .write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
            }
            let tail = self.format_prepared_delimited_type(
                head.body,
                TypeContext::Root,
                TypeLayout::CallerIndented,
                writer,
            )?;
            self.format_expression_tail(tail, writer)?;
            match head.closing {
                TypeExpressionGap::Source(span) => {
                    self.format_verbatim(span, writer)?;
                    Ok(None)
                }
                TypeExpressionGap::Formatted(boundary) => {
                    self.format_comment_boundary(&boundary, writer)?;
                    Ok(Some(boundary.gap()))
                }
            }
        })?;
        if let Some(closing) = closing {
            closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
        }
        writer.text(">");
        Ok(())
    }

    fn format_type_assertion_operand(
        &mut self,
        operand: TypeAssertionOperand<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match operand.head {
            TypeAssertionOperandHead::Prepared { head, content } => {
                self.comments.claim(head.head.comments)?;
                self.format_prepared_expression_with_resolved_head(
                    operand.expression,
                    content,
                    head,
                    operand.context,
                    writer,
                )
            }
            TypeAssertionOperandHead::Boundary { before, content } => {
                self.format_type_expression_gap(before, FlatSeparator::Empty, writer)?;
                self.format_expression_prepared_with_array_comments(
                    operand.expression,
                    content,
                    operand.context,
                    None,
                    writer,
                )
            }
        }
    }

    fn prepare_type_expression_gap(
        &self,
        gap: Span,
        comments: &'ast [Comment],
        source: bool,
    ) -> TypeExpressionGap<'ast> {
        if source {
            TypeExpressionGap::Source(gap)
        } else {
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
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::ConditionalTest
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
        )),
        ExprData::TsTypeAssertion(_) => Some(matches!(
            position,
            ExpressionPosition::MemberObject(_)
                | ExpressionPosition::MemberAssignmentObject
                | ExpressionPosition::AssignmentTarget
                | ExpressionPosition::UpdateArgument
                | ExpressionPosition::CallCallee { .. }
                | ExpressionPosition::TemplateTag
                | ExpressionPosition::NewCallee
                | ExpressionPosition::Superclass
                | ExpressionPosition::TypeBinaryOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
                | ExpressionPosition::BinaryOperand {
                    operator: BinaryOperatorKind::Binary(BinaryOperator::Exponent),
                    side: OperandSide::Left,
                }
        )),
        ExprData::TsInstantiationExpression(_)
            if matches!(
                position,
                ExpressionPosition::MemberObject(_)
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::CallCallee { .. }
                    | ExpressionPosition::TemplateTag
                    | ExpressionPosition::NewCallee
                    | ExpressionPosition::Superclass
                    | ExpressionPosition::NonNullOperand
                    | ExpressionPosition::InstantiationOperand
                    | ExpressionPosition::BinaryOperand {
                        operator: BinaryOperatorKind::Binary(_),
                        side: OperandSide::Left,
                    }
            ) =>
        {
            Some(true)
        }
        _ => None,
    }
}
