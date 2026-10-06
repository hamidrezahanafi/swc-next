//! Expression dispatch and immediate grammar and statement-start contexts.

pub(super) mod arguments;
mod arrays;
mod arrows;
pub(super) mod assignment;
mod binary;
mod callbacks;
mod calls;
mod chains;
mod conditional;
mod constructors;
mod for_initializer;
pub(super) mod imports;
mod members;
pub(super) mod objects;
pub(super) mod parentheses;
mod sequence;
mod suspension;
pub(super) mod targets;
mod typescript;
mod unary;

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, AssignmentTargetData, CallExpression, Expr, ExprData,
    GetSpan, ObjectExpression, SimpleAssignmentTargetData, Span, UnaryOperator, UpdateOperator,
};

use self::arrays::{ArrayArgumentArity, RecordedArray, RecordedArrayArgument};
use self::assignment::AssignmentParent;
use self::binary::{BinaryLike, BinaryOperatorKind, OperandSide};
use self::chains::FluentChainEligibility;
use self::objects::RecordedObject;
use super::literals::StringContext;
use super::suppression::ContentMode;
use super::{
    Formatter,
    comments::{ArrayCommentOwner, CommentRunLayout, ExpressionTail, NodeHead},
};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AccessKind {
    Ordinary,
    Optional,
}

#[derive(Clone, Copy)]
pub(super) enum ExpressionPosition {
    Statement,
    ExportDefault,
    Nested,
    UnaryArgument(UnaryOperator),
    ParenthesizedUnaryArgument(UnaryOperator),
    BinaryOperand {
        operator: BinaryOperatorKind,
        side: OperandSide,
    },
    TypeBinaryOperand,
    TypeAssertionOperand,
    NonNullOperand,
    InstantiationOperand,
    SpreadArgument,
    PropertyValue,
    MemberObject(AccessKind),
    CallCallee {
        access: AccessKind,
        curried_call: Option<CallExpression>,
    },
    NewCallee,
    Superclass,
    ArrowBody,
    RestrictedArgument,
    YieldArgument,
    AwaitArgument,
    MemberAssignmentObject,
    ConditionalTest,
    ControlCondition,
    ForHead,
    ForInitializer {
        parentheses: bool,
    },
    ForLeft {
        parentheses: bool,
    },
    ConditionalConsequent,
    ConditionalAlternate,
    AssignmentRight(AssignmentParent),
    EnumInitializer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpressionStart {
    None,
    StatementPending,
    StatementGuard,
    StatementGuarded,
    ArrowBody,
}

impl ExpressionStart {
    fn is_statement(self) -> bool {
        matches!(
            self,
            Self::StatementPending | Self::StatementGuard | Self::StatementGuarded
        )
    }

    fn needs_asi_guard(self) -> bool {
        matches!(self, Self::StatementPending | Self::StatementGuard)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HeadComments {
    Pending,
    Claimed,
}

/// A surrounding operation may own the operand's trailing comment boundary.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TailOwner {
    Expression,
    ParentOperation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParenthesesOwner {
    Expression,
    Parent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MemberLayout {
    Normal,
    Tight,
}

/// Assignment and update target spines retain member-owned lookup groups.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MemberChainLayout {
    Fluent,
    Local,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum OperandIndent {
    Expression,
    Parent,
}

/// Optional breaks for a complete expression belong to its actual source line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ExpressionLayout {
    Normal,
    Flat,
}

impl ExpressionLayout {
    pub(super) fn write<'a, T>(
        self,
        writer: &mut Writer<'a>,
        write: impl FnOnce(&mut Writer<'a>) -> T,
    ) -> T {
        match self {
            Self::Normal => write(writer),
            Self::Flat => writer.flat(write),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionContext {
    position: ExpressionPosition,
    expression_start: ExpressionStart,
    head_comments: HeadComments,
    content_mode: ContentMode,
    tail_owner: TailOwner,
    parentheses_owner: ParenthesesOwner,
    member_layout: MemberLayout,
    member_chain_layout: MemberChainLayout,
    arrow_layout: arrows::ArrowLayout,
    operand_indent: OperandIndent,
}

impl ExpressionContext {
    pub(super) fn variable_initializer() -> Self {
        Self::at(ExpressionPosition::AssignmentRight(
            AssignmentParent::Statement,
        ))
        .with_arrow_layout(arrows::ArrowLayout::Assignment(
            arrows::ArrowHeadGap::Expression,
        ))
    }

    pub(super) fn binding_initializer() -> Self {
        Self::at(ExpressionPosition::AssignmentRight(
            AssignmentParent::Nested,
        ))
        .with_arrow_layout(arrows::ArrowLayout::Ordinary)
    }

    pub(super) fn at(position: ExpressionPosition) -> Self {
        Self {
            position,
            expression_start: match position {
                ExpressionPosition::Statement => ExpressionStart::StatementPending,
                ExpressionPosition::ArrowBody => ExpressionStart::ArrowBody,
                _ => ExpressionStart::None,
            },
            head_comments: HeadComments::Pending,
            content_mode: ContentMode::Formatted,
            tail_owner: TailOwner::Expression,
            parentheses_owner: ParenthesesOwner::Expression,
            member_layout: MemberLayout::Normal,
            member_chain_layout: MemberChainLayout::Fluent,
            operand_indent: OperandIndent::Expression,
            arrow_layout: match position {
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::NewCallee => {
                    arrows::ArrowLayout::Callee
                }
                ExpressionPosition::AssignmentRight(_) => {
                    arrows::ArrowLayout::Assignment(arrows::ArrowHeadGap::Expression)
                }
                ExpressionPosition::BinaryOperand { .. } => arrows::ArrowLayout::Argument,
                _ => arrows::ArrowLayout::Ordinary,
            },
        }
    }

    pub(super) fn with_operand_indent(mut self, owner: OperandIndent) -> Self {
        self.operand_indent = owner;
        self
    }

    fn with_arrow_layout(mut self, layout: arrows::ArrowLayout) -> Self {
        self.arrow_layout = layout;
        self
    }

    pub(super) fn with_expression_start(mut self, start: ExpressionStart) -> Self {
        self.expression_start = start;
        self
    }

    fn inside_parentheses(mut self) -> Self {
        self.parentheses_owner = ParenthesesOwner::Parent;
        self.expression_start = ExpressionStart::None;
        self
    }

    pub(super) fn with_content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }

    fn member_receiver(self, access: AccessKind, mode: ContentMode) -> Self {
        let position = if matches!(self.position, ExpressionPosition::MemberAssignmentObject) {
            ExpressionPosition::MemberAssignmentObject
        } else {
            ExpressionPosition::MemberObject(access)
        };
        Self::at(position)
            .with_expression_start(self.expression_start)
            .with_tail_owner(TailOwner::ParentOperation)
            .with_content_mode(mode)
            .with_member_layout(self.member_layout)
            .with_member_chain_layout(self.member_chain_layout)
    }

    fn with_member_layout(mut self, layout: MemberLayout) -> Self {
        self.member_layout = layout;
        self
    }

    fn with_member_chain_layout(mut self, layout: MemberChainLayout) -> Self {
        self.member_chain_layout = layout;
        self
    }

    pub(super) fn with_tail_owner(mut self, owner: TailOwner) -> Self {
        self.tail_owner = owner;
        self
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_expression_with_array_comments(expression, context, None, writer)
    }

    fn format_expression_with_array_comments(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        mut array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.content_mode == ContentMode::Verbatim {
            let span = expression.span(self.ast);
            if matches!(context.position, ExpressionPosition::ForInitializer { .. }) {
                // This owner was parsed in NoIn grammar. Its source delimiters
                // remain necessary even when the ignored child would print alone.
                self.format_verbatim(span, writer)?;
                if let Some(owner) = array_comments {
                    owner.finish_source(self, span.end);
                }
                return Ok(None);
            }
            if let ExprData::ParenthesizedExpression(parenthesized) = self.ast.expr_data(expression)
                && !matches!(context.position, ExpressionPosition::Statement)
            {
                let child = parenthesized.expression(self.ast);
                let child_span = child.span(self.ast);
                let boundary = parentheses::SourceParentheses {
                    outer: span,
                    inner: child_span,
                    inner_target: parentheses::ParenthesisInner::Expression(child),
                };
                if self.parentheses_are_removable(boundary)
                    && self.expression_needs_parentheses(
                        self.ast.expr_data(child),
                        context.position,
                        ContentMode::Verbatim,
                    ) == Some(false)
                {
                    let context = context.with_content_mode(ContentMode::Verbatim);
                    return match array_comments {
                        Some(owner) => self.format_source_parentheses_with_array_comments(
                            parenthesized,
                            context,
                            Some(owner),
                            writer,
                        ),
                        None => self.format_source_parentheses(parenthesized, context, writer),
                    };
                }
            }
            self.format_verbatim_expression(expression, context, writer)?;
            if let Some(owner) = array_comments {
                owner.finish_source(self, span.end);
            }
            return Ok(None);
        }
        let layout = self.expression_layout(expression);
        let tail = layout.write(writer, |writer| {
            self.format_expression_content(
                expression,
                context,
                layout,
                array_comments.as_deref_mut(),
                writer,
            )
        })?;
        if let Some(owner) = array_comments {
            // A pending tail is still owned by the enclosing source boundary.
            let end = match &tail {
                Some(tail) => tail.source_end,
                None => expression.span(self.ast).end,
            };
            owner.finish_source(self, end);
        }
        Ok(tail)
    }

    fn expression_layout(&self, expression: Expr) -> ExpressionLayout {
        if self.diagnostic_lines.is_empty() {
            return ExpressionLayout::Normal;
        }
        let span = expression.span(self.ast);
        if self.diagnostic_line_is_sensitive(span.start)
            && self.line_breaks(span.start, span.end) == 0
        {
            ExpressionLayout::Flat
        } else {
            ExpressionLayout::Normal
        }
    }

    fn format_verbatim_expression(
        &mut self,
        expression: Expr,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let data = self.ast.expr_data(expression);
        let span = expression.span(self.ast);
        let source = self.source(span);
        let parenthesize = self.expression_is_parenthesized(data, context);
        let starts_update = matches!(data, ExprData::UpdateExpression(update) if update.prefix(self.ast))
            && (source.starts_with("++") || source.starts_with("--"));
        if self.options.semicolons == Semicolons::AsNeeded
            && context.expression_start.needs_asi_guard()
            && (parenthesize
                || matches!(data, ExprData::TsTypeAssertion(_))
                || !starts_update && source.starts_with(['(', '[', '/', '+', '-', '`']))
        {
            writer.text(";");
        }
        if parenthesize {
            writer.text("(");
        }
        self.format_verbatim(span, writer)?;
        if parenthesize {
            writer.text(")");
        }
        Ok(())
    }

    fn format_expression_content(
        &mut self,
        expression: Expr,
        mut context: ExpressionContext,
        layout: ExpressionLayout,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let data = self.ast.expr_data(expression);
        if let ExprData::ParenthesizedExpression(parenthesized) = data {
            return match array_comments {
                Some(owner) => self.format_source_parentheses_with_array_comments(
                    parenthesized,
                    context,
                    Some(owner),
                    writer,
                ),
                None => self.format_source_parentheses(parenthesized, context, writer),
            };
        }
        if let ExprData::SequenceExpression(sequence) = data {
            return self.format_sequence(sequence, context, writer);
        }
        let binary = BinaryLike::from_data(data, self.ast);
        let parenthesize = self.expression_is_parenthesized(data, context);
        if parenthesize
            && matches!(
                context.position,
                ExpressionPosition::ForInitializer { parentheses: true }
                    | ExpressionPosition::ForLeft { parentheses: true }
            )
        {
            return self.format_parenthesized_expression(
                expression,
                NodeHead {
                    comments: &[],
                    content_start: expression.span(self.ast).start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                context,
                writer,
            );
        }
        if context.expression_start == ExpressionStart::StatementPending
            && self.options.semicolons == Semicolons::AsNeeded
        {
            let mut asi_guard = parenthesize
                || match data {
                    ExprData::ArrayExpression(_)
                    | ExprData::RegExpLiteral(_)
                    | ExprData::TsTypeAssertion(_) => true,
                    ExprData::UnaryExpression(unary) => matches!(
                        unary.operator(self.ast),
                        UnaryOperator::Positive | UnaryOperator::Negate
                    ),
                    _ => false,
                };
            if let Some(binary) = binary {
                let left = self.expression_content(binary.left).data;
                asi_guard |= self.expression_needs_parentheses(
                    left,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                    ContentMode::Formatted,
                ) == Some(true);
            }
            if asi_guard {
                context.expression_start = ExpressionStart::StatementGuard;
            }
        }
        let conditional = matches!(data, ExprData::ConditionalExpression(_));
        if context.expression_start == ExpressionStart::StatementGuard
            && (binary.is_none() && !conditional || parenthesize)
        {
            writer.text(";");
            context.expression_start = ExpressionStart::None;
        }
        if parenthesize
            && self.has_type_casts
            && self
                .source_cast_before(expression.span(self.ast).start)
                .is_some()
        {
            return self.format_parenthesized_expression(
                expression,
                NodeHead {
                    comments: &[],
                    content_start: expression.span(self.ast).start,
                    layout: CommentRunLayout::Source,
                    gap_end: None,
                },
                context,
                writer,
            );
        }
        if let Some(binary) = binary {
            if parenthesize
                && matches!(
                    context.position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                )
            {
                return writer.group(false, |writer| {
                    writer.text("(");
                    writer.indent(|writer| {
                        writer.soft_line();
                        let tail = self.format_binary_enclosed(binary, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    writer.soft_line();
                    writer.text(")");
                    Ok(None)
                });
            }
            if parenthesize {
                writer.text("(");
            }
            let tail = self.format_binary(binary, context, writer)?;
            if parenthesize {
                writer.text(")");
            }
            return Ok(tail);
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && parenthesize
            && matches!(
                context.position,
                ExpressionPosition::MemberObject(_)
                    | ExpressionPosition::MemberAssignmentObject
                    | ExpressionPosition::CallCallee { .. }
                    | ExpressionPosition::NewCallee
            )
        {
            return writer.group(false, |writer| {
                writer.text("(");
                let tail = self.format_conditional(conditional, context, writer)?;
                self.format_expression_tail(tail, writer)?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        if let ExprData::ConditionalExpression(conditional) = data
            && matches!(context.position, ExpressionPosition::ConditionalTest)
        {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = self.format_conditional(conditional, context, writer)?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        if parenthesize {
            writer.text("(");
        }
        let tail = match data {
            ExprData::StringLiteral(literal) => {
                self.format_string_literal(literal, StringContext::Expression, writer);
                Ok(None)
            }
            ExprData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(None)
            }
            ExprData::Super(_) => {
                writer.text("super");
                Ok(None)
            }
            ExprData::PrivateIdentifier(identifier) => {
                writer.text("#");
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(None)
            }
            ExprData::ThisExpression(_) => {
                writer.text("this");
                Ok(None)
            }
            ExprData::BooleanLiteral(literal) => {
                writer.text(if literal.value(self.ast) {
                    "true"
                } else {
                    "false"
                });
                Ok(None)
            }
            ExprData::NullLiteral(_) => {
                writer.text("null");
                Ok(None)
            }
            ExprData::NumericLiteral(literal) => {
                self.format_numeric_literal(literal, writer);
                Ok(None)
            }
            ExprData::BigIntLiteral(literal) => {
                self.format_bigint_literal(literal, writer);
                Ok(None)
            }
            ExprData::RegExpLiteral(literal) => {
                self.format_regexp_literal(literal, writer);
                Ok(None)
            }
            ExprData::ArrayExpression(array) => self.format_array(array, writer).map(|()| None),
            ExprData::ObjectExpression(object) => self.format_object(object, writer).map(|()| None),
            ExprData::MemberExpression(member) => {
                let eligibility = if context.member_chain_layout == MemberChainLayout::Local {
                    FluentChainEligibility::Unqualified
                } else {
                    self.fluent_chain_eligibility(data, context, layout)
                };
                match eligibility {
                    FluentChainEligibility::Fluent => {
                        self.format_fluent_chain(expression, context, writer)
                    }
                    FluentChainEligibility::MemberOnly => self.format_member(
                        member,
                        context.with_member_chain_layout(MemberChainLayout::Local),
                        writer,
                    ),
                    FluentChainEligibility::Unqualified => {
                        self.format_member(member, context, writer)
                    }
                }
            }
            ExprData::CallExpression(call) => {
                if !chains::atomic_chain_head(self.ast.expr_data(call.callee(self.ast)))
                    && matches!(
                        self.fluent_chain_eligibility(data, context, layout),
                        FluentChainEligibility::Fluent
                    )
                {
                    self.format_fluent_chain(expression, context, writer)
                } else {
                    self.format_call(call, context, writer)
                }
            }
            ExprData::ChainExpression(chain) => {
                let mut child_context = context;
                child_context.position = ExpressionPosition::Nested;
                if parenthesize {
                    child_context = child_context.inside_parentheses();
                }
                self.format_expression(chain.expression(self.ast), child_context, writer)
            }
            ExprData::NewExpression(constructor) => self.format_new(constructor, writer),
            ExprData::Function(function) => self.format_function(function, writer).map(|()| None),
            ExprData::Class(class) => self.format_class(class, writer).map(|()| None),
            ExprData::ArrowFunctionExpression(arrow) => self.format_arrow(arrow, context, writer),
            ExprData::AwaitExpression(await_expression) => {
                self.format_await(await_expression, writer)
            }
            ExprData::YieldExpression(yield_expression) => {
                self.format_yield(yield_expression, writer)
            }
            ExprData::MetaProperty(meta) => self.format_meta_property(meta, writer).map(|()| None),
            ExprData::ImportExpression(import) => self.format_import_expression(import, writer),
            ExprData::UnaryExpression(unary) => self.format_unary(unary, writer),
            ExprData::UpdateExpression(update) => self.format_update(update, context, writer),
            ExprData::ConditionalExpression(conditional) => {
                self.format_conditional(conditional, context, writer)
            }
            ExprData::AssignmentExpression(assignment) => {
                self.format_assignment(assignment, context, writer)
            }
            ExprData::TsAsExpression(node) => self.format_binary_type_expression(
                node.expression(self.ast),
                node.type_annotation(self.ast),
                typescript::TypeBinaryOperator::As,
                context,
                writer,
            ),
            ExprData::TsSatisfiesExpression(node) => self.format_binary_type_expression(
                node.expression(self.ast),
                node.type_annotation(self.ast),
                typescript::TypeBinaryOperator::Satisfies,
                context,
                writer,
            ),
            ExprData::TsTypeAssertion(node) => self.format_type_assertion(node, context, writer),
            ExprData::TsNonNullExpression(node) => self.format_non_null(node, context, writer),
            ExprData::TsInstantiationExpression(node) => {
                self.format_instantiation(node, context, writer)
            }
            _ => Err(self.unsupported("expression", expression)),
        }?;
        if parenthesize {
            writer.text(")");
        }
        Ok(tail)
    }

    fn expression_is_parenthesized(&self, data: ExprData, context: ExpressionContext) -> bool {
        context.parentheses_owner == ParenthesesOwner::Expression
            && (self.expression_needs_parentheses(data, context.position, context.content_mode)
                == Some(true)
                || matches!(context.position, ExpressionPosition::TypeBinaryOperand)
                    && context.expression_start.is_statement()
                    && self.type_binary_head_needs_parentheses(data)
                || matches!(data, ExprData::ObjectExpression(_))
                    && context.expression_start != ExpressionStart::None
                || matches!(data, ExprData::Function(_) | ExprData::Class(_))
                    && context.expression_start.is_statement())
    }

    /// Follow the printed head only when a guard must precede protected comments.
    fn expression_needs_asi_guard(
        &self,
        mut expression: Expr,
        mut context: ExpressionContext,
        mut content: Option<ExprData>,
    ) -> Result<bool, FormatError> {
        loop {
            if context.content_mode == ContentMode::Verbatim {
                let data = self.ast.expr_data(expression);
                let source = self.source(expression.span(self.ast));
                let update = matches!(data, ExprData::UpdateExpression(node) if node.prefix(self.ast))
                    && (source.starts_with("++") || source.starts_with("--"));
                return Ok(self.expression_is_parenthesized(data, context)
                    || matches!(data, ExprData::TsTypeAssertion(_))
                    || !update && source.starts_with(['(', '[', '/', '+', '-', '`']));
            }
            let data = content
                .take()
                .unwrap_or_else(|| self.expression_content(expression).data);
            if self.expression_is_parenthesized(data, context)
                || matches!(
                    data,
                    ExprData::ParenthesizedExpression(_)
                        | ExprData::SequenceExpression(_)
                        | ExprData::ArrayExpression(_)
                        | ExprData::RegExpLiteral(_)
                        | ExprData::TsTypeAssertion(_)
                )
                || matches!(data, ExprData::UnaryExpression(node) if matches!(node.operator(self.ast), UnaryOperator::Positive | UnaryOperator::Negate))
            {
                return Ok(true);
            }
            if let Some(binary) = BinaryLike::from_data(data, self.ast) {
                expression = binary.left;
                context.position = ExpressionPosition::BinaryOperand {
                    operator: binary.operator,
                    side: OperandSide::Left,
                };
                continue;
            }
            match data {
                ExprData::CallExpression(call) => {
                    let callee = self.source_call_callee(call, context)?;
                    expression = callee.expression;
                    context = callee.context(context);
                }
                ExprData::MemberExpression(member) => {
                    (expression, context) = self.source_member_receiver(member, context)?;
                }
                ExprData::TsAsExpression(node) => {
                    expression = node.expression(self.ast);
                    context.position = ExpressionPosition::TypeBinaryOperand;
                }
                ExprData::TsSatisfiesExpression(node) => {
                    expression = node.expression(self.ast);
                    context.position = ExpressionPosition::TypeBinaryOperand;
                }
                ExprData::TsNonNullExpression(node) => {
                    expression = node.expression(self.ast);
                    context.position = ExpressionPosition::NonNullOperand;
                }
                ExprData::TsInstantiationExpression(node) => {
                    expression = node.expression(self.ast);
                    context.position = ExpressionPosition::InstantiationOperand;
                    if self.has_suppression {
                        let arguments = self.prepare_expression_type_arguments(
                            expression,
                            node.type_arguments(self.ast),
                            AccessKind::Ordinary,
                        )?;
                        context.content_mode = arguments.callee_mode;
                    }
                }
                ExprData::ChainExpression(chain) => {
                    expression = chain.expression(self.ast);
                    context.position = ExpressionPosition::Nested;
                }
                ExprData::ConditionalExpression(conditional) => {
                    expression = conditional.test(self.ast);
                    context.position = ExpressionPosition::ConditionalTest;
                }
                ExprData::ArrowFunctionExpression(arrow) => {
                    return self.arrow_needs_asi_guard(arrow);
                }
                ExprData::AssignmentExpression(assignment) => {
                    let target = self.assignment_source_target(assignment)?;
                    return self.source_target_needs_asi_guard(&target);
                }
                ExprData::UpdateExpression(update) if !update.prefix(self.ast) => {
                    let target = self.update_source_target(update)?;
                    return self.source_target_needs_asi_guard(&target);
                }
                _ => return Ok(false),
            }
        }
    }

    /// Unknown grammar keeps source parentheses until its formatting rules are implemented.
    /// A default export cannot expose a function/class declaration head followed
    /// by expression syntax. Stop once an inner owner already supplies parentheses.
    fn default_export_head_needs_parentheses(&self, mut data: ExprData, mode: ContentMode) -> bool {
        loop {
            let (expression, position) = if let Some(binary) = BinaryLike::from_data(data, self.ast)
            {
                (
                    binary.left,
                    ExpressionPosition::BinaryOperand {
                        operator: binary.operator,
                        side: OperandSide::Left,
                    },
                )
            } else {
                match data {
                    ExprData::Function(_) | ExprData::Class(_) => return true,
                    ExprData::CallExpression(call) => (
                        call.callee(self.ast),
                        ExpressionPosition::CallCallee {
                            access: if call.optional(self.ast) {
                                AccessKind::Optional
                            } else {
                                AccessKind::Ordinary
                            },
                            curried_call: None,
                        },
                    ),
                    ExprData::MemberExpression(member) => (
                        member.object(self.ast),
                        ExpressionPosition::MemberObject(if member.optional(self.ast) {
                            AccessKind::Optional
                        } else {
                            AccessKind::Ordinary
                        }),
                    ),
                    ExprData::ConditionalExpression(conditional) => (
                        conditional.test(self.ast),
                        ExpressionPosition::ConditionalTest,
                    ),
                    ExprData::TsAsExpression(node) => (
                        node.expression(self.ast),
                        ExpressionPosition::TypeBinaryOperand,
                    ),
                    ExprData::TsSatisfiesExpression(node) => (
                        node.expression(self.ast),
                        ExpressionPosition::TypeBinaryOperand,
                    ),
                    ExprData::TsNonNullExpression(node) => (
                        node.expression(self.ast),
                        ExpressionPosition::NonNullOperand,
                    ),
                    ExprData::TsInstantiationExpression(node) => (
                        node.expression(self.ast),
                        ExpressionPosition::InstantiationOperand,
                    ),
                    ExprData::ChainExpression(chain) => {
                        (chain.expression(self.ast), ExpressionPosition::Nested)
                    }
                    ExprData::AssignmentExpression(assignment) => {
                        let AssignmentTargetData::SimpleAssignmentTarget(target) =
                            self.ast.assignment_target_data(assignment.left(self.ast))
                        else {
                            return false;
                        };
                        let SimpleAssignmentTargetData::MemberExpression(member) =
                            self.ast.simple_assignment_target_data(target)
                        else {
                            return false;
                        };
                        (Expr::MemberExpression(member), ExpressionPosition::Nested)
                    }
                    ExprData::UpdateExpression(update) if !update.prefix(self.ast) => {
                        let SimpleAssignmentTargetData::MemberExpression(member) = self
                            .ast
                            .simple_assignment_target_data(update.argument(self.ast))
                        else {
                            return false;
                        };
                        (Expr::MemberExpression(member), ExpressionPosition::Nested)
                    }
                    _ => return false,
                }
            };
            data = self.expression_content(expression).data;
            if self.expression_needs_parentheses(data, position, mode) == Some(true) {
                return false;
            }
        }
    }

    fn expression_needs_parentheses(
        &self,
        data: ExprData,
        position: ExpressionPosition,
        mode: ContentMode,
    ) -> Option<bool> {
        if mode == ContentMode::Formatted
            && matches!(
                position,
                ExpressionPosition::ForInitializer { parentheses: true }
                    | ExpressionPosition::ForLeft { parentheses: true }
            )
        {
            return Some(true);
        }
        if matches!(position, ExpressionPosition::ExportDefault)
            && self.default_export_head_needs_parentheses(data, mode)
        {
            return Some(true);
        }
        if let Some(needed) = typescript::type_expression_needs_parentheses(data, position) {
            return Some(needed);
        }
        if matches!(position, ExpressionPosition::NonNullOperand)
            && matches!(
                data,
                ExprData::UnaryExpression(_) | ExprData::UpdateExpression(_)
            )
        {
            return Some(true);
        }
        if matches!(
            position,
            ExpressionPosition::TypeBinaryOperand
                | ExpressionPosition::TypeAssertionOperand
                | ExpressionPosition::NonNullOperand
                | ExpressionPosition::InstantiationOperand
        ) && matches!(
            data,
            ExprData::ArrowFunctionExpression(_)
                | ExprData::AssignmentExpression(_)
                | ExprData::ConditionalExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::BinaryExpression(_)
                | ExprData::YieldExpression(_)
                | ExprData::AwaitExpression(_)
        ) {
            return Some(true);
        }
        match data {
            ExprData::SequenceExpression(_) => {
                return Some(!matches!(
                    position,
                    ExpressionPosition::ForHead | ExpressionPosition::ForInitializer { .. }
                ));
            }
            ExprData::ArrowFunctionExpression(_) => {
                return Some(matches!(
                    position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::ParenthesizedUnaryArgument(_)
                        | ExpressionPosition::AwaitArgument
                        | ExpressionPosition::BinaryOperand { .. }
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::Superclass
                        | ExpressionPosition::ConditionalTest
                ));
            }
            ExprData::AwaitExpression(_) | ExprData::YieldExpression(_) => {
                let low_precedence_parent = matches!(
                    position,
                    ExpressionPosition::UnaryArgument(_)
                        | ExpressionPosition::ParenthesizedUnaryArgument(_)
                        | ExpressionPosition::BinaryOperand { .. }
                        | ExpressionPosition::SpreadArgument
                        | ExpressionPosition::MemberObject(_)
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::CallCallee { .. }
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::Superclass
                        | ExpressionPosition::ConditionalTest
                );
                let awaited_yield = matches!(data, ExprData::YieldExpression(_))
                    && matches!(position, ExpressionPosition::AwaitArgument);
                return Some(low_precedence_parent || awaited_yield);
            }
            ExprData::ChainExpression(_) => {
                return Some(matches!(
                    position,
                    ExpressionPosition::MemberObject(AccessKind::Ordinary)
                        | ExpressionPosition::CallCallee {
                            access: AccessKind::Ordinary,
                            ..
                        }
                        | ExpressionPosition::MemberAssignmentObject
                        | ExpressionPosition::NewCallee
                        | ExpressionPosition::Superclass
                        | ExpressionPosition::NonNullOperand
                ));
            }
            ExprData::NewExpression(constructor) => {
                if mode == ContentMode::Formatted
                    || !matches!(
                        position,
                        ExpressionPosition::CallCallee { .. }
                            | ExpressionPosition::MemberObject(_)
                            | ExpressionPosition::MemberAssignmentObject
                            | ExpressionPosition::NewCallee
                    )
                {
                    return Some(false);
                }
                let suffix_end = constructor.type_arguments(self.ast).map_or_else(
                    || constructor.callee(self.ast).span(self.ast).end,
                    |arguments| arguments.span(self.ast).end,
                );
                return Some(constructor.span(self.ast).end == suffix_end);
            }
            ExprData::IdentifierReference(_)
            | ExprData::Function(_)
            | ExprData::Class(_)
            | ExprData::Super(_)
            | ExprData::PrivateIdentifier(_)
            | ExprData::MetaProperty(_)
            | ExprData::ThisExpression(_)
            | ExprData::StringLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_)
            | ExprData::ArrayExpression(_)
            | ExprData::ObjectExpression(_)
            | ExprData::MemberExpression(_)
            | ExprData::CallExpression(_)
            | ExprData::ImportExpression(_)
            | ExprData::UnaryExpression(_)
            | ExprData::UpdateExpression(_)
            | ExprData::BinaryExpression(_)
            | ExprData::LogicalExpression(_)
            | ExprData::ConditionalExpression(_)
            | ExprData::AssignmentExpression(_)
            | ExprData::ParenthesizedExpression(_)
            | ExprData::TsNonNullExpression(_)
            | ExprData::TsInstantiationExpression(_) => {}
            _ => return None,
        }
        if matches!(position, ExpressionPosition::NewCallee)
            && self.new_callee_contains_call(data, mode)
        {
            return Some(true);
        }
        if matches!(position, ExpressionPosition::Superclass) {
            return Some(matches!(
                data,
                ExprData::ArrowFunctionExpression(_)
                    | ExprData::SequenceExpression(_)
                    | ExprData::AssignmentExpression(_)
                    | ExprData::ConditionalExpression(_)
                    | ExprData::UnaryExpression(_)
                    | ExprData::UpdateExpression(_)
                    | ExprData::BinaryExpression(_)
                    | ExprData::LogicalExpression(_)
                    | ExprData::YieldExpression(_)
                    | ExprData::AwaitExpression(_)
                    | ExprData::ChainExpression(_)
            ));
        }
        Some(match (position, data) {
            (
                ExpressionPosition::NewCallee | ExpressionPosition::CallCallee { .. },
                ExprData::Function(_),
            ) => true,
            (
                ExpressionPosition::MemberObject(_) | ExpressionPosition::MemberAssignmentObject,
                ExprData::NumericLiteral(_)
                | ExprData::UnaryExpression(_)
                | ExprData::UpdateExpression(_)
                | ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ConditionalExpression(_),
            ) => true,
            (
                ExpressionPosition::CallCallee { .. } | ExpressionPosition::NewCallee,
                ExprData::UnaryExpression(_)
                | ExprData::UpdateExpression(_)
                | ExprData::BinaryExpression(_)
                | ExprData::LogicalExpression(_)
                | ExprData::ConditionalExpression(_),
            ) => true,
            (ExpressionPosition::Statement, ExprData::AssignmentExpression(node)) => matches!(
                self.ast.assignment_target_data(node.left(self.ast)),
                swc_next_ecma_ast::AssignmentTargetData::ObjectAssignmentTarget(_)
            ),
            (position, ExprData::AssignmentExpression(_)) => !matches!(
                position,
                ExpressionPosition::Statement
                    | ExpressionPosition::ExportDefault
                    | ExpressionPosition::AssignmentRight(_)
                    | ExpressionPosition::EnumInitializer
                    | ExpressionPosition::ForHead
                    | ExpressionPosition::ForInitializer { .. }
            ),
            (
                ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::ParenthesizedUnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument
                | ExpressionPosition::BinaryOperand { .. }
                | ExpressionPosition::ConditionalTest,
                ExprData::ConditionalExpression(_),
            ) => true,
            (
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent),
                ExprData::UnaryExpression(unary),
            ) => {
                matches!(parent, UnaryOperator::Positive | UnaryOperator::Negate)
                    && unary.operator(self.ast) == parent
            }
            (
                ExpressionPosition::UnaryArgument(parent)
                | ExpressionPosition::ParenthesizedUnaryArgument(parent),
                ExprData::UpdateExpression(update),
            ) => {
                update.prefix(self.ast)
                    && matches!(
                        (parent, update.operator(self.ast)),
                        (UnaryOperator::Positive, UpdateOperator::Increment)
                            | (UnaryOperator::Negate, UpdateOperator::Decrement)
                    )
            }
            (
                ExpressionPosition::UnaryArgument(_)
                | ExpressionPosition::AwaitArgument
                | ExpressionPosition::SpreadArgument,
                ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_),
            ) => true,
            (ExpressionPosition::BinaryOperand { operator, side }, _) => {
                match BinaryOperatorKind::from_data(data, self.ast) {
                    Some(child) => operator.needs_parentheses(child, side),
                    None => matches!(
                        (side, operator, data),
                        (
                            OperandSide::Left,
                            BinaryOperatorKind::Binary(swc_next_ecma_ast::BinaryOperator::Exponent),
                            ExprData::UnaryExpression(_) | ExprData::UpdateExpression(_)
                        ) | (
                            OperandSide::Left,
                            BinaryOperatorKind::Binary(
                                swc_next_ecma_ast::BinaryOperator::In
                                    | swc_next_ecma_ast::BinaryOperator::Instanceof
                            ),
                            ExprData::UnaryExpression(_)
                        )
                    ),
                }
            }
            (
                ExpressionPosition::Statement,
                ExprData::StringLiteral(_)
                | ExprData::ObjectExpression(_)
                | ExprData::Function(_)
                | ExprData::Class(_),
            ) => true,
            _ => false,
        })
    }

    fn record_object_argument(
        &mut self,
        expression: Expr,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedObject<'ast>, FormatError> {
        let layout = self.expression_layout(expression);
        match layout {
            ExpressionLayout::Flat => {
                let (normal, ()) = writer.record_with_value(|writer| {
                    layout.write(writer, |writer| self.format_object(object, writer))
                })?;
                Ok(RecordedObject {
                    normal,
                    forced: None,
                })
            }
            ExpressionLayout::Normal => self.record_object(object, writer),
        }
    }

    fn record_array_argument(
        &mut self,
        expression: Expr,
        array: ArrayExpression,
        arity: ArrayArgumentArity,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrayArgument<'ast>, FormatError> {
        let layout = self.expression_layout(expression);
        match layout {
            ExpressionLayout::Flat => {
                let (normal, ()) = writer.record_with_value(|writer| {
                    layout.write(writer, |writer| self.format_array(array, writer))
                })?;
                Ok(RecordedArrayArgument::Selected(RecordedArray {
                    normal,
                    forced: None,
                }))
            }
            ExpressionLayout::Normal => self.record_array(array, arity, writer),
        }
    }

    fn format_argument(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_argument_with_array_comments(argument, mode, None, writer)
    }

    fn format_argument_with_array_comments(
        &mut self,
        argument: Argument,
        mode: ContentMode,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_argument_data_with_array_comments(
            self.ast.argument_data(argument),
            mode,
            array_comments,
            writer,
        )
    }

    fn format_argument_data(
        &mut self,
        argument: ArgumentData,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_argument_data_with_array_comments(argument, mode, None, writer)
    }

    fn format_argument_data_with_array_comments(
        &mut self,
        argument: ArgumentData,
        mode: ContentMode,
        array_comments: Option<&mut ArrayCommentOwner<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match argument {
            ArgumentData::Expr(expression) => self.format_expression_with_array_comments(
                expression,
                ExpressionContext::at(ExpressionPosition::Nested)
                    .with_content_mode(mode)
                    .with_arrow_layout(arrows::ArrowLayout::Argument),
                array_comments,
                writer,
            ),
            ArgumentData::SpreadElement(spread) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(spread.span(self.ast), writer)?;
                    return Ok(None);
                }
                let argument = spread.argument(self.ast);
                writer.text("...");
                let comments = self.comments.between(Span::new(
                    spread.span(self.ast).start + 3,
                    argument.span(self.ast).start,
                ))?;
                let comments = self.comments.take(comments.len());
                let mode = self.content_mode(comments);
                self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
                if let Some(last) = comments.last() {
                    self.comment_gap(last.span.end, argument.span(self.ast).start, writer);
                }
                self.format_expression(
                    argument,
                    ExpressionContext::at(ExpressionPosition::SpreadArgument)
                        .with_content_mode(mode),
                    writer,
                )
            }
        }
    }
}
