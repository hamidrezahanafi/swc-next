//! Interpretation of flat formatting instructions and bounded fit probes.

use super::{FormatElement, text::Text};
use crate::{FormatOptions, IndentStyle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Flat,
    Break,
}

#[derive(Clone, Copy)]
enum ProbeKind {
    Group,
    Fill,
}

struct FillState {
    flat: bool,
    separator: bool,
    separator_mode: Mode,
}

struct GroupState {
    mode: Mode,
    restore_mode: Mode,
}

struct Output {
    text: String,
    column: usize,
    pending_indent: Indentation,
    pending_space: bool,
}

#[derive(Clone, Copy, Default)]
struct Indentation {
    levels: usize,
    alignment: usize,
    alignments: usize,
}

impl Indentation {
    fn width(self, options: FormatOptions) -> usize {
        self.levels * usize::from(options.indent_width.value()) + self.alignment
    }

    fn indented(mut self, options: FormatOptions) -> Self {
        if options.indent_style == IndentStyle::Tab {
            self.levels += self.alignments;
            self.alignment = 0;
            self.alignments = 0;
        }
        self.levels += 1;
        self
    }
}

pub(super) fn print(
    elements: &[FormatElement<'_>],
    group_count: usize,
    options: FormatOptions,
) -> String {
    let mut output = Output {
        text: String::new(),
        column: 0,
        pending_indent: Indentation::default(),
        pending_space: false,
    };
    let mut suffixes = Vec::new();
    let mut indent = Indentation::default();
    let mut indents = Vec::new();
    let mut modes = Vec::new();
    let mut group_modes = vec![Mode::Flat; group_count];
    let mut mode = Mode::Break;
    let mut probe = Vec::new();
    let mut fills = Vec::new();
    let mut entries = Vec::new();
    for (index, element) in elements.iter().enumerate() {
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
                output.line(Indentation::default(), options);
            }
            FormatElement::IfBreak(text) | FormatElement::IfFlat(text) => {
                let group_mode = modes
                    .last()
                    .map_or(Mode::Break, |group: &GroupState| group.mode);
                let active = match element {
                    FormatElement::IfBreak(_) => group_mode == Mode::Break,
                    _ => group_mode == Mode::Flat,
                };
                if active {
                    output.write(text, options);
                }
            }
            FormatElement::LineSuffix(text) => suffixes.push(text),
            FormatElement::StartGroup { expand, id } => {
                let flat = !expand
                    && (mode == Mode::Flat
                        || fits(
                            &elements[index + 1..],
                            &modes,
                            &mut probe,
                            output.column,
                            output.pending_space,
                            options,
                            ProbeKind::Group,
                        ));
                let group_mode = if flat { Mode::Flat } else { Mode::Break };
                if let Some(id) = id {
                    group_modes[id.0] = group_mode;
                }
                modes.push(GroupState {
                    mode: group_mode,
                    restore_mode: mode,
                });
                mode = group_mode;
            }
            FormatElement::EndGroup => {
                mode = modes
                    .pop()
                    .expect("group scopes must close after opening")
                    .restore_mode;
            }
            FormatElement::FlatGroup(id) => group_modes[id.0] = Mode::Flat,
            FormatElement::StartIndent => {
                indents.push(indent);
                indent = indent.indented(options);
            }
            FormatElement::StartIndentIfBreak(id) => {
                indents.push(indent);
                if group_modes[id.0] == Mode::Break {
                    indent = indent.indented(options);
                }
            }
            FormatElement::EndIndent => {
                indent = indents
                    .pop()
                    .expect("indent scopes must close after opening");
            }
            FormatElement::StartAlign { width } => {
                indent.alignment += width;
                indent.alignments += 1;
            }
            FormatElement::EndAlign { width } => {
                indent.alignment = indent
                    .alignment
                    .checked_sub(width)
                    .expect("alignment scopes must close after opening");
                indent.alignments = indent
                    .alignments
                    .checked_sub(1)
                    .expect("alignment scopes must close after opening");
            }
            FormatElement::StartFill => fills.push(FillState {
                flat: mode == Mode::Flat,
                separator: false,
                separator_mode: Mode::Flat,
            }),
            FormatElement::EndFill => {
                fills.pop().expect("fill scopes must close after opening");
            }
            FormatElement::StartEntry { end_offset } => {
                let end = index + end_offset;
                let fill = fills.last_mut().expect("entries belong to an active fill");
                let entry_mode = if fill.separator {
                    fill.separator = false;
                    fill.separator_mode
                } else {
                    fill.separator = true;
                    if fill.flat {
                        Mode::Flat
                    } else {
                        let separator_start = end + 1;
                        let next_end = match elements.get(separator_start) {
                            Some(FormatElement::StartEntry { end_offset }) => {
                                let separator_end = separator_start + end_offset;
                                let content_start = separator_end + 1;
                                match elements.get(content_start) {
                                    Some(FormatElement::StartEntry { end_offset }) => {
                                        content_start + end_offset
                                    }
                                    _ => separator_end,
                                }
                            }
                            _ => end,
                        };
                        let pair_fits = fits(
                            &elements[index + 1..next_end],
                            &modes,
                            &mut probe,
                            output.column,
                            output.pending_space,
                            options,
                            ProbeKind::Fill,
                        );
                        if pair_fits {
                            fill.separator_mode = Mode::Flat;
                            Mode::Flat
                        } else {
                            fill.separator_mode = Mode::Break;
                            if next_end != end
                                && fits(
                                    &elements[index + 1..end],
                                    &modes,
                                    &mut probe,
                                    output.column,
                                    output.pending_space,
                                    options,
                                    ProbeKind::Fill,
                                )
                            {
                                Mode::Flat
                            } else {
                                Mode::Break
                            }
                        }
                    }
                };
                entries.push(mode);
                mode = entry_mode;
            }
            FormatElement::EndEntry => {
                mode = entries
                    .pop()
                    .expect("fill entries must close after opening");
            }
        }
    }
    flush_suffixes(&mut suffixes, &mut output, options);
    debug_assert_eq!(indent.levels, 0, "document must close every indent scope");
    debug_assert!(indents.is_empty(), "document must close every indent scope");
    debug_assert_eq!(
        indent.alignment, 0,
        "document must close every alignment scope"
    );
    debug_assert!(modes.is_empty(), "document must close every group scope");
    debug_assert!(fills.is_empty(), "document must close every fill scope");
    debug_assert!(entries.is_empty(), "document must close every fill entry");
    output.text
}

impl Output {
    fn write(&mut self, text: Text<'_>, options: FormatOptions) {
        let (character, count) = match options.indent_style {
            IndentStyle::Tab => ('\t', self.pending_indent.levels),
            IndentStyle::Space => (
                ' ',
                self.pending_indent.levels * usize::from(options.indent_width.value()),
            ),
        };
        self.text.extend(std::iter::repeat_n(character, count));
        self.text
            .extend(std::iter::repeat_n(' ', self.pending_indent.alignment));
        if self.pending_space {
            self.text.push(' ');
            self.column += 1;
        }
        self.text.push_str(text.source);
        self.column = text.last_line(self.column, options.indent_width.value());
        self.pending_indent = Indentation::default();
        self.pending_space = false;
    }

    fn line(&mut self, indent: Indentation, options: FormatOptions) {
        self.text.push_str(options.line_ending.as_str());
        self.column = indent.width(options);
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
    parents: &[GroupState],
    modes: &mut Vec<Mode>,
    mut column: usize,
    mut pending_space: bool,
    options: FormatOptions,
    kind: ProbeKind,
) -> bool {
    modes.clear();
    if matches!(kind, ProbeKind::Group) {
        modes.push(Mode::Flat);
    }
    let mut parent_len = parents.len();
    let mut mode = Mode::Flat;
    for element in elements {
        if column > usize::from(options.line_width) {
            return false;
        }
        match *element {
            FormatElement::Text(text)
            | FormatElement::IfBreak(text)
            | FormatElement::IfFlat(text) => {
                if matches!(
                    element,
                    FormatElement::IfBreak(_) | FormatElement::IfFlat(_)
                ) {
                    let group_mode = modes
                        .last()
                        .copied()
                        .or_else(|| parents[..parent_len].last().map(|group| group.mode))
                        .unwrap_or(Mode::Break);
                    let active = match element {
                        FormatElement::IfBreak(_) => group_mode == Mode::Break,
                        _ => group_mode == Mode::Flat,
                    };
                    if !active {
                        continue;
                    }
                }
                column += usize::from(pending_space);
                pending_space = false;
                let (next_column, multiline) =
                    text.first_line(column, options.indent_width.value());
                column = next_column;
                if multiline {
                    return column <= usize::from(options.line_width);
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
            FormatElement::StartGroup { expand, .. } => {
                if expand && matches!(kind, ProbeKind::Fill) {
                    return false;
                }
                mode = if expand { Mode::Break } else { mode };
                modes.push(mode);
            }
            FormatElement::EndGroup => {
                if modes.pop().is_none() {
                    parent_len = parent_len
                        .checked_sub(1)
                        .expect("fit probe closes an active group scope");
                }
                mode = if matches!(kind, ProbeKind::Fill) && modes.is_empty() {
                    Mode::Flat
                } else {
                    modes
                        .last()
                        .copied()
                        .or_else(|| parents[..parent_len].last().map(|group| group.mode))
                        .unwrap_or(Mode::Break)
                };
            }
            FormatElement::StartIndent
            | FormatElement::FlatGroup(_)
            | FormatElement::StartIndentIfBreak(_)
            | FormatElement::EndIndent
            | FormatElement::StartAlign { .. }
            | FormatElement::EndAlign { .. }
            | FormatElement::StartFill
            | FormatElement::EndFill
            | FormatElement::StartEntry { .. }
            | FormatElement::EndEntry => {}
        }
    }
    column <= usize::from(options.line_width)
}
