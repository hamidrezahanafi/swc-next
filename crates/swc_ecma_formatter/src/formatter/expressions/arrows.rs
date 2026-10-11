//! Expression arrows with parameter, arrow-token and body boundary ownership.

use swc_next_ecma_ast::{
    ArrowFunctionBody, ArrowFunctionBodyData, ArrowFunctionExpression, Comment, CommentKind, Expr,
    ExprData, FormalParameterItemData, FormalParameterPatternData, FormalParameters, FunctionBody,
    GetSpan, Span, TsTypeAnnotation, TsTypeParameterDeclaration,
};

use super::arguments::TrailingCommaPolicy;
use super::parentheses::{ExpressionContent, SourceParentheses};
use super::{
    ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter, JsxCallbackContext,
    TailOwner,
};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::functions::SignatureBoundaryRole;
use crate::formatter::parameters::{ArrowParameterParentheses, SourceParameters};
use crate::formatter::statements::IncomingStatement;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::callable::{
    SignatureReturn, SignatureReturnDelimiter, SignatureReturnEolTarget,
};
use crate::formatter::types::generics::TypeParameterContext;
use crate::formatter::types::{TypeContext, TypeGap};
use crate::{
    ArrowParentheses, FormatError, Semicolons, TrailingCommas,
    document::{Fragment, RecordedGroup, Writer},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowLayout {
    Ordinary,
    Assignment(ArrowHeadGap),
    Callee,
    Argument,
    JsxExpression,
}

/// A leading comment run owns the gap to the arrow's first signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowHeadGap {
    Expression,
    LeadingComments,
}

impl ArrowLayout {
    pub(super) fn with_leading_comments(self) -> Self {
        match self {
            Self::Assignment(_) => Self::Assignment(ArrowHeadGap::LeadingComments),
            _ => self,
        }
    }
}

pub(super) enum RecordedArrow<'a> {
    Verbatim(Fragment<'a>),
    Formatted(FormattedRecordedArrow<'a>),
}

pub(super) struct FormattedRecordedArrow<'a> {
    pub(super) signature: Fragment<'a>,
    pub(super) body: RecordedGroup<'a>,
    pub(super) body_layout: ArrowBodyLayout,
    pub(super) normal: Fragment<'a>,
    pub(super) tail: Option<ExpressionTail<'a>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowBodyLayout {
    Hug,
    Block,
    Expression,
}

struct ArrowLink<'a> {
    signature: Fragment<'a>,
    following: Option<Fragment<'a>>,
    trailing: Option<Span>,
}

struct RecordedArrowChain<'a> {
    first: ArrowLink<'a>,
    rest: Vec<ArrowLink<'a>>,
    body: RecordedGroup<'a>,
    tail: Option<ExpressionTail<'a>>,
    should_break: bool,
    body_layout: ArrowBodyLayout,
}

struct ArrowChainChild<'a> {
    head: FormattedArrowHead<'a>,
    leading: &'a [Comment],
    start: u32,
    parentheses: Option<SourceParentheses>,
}

pub(super) struct VerbatimArrow {
    span: Span,
    is_async: bool,
}

struct ArrowReturnInput {
    annotation: TsTypeAnnotation,
    mode: ContentMode,
}

struct PreparedArrowGenerics<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: TypeGap<'ast>,
    following: TypeGap<'ast>,
    tail: TrailingCommaPolicy,
}

pub(super) struct FormattedArrowHead<'a> {
    arrow: ArrowFunctionExpression,
    parameters: SourceParameters,
    generics: Option<PreparedArrowGenerics<'a>>,
    body_mode: ContentMode,
    result: Option<ArrowReturnInput>,
    parameter_start: u32,
    signature_end: u32,
    body: ArrowFunctionBody,
    body_span: Span,
    operator: u32,
    comments: &'a [Comment],
    before_count: usize,
}

pub(super) enum SourceArrow<'a> {
    Verbatim(VerbatimArrow),
    Formatted(FormattedArrowHead<'a>),
}

pub(super) struct FormattedSourceArrow<'a> {
    arrow: ArrowFunctionExpression,
    next: Option<ArrowChainChild<'a>>,
    parameters: SourceParameters,
    generics: Option<PreparedArrowGenerics<'a>>,
    parameter_start: u32,
    signature_end: u32,
    result: Option<SignatureReturn<'a>>,
    before: &'a [Comment],
    body: SourceArrowBody<'a>,
}

struct ArrowBoundary<'ast> {
    parameters: FormalParameters,
    parameter_span: Span,
    signature_end: u32,
    body: ArrowFunctionBody,
    body_span: Span,
    operator: u32,
    comments: &'ast [Comment],
    before_count: usize,
}

pub(super) enum PreparedArrowBody {
    FunctionBody(FunctionBody),
    Expression {
        expression: Expr,
        content: ExpressionContent,
    },
}

pub(super) struct SourceArrowBody<'a> {
    body: PreparedArrowBody,
    parent_layout: ArrowLayout,
    body_span: Span,
    operator: u32,
    leading: &'a [Comment],
    mode: ContentMode,
    first_statement: Option<IncomingStatement>,
    trailing_constraint: LineConstraint,
    leading_layout: CommentRunLayout,
    body_starts_line: bool,
    hug_body: bool,
    jsx_callback: JsxCallbackContext,
}

impl SourceArrowBody<'_> {
    fn layout(&self) -> ArrowBodyLayout {
        if self.hug_body {
            ArrowBodyLayout::Hug
        } else if matches!(self.body, PreparedArrowBody::FunctionBody(_)) {
            ArrowBodyLayout::Block
        } else {
            ArrowBodyLayout::Expression
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn arrow_needs_asi_guard(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<bool, FormatError> {
        if arrow.r#async(self.ast) {
            return Ok(false);
        }
        if self.options.arrow_parentheses == ArrowParentheses::Always {
            return Ok(true);
        }
        let annotation = arrow.return_type(self.ast);
        let generics = arrow.type_parameters(self.ast);
        if generics.is_some() || annotation.is_some() {
            self.check_arrow_header_domain(arrow, generics, annotation)?;
            // Generic heads start with `<`, nongeneric typed heads with `(`.
            // Their ASI protection is separate from parameter parentheses.
            return Ok(true);
        }
        let boundary = self.source_arrow_boundary(arrow, generics, annotation)?;
        let parentheses = if boundary.before_count == 0 {
            ArrowParameterParentheses::Optional
        } else {
            ArrowParameterParentheses::Required
        };
        let parameters = self.source_arrow_parameters(boundary.parameters, parentheses)?;
        Ok(parameters.starts_with_parenthesis())
    }

    fn check_arrow_header_domain(
        &self,
        arrow: ArrowFunctionExpression,
        generics: Option<TsTypeParameterDeclaration>,
        annotation: Option<TsTypeAnnotation>,
    ) -> Result<(), FormatError> {
        if (generics.is_some() || annotation.is_some()) && !self.ast.is_ts() {
            return Err(self.unsupported("typed arrow", arrow));
        }
        Ok(())
    }

    #[inline(always)]
    fn source_arrow_boundary(
        &self,
        arrow: ArrowFunctionExpression,
        generics: Option<TsTypeParameterDeclaration>,
        annotation: Option<TsTypeAnnotation>,
    ) -> Result<ArrowBoundary<'ast>, FormatError> {
        self.check_arrow_header_domain(arrow, generics, annotation)?;
        let params = arrow.params(self.ast);
        let parameter_span = params.span(self.ast);
        let body = arrow.body(self.ast);
        let body_span = body.span(self.ast);
        let signature_end = annotation.map_or(parameter_span.end, |node| node.span(self.ast).end);
        let gap = Span::new(signature_end, body_span.start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=>") else {
            return Err(self.unsupported("arrow token boundary", arrow));
        };
        let before_count = comments.partition_point(|comment| comment.span.end <= operator);
        Ok(ArrowBoundary {
            parameters: params,
            parameter_span,
            signature_end,
            body,
            body_span,
            operator,
            comments,
            before_count,
        })
    }

    pub(super) fn source_arrow(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<SourceArrow<'ast>, FormatError> {
        let annotation = arrow.return_type(self.ast);
        let generic_node = arrow.type_parameters(self.ast);
        let boundary = self.source_arrow_boundary(arrow, generic_node, annotation)?;
        let parentheses =
            if generic_node.is_some() || annotation.is_some() || boundary.before_count != 0 {
                ArrowParameterParentheses::Required
            } else {
                ArrowParameterParentheses::Optional
            };
        let mut parameters = self.source_arrow_parameters(boundary.parameters, parentheses)?;
        if parameters.enclosing_mode() == ContentMode::Verbatim {
            return Ok(SourceArrow::Verbatim(VerbatimArrow {
                span: arrow.span(self.ast),
                is_async: arrow.r#async(self.ast),
            }));
        }
        let mut source_return = None;
        let mut body_mode = ContentMode::Formatted;
        let generics = generic_node.map(|node| {
            let generic_span = node.span(self.ast);
            let prefix_start =
                arrow.span(self.ast).start + if arrow.r#async(self.ast) { 5 } else { 0 };
            let prefix_gap = Span::new(prefix_start, generic_span.start);
            let prefix_source = self.has_suppression
                && self.content_mode(self.comments.in_range(prefix_gap)) == ContentMode::Verbatim;
            let mut mode = if prefix_source {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
            let following_gap = Span::new(generic_span.end, boundary.parameter_span.start);
            let mut following_source = false;
            if self.has_suppression {
                let comments = self.comments.in_range(following_gap);
                following_source = self.content_mode(comments) == ContentMode::Verbatim;
                if following_source {
                    for (_, position) in self.suppressed_comment_runs(comments, following_gap.end) {
                        if matches!(position, SuppressionPosition::EndOfLine)
                            || (!parameters.has_parameters()
                                && matches!(position, SuppressionPosition::Inline))
                        {
                            mode = ContentMode::Verbatim;
                        } else if parameters.has_parameters() {
                            parameters.set_leading_mode(ContentMode::Verbatim);
                        } else if let Some(annotation) = annotation {
                            source_return = Some(annotation);
                        } else {
                            body_mode = ContentMode::Verbatim;
                        }
                    }
                }
            }
            let members = node.params(self.ast);
            let requires_comma = self.ast.is_jsx()
                && members.len() == 1
                && members.iter().next().is_some_and(|id| {
                    let member = self.ast.get_node_in_sub_range(id);
                    member.constraint(self.ast).is_none()
                });
            let tail = if requires_comma {
                TrailingCommaPolicy::Required
            } else if self.options.trailing_commas == TrailingCommas::None {
                TrailingCommaPolicy::Forbidden
            } else {
                TrailingCommaPolicy::IfBreak
            };
            PreparedArrowGenerics {
                node,
                mode,
                prefix: self.prepare_type_gap(prefix_gap, prefix_source),
                following: self.prepare_type_gap(following_gap, following_source),
                tail,
            }
        });
        let result = annotation.map(|annotation| ArrowReturnInput {
            annotation,
            mode: if source_return.is_some() {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            },
        });
        Ok(SourceArrow::Formatted(FormattedArrowHead {
            arrow,
            parameters,
            generics,
            body_mode,
            result,
            parameter_start: boundary.parameter_span.start,
            signature_end: boundary.signature_end,
            body: boundary.body,
            body_span: boundary.body_span,
            operator: boundary.operator,
            comments: boundary.comments,
            before_count: boundary.before_count,
        }))
    }

    pub(super) fn prepare_arrow_body(&self, body: ArrowFunctionBody) -> PreparedArrowBody {
        match self.ast.arrow_function_body_data(body) {
            ArrowFunctionBodyData::FunctionBody(body) => PreparedArrowBody::FunctionBody(body),
            ArrowFunctionBodyData::Expr(expression) => PreparedArrowBody::Expression {
                expression,
                content: self.expression_content(expression),
            },
        }
    }

    fn source_formatted_arrow(
        &self,
        head: FormattedArrowHead<'ast>,
        prepared_body: Option<PreparedArrowBody>,
    ) -> Result<FormattedSourceArrow<'ast>, FormatError> {
        let FormattedArrowHead {
            arrow,
            mut parameters,
            mut generics,
            body_mode,
            result,
            parameter_start,
            signature_end,
            body,
            body_span,
            operator,
            comments,
            before_count,
        } = head;
        let result = if let Some(input) = result {
            let target = if parameters.has_parameters() {
                SignatureReturnEolTarget::Parameter
            } else if let Some(generic) = &mut generics {
                SignatureReturnEolTarget::Header(&mut generic.mode)
            } else {
                SignatureReturnEolTarget::Annotation
            };
            let source_return = (input.mode == ContentMode::Verbatim).then_some(input.annotation);
            let result = self.prepare_signature_return(
                parameters.span(self.ast).end,
                target,
                input.annotation,
                source_return,
                SignatureReturnDelimiter::Colon,
            );
            if result.preceding_mode == ContentMode::Verbatim {
                parameters.suppress_last(self.ast)?;
            }
            Some(result)
        } else {
            None
        };
        let (before, leading) = comments.split_at(before_count);
        let body = prepared_body.unwrap_or_else(|| self.prepare_arrow_body(body));
        let (mode, first_statement, trailing_constraint) = match &body {
            PreparedArrowBody::FunctionBody(body) if self.has_suppression => {
                let modes = self.signature_body_modes(
                    *body,
                    leading,
                    body_span.start,
                    operator + 2,
                    SignatureBoundaryRole::Arrow {
                        has_parameters: parameters.has_parameters(),
                    },
                );
                (modes.body, modes.first_statement, modes.constraint)
            }
            PreparedArrowBody::FunctionBody(_) => {
                (ContentMode::Formatted, None, LineConstraint::Unconstrained)
            }
            PreparedArrowBody::Expression { .. } => (
                self.content_mode(leading),
                None,
                LineConstraint::Unconstrained,
            ),
        };
        let (head_end, hugs_shape, leading_layout) = match &body {
            PreparedArrowBody::Expression { content, .. } => (
                content.expression.span(self.ast).start,
                matches!(
                    content.data,
                    ExprData::ArrayExpression(_)
                        | ExprData::ObjectExpression(_)
                        | ExprData::SequenceExpression(_)
                        | ExprData::ArrowFunctionExpression(_)
                        | ExprData::JsxElement(_)
                        | ExprData::JsxFragment(_)
                ),
                CommentRunLayout::GroupedLeading(body_span.start),
            ),
            PreparedArrowBody::FunctionBody(_) => (
                body_span.start,
                true,
                CommentRunLayout::FlatLeading(body_span.start),
            ),
        };
        let head_comments = self.comments.in_range(Span::new(operator + 2, head_end));
        let body_starts_line = head_comments.iter().enumerate().any(|(index, comment)| {
            let next_start = head_comments
                .get(index + 1)
                .map_or(head_end, |next| next.span.start);
            comment.kind == CommentKind::Line
                || (matches!(leading_layout, CommentRunLayout::GroupedLeading(_))
                    && self.source_position_starts_line(comment.span.start)
                    && self.line_breaks(comment.span.end, next_start) != 0)
        });
        let hug_body = hugs_shape && !body_starts_line;
        let mut body = SourceArrowBody {
            body,
            parent_layout: ArrowLayout::Ordinary,
            body_span,
            operator,
            leading,
            mode,
            first_statement,
            trailing_constraint,
            leading_layout,
            body_starts_line,
            hug_body,
            jsx_callback: JsxCallbackContext::Ordinary,
        };
        if body_mode == ContentMode::Verbatim {
            body.mode = body_mode;
        }
        let next = self.arrow_chain_child(&mut body)?;
        Ok(FormattedSourceArrow {
            arrow,
            next,
            parameters,
            generics,
            parameter_start,
            signature_end,
            result,
            before,
            body,
        })
    }

    pub(super) fn format_arrow(
        &mut self,
        arrow: ArrowFunctionExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let mut source = match self.source_arrow(arrow)? {
            SourceArrow::Verbatim(source) => {
                self.format_verbatim_arrow(source, context, writer)?;
                return Ok(None);
            }
            SourceArrow::Formatted(head) => self.source_formatted_arrow(head, None)?,
        };
        if let Some(child) = source.next.take() {
            let chain = self.record_arrow_chain(source, child, context, writer)?;
            chain.write(context.arrow_layout, writer);
            return Ok(chain.tail);
        }
        writer.group(false, |writer| {
            let body = self.format_arrow_head(source, context, writer)?;
            self.format_arrow_body(body, writer)
        })
    }

    fn format_verbatim_arrow(
        &mut self,
        source: VerbatimArrow,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !source.is_async
        {
            writer.text(";");
        }
        self.format_verbatim(source.span, writer)
    }

    fn arrow_chain_child(
        &self,
        body: &mut SourceArrowBody<'ast>,
    ) -> Result<Option<ArrowChainChild<'ast>>, FormatError> {
        let PreparedArrowBody::Expression {
            expression,
            content,
        } = &body.body
        else {
            return Ok(None);
        };
        let ExprData::ArrowFunctionExpression(arrow) = content.data else {
            return Ok(None);
        };
        let start = arrow.span(self.ast).start;
        let leading = self.comments.in_range(Span::new(body.operator + 2, start));
        if body.mode != ContentMode::Formatted
            || !self
                .comment_binding(leading, Span::new(body.operator + 2, start), None)
                .allows_relocation(None)
        {
            return Ok(None);
        }
        if self.expression_layout(*expression) == ExpressionLayout::Flat
            || self.expression_layout(content.expression) == ExpressionLayout::Flat
        {
            return Ok(None);
        }
        let head = match self.source_arrow(arrow)? {
            SourceArrow::Verbatim(_) => {
                body.mode = ContentMode::Verbatim;
                return Ok(None);
            }
            SourceArrow::Formatted(head) => head,
        };
        Ok(Some(ArrowChainChild {
            head,
            leading,
            start,
            parentheses: content.parentheses.map(|removed| removed.source),
        }))
    }

    fn arrow_chain_breaks(&self, source: &FormattedSourceArrow<'ast>) -> bool {
        if source.generics.is_some()
            || source.result.is_some() && source.parameters.has_parameters()
        {
            return true;
        }
        let ast = self.ast;
        let params = source.arrow.params(ast);
        params.rest(ast).is_some()
            || params.items(ast).iter().any(|id| {
                let parameter = ast.get_node_in_sub_range(id);
                match ast.formal_parameter_item_data(parameter) {
                    FormalParameterItemData::FormalParameter(parameter) => {
                        parameter.initializer(ast).is_some()
                            || !matches!(
                                ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                                FormalParameterPatternData::BindingIdentifier(_)
                            )
                    }
                    FormalParameterItemData::TsParameterProperty(_) => true,
                }
            })
    }

    pub(super) fn record_arrow(
        &mut self,
        source: SourceArrow<'ast>,
        prepared_body: Option<PreparedArrowBody>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrow<'ast>, FormatError> {
        let mut source = match source {
            SourceArrow::Verbatim(source) => {
                let (value, ()) = writer.record_with_value(|writer| {
                    self.format_verbatim_arrow(source, context, writer)
                })?;
                return Ok(RecordedArrow::Verbatim(value));
            }
            SourceArrow::Formatted(head) => self.source_formatted_arrow(head, prepared_body)?,
        };
        let Some(first_child) = source.next.take() else {
            let (signature, source_body) = writer
                .record_with_value(|writer| self.format_arrow_head(source, context, writer))?;
            let body_layout = source_body.layout();
            let (body, tail) = writer.record_group_with_value(|writer| {
                self.format_arrow_body_content(source_body, writer)
            })?;
            let (normal, ()) = writer.record_with_value(|writer| {
                writer.group(false, |writer| {
                    writer.write_fragment(signature);
                    writer.write_fragment(if body_layout == ArrowBodyLayout::Hug {
                        body.contents()
                    } else {
                        body.grouped()
                    });
                });
                Ok::<_, FormatError>(())
            })?;
            return Ok(RecordedArrow::Formatted(FormattedRecordedArrow {
                signature,
                body,
                body_layout,
                normal,
                tail,
            }));
        };
        let chain = self.record_arrow_chain(source, first_child, context, writer)?;
        let (signature, ()) = writer.record_with_value(|writer| {
            for link in std::iter::once(&chain.first).chain(&chain.rest) {
                writer.write_fragment(link.signature);
                if let Some(following) = link.following {
                    writer.write_fragment(following);
                }
            }
            writer.text("=>");
            Ok::<_, FormatError>(())
        })?;
        let (normal, ()) = writer.record_with_value(|writer| {
            chain.write(context.arrow_layout, writer);
            Ok::<_, FormatError>(())
        })?;
        Ok(RecordedArrow::Formatted(FormattedRecordedArrow {
            signature,
            body: chain.body,
            body_layout: chain.body_layout,
            normal,
            tail: chain.tail,
        }))
    }

    fn record_arrow_chain(
        &mut self,
        mut source: FormattedSourceArrow<'ast>,
        first_child: ArrowChainChild<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrowChain<'ast>, FormatError> {
        let mut should_break = self.arrow_chain_breaks(&source);
        let (signature, source_body) = writer
            .record_with_value(|writer| self.format_arrow_signature(source, context, writer))?;
        let (following, ()) = writer.record_with_value(|writer| {
            self.format_arrow_link(source_body, &first_child, writer)
        })?;
        let first = ArrowLink {
            signature,
            following: Some(following),
            trailing: first_child
                .parentheses
                .map(|boundary| Span::new(boundary.inner.end, boundary.outer.end)),
        };
        source = self.source_formatted_arrow(first_child.head, None)?;
        let mut links = Vec::new();
        let (body, mut tail, body_layout) = loop {
            should_break |= self.arrow_chain_breaks(&source);
            let next = source.next.take();
            let (signature, source_body) = writer.record_with_value(|writer| {
                self.format_arrow_signature(
                    source,
                    ExpressionContext::at(ExpressionPosition::ArrowBody),
                    writer,
                )
            })?;
            if let Some(child) = next {
                let (following, ()) = writer.record_with_value(|writer| {
                    self.format_arrow_link(source_body, &child, writer)
                })?;
                links.push(ArrowLink {
                    signature,
                    following: Some(following),
                    trailing: child
                        .parentheses
                        .map(|boundary| Span::new(boundary.inner.end, boundary.outer.end)),
                });
                source = self.source_formatted_arrow(child.head, None)?;
            } else {
                let body_layout = source_body.layout();
                let (body, tail) = writer.record_group_with_value(|writer| {
                    self.format_arrow_body_content(source_body, writer)
                })?;
                links.push(ArrowLink {
                    signature,
                    following: None,
                    trailing: None,
                });
                break (body, tail, body_layout);
            }
        };
        for link in links.iter().rev().chain(std::iter::once(&first)) {
            if let Some(trailing) = link.trailing {
                tail = self.capture_expression_tail(trailing, tail)?;
            }
        }
        Ok(RecordedArrowChain {
            first,
            rest: links,
            body,
            tail,
            should_break,
            body_layout,
        })
    }

    fn format_arrow_link(
        &mut self,
        source: SourceArrowBody<'ast>,
        child: &ArrowChainChild<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("=>");
        let first = child
            .leading
            .first()
            .map_or(child.start, |comment| comment.span.start);
        self.line_constraint(source.operator + 2, first).write(
            writer,
            FlatSeparator::Space,
            |writer| {
                if source.body_starts_line {
                    writer.hard_line();
                } else {
                    writer.line();
                }
            },
        );
        self.comments.claim(child.leading)?;
        self.format_comment_run(
            child.leading,
            CommentRunLayout::GroupedLeading(child.start),
            writer,
        )
    }

    pub(super) fn format_arrow_head(
        &mut self,
        source: FormattedSourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let body = self.format_arrow_signature(source, context, writer)?;
        writer.text("=>");
        Ok(body)
    }

    fn format_arrow_signature(
        &mut self,
        source: FormattedSourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let FormattedSourceArrow {
            arrow,
            parameters,
            generics,
            parameter_start,
            signature_end,
            result,
            before,
            mut body,
            ..
        } = source;
        if context.expression_start.needs_asi_guard()
            && self.options.semicolons == Semicolons::AsNeeded
            && !arrow.r#async(self.ast)
            && (generics.is_some() || parameters.starts_with_parenthesis())
        {
            writer.text(";");
        }
        if arrow.r#async(self.ast) {
            writer.text("async");
            if generics.is_none() {
                self.format_source_token_gap(
                    arrow.span(self.ast).start + 5,
                    parameter_start,
                    true,
                    writer,
                )?;
            }
        }
        if let Some(generic) = generics {
            let prefix_separator = if arrow.r#async(self.ast) {
                FlatSeparator::Space
            } else {
                FlatSeparator::Empty
            };
            self.format_type_gap(generic.prefix, prefix_separator, writer)?;
            self.format_type_parameters_with_tail(
                generic.node,
                TypeParameterContext::Callable,
                generic.mode,
                generic.tail,
                writer,
            )?;
            self.format_type_gap(generic.following, FlatSeparator::Empty, writer)?;
        }
        self.format_source_parameters(parameters, writer)?;
        if let Some(result) = result {
            let tail = self.format_signature_return_with_context(
                result,
                TypeContext::ArrowReturn,
                writer,
            )?;
            self.format_expression_tail(tail, writer)?;
        }
        self.comments.claim(before)?;
        self.format_comment_run(before, CommentRunLayout::SourceAfter(signature_end), writer)?;
        let before_end = before
            .last()
            .map_or(signature_end, |comment| comment.span.end);
        self.line_constraint(before_end, body.operator).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        body.parent_layout = context.arrow_layout;
        body.jsx_callback = context.jsx_callback.body(arrow);
        Ok(body)
    }

    pub(super) fn format_arrow_body(
        &mut self,
        source: SourceArrowBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if source.hug_body {
            self.format_arrow_body_content(source, writer)
        } else {
            writer.group(false, |writer| {
                self.format_arrow_body_content(source, writer)
            })
        }
    }

    fn format_arrow_body_content(
        &mut self,
        source: SourceArrowBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let SourceArrowBody {
            body,
            parent_layout,
            body_span,
            operator,
            leading,
            mode,
            first_statement,
            trailing_constraint,
            leading_layout,
            body_starts_line,
            hug_body,
            jsx_callback,
        } = source;
        let first = leading
            .first()
            .map_or(body_span.start, |comment| comment.span.start);
        let write_body = |writer: &mut Writer<'ast>| {
            if !matches!(body, PreparedArrowBody::FunctionBody(_))
                || mode != ContentMode::Verbatim && first_statement.is_none()
            {
                self.line_constraint(operator + 2, first).write(
                    writer,
                    FlatSeparator::Space,
                    |writer| {
                        if body_starts_line {
                            writer.hard_line();
                        } else if hug_body {
                            writer.space();
                        } else {
                            writer.line();
                        }
                    },
                );
            }
            self.comments.claim(leading)?;
            match body {
                PreparedArrowBody::Expression {
                    expression,
                    content,
                } => self.format_prepared_expression_with_leading_comments(
                    expression,
                    content,
                    NodeHead {
                        comments: leading,
                        content_start: body_span.start,
                        layout: leading_layout,
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::ArrowBody)
                        .with_content_mode(mode)
                        .with_jsx_callback(jsx_callback)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                ),
                PreparedArrowBody::FunctionBody(body) => {
                    if mode == ContentMode::Verbatim || first_statement.is_some() {
                        self.format_comment_run(
                            leading,
                            CommentRunLayout::SourceAfter(operator + 2),
                            writer,
                        )?;
                        trailing_constraint.write(writer, FlatSeparator::Space, Writer::space);
                    } else {
                        self.format_comment_run(leading, leading_layout, writer)?;
                    }
                    self.format_function_body(body, mode, first_statement, writer)?;
                    Ok(None)
                }
            }
        };
        let tail = if hug_body {
            write_body(writer)
        } else {
            writer.indent(write_body)
        }?;
        if parent_layout == ArrowLayout::JsxExpression && !hug_body {
            writer.soft_line();
        }
        Ok(tail)
    }
}

impl<'a> RecordedArrowChain<'a> {
    fn write(&self, layout: ArrowLayout, writer: &mut Writer<'a>) {
        let Self {
            first,
            rest,
            body,
            should_break,
            body_layout,
            ..
        } = self;
        let hug_body = *body_layout == ArrowBodyLayout::Hug;
        writer.group(false, |writer| {
            let callee = layout == ArrowLayout::Callee;
            // A broken callee can keep its signatures flat. The final arrow and
            // body observe the enclosing group independently of the signatures.
            let group = writer.named_group(callee && !hug_body, |writer| {
                let write_remaining = |writer: &mut Writer<'a>| {
                    if let Some(following) = first.following {
                        writer.write_fragment(following);
                    }
                    for link in rest {
                        writer.write_fragment(link.signature);
                        if let Some(following) = link.following {
                            writer.write_fragment(following);
                        }
                    }
                };
                let write_signatures = |writer: &mut Writer<'a>| {
                    writer.group(*should_break, |writer| {
                        writer.write_fragment(first.signature);
                        if layout == ArrowLayout::Argument {
                            writer.indent(write_remaining);
                        } else {
                            write_remaining(writer);
                        }
                    });
                };
                match layout {
                    ArrowLayout::Argument => write_signatures(writer),
                    ArrowLayout::Callee | ArrowLayout::Assignment(ArrowHeadGap::Expression) => {
                        writer.indent(|writer| {
                            writer.soft_line();
                            write_signatures(writer);
                        })
                    }
                    ArrowLayout::Ordinary
                    | ArrowLayout::JsxExpression
                    | ArrowLayout::Assignment(ArrowHeadGap::LeadingComments) => {
                        writer.indent(write_signatures)
                    }
                }
            });
            writer.text("=>");
            writer.indent_if_break(group, |writer| writer.write_fragment(body.contents()));
            if callee {
                writer.soft_line_if_break(group);
            } else if layout == ArrowLayout::JsxExpression && !hug_body {
                writer.soft_line();
            }
        });
    }
}
