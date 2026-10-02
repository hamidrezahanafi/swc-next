//! Expression arrows with parameter, arrow-token and body boundary ownership.

use swc_next_ecma_ast::{
    ArrowFunctionBodyData, ArrowFunctionExpression, Comment, CommentKind, ExprData,
    FormalParameterItemData, FormalParameterPatternData, GetSpan, Span,
};

use super::parentheses::SourceParentheses;
use super::{ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::{CommentRunLayout, CommentSemantics, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::{parameters::SourceParameters, suppression::ContentMode};
use crate::{
    FormatError, Semicolons,
    document::{Fragment, RecordedGroup, Writer},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowLayout {
    Ordinary,
    Assignment(ArrowHeadGap),
    Callee,
    Argument,
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

pub(super) struct RecordedArrow<'a> {
    pub(super) signature: Fragment<'a>,
    pub(super) body: Fragment<'a>,
    pub(super) normal: Fragment<'a>,
    pub(super) tail: Option<ExpressionTail<'a>>,
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
    hug_body: bool,
}

#[derive(Clone, Copy)]
struct ArrowChainChild<'a> {
    arrow: ArrowFunctionExpression,
    leading: &'a [Comment],
    start: u32,
    parentheses: Option<SourceParentheses>,
}

pub(super) struct SourceArrow<'a> {
    arrow: ArrowFunctionExpression,
    next: Option<ArrowChainChild<'a>>,
    parameters: SourceParameters,
    parameter_span: Span,
    before: &'a [Comment],
    body: SourceArrowBody<'a>,
}

pub(super) struct SourceArrowBody<'a> {
    body: ArrowFunctionBodyData,
    body_span: Span,
    operator: u32,
    leading: &'a [Comment],
    mode: ContentMode,
    leading_layout: CommentRunLayout,
    body_starts_line: bool,
    hug_body: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arrow(
        &self,
        arrow: ArrowFunctionExpression,
    ) -> Result<SourceArrow<'ast>, FormatError> {
        if arrow.type_parameters(self.ast).is_some() || arrow.return_type(self.ast).is_some() {
            return Err(self.unsupported("typed arrow", arrow));
        }
        let body = self.ast.arrow_function_body_data(arrow.body(self.ast));
        let params = arrow.params(self.ast);
        let parameter_span = params.span(self.ast);
        let body_span = arrow.body(self.ast).span(self.ast);
        let gap = Span::new(parameter_span.end, body_span.start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=>") else {
            return Err(self.unsupported("arrow token boundary", arrow));
        };
        let (before, leading) =
            comments.split_at(comments.partition_point(|comment| comment.span.end <= operator));
        let parameters = self.source_arrow_parameters(params, !before.is_empty())?;
        let mode = self.content_mode(leading);
        let (head_end, hugs_shape, leading_layout) = match body {
            ArrowFunctionBodyData::Expr(expression) => {
                let content = self.expression_content(expression);
                (
                    content.expression.span(self.ast).start,
                    matches!(
                        content.data,
                        ExprData::ArrayExpression(_)
                            | ExprData::ObjectExpression(_)
                            | ExprData::SequenceExpression(_)
                            | ExprData::ArrowFunctionExpression(_)
                    ),
                    CommentRunLayout::GroupedLeading(body_span.start),
                )
            }
            ArrowFunctionBodyData::FunctionBody(_) => (
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
        let body = SourceArrowBody {
            body,
            body_span,
            operator,
            leading,
            mode,
            leading_layout,
            body_starts_line,
            hug_body,
        };
        let next = self.arrow_chain_child(&body);
        Ok(SourceArrow {
            arrow,
            next,
            parameters,
            parameter_span,
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
        let source = self.source_arrow(arrow)?;
        if let Some(child) = source.next {
            let chain = self.record_arrow_chain(source, child, context, writer)?;
            chain.write(context.arrow_layout, writer);
            return Ok(chain.tail);
        }
        writer.group(false, |writer| {
            let body = self.format_arrow_head(source, context, writer)?;
            self.format_arrow_body(body, writer)
        })
    }

    fn arrow_chain_child(&self, body: &SourceArrowBody<'ast>) -> Option<ArrowChainChild<'ast>> {
        let ArrowFunctionBodyData::Expr(expression) = body.body else {
            return None;
        };
        let content = self.expression_content(expression);
        let ExprData::ArrowFunctionExpression(arrow) = content.data else {
            return None;
        };
        let start = arrow.span(self.ast).start;
        let leading = self.comments.in_range(Span::new(body.operator + 2, start));
        if body.mode != ContentMode::Formatted
            || leading.iter().any(|&comment| {
                !matches!(self.comment_semantics(comment), CommentSemantics::Ordinary)
            })
        {
            return None;
        }
        if self.expression_layout(expression) == ExpressionLayout::Flat
            || self.expression_layout(content.expression) == ExpressionLayout::Flat
            || self.diagnostic_range_is_sensitive(Span::new(body.operator + 2, start))
        {
            return None;
        }
        Some(ArrowChainChild {
            arrow,
            leading,
            start,
            parentheses: content.parentheses,
        })
    }

    fn arrow_chain_breaks(&self, arrow: ArrowFunctionExpression) -> bool {
        let ast = self.ast;
        let params = arrow.params(ast);
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
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrow<'ast>, FormatError> {
        let Some(first_child) = source.next else {
            let (signature, source_body) = writer
                .record_with_value(|writer| self.format_arrow_head(source, context, writer))?;
            let (body, tail) =
                writer.record_with_value(|writer| self.format_arrow_body(source_body, writer))?;
            let (normal, ()) = writer.record_with_value(|writer| {
                writer.group(false, |writer| {
                    writer.write_fragment(signature);
                    writer.write_fragment(body);
                });
                Ok::<_, FormatError>(())
            })?;
            return Ok(RecordedArrow {
                signature,
                body,
                normal,
                tail,
            });
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
        let body = if chain.hug_body {
            chain.body.contents
        } else {
            chain.body.grouped
        };
        Ok(RecordedArrow {
            signature,
            body,
            normal,
            tail: chain.tail,
        })
    }

    fn record_arrow_chain(
        &mut self,
        mut source: SourceArrow<'ast>,
        first_child: ArrowChainChild<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedArrowChain<'ast>, FormatError> {
        let mut should_break = self.arrow_chain_breaks(source.arrow);
        let (signature, source_body) = writer
            .record_with_value(|writer| self.format_arrow_signature(source, context, writer))?;
        let (following, ()) = writer
            .record_with_value(|writer| self.format_arrow_link(source_body, first_child, writer))?;
        let first = ArrowLink {
            signature,
            following: Some(following),
            trailing: first_child
                .parentheses
                .map(|boundary| Span::new(boundary.inner.end, boundary.outer.end)),
        };
        source = self.source_arrow(first_child.arrow)?;
        let mut links = Vec::new();
        let (body, mut tail, hug_body) = loop {
            should_break |= self.arrow_chain_breaks(source.arrow);
            let next = source.next;
            let (signature, source_body) = writer.record_with_value(|writer| {
                self.format_arrow_signature(
                    source,
                    ExpressionContext::at(ExpressionPosition::ArrowBody),
                    writer,
                )
            })?;
            if let Some(child) = next {
                let (following, ()) = writer.record_with_value(|writer| {
                    self.format_arrow_link(source_body, child, writer)
                })?;
                links.push(ArrowLink {
                    signature,
                    following: Some(following),
                    trailing: child
                        .parentheses
                        .map(|boundary| Span::new(boundary.inner.end, boundary.outer.end)),
                });
                source = self.source_arrow(child.arrow)?;
            } else {
                let hug_body = source_body.hug_body;
                let (body, tail) = writer.record_group_with_value(|writer| {
                    self.format_arrow_body_content(source_body, writer)
                })?;
                links.push(ArrowLink {
                    signature,
                    following: None,
                    trailing: None,
                });
                break (body, tail, hug_body);
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
            hug_body,
        })
    }

    fn format_arrow_link(
        &mut self,
        source: SourceArrowBody<'ast>,
        child: ArrowChainChild<'ast>,
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
        source: SourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let body = self.format_arrow_signature(source, context, writer)?;
        writer.text("=>");
        Ok(body)
    }

    fn format_arrow_signature(
        &mut self,
        source: SourceArrow<'ast>,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceArrowBody<'ast>, FormatError> {
        let SourceArrow {
            arrow,
            parameters,
            parameter_span,
            before,
            body,
            ..
        } = source;
        if context.expression_start.is_statement()
            && self.options.semicolons == Semicolons::AsNeeded
            && !arrow.r#async(self.ast)
            && parameters.starts_with_parenthesis()
        {
            writer.text(";");
        }
        if arrow.r#async(self.ast) {
            writer.text("async");
            self.format_source_token_gap(
                arrow.span(self.ast).start + 5,
                parameter_span.start,
                true,
                writer,
            )?;
        }
        self.format_source_parameters(parameters, writer)?;
        self.comments.claim(before)?;
        self.format_comment_run(
            before,
            CommentRunLayout::SourceAfter(parameter_span.end),
            writer,
        )?;
        let before_end = before
            .last()
            .map_or(parameter_span.end, |comment| comment.span.end);
        self.line_constraint(before_end, body.operator).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
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
            body_span,
            operator,
            leading,
            mode,
            leading_layout,
            body_starts_line,
            hug_body,
        } = source;
        let first = leading
            .first()
            .map_or(body_span.start, |comment| comment.span.start);
        let mut write_body = |writer: &mut Writer<'ast>| {
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
            self.comments.claim(leading)?;
            match body {
                ArrowFunctionBodyData::Expr(expression) => self
                    .format_expression_with_leading_comments(
                        expression,
                        NodeHead {
                            comments: leading,
                            content_start: body_span.start,
                            layout: leading_layout,
                            gap_end: None,
                        },
                        ExpressionContext::at(ExpressionPosition::ArrowBody)
                            .with_content_mode(mode)
                            .with_tail_owner(TailOwner::ParentOperation),
                        writer,
                    ),
                ArrowFunctionBodyData::FunctionBody(body) => {
                    self.format_comment_run(leading, leading_layout, writer)?;
                    self.format_function_body(body, mode, writer)?;
                    Ok(None)
                }
            }
        };
        if hug_body {
            write_body(writer)
        } else {
            writer.indent(write_body)
        }
    }
}

impl<'a> RecordedArrowChain<'a> {
    fn write(&self, layout: ArrowLayout, writer: &mut Writer<'a>) {
        let Self {
            first,
            rest,
            body,
            should_break,
            hug_body,
            ..
        } = self;
        writer.group(false, |writer| {
            let callee = layout == ArrowLayout::Callee;
            // A broken callee can keep its signatures flat. The final arrow and
            // body observe the enclosing group independently of the signatures.
            let group = writer.named_group(callee && !*hug_body, |writer| {
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
                    | ArrowLayout::Assignment(ArrowHeadGap::LeadingComments) => {
                        writer.indent(write_signatures)
                    }
                }
            });
            writer.text("=>");
            writer.indent_if_break(group, |writer| writer.write_fragment(body.contents));
            if callee {
                writer.soft_line_if_break(group);
            }
        });
    }
}
