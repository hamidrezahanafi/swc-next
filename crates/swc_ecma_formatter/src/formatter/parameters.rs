//! Actual runtime parameter boundaries and receiver/formal/rest ownership.

use swc_next_ecma_ast::{
    Ast, ExprData, FormalParameter, FormalParameterItemData, FormalParameterPatternData,
    FormalParameterRest, FormalParameters, FunctionBody, GetSpan, Span, TsThisParameter,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use super::types::callable::CallableParameter;
use crate::{ArrowParentheses, FormatError, TrailingCommas, document::Writer};

enum ParameterRole {
    Arrow { has_trailing_comments: bool },
    Function,
    PrivateMethod(FunctionBody),
}

enum ParameterSource {
    Parenthesized(Span),
    Bare(FormalParameter),
    PrivateEmpty(Span),
}

struct PrivateParameterBoundary {
    parameter: CallableParameter,
    relation: DelimitedSuppression,
    closing: LineConstraint,
}

pub(super) struct SourceParameters {
    node: FormalParameters,
    source: ParameterSource,
    first: Option<CallableParameter>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: bool,
    leading_mode: ContentMode,
    enclosing_mode: ContentMode,
    suppressed_tail: Option<CallableParameter>,
    private_boundary: Option<PrivateParameterBoundary>,
    key_mode: ContentMode,
    body_target: Option<(FunctionBody, ContentMode)>,
}

impl SourceParameters {
    pub(super) fn key_mode(&self) -> ContentMode {
        self.key_mode
    }
    pub(super) fn body_mode(&self, body: FunctionBody) -> ContentMode {
        self.body_target
            .filter(|(target, _)| *target == body)
            .map_or(ContentMode::Formatted, |(_, mode)| mode)
    }

    pub(super) fn enclosing_mode(&self) -> ContentMode {
        self.enclosing_mode
    }

    pub(super) fn has_parameters(&self) -> bool {
        self.first.is_some() || self.rest.is_some()
    }

    pub(super) fn suppress_last(&mut self, ast: &Ast<'_>) -> Result<(), FormatError> {
        self.suppressed_tail = if let Some(rest) = self.rest {
            Some(CallableParameter::Rest(rest))
        } else if let Some(id) = self.node.items(ast).iter().next_back() {
            let item = ast.get_node_in_sub_range(id);
            match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(parameter) => {
                    Some(CallableParameter::Formal(parameter))
                }
                FormalParameterItemData::TsParameterProperty(_) => {
                    return Err(FormatError::unsupported(
                        "typed parameter property",
                        None,
                        item.span(ast),
                    ));
                }
            }
        } else {
            self.first
                .filter(|parameter| matches!(parameter, CallableParameter::This(_)))
        };
        Ok(())
    }

    pub(super) fn set_leading_mode(&mut self, mode: ContentMode) {
        self.leading_mode = mode;
    }

    pub(super) fn starts_with_parenthesis(&self) -> bool {
        !self.omit_parentheses
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn source_arrow_parameters(
        &self,
        node: FormalParameters,
        has_trailing_comments: bool,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(
            node,
            None,
            ParameterRole::Arrow {
                has_trailing_comments,
            },
        )
    }

    pub(super) fn source_function_parameters(
        &self,
        node: FormalParameters,
        receiver: Option<TsThisParameter>,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, receiver, ParameterRole::Function)
    }

    pub(super) fn source_private_method_parameters(
        &self,
        node: FormalParameters,
        body: FunctionBody,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, None, ParameterRole::PrivateMethod(body))
    }

    fn source_parameters(
        &self,
        node: FormalParameters,
        receiver: Option<TsThisParameter>,
        role: ParameterRole,
    ) -> Result<SourceParameters, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let items = node.items(ast);
        let rest = node.rest(ast);
        let first = if let Some(receiver) = receiver {
            Some(CallableParameter::This(receiver))
        } else {
            let first = items.iter().next().map(|id| ast.get_node_in_sub_range(id));
            match first.map(|item| ast.formal_parameter_item_data(item)) {
                Some(FormalParameterItemData::FormalParameter(parameter)) => {
                    Some(CallableParameter::Formal(parameter))
                }
                Some(FormalParameterItemData::TsParameterProperty(_)) => {
                    return Err(self.unsupported("arrow parameter property", node));
                }
                None => None,
            }
        };
        let mut source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
            ParameterSource::Parenthesized(span)
        } else if matches!(role, ParameterRole::Arrow { .. })
            && let Some(CallableParameter::Formal(parameter)) = first
            && items.len() == 1
            && rest.is_none()
            && parameter.span(ast) == span
            && matches!(
                ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                FormalParameterPatternData::BindingIdentifier(_)
            )
            && parameter.initializer(ast).is_none()
        {
            ParameterSource::Bare(parameter)
        } else {
            return Err(self.unsupported("arrow parameter source boundary", node));
        };
        let enclosing_mode = if self.has_suppression
            && !matches!(role, ParameterRole::PrivateMethod(_))
            && first.is_none()
            && rest.is_none()
        {
            self.content_mode(self.comments.in_range(span))
        } else {
            ContentMode::Formatted
        };
        let single = match first {
            Some(CallableParameter::Formal(parameter)) if items.len() == 1 && rest.is_none() => {
                Some(parameter)
            }
            _ => None,
        };
        let omit_parentheses = matches!(
            role,
            ParameterRole::Arrow {
                has_trailing_comments: false
            }
        ) && self.options.arrow_parentheses == ArrowParentheses::AsNeeded
            && single.is_some_and(|parameter| {
                matches!(
                    ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                    FormalParameterPatternData::BindingIdentifier(_)
                ) && parameter.initializer(ast).is_none()
                    && parameter.type_annotation(ast).is_none()
                    && !parameter.optional(ast)
                    && parameter.decorators(ast).is_empty()
            })
            && self.comments.in_range(span).is_empty()
            && !self.diagnostic_range_is_sensitive(span);
        let hug = single.is_some_and(|parameter| {
            let destructuring = matches!(
                ast.formal_parameter_pattern_data(parameter.pattern(ast)),
                FormalParameterPatternData::ArrayPattern(_)
                    | FormalParameterPatternData::ObjectPattern(_)
            );
            let default_hugs = parameter.initializer(ast).is_none_or(|expression| {
                match self.expression_content(expression).data {
                    ExprData::IdentifierReference(_) => true,
                    ExprData::ArrayExpression(array) => array.elements(ast).is_empty(),
                    ExprData::ObjectExpression(object) => object.properties(ast).is_empty(),
                    _ => false,
                }
            });
            let parameter_span = parameter.span(ast);
            destructuring
                && default_hugs
                && !self.diagnostic_range_is_sensitive(span)
                && self
                    .comments
                    .in_range(Span::new(span.start, parameter_span.start))
                    .is_empty()
                && self
                    .comments
                    .in_range(Span::new(parameter_span.end, span.end))
                    .is_empty()
        });
        let mut key_mode = ContentMode::Formatted;
        let mut body_mode = ContentMode::Formatted;
        let mut private_boundary = None;
        if self.has_suppression
            && let ParameterRole::PrivateMethod(_) = role
        {
            let content = Span::new(span.start + 1, span.end - 1);
            let tail = if let Some(rest) = rest {
                Some(CallableParameter::Rest(rest))
            } else {
                items.iter().next_back().and_then(|id| {
                    match ast.formal_parameter_item_data(ast.get_node_in_sub_range(id)) {
                        FormalParameterItemData::FormalParameter(parameter) => {
                            Some(CallableParameter::Formal(parameter))
                        }
                        FormalParameterItemData::TsParameterProperty(_) => None,
                    }
                })
            };
            let end = tail.map_or(content.start, |tail| tail.span(ast).end);
            let gap = Span::new(end, content.end);
            let comments = self.comments.in_range(gap);
            if self.content_mode(comments) == ContentMode::Verbatim {
                let mut preceding = None;
                for position in self.suppression_positions(comments, content.end) {
                    match position {
                        SuppressionPosition::OwnLine => body_mode = ContentMode::Verbatim,
                        SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                            if let Some(tail) = tail {
                                preceding = Some(tail.span(ast));
                            } else {
                                key_mode = ContentMode::Verbatim;
                            }
                        }
                    }
                }
                if let Some(parameter) = tail {
                    let last = comments.last().map_or(end, |comment| comment.span.end);
                    private_boundary = Some(PrivateParameterBoundary {
                        parameter,
                        relation: DelimitedSuppression {
                            preceding,
                            following: body_mode,
                        },
                        closing: self.source_line_constraint(last, content.end),
                    });
                } else {
                    source = ParameterSource::PrivateEmpty(span);
                }
            }
            // The function signature consumes this actual body target below.
        }
        Ok(SourceParameters {
            node,
            source,
            first,
            rest,
            omit_parentheses,
            hug,
            leading_mode: ContentMode::Formatted,
            enclosing_mode,
            suppressed_tail: None,
            private_boundary,
            key_mode,
            body_target: match role {
                ParameterRole::PrivateMethod(body) if body_mode == ContentMode::Verbatim => {
                    Some((body, body_mode))
                }
                _ => None,
            },
        })
    }

    pub(super) fn format_source_parameters(
        &mut self,
        parameters: SourceParameters,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let SourceParameters {
            node,
            source,
            first,
            rest,
            omit_parentheses,
            hug,
            leading_mode,
            enclosing_mode: _,
            suppressed_tail,
            private_boundary,
            key_mode: _,
            body_target: _,
        } = parameters;
        if let ParameterSource::PrivateEmpty(span) = source {
            return self.format_verbatim(span, writer);
        }
        let has_parameters = first.is_some() || rest.is_some();
        let content = match source {
            ParameterSource::Bare(parameter) => parameter.span(ast),
            ParameterSource::Parenthesized(span) | ParameterSource::PrivateEmpty(span) => {
                Span::new(span.start + 1, span.end - 1)
            }
        };
        if omit_parentheses && let Some(CallableParameter::Formal(parameter)) = first {
            let tail = self.format_formal_parameter(parameter, ContentMode::Formatted, writer)?;
            self.format_expression_tail(tail, writer)?;
            return Ok(());
        }
        writer.group(false, |writer| {
            writer.text("(");
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hug && (has_parameters || !self.comments.in_range(content).is_empty()) {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut formal_ids = node.items(ast).iter();
                if matches!(first, Some(CallableParameter::Formal(_))) {
                    // The prepared head already carries this actual formal.
                    formal_ids.next();
                }
                let formals = formal_ids.map(|id| {
                    let item = ast.get_node_in_sub_range(id);
                    match ast.formal_parameter_item_data(item) {
                        FormalParameterItemData::FormalParameter(parameter) => {
                            Ok(CallableParameter::Formal(parameter))
                        }
                        FormalParameterItemData::TsParameterProperty(_) => {
                            Err(FormatError::unsupported(
                                "arrow parameter property",
                                Some(ast.node_kind(item.node_id())),
                                item.span(ast),
                            ))
                        }
                    }
                });
                let mut parameters = first
                    .into_iter()
                    .map(Ok)
                    .chain(formals)
                    .chain(rest.map(CallableParameter::Rest).into_iter().map(Ok));
                let mut next = parameters.next().transpose()?;
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                let mut following_mode =
                    (leading_mode == ContentMode::Verbatim).then_some(leading_mode);
                while let Some(parameter) = next {
                    next = parameters.next().transpose()?;
                    let span = parameter.span(ast);
                    if has_previous {
                        if matches!(separator, Separator::EmptyLine) {
                            writer.break_parent();
                        }
                        let next = self
                            .comments
                            .between(Span::new(end, span.start))?
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = next
                        .map(|parameter| parameter.span(ast))
                        .map_or(FollowingItem::End, FollowingItem::Node);
                    let following = private_boundary
                        .as_ref()
                        .filter(|source| source.parameter == parameter)
                        .map_or(following, |source| {
                            FollowingItem::ResolvedTerminal(source.relation)
                        });
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, content.end),
                        !matches!(parameter, CallableParameter::Rest(_))
                            && !hug
                            && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if suppressed_tail == Some(parameter) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let tail = match parameter {
                        CallableParameter::Formal(parameter) => {
                            self.format_formal_parameter(parameter, item.mode, writer)?
                        }
                        CallableParameter::Rest(rest) => {
                            self.format_rest_parameter(rest, item.mode, writer)?
                        }
                        CallableParameter::This(receiver) => {
                            let prepared =
                                self.prepare_runtime_this_parameter(receiver, item.mode)?;
                            self.format_prepared_parameter_item(prepared, writer)?
                        }
                    };
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                    has_previous = true;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                self.comments.claim(dangling)?;
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                if dangling
                    .iter()
                    .any(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                {
                    writer.break_parent();
                }
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            };
            let end = if hug {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hug && (has_parameters || end != content.start) {
                private_boundary
                    .as_ref()
                    .map_or_else(
                        || self.line_constraint(end, content.end),
                        |source| source.closing,
                    )
                    .write(writer, FlatSeparator::Empty, Writer::soft_line);
            }
            writer.text(")");
            Ok(())
        })
    }

    fn format_formal_parameter(
        &mut self,
        parameter: FormalParameter,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(parameter.span(self.ast), writer)?;
            return Ok(None);
        }
        if self.ast.is_ts() {
            let annotation = parameter.type_annotation(self.ast);
            let optional = parameter.optional(self.ast);
            if annotation.is_some() || optional {
                let prepared =
                    self.prepare_typed_formal_parameter(parameter, mode, annotation, optional)?;
                return writer.group(false, |writer| {
                    self.format_prepared_parameter_item(prepared, writer)
                });
            }
        }
        let pattern = parameter.pattern(self.ast);
        let default = parameter
            .initializer(self.ast)
            .map(|initializer| {
                self.prepare_binding_default(
                    self.parameter_binding(pattern),
                    initializer,
                    parameter.span(self.ast),
                )
            })
            .transpose()?;
        if default
            .as_ref()
            .is_none_or(|default| !default.is_verbatim())
            && !parameter.decorators(self.ast).is_empty()
        {
            return Err(self.unsupported("typed or decorated parameter", parameter));
        }
        writer.group(false, |writer| match default {
            Some(default) => self.format_binding_default(default, writer),
            None => self.format_parameter_binding(pattern, mode, writer),
        })
    }

    fn format_rest_parameter(
        &mut self,
        rest: FormalParameterRest,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(rest.span(self.ast), writer)?;
            return Ok(None);
        }
        if self.ast.is_ts()
            && let Some(annotation) = rest.type_annotation(self.ast)
        {
            let prepared = self.prepare_typed_rest_parameter(rest, mode, annotation)?;
            return self.format_prepared_parameter_item(prepared, writer);
        }
        if !rest.decorators(self.ast).is_empty() {
            return Err(self.unsupported("typed or decorated rest parameter", rest));
        }
        let argument = rest.argument(self.ast);
        writer.text("...");
        let leading = self.comments.between(Span::new(
            rest.span(self.ast).start + 3,
            argument.span(self.ast).start,
        ))?;
        let mode = self.content_mode(leading);
        self.comments.claim(leading)?;
        self.format_comment_run(
            leading,
            CommentRunLayout::FlatLeading(argument.span(self.ast).start),
            writer,
        )?;
        self.format_parameter_binding(argument, mode, writer)
    }
}
