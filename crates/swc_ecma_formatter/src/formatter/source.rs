//! Line boundaries in source gaps, including ECMAScript line terminators.

use swc_next_ecma_ast::Span;

use super::Formatter;

impl Formatter<'_, '_> {
    pub(super) fn source(&self, span: Span) -> &str {
        &self.ast.source()[span.start as usize..span.end as usize]
    }

    pub(super) fn line_breaks(&self, start: u32, end: u32) -> u8 {
        let mut characters = self.source(Span::new(start, end)).chars().peekable();
        let mut count = 0;
        while let Some(character) = characters.next() {
            match character {
                '\r' => {
                    if characters.peek() == Some(&'\n') {
                        characters.next();
                    }
                }
                '\n' | '\u{2028}' | '\u{2029}' => {}
                _ => continue,
            }
            count += 1;
            if count == 2 {
                break;
            }
        }
        count
    }
}
