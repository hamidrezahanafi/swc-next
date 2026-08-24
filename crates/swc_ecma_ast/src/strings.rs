use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec, wtf8::Wtf8,
};

macro_rules! define_ast_string {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
        pub struct $name {
            pub(crate) start: u32,
            pub(crate) end: u32,
        }

        impl $name {
            pub const EMPTY: Self = Self { start: 0, end: 0 };

            #[inline]
            pub const fn len(self) -> u32 {
                debug_assert!(self.end >= self.start);
                self.end - self.start
            }

            #[inline]
            pub const fn is_empty(self) -> bool {
                self.start == self.end
            }

            #[inline]
            pub(crate) const fn from_raw(raw: u64) -> Self {
                Self {
                    start: raw as u32,
                    end: (raw >> 32) as u32,
                }
            }

            #[inline]
            pub(crate) const fn raw(self) -> u64 {
                self.start as u64 | ((self.end as u64) << 32)
            }
        }
    };
}

define_ast_string!(AstUtf8);
define_ast_string!(AstWtf8);

impl From<AstUtf8> for AstWtf8 {
    #[inline]
    fn from(value: AstUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

pub(crate) struct StringPool<'a> {
    allocator: &'a Allocator,
    pub(crate) source: &'a str,
    pub(crate) extra: ArenaVec<'a, u8>,
    /// Canonical ranges for strings whose identity is used by semantic analysis.
    interned: ArenaHashMap<'a, ArenaVec<'a, u8>, (u32, u32)>,
}

impl<'a> StringPool<'a> {
    pub fn new_in(allocator: &'a Allocator, source: &'a str) -> Self {
        assert!(source.len() <= u32::MAX as usize);
        Self {
            allocator,
            source,
            extra: ArenaVec::new_in(allocator),
            interned: ArenaHashMap::new_in(allocator),
        }
    }

    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> &str {
        // SAFETY: `AstUtf8` is guaranteed to reference valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &Wtf8 {
        // SAFETY: `AstWtf8` is guaranteed to reference valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(self.get_bytes(id.start, id.end)) }
    }

    #[inline]
    fn get_bytes(&self, start: u32, end: u32) -> &[u8] {
        debug_assert!(end >= start);
        if start == end {
            return b"";
        }
        let source_len = self.source.len() as u32;
        if start < source_len {
            debug_assert!(end <= source_len);
            &self.source.as_bytes()[start as usize..end as usize]
        } else {
            let start = (start - source_len) as usize;
            let end = (end - source_len) as usize;
            &self.extra[start..end]
        }
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        debug_assert!(self.source.get(start as usize..end as usize).is_some());
        AstUtf8 { start, end }
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8 {
        debug_assert!(self.source.get(start as usize..end as usize).is_some());
        AstWtf8 { start, end }
    }

    #[inline]
    pub fn add_utf8(&mut self, text: &str) -> AstUtf8 {
        let (start, end) = self.append_bytes(text.as_bytes());
        AstUtf8 { start, end }
    }

    #[inline]
    pub fn add_wtf8(&mut self, text: &Wtf8) -> AstWtf8 {
        let (start, end) = self.append_bytes(text.as_bytes());
        AstWtf8 { start, end }
    }

    /// Returns the canonical range for a semantic UTF-8 string.
    #[inline]
    pub fn intern_utf8(&mut self, text: &str) -> AstUtf8 {
        let (start, end) = self.intern_bytes(text.as_bytes());
        AstUtf8 { start, end }
    }

    /// Canonicalizes an existing UTF-8 range.
    #[inline]
    pub fn intern_utf8_slice(&mut self, text: AstUtf8) -> AstUtf8 {
        let (start, end) = self.intern_range(text.start, text.end);
        AstUtf8 { start, end }
    }

    fn intern_bytes(&mut self, text: &[u8]) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }
        if let Some(&id) = self.interned.get(text) {
            return id;
        }
        let range = self.append_bytes(text);
        let mut key = ArenaVec::with_capacity_in(text.len(), self.allocator);
        key.extend_from_slice(text);
        self.interned.insert(key, range);
        range
    }

    fn intern_range(&mut self, start: u32, end: u32) -> (u32, u32) {
        if start == end {
            return (0, 0);
        }
        if let Some(id) = {
            let text = self.get_bytes(start, end);
            self.interned.get(text).copied()
        } {
            return id;
        }
        let mut key = ArenaVec::with_capacity_in((end - start) as usize, self.allocator);
        key.extend_from_slice(self.get_bytes(start, end));
        self.interned.insert(key, (start, end));
        (start, end)
    }

    fn append_bytes(&mut self, text: &[u8]) -> (u32, u32) {
        if text.is_empty() {
            return (0, 0);
        }
        let start = self
            .source
            .len()
            .checked_add(self.extra.len())
            .expect("string pool offset overflow");
        let end = start
            .checked_add(text.len())
            .expect("string pool offset overflow");
        assert!(end <= u32::MAX as usize, "string pool exceeds u32 range");
        self.extra.extend_from_slice(text);
        (start as u32, end as u32)
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::StringPool;
    use crate::{Ast, Span};

    #[test]
    fn only_explicitly_interned_strings_share_a_range() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "name name");

        let first = strings.source_slice(0, 4);
        let second = strings.source_slice(5, 9);
        assert_ne!(first, second);
        assert_eq!(
            strings.intern_utf8_slice(first),
            strings.intern_utf8_slice(second)
        );

        let first = strings.add_utf8("decoded");
        let second = strings.add_utf8("decoded");
        assert_ne!(first, second);
        assert_eq!(strings.get_utf8(first), strings.get_utf8(second));
        assert_eq!(
            strings.intern_utf8_slice(first),
            strings.intern_utf8_slice(second)
        );
    }

    #[test]
    fn semantic_name_fields_intern_but_literal_fields_remain_slices() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("name name same same", &allocator);

        let first_name = ast.source_slice(0, 4);
        let second_name = ast.source_slice(5, 9);
        let first_identifier = ast.identifier_reference(Span::new(0, 4), first_name);
        let second_identifier = ast.identifier_reference(Span::new(5, 9), second_name);
        assert_eq!(first_identifier.name(&ast), second_identifier.name(&ast));

        let first_literal_value = ast.source_slice_wtf8(10, 14);
        let second_literal_value = ast.source_slice_wtf8(15, 19);
        let first_literal = ast.string_literal(
            Span::new(10, 14),
            first_literal_value,
            ast.source_slice(10, 14),
        );
        let second_literal = ast.string_literal(
            Span::new(15, 19),
            second_literal_value,
            ast.source_slice(15, 19),
        );
        assert_ne!(first_literal.value(&ast), second_literal.value(&ast));
    }
}
