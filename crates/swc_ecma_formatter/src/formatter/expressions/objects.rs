//! Object literal properties and their source-ordered comment boundaries.

use swc_next_ecma_ast::{
    Argument, Comment, CommentKind, ExprData, GetSpan, ObjectExpression, ObjectProperty,
    ObjectPropertyKindData, PropertyKeyData, PropertyKind, Span,
};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, QuoteProperties, TrailingCommas, document::Writer};

struct ObjectBoundary<'ast> {
    inline: &'ast [Comment],
    own_line: &'ast [Comment],
    leading: &'ast [Comment],
    trailing_end: u32,
    separator: Span,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_object(
        &mut self,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = object.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        if self.diagnostic_range_is_sensitive(content) {
            return Err(self.unsupported("object diagnostic boundary", object));
        }
        let mut properties = object
            .properties(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let Some(first) = properties.peek().copied() else {
            let comments = self.comments.between(content)?;
            if self.content_mode(comments) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            let comments = self.comments.take(comments.len());
            return writer.group(false, |writer| {
                writer.text("{");
                if !comments.is_empty() {
                    writer.indent(|writer| {
                        writer.soft_line();
                        self.format_comment_run(comments, CommentRunLayout::Lines, writer)
                    })?;
                    writer.soft_line();
                    if comments
                        .iter()
                        .any(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.break_parent();
                    }
                }
                writer.text("}");
                Ok(())
            });
        };
        let expand = self.line_breaks(content.start, first.span(ast).start) != 0;
        writer.group(expand, |writer| {
            writer.text("{");
            writer.indent(|writer| {
                self.object_brace_line(writer);
                let leading = self
                    .comments
                    .between(Span::new(content.start, first.span(ast).start))?;
                let mut leading = leading;
                while let Some(property) = properties.next() {
                    let property_span = property.span(ast);
                    let terminal = properties.peek().is_none();
                    let next_start = properties
                        .peek()
                        .map_or(content.end, |next| next.span(ast).start);
                    let boundary =
                        self.object_boundary(Span::new(property_span.end, next_start), terminal);
                    let mode = if self.content_mode(leading) == ContentMode::Verbatim
                        || self.content_mode(boundary.inline) == ContentMode::Verbatim
                        || self.content_mode(boundary.own_line) == ContentMode::Verbatim
                    {
                        ContentMode::Verbatim
                    } else {
                        ContentMode::Formatted
                    };
                    self.comments.claim(leading)?;
                    self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
                    if let Some(last) = leading.last() {
                        self.comment_gap(last.span.end, property_span.start, writer);
                    }
                    let tail = if mode == ContentMode::Verbatim {
                        self.format_verbatim(property_span, writer)?;
                        None
                    } else {
                        match ast.object_property_kind_data(property) {
                            ObjectPropertyKindData::ObjectProperty(property) => {
                                self.format_object_property(property, writer)?
                            }
                            ObjectPropertyKindData::SpreadElement(spread) => {
                                self.format_argument(Argument::SpreadElement(spread), mode, writer)?
                            }
                        }
                    };
                    self.format_expression_tail(tail, writer)?;
                    self.comments.claim(boundary.inline)?;
                    self.format_inline_trailing_comments(boundary.inline, writer)?;
                    if !terminal {
                        writer.text(",");
                        writer.line();
                        if self.next_line_is_empty(boundary.separator.start, boundary.separator.end)
                        {
                            writer.hard_line();
                        }
                    } else {
                        if self.options.trailing_commas != TrailingCommas::None {
                            writer.if_break(",");
                        }
                        self.comments.claim(boundary.own_line)?;
                        if let Some(first) = boundary.own_line.first() {
                            writer.hard_line();
                            if self.next_line_is_empty(boundary.trailing_end, first.span.start) {
                                writer.hard_line();
                            }
                            self.format_comment_run(
                                boundary.own_line,
                                CommentRunLayout::Lines,
                                writer,
                            )?;
                        }
                    }
                    leading = boundary.leading;
                }
                Ok(())
            })?;
            self.object_brace_line(writer);
            writer.text("}");
            Ok(())
        })
    }

    fn object_boundary(&self, span: Span, terminal: bool) -> ObjectBoundary<'ast> {
        let comments = self.comments.in_range(span);
        let comma = self.find_comma(span, comments);
        let mut inline_end = span.start;
        let mut count = 0;
        for (index, comment) in comments.iter().enumerate() {
            if self.line_breaks(inline_end, comment.span.start) != 0 {
                break;
            }
            inline_end = comment.span.end;
            if comment.kind == CommentKind::Line {
                count = index + 1;
                break;
            }
            let following_start = comments
                .get(index + 1)
                .map_or(span.end, |next| next.span.start);
            if terminal
                || comma.is_none_or(|comma| comment.span.start < comma)
                || self.line_breaks(comment.span.end, following_start) != 0
            {
                count = index + 1;
            }
        }
        let (inline, remaining) = comments.split_at(count);
        let trailing_end = inline.last().map_or(span.start, |comment| comment.span.end);
        let next_start = remaining
            .first()
            .map_or(span.end, |comment| comment.span.start);
        // The next property's leading comment can precede the source comma.
        let separator_start = match comma {
            Some(comma) if comma < next_start => trailing_end.max(comma + 1),
            _ => trailing_end,
        };
        ObjectBoundary {
            inline,
            own_line: if terminal { remaining } else { &[] },
            leading: if terminal { &[] } else { remaining },
            trailing_end,
            separator: Span::new(separator_start, next_start),
        }
    }

    fn object_brace_line(&self, writer: &mut Writer<'ast>) {
        if self.options.bracket_spacing {
            writer.line();
        } else {
            writer.soft_line();
        }
    }

    fn format_object_property(
        &mut self,
        property: ObjectProperty,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if property.method(ast) || property.kind(ast) != PropertyKind::Init {
            return Err(self.unsupported("object method", property));
        }
        if self.has_suppression {
            let header = Span::new(
                property.span(ast).start,
                property.value(ast).span(ast).start,
            );
            if self.content_mode(self.comments.in_range(header)) == ContentMode::Verbatim {
                return Err(self.unsupported("object property suppression boundary", property));
            }
        }
        writer.group(false, |writer| {
            let key_end = self.format_object_key(property, writer)?;
            if property.shorthand(ast) {
                return Ok(None);
            }
            let value = property.value(ast);
            let value_start = value.span(ast).start;
            let gap = Span::new(key_end, value_start);
            let comments = self.comments.between(gap)?;
            let Some(colon) = self.find_token(gap, comments, ":") else {
                return Err(self.unsupported("object property colon", property));
            };
            let count = comments.partition_point(|comment| comment.span.end <= colon);
            let before = self.comments.take(count);
            self.format_comment_run(before, CommentRunLayout::SourceAfter(key_end), writer)?;
            writer.text(":");
            let leading = self.comments.between(Span::new(colon + 1, value_start))?;
            let leading = self.comments.take(leading.len());
            let own_line = leading.first().is_some_and(|comment| {
                comment.kind == CommentKind::Line
                    || self.line_breaks(colon + 1, comment.span.start) != 0
            });
            let data = self.expression_content(value).data;
            let break_after = own_line || self.assignment_right_breaks_after_operator(data);
            let mut write_value = |writer: &mut Writer<'ast>| {
                self.format_expression_with_leading_comments(
                    value,
                    NodeHead {
                        comments: leading,
                        content_start: value_start,
                        layout: CommentRunLayout::Leading(value_start),
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::PropertyValue)
                        .with_content_mode(self.content_mode(leading)),
                    writer,
                )
            };
            if break_after {
                writer.group(false, |writer| {
                    writer.indent(|writer| {
                        if own_line {
                            writer.hard_line();
                        } else {
                            writer.line();
                        }
                        write_value(writer)
                    })
                })
            } else if matches!(
                data,
                ExprData::BooleanLiteral(_) | ExprData::NumericLiteral(_)
            ) {
                writer.space();
                write_value(writer)
            } else {
                let group = writer.named_group(false, |writer| writer.indent(Writer::line));
                writer.indent_if_break(group, write_value)
            }
        })
    }

    fn format_object_key(
        &mut self,
        property: ObjectProperty,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let ast = self.ast;
        let key = property.key(ast);
        let key_span = key.span(ast);
        if property.computed(ast) {
            let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
                return Err(self.unsupported("computed object key", key));
            };
            let gap = Span::new(key_span.end, property.value(ast).span(ast).start);
            let comments = self.comments.in_range(gap);
            let Some(closing) = self.find_token(gap, comments, "]") else {
                return Err(self.unsupported("computed object key boundary", property));
            };
            writer.group(false, |writer| {
                writer.text("[");
                writer.indent(|writer| {
                    writer.soft_line();
                    let leading = self
                        .comments
                        .between(Span::new(property.span(ast).start + 1, key_span.start))?;
                    let leading = self.comments.take(leading.len());
                    let tail = self.format_expression_with_leading_comments(
                        expression,
                        NodeHead {
                            comments: leading,
                            content_start: key_span.start,
                            layout: CommentRunLayout::Leading(key_span.start),
                            gap_end: None,
                        },
                        ExpressionContext::at(ExpressionPosition::Nested)
                            .with_content_mode(self.content_mode(leading)),
                        writer,
                    )?;
                    let tail =
                        self.capture_expression_tail(Span::new(key_span.end, closing), tail)?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text("]");
                Ok(())
            })?;
            return Ok(closing + 1);
        }
        match ast.property_key_data(key) {
            PropertyKeyData::IdentifierName(identifier) => {
                writer.text(ast.get_utf8(identifier.name(ast)))
            }
            PropertyKeyData::Expr(expression) => match ast.expr_data(expression) {
                ExprData::StringLiteral(literal) => {
                    let raw = ast.get_utf8(literal.raw(ast));
                    let content = &raw[1..raw.len() - 1];
                    if self.options.quote_properties == QuoteProperties::AsNeeded
                        && (swc_next_ecma_ast::is_identifier_es5_and_esnext(content.as_bytes())
                            || !ast.lang().is_typescript() && numeric_property_name(content))
                    {
                        writer.text(content);
                    } else {
                        self.format_string_literal(literal, StringContext::Expression, writer);
                    }
                }
                ExprData::NumericLiteral(literal) => self.format_numeric_literal(literal, writer),
                ExprData::BigIntLiteral(literal) => self.format_bigint_literal(literal, writer),
                _ => return Err(self.unsupported("object property key", key)),
            },
        }
        Ok(key_span.end)
    }
}

fn numeric_property_name(content: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let (integer, fraction) = content
        .split_once('.')
        .map_or((content, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    if !digits(integer) || fraction.is_some_and(|fraction| !digits(fraction)) {
        return false;
    }
    // Numeric keys must round-trip through ECMAScript's number-to-string conversion.
    content
        .parse::<f64>()
        .is_ok_and(|number| zmij_ecma::Buffer::new().format(number) == content)
}
