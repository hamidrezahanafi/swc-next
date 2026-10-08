//! A forward JSX sibling stream keeps semantic text and child-ignore barriers.

use std::borrow::Cow;

use swc_next_ecma_ast::{GetSpan, JsxChild, JsxChildData, JsxExpressionData, SubRange};

use super::Formatter;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_jsx_children(
        &mut self,
        children: SubRange<JsxChild>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if children.is_empty() {
            return Ok(());
        }
        let ast = self.ast;
        // A mixed text sequence owns every boundary space. Only a sequence with
        // no semantic text permits structural newlines between its children.
        let mut contains_text = false;
        let mut child_count = 0;
        let mut has_tag = false;
        let mut source_break = false;
        for id in children.iter() {
            let child = ast.get_node_in_sub_range(id);
            match ast.jsx_child_data(child) {
                JsxChildData::JsxText(text) => {
                    let raw = ast.get_utf8(text.value(ast));
                    contains_text |= jsx_text_has_content(raw);
                    source_break |= raw.contains(['\n', '\r']);
                }
                JsxChildData::JsxElement(_) | JsxChildData::JsxFragment(_) => {
                    has_tag = true;
                    child_count += 1;
                }
                _ => child_count += 1,
            }
        }
        let expand = !contains_text && (source_break || has_tag || child_count > 1);
        writer.group(expand, |writer| {
            writer
                .indent(|writer| {
                    let mut previous_ignore = false;
                    let mut emitted = false;
                    for id in children.iter() {
                        let child = ast.get_node_in_sub_range(id);
                        let data = ast.jsx_child_data(child);
                        if let JsxChildData::JsxText(text) = data {
                            let raw = ast.get_utf8(text.value(ast));
                            // A same-line space is a real sibling and blocks ignore,
                            // even when a future layout pass chooses another spelling.
                            let meaningful = jsx_text_is_meaningful(raw);
                            if meaningful {
                                previous_ignore = false;
                            }
                            let text = jsx_semantic_text(raw);
                            if !text.is_empty() {
                                writer.literal_text(text);
                                emitted = true;
                            }
                            continue;
                        }
                        if !contains_text {
                            writer.soft_line();
                        }
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
                                    ) && self
                                        .content_mode(self.comments.in_range(container.span(ast)))
                                        == ContentMode::Verbatim
                                }
                                _ => false,
                            }
                        } else {
                            false
                        };
                        if ignored {
                            self.format_verbatim(child.span(ast), writer)?;
                        } else {
                            match data {
                                JsxChildData::JsxText(_) => {
                                    unreachable!("text siblings are consumed before child dispatch")
                                }
                                JsxChildData::JsxExpressionContainer(container) => {
                                    self.format_jsx_container(container, writer)?
                                }
                                JsxChildData::JsxSpreadChild(spread) => self
                                    .format_jsx_braced_expression(
                                        spread.expression(ast),
                                        spread.span(ast),
                                        true,
                                        writer,
                                    )?,
                                JsxChildData::JsxElement(element) => {
                                    self.format_jsx_element(element, writer)?
                                }
                                JsxChildData::JsxFragment(fragment) => {
                                    self.format_jsx_fragment(fragment, writer)?
                                }
                            }
                        }
                        previous_ignore = next_ignore;
                        emitted = true;
                    }
                    Ok::<_, FormatError>(emitted)
                })
                .map(|emitted| {
                    if emitted && !contains_text {
                        writer.soft_line();
                    }
                })
        })
    }
}

/// Prettier's sibling-ignore predicate only skips LF-containing formatting
/// whitespace. Other Unicode whitespace and same-line blanks remain barriers.
fn jsx_text_is_meaningful(raw: &str) -> bool {
    !raw.contains('\n')
        || !raw
            .chars()
            .all(|character| matches!(character, ' ' | '\t' | '\r' | '\n'))
}

/// Common JSX physical lines trim ASCII edges before entity decoding. Unicode
/// edges have different consumer rules; their borrowed literal keeps both rules
/// intact instead of flattening them with one consumer's whitespace table.
fn jsx_semantic_text(raw: &str) -> Cow<'_, str> {
    if !raw.contains(['\r', '\n']) || jsx_text_needs_literal(raw) {
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

fn jsx_text_has_content(raw: &str) -> bool {
    if !raw.contains(['\r', '\n']) || jsx_text_needs_literal(raw) {
        return !raw.is_empty();
    }
    raw.chars()
        .any(|character| !matches!(character, ' ' | '\t' | '\r' | '\n'))
}
