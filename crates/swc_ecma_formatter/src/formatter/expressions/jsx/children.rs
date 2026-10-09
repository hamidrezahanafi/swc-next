//! One forward JSX sibling stream prepares borrowed words and their boundaries.

use std::borrow::Cow;

use swc_next_ecma_ast::{
    Expr, ExprData, GetSpan, JsxChild, JsxChildData, JsxExpressionContainer, JsxExpressionData,
    SubRange,
};

use super::super::parentheses::ExpressionContent;
use super::{Formatter, JsxBraceRole};
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, QuoteStyle,
    document::{Fragment, Writer},
};

#[derive(Default)]
pub(super) struct JsxChildrenLayout {
    pub(super) has_content: bool,
    pub(super) contains_tag: bool,
    pub(super) multiple_expressions: bool,
    pub(super) leading_inline: bool,
    pub(super) trailing_inline: bool,
    grouped_text: bool,
    leading_space: bool,
    trailing_space: bool,
    only_space: bool,
    single_quote: bool,
}

impl JsxChildrenLayout {
    pub(super) fn write<'ast>(&self, content: Fragment<'ast>, writer: &mut Writer<'ast>) {
        if self.only_space {
            write_jsx_space(self.single_quote, false, writer);
            return;
        }
        let write = |writer: &mut Writer<'ast>| {
            if self.leading_space {
                write_jsx_space(self.single_quote, true, writer);
            }
            writer.write_fragment(content);
            if self.trailing_space {
                write_jsx_space(self.single_quote, false, writer);
            }
        };
        if self.grouped_text {
            writer.group(false, write);
        } else {
            write(writer);
        }
    }
}

#[derive(Clone, Copy)]
enum JsxSeparator {
    Words,
    Space,
    Newline { empty_line: bool },
}

#[derive(Clone, Copy)]
enum JsxContentEdge {
    Word { single_character: bool },
    NonText { self_closing: bool },
    Literal,
}

#[derive(Default)]
struct JsxChildrenStream {
    layout: JsxChildrenLayout,
    previous: Option<JsxContentEdge>,
    pending: Option<JsxSeparator>,
    contains_text: bool,
    seen_expression: bool,
    has_empty_line: bool,
}

impl JsxChildrenStream {
    fn separator(&mut self, separator: JsxSeparator) {
        // Canonical space supersedes adjacent physical formatting newlines.
        if !matches!(self.pending, Some(JsxSeparator::Space)) {
            self.pending = Some(separator);
        }
        self.contains_text |= matches!(separator, JsxSeparator::Space);
    }

    fn before(&mut self, next: JsxContentEdge, writer: &mut Writer<'_>) {
        let pending = self.pending.take();
        let Some(previous) = self.previous else {
            self.layout.leading_space = matches!(pending, Some(JsxSeparator::Space));
            self.layout.leading_inline =
                matches!(next, JsxContentEdge::Literal) && !self.layout.leading_space;
            return;
        };
        writer.fill_entry(|writer| match pending {
            Some(JsxSeparator::Words) => writer.line(),
            Some(JsxSeparator::Space) => self.write_space(writer),
            Some(JsxSeparator::Newline { empty_line }) => {
                writer.hard_line();
                if empty_line {
                    writer.hard_line();
                    self.has_empty_line = true;
                }
            }
            None => match (previous, next) {
                (JsxContentEdge::Literal, _) | (_, JsxContentEdge::Literal) => {}
                (JsxContentEdge::NonText { .. }, JsxContentEdge::NonText { .. }) => {
                    writer.hard_line();
                }
                (
                    JsxContentEdge::Word { single_character },
                    JsxContentEdge::NonText { self_closing },
                )
                | (
                    JsxContentEdge::NonText { self_closing },
                    JsxContentEdge::Word { single_character },
                ) => {
                    if self_closing && !single_character {
                        writer.hard_line();
                    } else {
                        writer.soft_line();
                    }
                }
                (JsxContentEdge::Word { .. }, JsxContentEdge::Word { .. }) => {}
            },
        });
    }

    /// Fill chooses this separator's mode independently of surrounding groups.
    fn write_space(&self, writer: &mut Writer<'_>) {
        writer.if_current_break(if self.layout.single_quote {
            "{' '}"
        } else {
            "{\" \"}"
        });
        writer.if_current_flat(" ");
        writer.soft_line();
    }

    fn after(&mut self, edge: JsxContentEdge) {
        self.previous = Some(edge);
        self.layout.has_content = true;
        self.layout.trailing_inline = matches!(edge, JsxContentEdge::Literal);
    }

    fn finish(mut self) -> JsxChildrenLayout {
        if matches!(self.pending, Some(JsxSeparator::Space)) {
            if self.previous.is_none() {
                self.layout.only_space = true;
                self.layout.has_content = true;
            } else {
                self.layout.trailing_space = true;
                self.layout.trailing_inline = false;
            }
        }
        self.layout.grouped_text = self.contains_text
            && !self.layout.leading_space
            && !self.layout.trailing_space
            && !self.layout.only_space;
        self.layout
    }
}

fn write_jsx_space<'ast>(single_quote: bool, following_line: bool, writer: &mut Writer<'ast>) {
    writer.if_break(if single_quote { "{' '}" } else { "{\" \"}" });
    writer.if_flat(" ");
    if following_line {
        writer.soft_line();
    }
}

/// Every slice still borrows the original JSXText. Only the four JSX formatting
/// characters separate words; Unicode whitespace is never Rust trim whitespace.
struct JsxTextChunks<'ast> {
    remaining: &'ast str,
}

enum JsxTextChunk<'ast> {
    Word(&'ast str),
    Whitespace { line_breaks: u8, has_lf: bool },
}

impl<'ast> Iterator for JsxTextChunks<'ast> {
    type Item = JsxTextChunk<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = *self.remaining.as_bytes().first()?;
        let whitespace = is_jsx_whitespace(first);
        let mut end = 0;
        let mut line_breaks = 0_u8;
        let mut has_lf = false;
        let mut previous_cr = false;
        for byte in self.remaining.bytes() {
            if is_jsx_whitespace(byte) != whitespace {
                break;
            }
            end += 1;
            if byte == b'\r' || byte == b'\n' && !previous_cr {
                line_breaks = (line_breaks + 1).min(2);
            }
            has_lf |= byte == b'\n';
            previous_cr = byte == b'\r';
        }
        // A boundary is an ASCII formatting character, so it cannot split UTF-8.
        let (chunk, remaining) = self.remaining.split_at(end);
        self.remaining = remaining;
        Some(if whitespace {
            JsxTextChunk::Whitespace {
                line_breaks,
                has_lf,
            }
        } else {
            JsxTextChunk::Word(chunk)
        })
    }
}

fn is_jsx_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

/// Child eligibility and its eventual printer share the actual prepared core.
struct JsxChildExpression {
    original: Expr,
    content: Option<ExpressionContent>,
    space: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_jsx_children(
        &mut self,
        children: SubRange<JsxChild>,
        writer: &mut Writer<'ast>,
    ) -> Result<JsxChildrenLayout, FormatError> {
        let ast = self.ast;
        let mut stream = JsxChildrenStream::default();
        stream.layout.single_quote = self.options.quote_style == QuoteStyle::Single;
        let mut previous_ignore = false;
        writer.fill_with_empty_line_policy(|writer| {
            for id in children.iter() {
                let child = ast.get_node_in_sub_range(id);
                let data = ast.jsx_child_data(child);
                let expression = match data {
                    JsxChildData::JsxExpressionContainer(container) => {
                        self.prepare_jsx_child_expression(container)
                    }
                    _ => None,
                };
                if expression
                    .as_ref()
                    .is_some_and(|expression| expression.space)
                {
                    previous_ignore = false;
                    stream.separator(JsxSeparator::Space);
                    continue;
                }
                if let JsxChildData::JsxText(text) = data {
                    let raw = ast.get_utf8(text.value(ast));
                    // Ignore uses the original sibling, not canonical spelling.
                    if previous_ignore && jsx_text_is_meaningful(raw) {
                        previous_ignore = false;
                    }
                    if let Some(literal) = jsx_literal_text(raw) {
                        stream.before(JsxContentEdge::Literal, writer);
                        writer.fill_entry(|writer| writer.literal_text(literal));
                        stream.after(JsxContentEdge::Literal);
                        stream.contains_text = true;
                        continue;
                    }
                    let mut chunks = JsxTextChunks { remaining: raw };
                    let mut first = true;
                    while let Some(chunk) = chunks.next() {
                        match chunk {
                            JsxTextChunk::Word(word) => {
                                let mut characters = word.chars();
                                let single_character =
                                    characters.next().is_some() && characters.next().is_none();
                                let edge = JsxContentEdge::Word { single_character };
                                stream.before(edge, writer);
                                writer.fill_entry(|writer| writer.text(word));
                                stream.after(edge);
                                stream.contains_text = true;
                            }
                            JsxTextChunk::Whitespace {
                                line_breaks,
                                has_lf,
                            } => {
                                stream.contains_text |= !has_lf;
                                let separator = if first || chunks.remaining.is_empty() {
                                    if line_breaks == 0 {
                                        JsxSeparator::Space
                                    } else {
                                        JsxSeparator::Newline {
                                            empty_line: line_breaks == 2,
                                        }
                                    }
                                } else {
                                    JsxSeparator::Words
                                };
                                stream.separator(separator);
                            }
                        }
                        first = false;
                    }
                    continue;
                }
                let self_closing = match data {
                    JsxChildData::JsxElement(element) => {
                        stream.layout.contains_tag = true;
                        element.opening_element(ast).self_closing(ast)
                    }
                    JsxChildData::JsxFragment(_) => {
                        stream.layout.contains_tag = true;
                        false
                    }
                    JsxChildData::JsxExpressionContainer(_) => {
                        stream.layout.multiple_expressions |= stream.seen_expression;
                        stream.seen_expression = true;
                        false
                    }
                    _ => false,
                };
                let edge = JsxContentEdge::NonText { self_closing };
                stream.before(edge, writer);
                let ignored = previous_ignore
                    && matches!(
                        data,
                        JsxChildData::JsxElement(_) | JsxChildData::JsxFragment(_)
                    );
                let next_ignore = if self.has_suppression {
                    match data {
                        JsxChildData::JsxExpressionContainer(container) => {
                            matches!(
                                ast.jsx_expression_data(container.expression(ast)),
                                JsxExpressionData::JsxEmptyExpression(_)
                            ) && self.content_mode(self.comments.in_range(container.span(ast)))
                                == ContentMode::Verbatim
                        }
                        _ => false,
                    }
                } else {
                    false
                };
                writer.fill_entry(|writer| {
                    if ignored {
                        return self.format_verbatim(child.span(ast), writer);
                    }
                    match data {
                        JsxChildData::JsxText(_) => {
                            unreachable!("text siblings are consumed before child dispatch")
                        }
                        JsxChildData::JsxExpressionContainer(container) => {
                            if let Some(expression) = expression {
                                self.format_jsx_braced_expression_prepared(
                                    expression.original,
                                    container.span(ast),
                                    false,
                                    JsxBraceRole::Child,
                                    expression.content,
                                    writer,
                                )
                            } else {
                                self.format_jsx_container(container, JsxBraceRole::Child, writer)
                            }
                        }
                        JsxChildData::JsxSpreadChild(spread) => self.format_jsx_braced_expression(
                            spread.expression(ast),
                            spread.span(ast),
                            true,
                            JsxBraceRole::Child,
                            writer,
                        ),
                        JsxChildData::JsxElement(element) => {
                            self.format_jsx_element(element, writer)
                        }
                        JsxChildData::JsxFragment(fragment) => {
                            self.format_jsx_fragment(fragment, writer)
                        }
                    }
                })?;
                previous_ignore = next_ignore;
                stream.after(edge);
            }
            let collapse_empty_lines = stream.has_empty_line && stream.contains_text;
            Ok((stream.finish(), collapse_empty_lines))
        })
    }

    fn prepare_jsx_child_expression(
        &self,
        container: JsxExpressionContainer,
    ) -> Option<JsxChildExpression> {
        let JsxExpressionData::Expr(original) =
            self.ast.jsx_expression_data(container.expression(self.ast))
        else {
            return None;
        };
        let data = self.ast.expr_data(original);
        let content = matches!(data, ExprData::ParenthesizedExpression(_))
            .then(|| self.expression_content(original));
        let printed = content.as_ref().map_or(data, |content| content.data);
        let space = matches!(
            printed,
            ExprData::StringLiteral(literal)
                if self.ast.get_wtf8(literal.value(self.ast)).as_str() == Some(" ")
        ) && self.comments.in_range(container.span(self.ast)).is_empty();
        Some(JsxChildExpression {
            original,
            content,
            space,
        })
    }
}

/// Ambiguous Unicode text stays borrowed. Only possible TAB inputs need the
/// existing common physical-edge fold; surviving TABs cannot gain trim edges.
fn jsx_literal_text(raw: &str) -> Option<Cow<'_, str>> {
    if jsx_text_needs_literal(raw) {
        return Some(Cow::Borrowed(raw));
    }
    if !raw.contains('\t') {
        return None;
    }
    let text = jsx_semantic_text(raw);
    text.contains('\t').then_some(text)
}

/// Prettier's sibling-ignore predicate only skips LF-containing formatting
/// whitespace. Other Unicode whitespace and same-line blanks remain barriers.
fn jsx_text_is_meaningful(raw: &str) -> bool {
    !raw.contains('\n') || !raw.bytes().all(is_jsx_whitespace)
}

/// Common physical lines trim ASCII edges before entity decoding.
fn jsx_semantic_text(raw: &str) -> Cow<'_, str> {
    if !raw.contains(['\r', '\n']) {
        return Cow::Borrowed(raw);
    }
    let mut output = String::with_capacity(raw.len());
    let mut lines = raw.split(['\r', '\n']).enumerate().peekable();
    while let Some((index, line)) = lines.next() {
        let line = if index == 0 {
            line
        } else {
            line.trim_start_matches([' ', '\t'])
        };
        let line = if lines.peek().is_none() {
            line
        } else {
            line.trim_end_matches([' ', '\t'])
        };
        if line.is_empty() {
            continue;
        }
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(line);
    }
    Cow::Owned(output)
}

fn jsx_text_needs_literal(raw: &str) -> bool {
    raw.chars().any(|character| {
        matches!(character, '\u{b}' | '\u{c}' | '\u{200b}' | '\u{feff}')
            || !character.is_ascii() && character.is_whitespace()
    })
}
