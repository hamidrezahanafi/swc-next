//! Interpretation of flat formatting instructions.

use super::FormatElement;
use crate::{FormatOptions, IndentStyle};

pub(super) fn print(elements: &[FormatElement<'_>], options: FormatOptions) -> String {
    let mut output = String::new();
    let mut indent = 0_usize;
    let mut pending_indent = 0;
    let mut pending_space = false;
    for element in elements {
        match element {
            FormatElement::Text(text) => {
                let (character, count) = match options.indent_style {
                    IndentStyle::Tab => ('\t', pending_indent),
                    IndentStyle::Space => (
                        ' ',
                        pending_indent * usize::from(options.indent_width.value()),
                    ),
                };
                output.extend(std::iter::repeat_n(character, count));
                if pending_space {
                    output.push(' ');
                }
                output.push_str(text);
                pending_indent = 0;
                pending_space = false;
            }
            FormatElement::Space => pending_space = true,
            FormatElement::HardLine => {
                output.push_str(options.line_ending.as_str());
                pending_indent = indent;
                pending_space = false;
            }
            FormatElement::LiteralLine => {
                output.push_str(options.line_ending.as_str());
                pending_indent = 0;
                pending_space = false;
            }
            FormatElement::StartIndent => indent += 1,
            FormatElement::EndIndent => {
                indent = indent
                    .checked_sub(1)
                    .expect("indent scopes must close after opening");
            }
        }
    }
    debug_assert_eq!(indent, 0, "document must close every indent scope");
    output
}
