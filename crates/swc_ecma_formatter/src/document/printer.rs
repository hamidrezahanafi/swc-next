//! Interpretation of flat formatting instructions and bounded fit probes.

use super::{FormatElement, text::Text};
use crate::{FormatOptions, IndentStyle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
}

struct Output {
    text: String,
    column: usize,
    pending_indent: usize,
    pending_space: bool,
}

pub(super) fn print(elements: &[FormatElement<'_>], options: FormatOptions) -> String {
    let mut output = Output {
        text: String::new(),
        column: 0,
        pending_indent: 0,
        pending_space: false,
    };
    let mut suffixes = Vec::new();
    let mut indent = 0_usize;
    let mut modes = Vec::new();
    let mut probe = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let mode = modes.last().copied().unwrap_or(Mode::Break);
        match *element {
            FormatElement::Text(text) => output.write(text, options),
            FormatElement::Space => output.pending_space = true,
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    output.pending_space = true;
                }
            }
            FormatElement::SoftLine | FormatElement::Line | FormatElement::HardLine => {
                flush_suffixes(&mut suffixes, &mut output, options);
                output.line(indent, options);
            }
            FormatElement::LiteralLine => {
                flush_suffixes(&mut suffixes, &mut output, options);
                output.line(0, options);
            }
            FormatElement::IfBreak(text) if mode == Mode::Break => output.write(text, options),
            FormatElement::IfBreak(_) => {}
            FormatElement::LineSuffix(text) => suffixes.push(text),
            FormatElement::StartGroup { expand } => {
                let flat = !expand
                    && (mode == Mode::Flat
                        || fits(
                            &elements[index + 1..],
                            &modes,
                            &mut probe,
                            output.column,
                            output.pending_space,
                            options,
                        ));
                modes.push(if flat { Mode::Flat } else { Mode::Break });
            }
            FormatElement::EndGroup => {
                modes.pop().expect("group scopes must close after opening");
            }
            FormatElement::StartIndent => indent += 1,
            FormatElement::EndIndent => {
                indent = indent
                    .checked_sub(1)
                    .expect("indent scopes must close after opening");
            }
        }
    }
    flush_suffixes(&mut suffixes, &mut output, options);
    debug_assert_eq!(indent, 0, "document must close every indent scope");
    debug_assert!(modes.is_empty(), "document must close every group scope");
    output.text
}

impl Output {
    fn write(&mut self, text: Text<'_>, options: FormatOptions) {
        let (character, count) = match options.indent_style {
            IndentStyle::Tab => ('\t', self.pending_indent),
            IndentStyle::Space => (
                ' ',
                self.pending_indent * usize::from(options.indent_width.value()),
            ),
        };
        self.text.extend(std::iter::repeat_n(character, count));
        if self.pending_space {
            self.text.push(' ');
            self.column += 1;
        }
        self.text.push_str(text.source);
        self.column = text.last_line(self.column, options.indent_width.value());
        self.pending_indent = 0;
        self.pending_space = false;
    }

    fn line(&mut self, indent: usize, options: FormatOptions) {
        self.text.push_str(options.line_ending.as_str());
        self.column = indent * usize::from(options.indent_width.value());
        self.pending_indent = indent;
        self.pending_space = false;
    }
}

fn flush_suffixes(suffixes: &mut Vec<Text<'_>>, output: &mut Output, options: FormatOptions) {
    for text in suffixes.drain(..) {
        output.pending_space = true;
        output.write(text, options);
    }
}

fn fits(
    elements: &[FormatElement<'_>],
    parents: &[Mode],
    modes: &mut Vec<Mode>,
    mut column: usize,
    mut pending_space: bool,
    options: FormatOptions,
) -> bool {
    modes.clear();
    modes.push(Mode::Flat);
    let mut parent_len = parents.len();
    for element in elements {
        if column > usize::from(options.line_width) {
            return false;
        }
        let mode = modes
            .last()
            .or_else(|| parents[..parent_len].last())
            .copied()
            .unwrap_or(Mode::Break);
        match *element {
            FormatElement::Text(text) | FormatElement::IfBreak(text) => {
                if matches!(element, FormatElement::Text(_)) || mode == Mode::Break {
                    column += usize::from(pending_space);
                    pending_space = false;
                    let (next_column, multiline) =
                        text.first_line(column, options.indent_width.value());
                    column = next_column;
                    if multiline {
                        return column <= usize::from(options.line_width);
                    }
                }
            }
            FormatElement::LineSuffix(_) => {}
            FormatElement::Space => pending_space = true,
            FormatElement::SoftLine | FormatElement::Line if mode == Mode::Flat => {
                if matches!(element, FormatElement::Line) {
                    pending_space = true;
                }
            }
            FormatElement::SoftLine
            | FormatElement::Line
            | FormatElement::HardLine
            | FormatElement::LiteralLine => return true,
            FormatElement::StartGroup { expand } => {
                modes.push(if expand { Mode::Break } else { mode })
            }
            FormatElement::EndGroup => {
                if modes.pop().is_none() {
                    parent_len = parent_len
                        .checked_sub(1)
                        .expect("fit probe closes an active group scope");
                }
            }
            FormatElement::StartIndent | FormatElement::EndIndent => {}
        }
    }
    column <= usize::from(options.line_width)
}
