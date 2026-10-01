//! Flat documents containing borrowed text and explicit line breaks.
//!
//! Construction owns a growable instruction buffer. Finishing transfers that
//! buffer to an immutable document; source text remains borrowed from its owner.

mod printer;
#[cfg(test)]
mod tests;
mod writer;

pub(crate) use writer::Writer;

use crate::FormatOptions;

#[derive(Debug, Clone, Copy)]
enum FormatElement<'a> {
    Text(&'a str),
    HardLine,
}

pub(crate) struct Doc<'a> {
    elements: Vec<FormatElement<'a>>,
}

impl Doc<'_> {
    pub(crate) fn print(self, options: FormatOptions) -> String {
        printer::print(&self.elements, options)
    }
}
