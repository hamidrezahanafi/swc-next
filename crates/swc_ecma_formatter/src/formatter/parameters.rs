//! Actual runtime parameter boundaries and receiver/formal/rest ownership.

use swc_next_ecma_ast::{
    Ast, ExprData, FormalParameter, FormalParameterItemData, FormalParameterPatternData,
    FormalParameterRest, FormalParameters, FunctionBody, GetSpan, MethodDefinition, Span,
    TsParameterProperty, TsThisParameter,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use super::types::TypeGap;
use super::types::callable::CallableParameter;
use crate::{ArrowParentheses, FormatError, TrailingCommas, document::Writer};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrowParameterParentheses {
    Optional,
    Required,
}

enum ParameterRole {
    Arrow(ArrowParameterParentheses),
    Function,
    Constructor(MethodDefinition),
    PrivateMethod(FunctionBody),
}

enum ParameterSource {
    Parenthesized(Span),
    Constructor { owner: MethodDefinition, span: Span },
    Bare(FormalParameter),
    PrivateEmpty(Span),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RuntimeParameter {
    Callable(CallableParameter),
    Property(TsParameterProperty),
}

impl RuntimeParameter {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Callable(parameter) => parameter.span(ast),
            Self::Property(property) => property.span(ast),
        }
    }

    fn from_formal(
        item: swc_next_ecma_ast::FormalParameterItem,
        ast: &Ast<'_>,
        constructor: Option<MethodDefinition>,
    ) -> Result<Self, FormatError> {
        match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => {
                Ok(Self::Callable(CallableParameter::Formal(parameter)))
            }
            FormalParameterItemData::TsParameterProperty(property)
                if constructor.is_some() && ast.is_ts() =>
            {
                Ok(Self::Property(property))
            }
            FormalParameterItemData::TsParameterProperty(_) => Err(FormatError::unsupported(
                "arrow parameter property",
                Some(ast.node_kind(item.node_id())),
                item.span(ast),
            )),
        }
    }
}

struct ParameterModifier<'ast> {
    keyword: &'static str,
    before: super::comments::CommentBoundary<'ast>,
}

struct ParameterPropertyPrefix<'ast> {
    modifiers: [Option<ParameterModifier<'ast>>; 3],
    following: super::comments::CommentBoundary<'ast>,
}

struct PrivateParameterBoundary {
    parameter: CallableParameter,
    relation: DelimitedSuppression,
    closing: LineConstraint,
}

pub(super) struct SourceParameters {
    node: FormalParameters,
    source: ParameterSource,
    first: Option<RuntimeParameter>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: bool,
    leading_mode: ContentMode,
    enclosing_mode: ContentMode,
    suppressed_tail: Option<RuntimeParameter>,
    private_boundary: Option<PrivateParameterBoundary>,
    key_mode: ContentMode,
    body_target: Option<(FunctionBody, ContentMode)>,
}

impl SourceParameters {
    pub(super) fn span(&self, ast: &Ast<'_>) -> Span {
        self.node.span(ast)
    }

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
        let constructor = match self.source {
            ParameterSource::Constructor { owner, .. } => Some(owner),
            _ => None,
        };
        self.suppressed_tail = if let Some(rest) = self.rest {
            Some(RuntimeParameter::Callable(CallableParameter::Rest(rest)))
        } else if let Some(id) = self.node.items(ast).iter().next_back() {
            let item = ast.get_node_in_sub_range(id);
            Some(
                RuntimeParameter::from_formal(item, ast, constructor).map_err(|_| {
                    FormatError::unsupported("typed parameter property", None, item.span(ast))
                })?,
            )
        } else {
            self.first.filter(|parameter| {
                matches!(
                    parameter,
                    RuntimeParameter::Callable(CallableParameter::This(_))
                )
            })
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
        parentheses: ArrowParameterParentheses,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, None, ParameterRole::Arrow(parentheses))
    }

    pub(super) fn source_function_parameters(
        &self,
        node: FormalParameters,
        receiver: Option<TsThisParameter>,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, receiver, ParameterRole::Function)
    }

    pub(super) fn source_constructor_parameters(
        &self,
        node: FormalParameters,
        owner: MethodDefinition,
    ) -> Result<SourceParameters, FormatError> {
        self.source_parameters(node, None, ParameterRole::Constructor(owner))
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
        let constructor = match role {
            ParameterRole::Constructor(owner) => Some(owner),
            _ => None,
        };
        let first = if let Some(receiver) = receiver {
            Some(RuntimeParameter::Callable(CallableParameter::This(
                receiver,
            )))
        } else {
            items
                .iter()
                .next()
                .map(|id| {
                    RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast, constructor)
                })
                .transpose()
                .map_err(|_| self.unsupported("arrow parameter property", node))?
        };
        let mut source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
            match constructor {
                Some(owner) => ParameterSource::Constructor { owner, span },
                None => ParameterSource::Parenthesized(span),
            }
        } else if matches!(role, ParameterRole::Arrow(_))
            && let Some(RuntimeParameter::Callable(CallableParameter::Formal(parameter))) = first
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
            Some(RuntimeParameter::Callable(CallableParameter::Formal(parameter)))
                if items.len() == 1 && rest.is_none() =>
            {
                Some(parameter)
            }
            _ => None,
        };
        let omit_parentheses = matches!(
            role,
            ParameterRole::Arrow(ArrowParameterParentheses::Optional)
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
        let constructor = match source {
            ParameterSource::Constructor { owner, .. } => Some(owner),
            _ => None,
        };
        let content = match source {
            ParameterSource::Bare(parameter) => parameter.span(ast),
            ParameterSource::Parenthesized(span)
            | ParameterSource::Constructor { span, .. }
            | ParameterSource::PrivateEmpty(span) => Span::new(span.start + 1, span.end - 1),
        };
        if omit_parentheses
            && let Some(RuntimeParameter::Callable(CallableParameter::Formal(parameter))) = first
        {
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
                if matches!(
                    first,
                    Some(
                        RuntimeParameter::Callable(CallableParameter::Formal(_))
                            | RuntimeParameter::Property(_)
                    )
                ) {
                    // The prepared head already carries this actual formal.
                    formal_ids.next();
                }
                let formals = formal_ids.map(|id| {
                    RuntimeParameter::from_formal(ast.get_node_in_sub_range(id), ast, constructor)
                });
                let mut parameters = first.into_iter().map(Ok).chain(formals).chain(
                    rest.map(|rest| RuntimeParameter::Callable(CallableParameter::Rest(rest)))
                        .into_iter()
                        .map(Ok),
                );
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
                        .filter(|source| RuntimeParameter::Callable(source.parameter) == parameter)
                        .map_or(following, |source| {
                            FollowingItem::ResolvedTerminal(source.relation)
                        });
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, content.end),
                        !matches!(
                            parameter,
                            RuntimeParameter::Callable(CallableParameter::Rest(_))
                        ) && !hug
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
                        RuntimeParameter::Callable(CallableParameter::Formal(parameter)) => {
                            self.format_formal_parameter(parameter, item.mode, writer)?
                        }
                        RuntimeParameter::Callable(CallableParameter::Rest(rest)) => {
                            self.format_rest_parameter(rest, item.mode, writer)?
                        }
                        RuntimeParameter::Property(property) => {
                            self.format_constructor_parameter_property(property, item.mode, writer)?
                        }
                        RuntimeParameter::Callable(CallableParameter::This(receiver)) => {
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

    fn format_constructor_parameter_property(
        &mut self,
        property: TsParameterProperty,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(property.span(self.ast), writer)?;
            return Ok(None);
        }
        if !property.decorators(self.ast).is_empty() {
            return Err(self.unsupported("constructor parameter property decorator", property));
        }
        let prepared = self.prepare_constructor_parameter_item(property)?;
        if prepared.is_source() {
            return self.format_prepared_parameter_item(prepared, writer);
        }
        let prefix = self.prepare_parameter_property_prefix(property)?;
        writer.group(false, |writer| {
            for (index, modifier) in prefix.modifiers.into_iter().flatten().enumerate() {
                let separator = if index == 0 {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                self.format_type_gap(TypeGap::Formatted(modifier.before), separator, writer)?;
                writer.text(modifier.keyword);
            }
            self.format_type_gap(
                TypeGap::Formatted(prefix.following),
                FlatSeparator::Space,
                writer,
            )?;
            self.format_prepared_parameter_item(prepared, writer)
        })
    }

    fn prepare_parameter_property_prefix(
        &self,
        property: TsParameterProperty,
    ) -> Result<ParameterPropertyPrefix<'ast>, FormatError> {
        let ast = self.ast;
        let span = Span::new(
            property.span(ast).start,
            property.parameter(ast).span(ast).start,
        );
        let mut comments = self.comments.in_range(span);
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            return Err(self.unsupported("parameter property modifier suppression", property));
        }
        let accessibility = match property.accessibility(ast) {
            swc_next_ecma_ast::Accessibility::None => None,
            swc_next_ecma_ast::Accessibility::Public => Some("public"),
            swc_next_ecma_ast::Accessibility::Protected => Some("protected"),
            swc_next_ecma_ast::Accessibility::Private => Some("private"),
        };
        let keywords = [
            accessibility,
            property.r#override(ast).then_some("override"),
            property.readonly(ast).then_some("readonly"),
        ];
        let mut tokens = [None; 3];
        for (slot, keyword) in tokens.iter_mut().zip(keywords) {
            let Some(keyword) = keyword else {
                continue;
            };
            let position = self.find_token(span, comments, keyword).ok_or_else(|| {
                self.unsupported("constructor parameter modifier boundary", property)
            })?;
            *slot = Some((position, keyword));
        }
        tokens.sort_unstable_by_key(|token| token.map(|(position, _)| position));
        let mut end = span.start;
        let mut modifiers = std::array::from_fn(|_| None);
        for (slot, (position, keyword)) in modifiers.iter_mut().zip(tokens.into_iter().flatten()) {
            let split = comments.partition_point(|comment| comment.span.end <= position);
            let (before, following) = comments.split_at(split);
            *slot = Some(ParameterModifier {
                keyword,
                before: self.prepare_comment_boundary(before, end, position),
            });
            comments = following;
            end = position + keyword.len() as u32;
        }
        Ok(ParameterPropertyPrefix {
            modifiers,
            following: self.prepare_comment_boundary(comments, end, span.end),
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
