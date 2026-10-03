//! Object literal properties and their source-ordered comment boundaries.

use swc_next_ecma_ast::{
    Argument, Comment, CommentKind, ExprData, GetSpan, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, ObjectPropertyKindData, PropertyKey, PropertyKeyData, PropertyKind, Span,
    SubRange,
};

use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::literals::StringContext;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{
    FormatError, QuoteProperties, TrailingCommas,
    document::{Fragment, Writer},
};

enum SourceObject<'ast> {
    Verbatim(Span),
    Formatted(FormattedObject<'ast>),
}

struct FormattedObject<'ast> {
    expand: bool,
    contents: ObjectContents<'ast>,
}

enum ObjectContents<'ast> {
    Empty {
        comments: &'ast [Comment],
    },
    Properties {
        content: Span,
        first: ObjectPropertyKind,
        remaining: SubRange<ObjectPropertyKind>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct RecordedObject<'ast> {
    pub(super) normal: Fragment<'ast>,
    pub(super) forced: Option<Fragment<'ast>>,
}

struct ObjectBoundary<'ast> {
    inline: &'ast [Comment],
    own_line: &'ast [Comment],
    leading: &'ast [Comment],
    trailing_end: u32,
    separator: Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::formatter) enum PropertySuppression {
    None,
    Value,
    Property,
}

struct ObjectPropertyBoundary<'ast> {
    suppression: PropertySuppression,
    before: &'ast [Comment],
    moved: &'ast [Comment],
    operator_line: &'ast [Comment],
    operator_line_start: u32,
    leading: &'ast [Comment],
    key_end: u32,
    colon: Span,
    value_start: u32,
    line_before_colon: bool,
    line_after_colon: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_object(
        &mut self,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.source_object(object)? {
            SourceObject::Verbatim(span) => self.format_verbatim(span, writer),
            SourceObject::Formatted(source) => writer.group(source.expand, |writer| {
                self.format_object_contents(source.contents, writer)
            }),
        }
    }

    fn source_object(
        &mut self,
        object: ObjectExpression,
    ) -> Result<SourceObject<'ast>, FormatError> {
        let ast = self.ast;
        let span = object.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        if self.diagnostic_range_is_sensitive(content) {
            return Err(self.unsupported("object diagnostic boundary", object));
        }
        let mut remaining = object.properties(ast);
        let Some(first) = remaining.first().map(|id| ast.get_node_in_sub_range(id)) else {
            let comments = self.comments.between(content)?;
            if self.content_mode(comments) == ContentMode::Verbatim {
                return Ok(SourceObject::Verbatim(span));
            }
            let comments = self.comments.take(comments.len());
            return Ok(SourceObject::Formatted(FormattedObject {
                expand: false,
                contents: ObjectContents::Empty { comments },
            }));
        };
        let remaining = remaining.split_off(1);
        let expand = self.line_breaks(content.start, first.span(ast).start) != 0;
        Ok(SourceObject::Formatted(FormattedObject {
            expand,
            contents: ObjectContents::Properties {
                content,
                first,
                remaining,
            },
        }))
    }

    pub(super) fn record_object(
        &mut self,
        object: ObjectExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<RecordedObject<'ast>, FormatError> {
        match self.source_object(object)? {
            SourceObject::Verbatim(span) => {
                let (normal, ()) =
                    writer.record_with_value(|writer| self.format_verbatim(span, writer))?;
                Ok(RecordedObject {
                    normal,
                    forced: None,
                })
            }
            SourceObject::Formatted(FormattedObject { expand, contents }) => {
                let (content, ()) = writer
                    .record_with_value(|writer| self.format_object_contents(contents, writer))?;
                let (normal, ()) = writer.record_with_value(|writer| {
                    writer.group(expand, |writer| writer.write_fragment(content));
                    Ok::<_, FormatError>(())
                })?;
                let forced = if normal.preferred_breaks() {
                    None
                } else {
                    let (forced, ()) = writer.record_with_value(|writer| {
                        writer.group(true, |writer| writer.write_fragment(content));
                        Ok::<_, FormatError>(())
                    })?;
                    Some(forced)
                };
                Ok(RecordedObject { normal, forced })
            }
        }
    }

    fn format_object_contents(
        &mut self,
        contents: ObjectContents<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        match contents {
            ObjectContents::Empty { comments } => {
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
            }
            ObjectContents::Properties {
                content,
                first,
                remaining,
            } => {
                writer.text("{");
                writer.indent(|writer| {
                    self.object_brace_line(writer);
                    let leading = self
                        .comments
                        .between(Span::new(content.start, first.span(ast).start))?;
                    let mut leading = leading;
                    let mut properties = remaining
                        .iter()
                        .map(|id| ast.get_node_in_sub_range(id))
                        .peekable();
                    let mut property = first;
                    loop {
                        let property_span = property.span(ast);
                        let terminal = properties.peek().is_none();
                        let next_start = properties
                            .peek()
                            .map_or(content.end, |next| next.span(ast).start);
                        let boundary = self
                            .object_boundary(Span::new(property_span.end, next_start), terminal);
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
                                ObjectPropertyKindData::SpreadElement(spread) => self
                                    .format_argument(
                                        Argument::SpreadElement(spread),
                                        mode,
                                        writer,
                                    )?,
                            }
                        };
                        self.format_expression_tail(tail, writer)?;
                        self.comments.claim(boundary.inline)?;
                        self.format_inline_trailing_comments(boundary.inline, writer)?;
                        if !terminal {
                            writer.text(",");
                            writer.line();
                            if self.next_line_is_empty(
                                boundary.separator.start,
                                boundary.separator.end,
                            ) {
                                writer.hard_line();
                            }
                        } else {
                            if self.options.trailing_commas != TrailingCommas::None {
                                writer.if_break(",");
                            }
                            self.comments.claim(boundary.own_line)?;
                            if let Some(first) = boundary.own_line.first() {
                                writer.hard_line();
                                if self.next_line_is_empty(boundary.trailing_end, first.span.start)
                                {
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
                        let Some(next) = properties.next() else {
                            break;
                        };
                        property = next;
                    }
                    Ok(())
                })?;
                self.object_brace_line(writer);
                writer.text("}");
                Ok(())
            }
        }
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
        let key_end = self.object_key_end(property)?;
        if property.shorthand(ast) {
            self.format_object_key(property, key_end, writer)?;
            return Ok(None);
        }
        let boundary = self.object_property_boundary(property, key_end)?;
        if boundary.suppression == PropertySuppression::Property {
            self.format_verbatim(property.span(ast), writer)?;
            return Ok(None);
        }
        let value = property.value(ast);
        let data = self.expression_content(value).data;
        let break_after =
            boundary.line_after_colon || self.assignment_right_breaks_after_operator(data);
        writer.group(false, |writer| {
            self.format_object_key(property, boundary.key_end, writer)?;
            self.comments.claim(boundary.before)?;
            self.format_comment_run(
                boundary.before,
                CommentRunLayout::SourceAfter(boundary.key_end),
                writer,
            )?;
            if boundary.line_before_colon {
                writer.hard_line();
            }
            writer.text(":");
            self.comments.claim(boundary.operator_line)?;
            self.format_comment_run(
                boundary.operator_line,
                CommentRunLayout::SourceAfter(boundary.operator_line_start),
                writer,
            )?;
            let mut write_value = |writer: &mut Writer<'ast>| {
                self.comments.claim(boundary.moved)?;
                self.format_comment_run(boundary.moved, CommentRunLayout::Source, writer)?;
                if let Some(last) = boundary.moved.last() {
                    let next = boundary
                        .leading
                        .first()
                        .map_or(boundary.value_start, |comment| comment.span.start);
                    let before = self.line_breaks(last.span.end, boundary.colon.start);
                    let after = self.line_breaks(boundary.colon.end, next);
                    // Moving the colon does not introduce an empty source line.
                    match before.max(after) {
                        0 => writer.space(),
                        1 => writer.hard_line(),
                        _ => {
                            writer.hard_line();
                            writer.hard_line();
                        }
                    }
                }
                self.comments.claim(boundary.leading)?;
                self.format_expression_with_leading_comments(
                    value,
                    NodeHead {
                        comments: boundary.leading,
                        content_start: boundary.value_start,
                        layout: CommentRunLayout::GroupedLeading(boundary.value_start),
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::PropertyValue).with_content_mode(
                        if boundary.suppression == PropertySuppression::Value {
                            ContentMode::Verbatim
                        } else {
                            ContentMode::Formatted
                        },
                    ),
                    writer,
                )
            };
            if break_after {
                writer.group(false, |writer| {
                    writer.indent(|writer| {
                        if boundary.line_after_colon {
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

    fn object_property_boundary(
        &self,
        property: ObjectProperty,
        key_end: u32,
    ) -> Result<ObjectPropertyBoundary<'ast>, FormatError> {
        let value_start = property.value(self.ast).span(self.ast).start;
        let gap = Span::new(key_end, value_start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("object property colon", property));
        };
        let colon = Span::new(colon_start, colon_start + 1);
        let count = comments.partition_point(|comment| comment.span.end <= colon.start);
        let (before, after) = comments.split_at(count);
        let suppression = self.property_suppression(before, after, colon, value_start);
        let ordinary_before = self
            .comment_run_binding(before.iter().copied(), |comment| {
                if self.comment_semantics(comment).is_ordinary() {
                    CommentBinding::Unbound
                } else {
                    CommentBinding::SourceBound
                }
            })
            .allows_relocation(None);
        let promoted = ordinary_before
            && before.first().is_some_and(|first| {
                self.line_breaks(key_end, first.span.start) != 0
                    || before.last().is_some_and(|last| {
                        self.line_breaks(last.span.start, last.span.end) != 0
                            && self.line_breaks(last.span.end, colon.start) != 0
                    })
            });
        let (before, moved) = if promoted {
            (&[][..], before)
        } else {
            (before, &[][..])
        };
        let line_before_colon = before
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line);
        let mut count = 0;
        let mut previous = colon.end;
        for (index, comment) in after.iter().enumerate() {
            if promoted || self.line_breaks(previous, comment.span.start) != 0 {
                break;
            }
            if comment.kind == CommentKind::Line {
                count = index + 1;
                break;
            }
            previous = comment.span.end;
        }
        let (operator_line, leading) = after.split_at(count);
        let (before, operator_line, operator_line_start, leading) =
            if ordinary_before && line_before_colon {
                (&[][..], before, key_end, after)
            } else {
                (before, operator_line, colon.end, leading)
            };
        let line_before_colon = line_before_colon && !ordinary_before;
        let line_after_colon = promoted
            || !operator_line.is_empty()
            || leading.first().is_some_and(|comment| {
                comment.kind == CommentKind::Line
                    || self.line_breaks(colon.end, comment.span.start) != 0
            });
        Ok(ObjectPropertyBoundary {
            suppression,
            before,
            moved,
            operator_line,
            operator_line_start,
            leading,
            key_end,
            colon,
            value_start,
            line_before_colon,
            line_after_colon,
        })
    }

    pub(in crate::formatter) fn property_suppression(
        &self,
        before: &[Comment],
        after: &[Comment],
        colon: Span,
        value_start: u32,
    ) -> PropertySuppression {
        let before = self.suppression_position(before, colon.start);
        let after = self.suppression_position(after, value_start);
        match (before, after) {
            (Some(SuppressionPosition::EndOfLine), _)
            | (_, Some(SuppressionPosition::EndOfLine)) => PropertySuppression::Property,
            (Some(SuppressionPosition::OwnLine), _)
            | (_, Some(SuppressionPosition::OwnLine | SuppressionPosition::Inline)) => {
                PropertySuppression::Value
            }
            _ => PropertySuppression::None,
        }
    }

    fn object_key_end(&self, property: ObjectProperty) -> Result<u32, FormatError> {
        let key_end = property.key(self.ast).span(self.ast).end;
        if !property.computed(self.ast) {
            return Ok(key_end);
        }
        let gap = Span::new(key_end, property.value(self.ast).span(self.ast).start);
        let comments = self.comments.in_range(gap);
        self.find_token(gap, comments, "]")
            .map(|closing| closing + 1)
            .ok_or_else(|| self.unsupported("computed object key boundary", property))
    }

    fn format_object_key(
        &mut self,
        property: ObjectProperty,
        key_end: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let computed = property
            .computed(self.ast)
            .then_some(Span::new(property.span(self.ast).start, key_end));
        self.format_property_key(property.key(self.ast), computed, writer)
    }

    pub(in crate::formatter) fn format_property_key(
        &mut self,
        key: PropertyKey,
        computed: Option<Span>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let key_span = key.span(ast);
        if let Some(computed) = computed {
            let PropertyKeyData::Expr(expression) = ast.property_key_data(key) else {
                return Err(self.unsupported("computed object key", key));
            };
            let closing = computed.end - 1;
            writer.group(false, |writer| {
                writer.text("[");
                writer.indent(|writer| {
                    writer.soft_line();
                    let leading = self
                        .comments
                        .between(Span::new(computed.start + 1, key_span.start))?;
                    let leading = self.comments.take(leading.len());
                    let tail = self.format_expression_with_leading_comments(
                        expression,
                        NodeHead {
                            comments: leading,
                            content_start: key_span.start,
                            layout: CommentRunLayout::GroupedLeading(key_span.start),
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
            return Ok(());
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
        Ok(())
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
