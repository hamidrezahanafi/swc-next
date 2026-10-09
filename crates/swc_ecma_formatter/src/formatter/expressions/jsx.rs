//! JSX names, attributes and brace containers retain their actual source owners.

mod children;

use std::borrow::Cow;

use swc_next_ecma_ast::{
    CommentKind, Expr, ExprData, GetSpan, JsxAttribute, JsxAttributeItemData, JsxAttributeName,
    JsxAttributeNameData, JsxAttributeValueData, JsxClosingElement, JsxElement, JsxElementName,
    JsxElementNameData, JsxExpressionContainer, JsxExpressionData, JsxFragment,
    JsxMemberExpression, JsxMemberExpressionObjectData, JsxNamespacedName, JsxOpeningElement, Span,
    StringLiteral,
};

use super::arrows::ArrowLayout;
use super::parentheses::ExpressionContent;
use super::{
    ExpressionContext, ExpressionPosition, Formatter, OperandIndent, ParenthesesOwner, TailOwner,
};
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::formatter::types::generics::TypeListKind;
use crate::{
    FormatError, LineEnding, QuoteStyle,
    document::{Fragment, Writer},
};

#[derive(Default)]
struct JsxOpeningLayout {
    multiple_attributes: bool,
}

#[derive(Clone, Copy)]
enum JsxBraceRole {
    Attribute,
    Child,
}

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
        let children = node.children(ast);
        if opening.self_closing(ast) {
            if !children.is_empty() || node.closing_element(ast).is_some() {
                return Err(self.unsupported("self-closing JSX children", node));
            }
            return self.format_jsx_opening(opening, writer);
        }
        let closing = node
            .closing_element(ast)
            .ok_or_else(|| self.unsupported("JSX closing element", node))?;
        if children.is_empty() {
            self.format_jsx_opening(opening, writer)?;
            return self.format_jsx_closing(closing, writer);
        }
        let (opening_doc, opening_layout) = self.record_jsx_opening(opening, writer)?;
        let (children_doc, children_layout) =
            writer.record_with_value(|writer| self.format_jsx_children(children, writer))?;
        let expand = children_layout.has_content
            && (opening_layout.multiple_attributes
                || children_layout.contains_tag
                || children_layout.multiple_expressions
                || opening_doc.preferred_breaks()
                || children_doc.preferred_breaks());
        writer.group(expand, |writer| {
            writer.write_fragment(opening_doc);
            writer.indent(|writer| {
                if children_layout.has_content && !children_layout.leading_inline {
                    writer.soft_line();
                }
                children_layout.write(children_doc, writer);
            });
            if children_layout.has_content && !children_layout.trailing_inline {
                writer.soft_line();
            }
            self.format_jsx_closing(closing, writer)
        })
    }

    fn format_jsx_closing(
        &mut self,
        closing: JsxClosingElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
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
        if node.children(ast).is_empty() {
            self.format_jsx_fragment_opening(opening, writer)?;
            self.format_jsx_closing_prefix(closing, closing.end - 1, writer)?;
            writer.text(">");
            return Ok(());
        }
        let (opening_doc, ()) =
            writer.record_with_value(|writer| self.format_jsx_fragment_opening(opening, writer))?;
        let (children_doc, layout) = writer
            .record_with_value(|writer| self.format_jsx_children(node.children(ast), writer))?;
        let expand = layout.has_content
            && (layout.contains_tag
                || layout.multiple_expressions
                || opening_doc.preferred_breaks()
                || children_doc.preferred_breaks());
        writer.group(expand, |writer| {
            writer.write_fragment(opening_doc);
            writer.indent(|writer| {
                if layout.has_content && !layout.leading_inline {
                    writer.soft_line();
                }
                layout.write(children_doc, writer);
            });
            if layout.has_content && !layout.trailing_inline {
                writer.soft_line();
            }
            self.format_jsx_closing_prefix(closing, closing.end - 1, writer)?;
            writer.text(">");
            Ok(())
        })
    }

    fn format_jsx_fragment_opening(
        &mut self,
        opening: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        writer.text("<");
        self.format_source_token_gap(opening.start + 1, opening.end - 1, false, writer)?;
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
        if node.attributes(self.ast).is_empty() {
            self.format_jsx_opening_content(node, writer)?;
            return Ok(());
        }
        let (opening, _) = self.record_jsx_opening(node, writer)?;
        writer.write_fragment(opening);
        Ok(())
    }

    fn record_jsx_opening(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<(Fragment<'ast>, JsxOpeningLayout), FormatError> {
        writer.record_with_value(|writer| {
            writer.group(false, |writer| {
                self.format_jsx_opening_content(node, writer)
            })
        })
    }

    fn format_jsx_opening_content(
        &mut self,
        node: JsxOpeningElement,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxOpeningLayout, FormatError> {
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
        let mut attribute_nodes = attributes.iter().map(|id| {
            let attribute = ast.get_node_in_sub_range(id);
            let data = ast.jsx_attribute_item_data(attribute);
            let string_has_lf = if let JsxAttributeItemData::JsxAttribute(attribute) = data
                && let Some(value) = attribute.value(ast)
                && let JsxAttributeValueData::StringLiteral(literal) =
                    ast.jsx_attribute_value_data(value)
            {
                Some(jsx_attribute_has_lf(ast.get_utf8(literal.raw(ast))))
            } else {
                None
            };
            (attribute, data, string_has_lf)
        });
        let first_attribute = attribute_nodes.next();
        let single_string = attributes.len() == 1
            && first_attribute.is_some_and(|(_, _, string_has_lf)| {
                string_has_lf == Some(false) && self.comments.in_range(node.span(ast)).is_empty()
            });
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
            return Ok(JsxOpeningLayout::default());
        }
        let closing_gap = writer.indent(|writer| {
            let mut after_attribute = false;
            for (attribute, data, string_has_lf) in
                first_attribute.into_iter().chain(attribute_nodes)
            {
                let span = attribute.span(ast);
                let comments = self.comments.between(Span::new(end, span.start))?;
                let first = comments
                    .first()
                    .map_or(span.start, |comment| comment.span.start);
                if single_string {
                    writer.space();
                } else {
                    self.line_constraint(end, first).write(
                        writer,
                        FlatSeparator::Space,
                        |writer| {
                            if after_attribute && self.next_line_is_empty(end, first) {
                                writer.hard_line();
                                writer.hard_line();
                            } else {
                                writer.line();
                            }
                        },
                    );
                }
                if string_has_lf == Some(true) {
                    writer.break_parent();
                }
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
                    match data {
                        JsxAttributeItemData::JsxAttribute(attribute) => {
                            self.format_jsx_attribute(attribute, writer)?
                        }
                        JsxAttributeItemData::JsxSpreadAttribute(spread) => {
                            self.format_jsx_braced_expression(
                                spread.argument(ast),
                                span,
                                true,
                                JsxBraceRole::Attribute,
                                writer,
                            )?;
                        }
                    }
                }
                end = span.end;
                after_attribute = true;
            }
            let comments = self.comments.between(Span::new(end, closing))?;
            let boundary = self.prepare_comment_boundary(comments, end, closing);
            self.format_comment_boundary(&boundary, writer)?;
            Ok::<_, FormatError>(boundary.gap())
        })?;
        closing_gap.write(writer, FlatSeparator::Empty, |writer| {
            if single_string {
                if node.self_closing(ast) {
                    writer.space();
                }
            } else if node.self_closing(ast) {
                writer.line();
            } else if !self.options.bracket_same_line {
                writer.soft_line();
            }
        });
        writer.text(if node.self_closing(ast) { "/>" } else { ">" });
        Ok(JsxOpeningLayout {
            multiple_attributes: attributes.len() > 1,
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
                self.format_jsx_container(container, JsxBraceRole::Attribute, writer)?
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
        role: JsxBraceRole,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match self.ast.jsx_expression_data(container.expression(self.ast)) {
            JsxExpressionData::Expr(expression) => self.format_jsx_braced_expression(
                expression,
                container.span(self.ast),
                false,
                role,
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
        role: JsxBraceRole,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_jsx_braced_expression_prepared(expression, envelope, spread, role, None, writer)
    }

    fn format_jsx_braced_expression_prepared(
        &mut self,
        expression: Expr,
        envelope: Span,
        spread: bool,
        role: JsxBraceRole,
        content: Option<ExpressionContent>,
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
        let mut content = content;
        let (hug, arrow_layout) = if !spread && mode == ContentMode::Formatted {
            let data = content
                .as_ref()
                .map_or_else(|| ast.expr_data(expression), |content| content.data);
            let data = if content.is_none() && matches!(data, ExprData::ParenthesizedExpression(_))
            {
                let prepared = self.expression_content(expression);
                let data = prepared.data;
                content = Some(prepared);
                data
            } else {
                data
            };
            let hug = matches!(
                data,
                ExprData::ObjectExpression(_)
                    | ExprData::ArrayExpression(_)
                    | ExprData::CallExpression(_)
                    | ExprData::Function(_)
                    | ExprData::ArrowFunctionExpression(_)
            ) && self.comments.in_range(envelope).is_empty();
            let arrow_layout = if hug && matches!(data, ExprData::ArrowFunctionExpression(_)) {
                if content.is_none() {
                    content = Some(ExpressionContent {
                        expression,
                        data,
                        parentheses: None,
                    });
                }
                ArrowLayout::JsxExpression
            } else {
                ArrowLayout::Ordinary
            };
            (hug, arrow_layout)
        } else {
            (false, ArrowLayout::Ordinary)
        };
        // Attribute braces own the indent only on this actual non-hug path.
        let operand_indent = if matches!(role, JsxBraceRole::Attribute)
            && mode == ContentMode::Formatted
            && !spread
            && !hug
        {
            OperandIndent::Parent
        } else {
            OperandIndent::Expression
        };
        writer.group(false, |writer| {
            writer.text("{");
            if let Some(marker) = spread_token {
                self.format_source_token_gap(envelope.start + 1, marker, false, writer)?;
                writer.text("...");
            }
            self.comments.claim(leading)?;
            let format_content = |writer: &mut Writer<'ast>| {
                let first = leading
                    .first()
                    .map_or(expression_span.start, |comment| comment.span.start);
                self.line_constraint(opening, first).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if !hug {
                            writer.soft_line();
                        }
                    },
                );
                let head = NodeHead {
                    comments: leading,
                    content_start: expression_span.start,
                    layout: CommentRunLayout::GroupedLeading(expression_span.start),
                    gap_end: None,
                };
                let context = ExpressionContext::at(ExpressionPosition::JsxExpression)
                    .with_content_mode(mode)
                    .with_arrow_layout(arrow_layout)
                    .with_operand_indent(operand_indent)
                    .with_tail_owner(TailOwner::ParentOperation);
                let tail = match content {
                    Some(content) => self.format_prepared_expression_with_leading_comments(
                        expression, content, head, context, writer,
                    ),
                    None => self
                        .format_expression_with_leading_comments(expression, head, context, writer),
                }?;
                let tail =
                    self.capture_expression_tail(Span::new(expression_span.end, closing), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(expression_span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>((
                    end,
                    last.is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            };
            let (end, line_comment) = if hug {
                format_content(writer)
            } else {
                writer.indent(format_content)
            }?;
            self.line_constraint(end, closing)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if line_comment {
                        writer.hard_line();
                    } else if !hug {
                        writer.soft_line();
                    }
                });
            writer.text("}");
            Ok(())
        })
    }
}

/// The lexer keeps JSX attribute entities in its raw value. The emitted LF
/// entity must keep the same opening layout on the next formatting pass.
fn jsx_attribute_has_lf(raw: &str) -> bool {
    raw.contains('\n')
        || raw.split("&#").skip(1).any(|entity| {
            let Some((digits, _)) = entity.split_once(';') else {
                return false;
            };
            let (digits, radix) = digits
                .strip_prefix(['x', 'X'])
                .map_or((digits, 10), |digits| (digits, 16));
            let valid = digits.bytes().all(|byte| {
                if radix == 10 {
                    byte.is_ascii_digit()
                } else {
                    byte.is_ascii_hexdigit()
                }
            });
            valid && u32::from_str_radix(digits, radix) == Ok(10)
        })
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
