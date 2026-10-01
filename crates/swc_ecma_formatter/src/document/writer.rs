//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::Allocator;

use super::{Doc, FormatElement, text::Text};

pub(crate) struct Writer<'a> {
    allocator: &'a Allocator,
    elements: Vec<FormatElement<'a>>,
    breaks: usize,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            elements: Vec::new(),
            breaks: 0,
        }
    }

    pub(crate) fn text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
    where
        'text: 'a,
    {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        let text = match text {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => self.allocator.alloc_str(&text),
        };
        self.elements.push(FormatElement::Text(Text::new(text)));
    }

    pub(crate) fn hard_line(&mut self) {
        self.elements.push(FormatElement::HardLine);
        self.breaks += 1;
    }

    pub(crate) fn literal_line(&mut self) {
        self.elements.push(FormatElement::LiteralLine);
        self.breaks += 1;
    }

    pub(crate) fn space(&mut self) {
        self.elements.push(FormatElement::Space);
    }

    pub(crate) fn soft_line(&mut self) {
        self.elements.push(FormatElement::SoftLine);
    }

    pub(crate) fn line(&mut self) {
        self.elements.push(FormatElement::Line);
    }

    pub(crate) fn if_break(&mut self, text: &'a str) {
        self.elements.push(FormatElement::IfBreak(Text::new(text)));
    }

    pub(crate) fn line_suffix(&mut self, text: &'a str) {
        self.elements
            .push(FormatElement::LineSuffix(Text::new(text)));
        self.break_parent();
    }

    pub(crate) fn break_parent(&mut self) {
        self.breaks += 1;
    }

    pub(crate) fn group<T>(&mut self, expand: bool, write: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.elements.len();
        let breaks = self.breaks;
        self.elements.push(FormatElement::StartGroup { expand });
        if expand {
            self.breaks += 1;
        }
        let result = write(self);
        let FormatElement::StartGroup { expand } = &mut self.elements[start] else {
            unreachable!("group scope retains its opening instruction");
        };
        *expand |= self.breaks != breaks;
        self.elements.push(FormatElement::EndGroup);
        result
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
