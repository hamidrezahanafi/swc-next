//! Direct instruction construction with borrowed source text.

use super::{Doc, FormatElement};

pub(crate) struct Writer<'a> {
    elements: Vec<FormatElement<'a>>,
}

impl<'a> Writer<'a> {
    pub(crate) fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub(crate) fn text(&mut self, text: &'a str) {
        if !text.is_empty() {
            self.elements.push(FormatElement::Text(text));
        }
    }

    pub(crate) fn hard_line(&mut self) {
        self.elements.push(FormatElement::HardLine);
    }

    pub(crate) fn literal_line(&mut self) {
        self.elements.push(FormatElement::LiteralLine);
    }

    pub(crate) fn space(&mut self) {
        self.elements.push(FormatElement::Space);
    }

    pub(crate) fn indent<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartIndent);
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub(crate) fn finish(self) -> Doc<'a> {
        Doc {
            elements: self.elements,
        }
    }
}
