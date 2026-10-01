//! Direct instruction construction with stable borrowed and generated text.

use std::borrow::Cow;

use swc_next_allocator::Allocator;

use super::{Doc, FormatElement, GroupId, text::Text};

pub(crate) struct Writer<'a> {
    allocator: &'a Allocator,
    elements: Vec<FormatElement<'a>>,
    breaks: usize,
    flattening: bool,
    group_count: usize,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            elements: Vec::new(),
            breaks: 0,
            flattening: false,
            group_count: 0,
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

    /// Copies transient text into storage owned by this formatting call.
    pub(crate) fn text_copied(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let text = self.allocator.alloc_str(text);
        self.elements.push(FormatElement::Text(Text::new(text)));
    }

    /// Literal source lines retain their contents without structural indentation.
    pub(crate) fn literal_text<'text>(&mut self, text: impl Into<Cow<'text, str>>)
    where
        'text: 'a,
    {
        let mut text = match text.into() {
            Cow::Borrowed(text) => text,
            Cow::Owned(text) => self.allocator.alloc_str(&text),
        };
        while let Some(index) = text.find(['\r', '\n']) {
            self.text(&text[..index]);
            self.literal_line();
            let crlf =
                text.as_bytes()[index] == b'\r' && text.as_bytes().get(index + 1) == Some(&b'\n');
            text = &text[index + 1 + usize::from(crlf)..];
        }
        self.text(text);
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

    pub(crate) fn if_flat(&mut self, text: &'a str) {
        self.elements.push(FormatElement::IfFlat(Text::new(text)));
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
        self.group_with_id(expand, None, write)
    }

    /// The returned handle refers to a group already written in this document.
    pub(crate) fn named_group(&mut self, expand: bool, write: impl FnOnce(&mut Self)) -> GroupId {
        let id = GroupId(self.group_count);
        self.group_count += 1;
        self.group_with_id(expand, Some(id), write);
        id
    }

    fn group_with_id<T>(
        &mut self,
        expand: bool,
        id: Option<GroupId>,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let start = self.elements.len();
        let breaks = self.breaks;
        self.elements.push(FormatElement::StartGroup { expand, id });
        if expand {
            self.breaks += 1;
        }
        let result = write(self);
        let FormatElement::StartGroup { expand, .. } = &mut self.elements[start] else {
            unreachable!("group scope retains its opening instruction");
        };
        *expand |= self.breaks != breaks;
        self.elements.push(FormatElement::EndGroup);
        result
    }

    /// Removes optional layout breaks in one complete scope. Required source
    /// breaks and line suffixes keep the original document unchanged.
    pub(crate) fn flat<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        if self.flattening {
            return write(self);
        }
        let start = self.elements.len();
        let breaks = self.breaks;
        self.flattening = true;
        let result = write(self);
        self.flattening = false;
        if self.elements[start..].iter().any(|element| match element {
            FormatElement::HardLine | FormatElement::LiteralLine | FormatElement::LineSuffix(_) => {
                true
            }
            FormatElement::Text(text) => text.source.contains(['\n', '\r', '\u{2028}', '\u{2029}']),
            _ => false,
        }) {
            return result;
        }
        let mut output = start;
        for index in start..self.elements.len() {
            let element = match self.elements[index] {
                FormatElement::Line => FormatElement::Space,
                FormatElement::IfFlat(text) => FormatElement::Text(text),
                FormatElement::StartGroup { id: Some(id), .. } => FormatElement::FlatGroup(id),
                element @ FormatElement::FlatGroup(_) => element,
                element @ (FormatElement::Text(_) | FormatElement::Space) => element,
                _ => continue,
            };
            self.elements[output] = element;
            output += 1;
        }
        self.elements.truncate(output);
        self.breaks = breaks;
        result
    }

    pub(crate) fn indent<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartIndent);
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub(crate) fn indent_if_break<T>(
        &mut self,
        group: GroupId,
        write: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.elements.push(FormatElement::StartIndentIfBreak(group));
        let result = write(self);
        self.elements.push(FormatElement::EndIndent);
        result
    }

    pub(crate) fn align<T>(&mut self, width: usize, write: impl FnOnce(&mut Self) -> T) -> T {
        if width == 0 {
            return write(self);
        }
        self.elements.push(FormatElement::StartAlign { width });
        let result = write(self);
        self.elements.push(FormatElement::EndAlign { width });
        result
    }

    pub(crate) fn fill<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        self.elements.push(FormatElement::StartFill);
        let result = write(self);
        self.elements.push(FormatElement::EndFill);
        result
    }

    /// Fill entries alternate between content and its following separator.
    pub(crate) fn fill_entry<T>(&mut self, write: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.elements.len();
        self.elements
            .push(FormatElement::StartEntry { end_offset: 0 });
        let result = write(self);
        let end_offset = self.elements.len() - start;
        self.elements[start] = FormatElement::StartEntry { end_offset };
        self.elements.push(FormatElement::EndEntry);
        result
    }

    pub(crate) fn finish(self) -> Doc<'a> {
        Doc {
            elements: self.elements,
            group_count: self.group_count,
        }
    }
}
