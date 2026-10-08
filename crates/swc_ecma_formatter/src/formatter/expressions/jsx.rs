//! JSX names, attributes and brace containers retain their actual source owners.

mod children;

use std::borrow::Cow;

use swc_next_ecma_ast::{
    CommentKind, Expr, GetSpan, JsxAttribute, JsxAttributeItemData, JsxAttributeName,
    JsxAttributeNameData, JsxAttributeValueData, JsxElement, JsxElementName, JsxElementNameData,
    JsxExpressionContainer, JsxExpressionData, JsxFragment, JsxMemberExpression,
    JsxMemberExpressionObjectData, JsxNamespacedName, JsxOpeningElement, Span, StringLiteral,
};

use super::{ExpressionContext, ExpressionPosition, Formatter, ParenthesesOwner, TailOwner};
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::generics::TypeListKind;
use crate::{FormatError, LineEnding, QuoteStyle, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_jsx_expression(
        &mut self,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
        content: impl FnOnce(&mut Self, &mut Writer<'ast>) -> Result<(), FormatError>,
    ) -> Result<(), FormatError> {
        if context.parentheses_owner == ParenthesesOwner::Parent
            || !matches!(
                context.position,
                ExpressionPosition::AssignmentRight(_)
                    | ExpressionPosition::ArrowBody
                    | ExpressionPosition::RestrictedArgument
                    | ExpressionPosition::YieldArgument
                    | ExpressionPosition::PropertyValue
                    | ExpressionPosition::ExportDefault
            )
        {
            return content(self, writer);
        }
        writer.group(false, |writer| {
            writer.if_break("(");
            writer.indent(|writer| {
                writer.soft_line();
                content(self, writer)
            })?;
            writer.soft_line();
            writer.if_break(")");
            Ok(())
        })
    }

    pub(super) fn format_jsx_element(
        &mut self,
        node: JsxElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let opening = node.opening_element(ast);
        self.format_jsx_opening(opening, writer)?;
        if opening.self_closing(ast) {
            if !node.children(ast).is_empty() || node.closing_element(ast).is_some() {
                return Err(self.unsupported("self-closing JSX children", node));
            }
            return Ok(());
        }
        let closing = node
            .closing_element(ast)
            .ok_or_else(|| self.unsupported("JSX closing element", node))?;
        self.format_jsx_children(node.children(ast), writer)?;
        let name = closing.name(ast);
        self.format_jsx_closing_prefix(closing.span(ast), name.span(ast).start, writer)?;
        self.format_jsx_name(name, writer)?;
        self.format_source_token_gap(name.span(ast).end, closing.span(ast).end - 1, false, writer)?;
        writer.text(">");
        Ok(())
    }

    pub(super) fn format_jsx_fragment(
        &mut self,
        node: JsxFragment,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let opening = node.opening_fragment(ast).span(ast);
        let closing = node.closing_fragment(ast).span(ast);
        writer.text("<");
        self.format_source_token_gap(opening.start + 1, opening.end - 1, false, writer)?;
        writer.text(">");
        self.format_jsx_children(node.children(ast), writer)?;
        self.format_jsx_closing_prefix(closing, closing.end - 1, writer)?;
        writer.text(">");
        Ok(())
    }

    fn format_jsx_closing_prefix(
        &mut self,
        closing: Span,
        content_start: u32,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let prefix = Span::new(closing.start + 1, content_start);
        let slash = self
            .find_token(prefix, self.comments.in_range(prefix), "/")
            .ok_or_else(|| FormatError::unsupported("JSX closing slash", None, prefix))?;
        writer.text("<");
        self.format_source_token_gap(prefix.start, slash, false, writer)?;
        writer.text("/");
        self.format_source_token_gap(slash + 1, prefix.end, false, writer)
    }

    fn format_jsx_opening(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let name = node.name(ast);
        let name_span = name.span(ast);
        let arguments = node.type_arguments(ast).map(|arguments| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(name_span.end, arguments.span(ast).start),
                TypeListKind::Arguments,
            );
            (arguments, prefix)
        });
        let attributes = node.attributes(ast);
        let closing = node.span(ast).end - if node.self_closing(ast) { 2 } else { 1 };
        writer.group(false, |writer| {
            writer.text("<");
            self.format_source_token_gap(node.span(ast).start + 1, name_span.start, false, writer)?;
            if arguments
                .as_ref()
                .is_some_and(|(_, prefix)| prefix.left_mode == ContentMode::Verbatim)
            {
                self.format_verbatim(name_span, writer)?;
            } else {
                self.format_jsx_name(name, writer)?;
            }
            let mut end = name_span.end;
            if let Some((arguments, prefix)) = arguments {
                let mode = prefix.right_mode;
                self.format_type_list_prefix(prefix, writer)?;
                self.format_type_arguments(arguments, mode, writer)?;
                end = arguments.span(ast).end;
            }
            if attributes.is_empty() {
                self.format_source_token_gap(end, closing, false, writer)?;
                if node.self_closing(ast) {
                    writer.text(" />");
                } else {
                    writer.text(">");
                }
                return Ok(());
            }
            let closing_gap = writer.indent(|writer| {
                for id in attributes.iter() {
                    let attribute = ast.get_node_in_sub_range(id);
                    let span = attribute.span(ast);
                    let comments = self.comments.between(Span::new(end, span.start))?;
                    let first = comments
                        .first()
                        .map_or(span.start, |comment| comment.span.start);
                    self.line_constraint(end, first).write(
                        writer,
                        FlatSeparator::Space,
                        Writer::line,
                    );
                    let mode = self.content_mode(comments);
                    self.comments.claim(comments)?;
                    self.format_comment_run(
                        comments,
                        CommentRunLayout::GroupedLeading(span.start),
                        writer,
                    )?;
                    if mode == ContentMode::Verbatim {
                        self.format_verbatim(span, writer)?;
                    } else {
                        match ast.jsx_attribute_item_data(attribute) {
                            JsxAttributeItemData::JsxAttribute(attribute) => {
                                self.format_jsx_attribute(attribute, writer)?
                            }
                            JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                                self.format_jsx_braced_expression(
                                    spread.argument(ast),
                                    span,
                                    true,
                                    writer,
                                )?;
                            }
                        }
                    }
                    end = span.end;
                }
                let comments = self.comments.between(Span::new(end, closing))?;
                let boundary = self.prepare_comment_boundary(comments, end, closing);
                self.format_comment_boundary(&boundary, writer)?;
                Ok::<_, FormatError>(boundary.gap())
            })?;
            closing_gap.write(writer, FlatSeparator::Empty, |writer| {
                if node.self_closing(ast) {
                    writer.line();
                } else if !self.options.bracket_same_line {
                    writer.soft_line();
                }
            });
            writer.text(if node.self_closing(ast) { "/>" } else { ">" });
            Ok(())
        })
    }

    fn format_jsx_name(
        &mut self,
        name: JsxElementName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_element_name_data(name) {
            JsxElementNameData::JsxIdentifier(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)))
            }
            JsxElementNameData::JsxNamespacedName(name) => {
                self.format_jsx_namespace(name, writer)?
            }
            JsxElementNameData::JsxMemberExpression(member) => {
                self.format_jsx_member(member, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_member(
        &mut self,
        member: JsxMemberExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let object = member.object(ast);
        match ast.jsx_member_expression_object_data(object) {
            JsxMemberExpressionObjectData::JsxIdentifier(identifier) => {
                writer.text(ast.get_utf8(identifier.name(ast)))
            }
            JsxMemberExpressionObjectData::JsxMemberExpression(member) => {
                self.format_jsx_member(member, writer)?
            }
        }
        let property = member.property(ast);
        let gap = Span::new(object.span(ast).end, property.span(ast).start);
        let comments = self.comments.in_range(gap);
        let dot = self
            .find_token(gap, comments, ".")
            .ok_or_else(|| self.unsupported("JSX member dot", member))?;
        self.format_source_token_gap(gap.start, dot, false, writer)?;
        writer.text(".");
        self.format_source_token_gap(dot + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(property.name(ast)));
        Ok(())
    }

    fn format_jsx_namespace(
        &mut self,
        name: JsxNamespacedName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let namespace = name.namespace(ast);
        let local = name.name(ast);
        let gap = Span::new(namespace.span(ast).end, local.span(ast).start);
        let comments = self.comments.in_range(gap);
        let colon = self
            .find_token(gap, comments, ":")
            .ok_or_else(|| self.unsupported("JSX namespace colon", name))?;
        writer.text(ast.get_utf8(namespace.name(ast)));
        self.format_source_token_gap(gap.start, colon, false, writer)?;
        writer.text(":");
        self.format_source_token_gap(colon + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(local.name(ast)));
        Ok(())
    }

    fn format_jsx_attribute_name(
        &mut self,
        name: JsxAttributeName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_attribute_name_data(name) {
            JsxAttributeNameData::JsxIdentifier(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)))
            }
            JsxAttributeNameData::JsxNamespacedName(name) => {
                self.format_jsx_namespace(name, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_attribute(
        &mut self,
        attribute: JsxAttribute,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let name = attribute.name(ast);
        self.format_jsx_attribute_name(name, writer)?;
        let Some(value) = attribute.value(ast) else {
            return Ok(());
        };
        let gap = Span::new(name.span(ast).end, value.span(ast).start);
        let comments = self.comments.in_range(gap);
        let equals = self
            .find_token(gap, comments, "=")
            .ok_or_else(|| self.unsupported("JSX attribute equals", attribute))?;
        self.format_source_token_gap(gap.start, equals, false, writer)?;
        writer.text("=");
        let leading = self.comments.between(Span::new(equals + 1, gap.end))?;
        let mode = self.content_mode(leading);
        self.comments.claim(leading)?;
        self.format_comment_run(leading, CommentRunLayout::FlatLeading(gap.end), writer)?;
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(value.span(ast), writer);
        }
        match ast.jsx_attribute_value_data(value) {
            JsxAttributeValueData::StringLiteral(literal) => {
                self.format_jsx_string(literal, writer)?
            }
            JsxAttributeValueData::JsxExpressionContainer(container) => {
                self.format_jsx_container(container, writer)?
            }
            JsxAttributeValueData::JsxElement(element) => {
                self.format_jsx_element(element, writer)?
            }
            JsxAttributeValueData::JsxFragment(fragment) => {
                self.format_jsx_fragment(fragment, writer)?
            }
        }
        Ok(())
    }

    fn format_jsx_string(
        &self,
        literal: StringLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let raw = self.ast.get_utf8(literal.raw(self.ast));
        let quote = raw.as_bytes().first().copied();
        if !matches!(quote, Some(b'\'' | b'"'))
            || raw.len() < 2
            || raw.as_bytes().last().copied() != quote
        {
            return Err(self.unsupported("JSX quoted attribute", literal));
        }
        writer.literal_text(jsx_attribute_string(
            raw,
            &raw[1..raw.len() - 1],
            self.options.jsx_quote_style,
            self.options.line_ending,
        ));
        Ok(())
    }

    fn format_jsx_container(
        &mut self,
        container: JsxExpressionContainer,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_expression_data(container.expression(self.ast)) {
            JsxExpressionData::Expr(expression) => self.format_jsx_braced_expression(
                expression,
                container.span(self.ast),
                false,
                writer,
            ),
            JsxExpressionData::JsxEmptyExpression(empty) => {
                let span = empty.span(self.ast);
                let comments = self.comments.between(span)?;
                if comments.is_empty() {
                    writer.text("{}");
                    return Ok(());
                }
                self.comments.claim(comments)?;
                writer.group(false, |writer| {
                    writer.text("{");
                    writer.indent(|writer| {
                        writer.soft_line();
                        self.format_comment_run(comments, CommentRunLayout::Lines, writer)
                    })?;
                    if comments
                        .last()
                        .is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    } else {
                        writer.soft_line();
                    }
                    writer.text("}");
                    Ok(())
                })
            }
        }
    }

    fn format_jsx_braced_expression(
        &mut self,
        expression: Expr,
        envelope: Span,
        spread: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let expression_span = expression.span(ast);
        let spread_token = if spread {
            let gap = Span::new(envelope.start + 1, expression_span.start);
            let comments = self.comments.in_range(gap);
            Some(
                self.find_token(gap, comments, "...")
                    .ok_or_else(|| self.unsupported("JSX spread boundary", expression))?,
            )
        } else {
            None
        };
        let opening = spread_token.map_or(envelope.start + 1, |marker| marker + 3);
        let closing = envelope.end - 1;
        if expression_span.start < opening || expression_span.end > closing {
            return Err(self.unsupported("JSX expression containment", expression));
        }
        let leading = self
            .comments
            .in_range(Span::new(opening, expression_span.start));
        let mode = if self.content_mode(leading) == ContentMode::Verbatim
            || self.trailing_content_mode(Span::new(expression_span.end, closing), true)
                == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        writer.group(false, |writer| {
            writer.text("{");
            if let Some(marker) = spread_token {
                self.format_source_token_gap(envelope.start + 1, marker, false, writer)?;
                writer.text("...");
            }
            self.comments.claim(leading)?;
            let (end, line_comment) = writer.indent(|writer| {
                let first = leading
                    .first()
                    .map_or(expression_span.start, |comment| comment.span.start);
                self.line_constraint(opening, first).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
                let tail = self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments: leading,
                        content_start: expression_span.start,
                        layout: CommentRunLayout::GroupedLeading(expression_span.start),
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::JsxExpression)
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?;
                let tail =
                    self.capture_expression_tail(Span::new(expression_span.end, closing), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(expression_span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>((
                    end,
                    last.is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            })?;
            self.line_constraint(end, closing)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if line_comment {
                        writer.hard_line();
                    } else {
                        writer.soft_line();
                    }
                });
            writer.text("}");
            Ok(())
        })
    }
}

fn jsx_attribute_string<'text>(
    raw: &'text str,
    interior: &'text str,
    preferred: QuoteStyle,
    line_ending: LineEnding,
) -> Cow<'text, str> {
    let decoded = if interior.contains("&apos;") || interior.contains("&quot;") {
        Cow::Owned(interior.replace("&apos;", "'").replace("&quot;", "\""))
    } else {
        Cow::Borrowed(interior)
    };
    let single = decoded.bytes().filter(|&byte| byte == b'\'').count();
    let double = decoded.bytes().filter(|&byte| byte == b'"').count();
    let quote = match preferred {
        QuoteStyle::Double if double <= single => '"',
        QuoteStyle::Single if single <= double => '\'',
        QuoteStyle::Double => '\'',
        QuoteStyle::Single => '"',
    };
    let encode_lines = !jsx_attribute_lines_match(&decoded, line_ending);
    if !encode_lines
        && matches!(decoded, Cow::Borrowed(_))
        && raw.starts_with(quote)
        && !decoded.contains(quote)
    {
        return Cow::Borrowed(raw);
    }
    let entity = if quote == '"' { "&quot;" } else { "&apos;" };
    let mut output = String::with_capacity(raw.len());
    output.push(quote);
    for character in decoded.chars() {
        match character {
            character if character == quote => output.push_str(entity),
            '\r' if encode_lines => output.push_str("&#13;"),
            '\n' if encode_lines => output.push_str("&#10;"),
            character => output.push(character),
        }
    }
    output.push(quote);
    Cow::Owned(output)
}

/// A literal document line uses the selected output EOL. Attribute values are
/// literal JSX data, so incompatible physical newlines need JSX entity spelling.
fn jsx_attribute_lines_match(raw: &str, ending: LineEnding) -> bool {
    let mut bytes = raw.bytes().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\r' if bytes.peek() == Some(&b'\n') => {
                bytes.next();
                if ending != LineEnding::CrLf {
                    return false;
                }
            }
            b'\r' if ending != LineEnding::Cr => return false,
            b'\n' if ending != LineEnding::Lf => return false,
            _ => {}
        }
    }
    true
}
