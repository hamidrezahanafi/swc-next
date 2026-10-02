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
    SoftLineIfBreak(GroupId),
    Line,
    HardLine,
    BreakParent,
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
    Fragment(Fragment<'a>),
    FlatFragment(FlatFragment<'a>),
    Alternatives(&'a Alternatives<'a>),
}

/// A complete immutable scope, owned by the current formatting call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Fragment<'a> {
    elements: &'a [FormatElement<'a>],
    breaks: bool,
    flat: bool,
    conditional_lines: bool,
}

pub(crate) struct RecordedGroup<'a> {
    pub(crate) grouped: Fragment<'a>,
    pub(crate) contents: Fragment<'a>,
}

impl<'a> Fragment<'a> {
    pub(crate) fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.flat.then_some(FlatFragment(self))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FlatFragment<'a>(Fragment<'a>);

#[derive(Debug)]
struct Alternatives<'a> {
    preferred: &'a [Fragment<'a>],
    fallback: Fragment<'a>,
}

impl<'a> Alternatives<'a> {
    /// Flat views and their eligibility always follow this exact path.
    fn first(&self) -> Fragment<'a> {
        self.preferred.first().copied().unwrap_or(self.fallback)
    }
}

impl FormatElement<'_> {
    fn permits_flat(self) -> bool {
        match self {
            Self::HardLine | Self::LiteralLine | Self::LineSuffix(_) | Self::BreakParent => false,
            Self::Text(text) => text.is_single_line(),
            Self::Fragment(fragment) => fragment.flat,
            Self::Alternatives(alternatives) => alternatives.first().flat,
            _ => true,
        }
    }

    /// Applies a flat view without changing any retained instructions.
    fn flattened(self) -> Option<Self> {
        match self {
            Self::Line => Some(Self::Space),
            Self::IfFlat(text) => Some(Self::Text(text)),
            Self::StartGroup { id: Some(id), .. } => Some(Self::FlatGroup(id)),
            Self::Fragment(fragment) => Some(Self::FlatFragment(FlatFragment(fragment))),
            Self::Alternatives(alternatives) => {
                Some(Self::FlatFragment(FlatFragment(alternatives.first())))
            }
            element
            @ (Self::Text(_) | Self::Space | Self::FlatGroup(_) | Self::FlatFragment(_)) => {
                Some(element)
            }
            _ => None,
        }
    }
}

pub(crate) struct Doc<'a> {
    elements: Vec<FormatElement<'a>>,
    group_count: usize,
    conditional_lines: bool,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(
            &self.elements,
            self.group_count,
            self.conditional_lines,
            options,
        )
    }
}
