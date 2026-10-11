//! Cached display widths for borrowed text; controls retain column context.

use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy)]
pub(super) struct Text<'a> {
    pub(super) source: &'a str,
    width: Width,
}

#[derive(Debug, Clone, Copy)]
enum Width {
    Ascii,
    Unicode(u32),
    Dynamic,
}

impl<'a> Text<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let width = if source.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            Width::Ascii
        } else if source.chars().any(char::is_control) {
            Width::Dynamic
        } else {
            Width::Unicode(u32::try_from(source.width()).unwrap_or(u32::MAX))
        };
        Self { source, width }
    }

    pub(super) fn is_single_line(self) -> bool {
        match self.width {
            Width::Ascii => true,
            _ => !self.source.contains(['\n', '\r', '\u{2028}', '\u{2029}']),
        }
    }

    /// Textual width has no column context and does not prove a flat render view.
    pub(super) fn clean_width(self) -> Option<usize> {
        Some(match self.width {
            Width::Ascii => self.source.len(),
            Width::Unicode(width) => width as usize,
            Width::Dynamic => {
                let mut width = 0;
                let mut start = 0;
                for (index, character) in self.source.char_indices() {
                    if matches!(character, '\r' | '\n') {
                        return None;
                    }
                    if character.is_control() {
                        width += self.source[start..index].width();
                        start = index + character.len_utf8();
                    }
                }
                width + self.source[start..].width()
            }
        })
    }

    pub(super) fn clean_ascii_width(self) -> Option<usize> {
        match self.width {
            Width::Ascii => Some(self.source.len()),
            Width::Dynamic
                if self
                    .source
                    .bytes()
                    .all(|byte| (0x20..=0x7f).contains(&byte)) =>
            {
                Some(self.source.len())
            }
            _ => None,
        }
    }

    pub(super) fn first_line(self, column: usize, tab_width: u8) -> (usize, bool) {
        match self.width {
            Width::Ascii => (column + self.source.len(), false),
            Width::Unicode(width) => (column + width as usize, false),
            Width::Dynamic => advance(self.source, column, tab_width),
        }
    }

    pub(super) fn last_line(self, column: usize, tab_width: u8) -> usize {
        if matches!(self.width, Width::Dynamic)
            && let Some((index, character)) = self
                .source
                .char_indices()
                .rfind(|(_, character)| matches!(character, '\r' | '\n'))
        {
            return advance(&self.source[index + character.len_utf8()..], 0, tab_width).0;
        }
        self.first_line(column, tab_width).0
    }
}

fn advance(source: &str, mut column: usize, tab_width: u8) -> (usize, bool) {
    let mut start = 0;
    for (index, character) in source.char_indices() {
        if matches!(character, '\r' | '\n') {
            column += source[start..index].width();
            return (column, true);
        }
        if character.is_control() {
            column += source[start..index].width();
            if character == '\t' {
                let width = usize::from(tab_width.max(1));
                column += width - column % width;
            }
            start = index + character.len_utf8();
        }
    }
    (column + source[start..].width(), false)
}
