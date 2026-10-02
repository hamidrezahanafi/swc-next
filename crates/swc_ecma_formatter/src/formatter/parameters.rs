//! Actual arrow parameter boundaries and typed formal/rest item ownership.

use swc_next_ecma_ast::{
    ExprData, FormalParameter, FormalParameterItemData, FormalParameterPatternData,
    FormalParameterRest, FormalParameters, GetSpan, Span,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::diagnostics::FlatSeparator;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::ContentMode;
use crate::{ArrowParentheses, FormatError, TrailingCommas, document::Writer};

enum ParameterSource {
    Parenthesized(Span),
    Bare(FormalParameter),
}

pub(super) struct SourceParameters {
    node: FormalParameters,
    source: ParameterSource,
    first: Option<FormalParameter>,
    rest: Option<FormalParameterRest>,
    omit_parentheses: bool,
    hug: bool,
}

impl SourceParameters {
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
        let ast = self.ast;
        let span = node.span(ast);
        let items = node.items(ast);
        let first = items.iter().next().map(|id| ast.get_node_in_sub_range(id));
        let first = match first.map(|item| ast.formal_parameter_item_data(item)) {
            Some(FormalParameterItemData::FormalParameter(parameter)) => Some(parameter),
            Some(FormalParameterItemData::TsParameterProperty(_)) => {
                return Err(self.unsupported("arrow parameter property", node));
            }
            None => None,
        };
        let source = if self.source(span).starts_with('(') && self.source(span).ends_with(')') {
            ParameterSource::Parenthesized(span)
        } else if let Some(parameter) = first
            && items.len() == 1
            && node.rest(ast).is_none()
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
        let single = match first {
            Some(parameter) if items.len() == 1 && node.rest(ast).is_none() => Some(parameter),
            _ => None,
        };
        let omit_parentheses = !has_trailing_comments
            && self.options.arrow_parentheses == ArrowParentheses::AsNeeded
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
        Ok(SourceParameters {
            node,
            source,
            first,
            rest: node.rest(ast),
            omit_parentheses,
            hug,
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
        } = parameters;
        let has_parameters = first.is_some() || rest.is_some();
        let (content, delimiters) = match source {
            ParameterSource::Bare(parameter) => (parameter.span(ast), None),
            ParameterSource::Parenthesized(span) => {
                (Span::new(span.start + 1, span.end - 1), Some(span))
            }
        };
        if !has_parameters
            && self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim
            && let Some(delimiters) = delimiters
        {
            return self.format_verbatim(delimiters, writer);
        }
        if omit_parentheses && let Some(parameter) = first {
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
                let mut formals = node
                    .items(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut end = content.start;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                while let Some(item) = formals.next() {
                    let parameter = match ast.formal_parameter_item_data(item) {
                        FormalParameterItemData::FormalParameter(parameter) => parameter,
                        FormalParameterItemData::TsParameterProperty(_) => {
                            return Err(self.unsupported("arrow parameter property", item));
                        }
                    };
                    if has_previous {
                        let next = self
                            .comments
                            .between(Span::new(end, parameter.span(ast).start))?
                            .first()
                            .map_or(parameter.span(ast).start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = formals
                        .peek()
                        .map(|item| item.span(ast))
                        .or_else(|| rest.map(|rest| rest.span(ast)))
                        .map_or(FollowingItem::End, FollowingItem::Node);
                    let item = self.prepare_delimited_item(
                        parameter.span(ast),
                        following,
                        Span::new(end, content.end),
                        !hug && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    let tail = self.format_formal_parameter(parameter, item.mode, writer)?;
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    separator = boundary.separator;
                    has_previous = true;
                }
                if let Some(rest) = rest {
                    if has_previous {
                        let next = self
                            .comments
                            .between(Span::new(end, rest.span(ast).start))?
                            .first()
                            .map_or(rest.span(ast).start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let item = self.prepare_delimited_item(
                        rest.span(ast),
                        FollowingItem::End,
                        Span::new(end, content.end),
                        false,
                        writer,
                    )?;
                    let tail = self.format_rest_parameter(rest, item.mode, writer)?;
                    end = self.finish_delimited_item(item, tail, writer)?.end;
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
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
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
            && (parameter.type_annotation(self.ast).is_some()
                || parameter.optional(self.ast)
                || !parameter.decorators(self.ast).is_empty())
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
        if rest.type_annotation(self.ast).is_some() || !rest.decorators(self.ast).is_empty() {
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
