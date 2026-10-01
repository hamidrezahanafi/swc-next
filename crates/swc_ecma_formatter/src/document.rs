//! Flat documents containing borrowed text and explicit line breaks.
//!
//! Construction owns a growable instruction buffer. Finishing transfers that
//! buffer to an immutable document; source text remains borrowed from its owner.

mod printer;
#[cfg(test)]
mod tests;
mod text;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;
use text::Text;

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space,
    SoftLine,
    Line,
    HardLine,
    LiteralLine,
    IfBreak(Text<'a>),
    LineSuffix(Text<'a>),
    StartGroup {
        expand: bool,
    },
    EndGroup,
    StartIndent,
    EndIndent,
    StartFill,
    EndFill,
    /// Distance from this opening instruction to its matching EndEntry.
    StartEntry {
        end_offset: usize,
    },
    EndEntry,
}

pub(crate) struct Doc<'a> {
    elements: Vec<FormatElement<'a>>,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(&self.elements, options)
    }
}
