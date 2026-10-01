//! Interpretation of flat formatting instructions.

use super::FormatElement;
use crate::FormatOptions;

pub(super) fn print(elements: &[FormatElement<'_>], options: FormatOptions) -> String {
    let mut output = String::new();
    for element in elements {
        match element {
            FormatElement::Text(text) => output.push_str(text),
            FormatElement::HardLine => output.push_str(options.line_ending.as_str()),
        }
    }
    output
}
