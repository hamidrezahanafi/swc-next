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
pub(crate) struct GroupId(usize);

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(Text<'a>),
    Space,
    SoftLine,
    Line,
    HardLine,
    LiteralLine,
    IfBreak(Text<'a>),
    IfFlat(Text<'a>),
    LineSuffix(Text<'a>),
    StartGroup {
        expand: bool,
        id: Option<GroupId>,
    },
    EndGroup,
    FlatGroup(GroupId),
    StartIndent,
    StartIndentIfBreak(GroupId),
    EndIndent,
    StartAlign {
        width: usize,
    },
    EndAlign {
        width: usize,
    },
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
    group_count: usize,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(&self.elements, self.group_count, options)
    }
}
