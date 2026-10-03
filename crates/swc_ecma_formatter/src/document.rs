//! Flat documents containing borrowed text and explicit line breaks.
//!
//! Construction owns a growable instruction buffer in the formatting arena.
//! Finishing seals an immutable slice; source text remains borrowed from its owner.

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
    capabilities: FragmentCapabilities,
    conditional_lines: bool,
}

pub(crate) struct RecordedGroup<'a> {
    pub(crate) grouped: Fragment<'a>,
    pub(crate) contents: Fragment<'a>,
}

impl<'a> Fragment<'a> {
    pub(crate) fn preferred_breaks(self) -> bool {
        self.breaks
    }

    pub(crate) fn try_flat(self) -> Option<FlatFragment<'a>> {
        self.capabilities
            .permits_flat()
            .then_some(FlatFragment(self))
    }

    pub(crate) fn try_neutral(self) -> Option<NeutralFragment<'a>> {
        matches!(self.capabilities, FragmentCapabilities::Neutral).then_some(NeutralFragment(self))
    }
}

/// A flat view whose complete scope cannot observe layout mode.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NeutralFragment<'a>(Fragment<'a>);

#[derive(Debug, Clone, Copy)]
enum FragmentCapabilities {
    General,
    FlatEligible,
    Neutral,
}

impl FragmentCapabilities {
    fn from_flat(flat: bool) -> Self {
        if flat {
            Self::FlatEligible
        } else {
            Self::General
        }
    }

    fn permits_flat(self) -> bool {
        !matches!(self, Self::General)
    }

    /// Fresh Writer records contain complete, balanced scopes. Once neutrality
    /// is ruled out, the same iterator resumes the existing flat-only scan.
    fn classify(elements: &[FormatElement<'_>]) -> Self {
        let mut elements = elements.iter().copied();
        while let Some(element) = elements.next() {
            let flat = match element {
                FormatElement::Text(text) => {
                    if !text.is_single_line() {
                        return Self::General;
                    }
                    continue;
                }
                FormatElement::Space
                | FormatElement::StartGroup {
                    expand: false,
                    id: None,
                }
                | FormatElement::EndGroup => continue,
                FormatElement::Fragment(fragment) => match fragment.capabilities {
                    Self::Neutral => continue,
                    Self::General => return Self::General,
                    Self::FlatEligible => true,
                },
                FormatElement::FlatFragment(fragment)
                    if matches!(fragment.0.capabilities, Self::Neutral) =>
                {
                    continue;
                }
                FormatElement::FlatFragment(_) => true,
                element => element.permits_flat(),
            };
            return Self::from_flat(flat && elements.all(FormatElement::permits_flat));
        }
        Self::Neutral
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
            Self::Fragment(fragment) => fragment.capabilities.permits_flat(),
            Self::Alternatives(alternatives) => alternatives.first().capabilities.permits_flat(),
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
    elements: &'a [FormatElement<'a>],
    group_count: usize,
    conditional_lines: bool,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(
            self.elements,
            self.group_count,
            self.conditional_lines,
            options,
        )
    }
}
