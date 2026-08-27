use std::{
    borrow::Cow,
    fmt,
    hash::{BuildHasher, Hash, Hasher},
};

use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec, wtf8::Wtf8,
};

const STORAGE_EXTRA: u32 = 1 << 31;
const GENERATED_NAME: u32 = 1 << 31;
const SUFFIX_SHIFT: u32 = 29;
const FLAT_OFFSET_MASK: u32 = (1 << 31) - 1;
const COMPACT_OFFSET_MASK: u32 = (1 << 29) - 1;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

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
                let is_generated = self.end & GENERATED_NAME != 0;
                let mask = if is_generated {
                    COMPACT_OFFSET_MASK
                } else {
                    FLAT_OFFSET_MASK
                };
                let start = self.start & mask;
                let end = self.end & mask;
                debug_assert!(end >= start);
                end - start + is_generated as u32
            }

            #[inline]
            pub const fn is_empty(self) -> bool {
                self.end & GENERATED_NAME == 0
                    && (self.start & FLAT_OFFSET_MASK) == (self.end & FLAT_OFFSET_MASK)
            }

            #[inline]
            #[allow(dead_code)]
            pub(crate) const fn from_raw(raw: u64) -> Self {
                Self {
                    start: raw as u32,
                    end: (raw >> 32) as u32,
                }
            }

            #[inline]
            #[allow(dead_code)]
            pub(crate) const fn raw(self) -> u64 {
                self.start as u64 | ((self.end as u64) << 32)
            }
        }
    };
}

define_ast_string!(AstUtf8);
define_ast_string!(AstInternedUtf8);
define_ast_string!(AstWtf8);

impl From<AstInternedUtf8> for AstUtf8 {
    #[inline]
    fn from(value: AstInternedUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StringId {
    start: u32,
    end: u32,
}

impl StringId {
    const EMPTY: Self = Self { start: 0, end: 0 };

    #[inline]
    const fn try_flat_source(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= FLAT_OFFSET_MASK {
            Some(Self { start, end })
        } else {
            None
        }
    }

    #[inline]
    const fn flat_source(start: u32, end: u32) -> Self {
        match Self::try_flat_source(start, end) {
            Some(id) => id,
            None => panic!("source string range exceeds 31-bit representation"),
        }
    }

    #[inline]
    const fn try_flat_extra(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= FLAT_OFFSET_MASK {
            Some(Self {
                start: STORAGE_EXTRA | start,
                end,
            })
        } else {
            None
        }
    }

    #[inline]
    const fn flat_extra(start: u32, end: u32) -> Self {
        match Self::try_flat_extra(start, end) {
            Some(id) => id,
            None => panic!("extra string range exceeds 31-bit representation"),
        }
    }

    #[inline]
    const fn try_generated(prefix: Self, suffix: u8) -> Option<Self> {
        if prefix.is_generated()
            || prefix.start_offset() > COMPACT_OFFSET_MASK
            || prefix.end_offset() > COMPACT_OFFSET_MASK
            || suffix >= 16
        {
            return None;
        }
        let storage = prefix.start & STORAGE_EXTRA;
        Some(Self {
            start: storage | (((suffix >> 2) as u32) << SUFFIX_SHIFT) | prefix.start_offset(),
            end: GENERATED_NAME | (((suffix & 0b11) as u32) << SUFFIX_SHIFT) | prefix.end_offset(),
        })
    }

    #[inline]
    const fn generated(prefix: Self, suffix: u8) -> Self {
        match Self::try_generated(prefix, suffix) {
            Some(id) => id,
            None => panic!("generated name prefix exceeds 29-bit representation"),
        }
    }

    #[inline]
    const fn is_extra(self) -> bool {
        self.start & STORAGE_EXTRA != 0
    }

    #[inline]
    const fn is_generated(self) -> bool {
        self.end & GENERATED_NAME != 0
    }

    #[inline]
    const fn offset_mask(self) -> u32 {
        if self.is_generated() {
            COMPACT_OFFSET_MASK
        } else {
            FLAT_OFFSET_MASK
        }
    }

    #[inline]
    const fn start_offset(self) -> u32 {
        self.start & self.offset_mask()
    }

    #[inline]
    const fn end_offset(self) -> u32 {
        self.end & self.offset_mask()
    }

    #[inline]
    const fn suffix(self) -> Option<u8> {
        if !self.is_generated() {
            return None;
        }
        Some(
            (((self.start >> SUFFIX_SHIFT) & 0b11) << 2 | ((self.end >> SUFFIX_SHIFT) & 0b11))
                as u8,
        )
    }

    #[inline]
    const fn can_be_compact_prefix(self) -> bool {
        !self.is_generated()
            && self.start_offset() <= COMPACT_OFFSET_MASK
            && self.end_offset() <= COMPACT_OFFSET_MASK
    }
}

impl From<AstUtf8> for StringId {
    #[inline]
    fn from(value: AstUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<AstInternedUtf8> for StringId {
    #[inline]
    fn from(value: AstInternedUtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<AstWtf8> for StringId {
    #[inline]
    fn from(value: AstWtf8) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<StringId> for AstUtf8 {
    #[inline]
    fn from(value: StringId) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<StringId> for AstInternedUtf8 {
    #[inline]
    fn from(value: StringId) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<StringId> for AstWtf8 {
    #[inline]
    fn from(value: StringId) -> Self {
        debug_assert!(!value.is_generated());
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InternEntry {
    hash: u64,
    id: StringId,
}

impl Hash for InternEntry {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Clone, Copy)]
struct LogicalBytes<'a> {
    prefix: &'a [u8],
    suffix: Option<u8>,
}

impl LogicalBytes<'_> {
    #[inline]
    fn len(self) -> usize {
        self.prefix.len() + usize::from(self.suffix.is_some())
    }

    #[inline]
    fn eq(self, other: Self) -> bool {
        match (self.suffix, other.suffix) {
            (None, None) => self.prefix == other.prefix,
            (Some(left), Some(right)) => left == right && self.prefix == other.prefix,
            (Some(suffix), None) => other
                .prefix
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == self.prefix),
            (None, Some(suffix)) => self
                .prefix
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == other.prefix),
        }
    }

    #[inline]
    fn eq_slice(self, other: &[u8]) -> bool {
        match self.suffix {
            None => self.prefix == other,
            Some(suffix) => other
                .split_last()
                .is_some_and(|(last, prefix)| *last == suffix && prefix == self.prefix),
        }
    }

    #[inline]
    fn hash(self) -> u64 {
        if let Some(suffix) = self.suffix {
            hash_segmented_bytes(self.prefix, suffix)
        } else {
            hash_bytes(self.prefix)
        }
    }
}

const HASH_SEED1: u64 = 0x243f_6a88_85a3_08d3;
const HASH_SEED2: u64 = 0x1319_8a2e_0370_7344;
const HASH_PREVENT_ZERO: u64 = 0xa409_3822_299f_31d0;

#[inline]
fn multiply_mix(left: u64, right: u64) -> u64 {
    let full = (left as u128).wrapping_mul(right as u128);
    full as u64 ^ (full >> 64) as u64
}

/// Fast content hash equivalent for flat and `prefix + suffix` strings.
/// This follows rustc-hash's byte mixer, but reads the final virtual byte
/// directly instead of materializing a contiguous candidate.
#[inline]
fn hash_bytes(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut s0 = HASH_SEED1;
    let mut s1 = HASH_SEED2;
    if len <= 16 {
        if len >= 8 {
            s0 ^= u64::from_le_bytes(bytes[..8].try_into().unwrap());
            s1 ^= u64::from_le_bytes(bytes[len - 8..].try_into().unwrap());
        } else if len >= 4 {
            s0 ^= u32::from_le_bytes(bytes[..4].try_into().unwrap()) as u64;
            s1 ^= u32::from_le_bytes(bytes[len - 4..].try_into().unwrap()) as u64;
        } else if len > 0 {
            s0 ^= u64::from(bytes[0]);
            s1 ^= (u64::from(bytes[len - 1]) << 8) | u64::from(bytes[len / 2]);
        }
    } else {
        let mut bulk = &bytes[..len - 1];
        while let Some((chunk, rest)) = bulk.split_first_chunk::<16>() {
            let x = u64::from_le_bytes(chunk[..8].try_into().unwrap());
            let y = u64::from_le_bytes(chunk[8..].try_into().unwrap());
            let next = multiply_mix(s0 ^ x, HASH_PREVENT_ZERO ^ y);
            s0 = s1;
            s1 = next;
            bulk = rest;
        }
        let suffix = &bytes[len - 16..];
        s0 ^= u64::from_le_bytes(suffix[..8].try_into().unwrap());
        s1 ^= u64::from_le_bytes(suffix[8..].try_into().unwrap());
    }
    multiply_mix(s0, s1) ^ len as u64
}

#[inline]
fn hash_segmented_bytes(prefix: &[u8], suffix: u8) -> u64 {
    let len = prefix.len() + 1;
    let byte = |index: usize| {
        if index == prefix.len() {
            suffix
        } else {
            prefix[index]
        }
    };
    let read_u32 = |start: usize| {
        u32::from_le_bytes([
            byte(start),
            byte(start + 1),
            byte(start + 2),
            byte(start + 3),
        ])
    };
    let read_u64 = |start: usize| {
        u64::from_le_bytes([
            byte(start),
            byte(start + 1),
            byte(start + 2),
            byte(start + 3),
            byte(start + 4),
            byte(start + 5),
            byte(start + 6),
            byte(start + 7),
        ])
    };

    let mut s0 = HASH_SEED1;
    let mut s1 = HASH_SEED2;
    if len <= 16 {
        if len >= 8 {
            s0 ^= read_u64(0);
            s1 ^= read_u64(len - 8);
        } else if len >= 4 {
            s0 ^= u64::from(read_u32(0));
            s1 ^= u64::from(read_u32(len - 4));
        } else {
            s0 ^= u64::from(byte(0));
            s1 ^= (u64::from(suffix) << 8) | u64::from(byte(len / 2));
        }
    } else {
        let mut bulk = prefix;
        while let Some((chunk, rest)) = bulk.split_first_chunk::<16>() {
            let x = u64::from_le_bytes(chunk[..8].try_into().unwrap());
            let y = u64::from_le_bytes(chunk[8..].try_into().unwrap());
            let next = multiply_mix(s0 ^ x, HASH_PREVENT_ZERO ^ y);
            s0 = s1;
            s1 = next;
            bulk = rest;
        }
        s0 ^= read_u64(len - 16);
        s1 ^= read_u64(len - 8);
    }
    multiply_mix(s0, s1) ^ len as u64
}

/// A non-allocating logical view of a UTF-8 handle.
#[derive(Clone, Copy, Debug)]
pub struct Utf8Chunks<'a> {
    prefix: &'a str,
    suffix: Option<u8>,
}

impl<'a> Utf8Chunks<'a> {
    #[inline]
    pub fn prefix(self) -> &'a str {
        self.prefix
    }

    #[inline]
    pub fn suffix(self) -> Option<u8> {
        self.suffix
    }

    #[inline]
    pub fn len(self) -> usize {
        self.prefix.len() + usize::from(self.suffix.is_some())
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn bytes(self) -> impl Iterator<Item = u8> + 'a {
        self.prefix.bytes().chain(self.suffix)
    }

    #[inline]
    pub fn eq_bytes(self, bytes: &[u8]) -> bool {
        self.len() == bytes.len() && self.bytes().eq(bytes.iter().copied())
    }

    #[inline]
    pub fn contains_byte(self, byte: u8) -> bool {
        self.prefix.as_bytes().contains(&byte) || self.suffix == Some(byte)
    }

    #[inline]
    pub fn first_byte(self) -> Option<u8> {
        self.prefix.as_bytes().first().copied().or(self.suffix)
    }

    pub fn to_cow(self) -> Cow<'a, str> {
        let Some(suffix) = self.suffix else {
            return Cow::Borrowed(self.prefix);
        };
        let mut output = String::with_capacity(self.prefix.len() + 1);
        output.push_str(self.prefix);
        output.push(char::from(suffix));
        Cow::Owned(output)
    }
}

impl fmt::Display for Utf8Chunks<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.prefix)?;
        if let Some(suffix) = self.suffix {
            formatter.write_str(unsafe {
                std::str::from_utf8_unchecked(std::slice::from_ref(&suffix))
            })?;
        }
        Ok(())
    }
}

pub(crate) struct StringPool<'a> {
    pub(crate) source: &'a str,
    pub(crate) extra: ArenaVec<'a, u8>,
    utf8_dedup: ArenaHashMap<'a, InternEntry, ()>,
    wtf8_dedup: ArenaHashMap<'a, InternEntry, ()>,
}

impl<'a> StringPool<'a> {
    pub fn new_in(allocator: &'a Allocator, source: &'a str) -> Self {
        assert!(
            source.len() <= FLAT_OFFSET_MASK as usize,
            "source exceeds 2 GiB string-handle range"
        );
        Self {
            source,
            extra: ArenaVec::new_in(allocator),
            utf8_dedup: ArenaHashMap::new_in(allocator),
            wtf8_dedup: ArenaHashMap::new_in(allocator),
        }
    }

    #[inline]
    pub fn get_flat_utf8(&self, id: AstUtf8) -> &str {
        let id = StringId::from(id);
        debug_assert!(!id.is_generated(), "compact UTF-8 requires a logical view");
        // SAFETY: `AstUtf8` is guaranteed to reference valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(self.get_flat_bytes(id)) }
    }

    #[inline]
    pub fn utf8_to_cow(&self, id: AstUtf8) -> Cow<'_, str> {
        self.utf8_chunks(id).to_cow()
    }

    #[inline]
    pub fn utf8_chunks(&self, id: AstUtf8) -> Utf8Chunks<'_> {
        let bytes = self.logical_bytes(id.into());
        // SAFETY: `AstUtf8` is valid UTF-8 and the optional suffix is ASCII.
        let prefix = unsafe { std::str::from_utf8_unchecked(bytes.prefix) };
        Utf8Chunks {
            prefix,
            suffix: bytes.suffix,
        }
    }

    #[inline]
    pub fn utf8_eq(&self, left: AstUtf8, right: AstUtf8) -> bool {
        self.logical_bytes(left.into())
            .eq(self.logical_bytes(right.into()))
    }

    #[inline]
    pub fn utf8_eq_bytes(&self, id: AstUtf8, bytes: &[u8]) -> bool {
        self.logical_bytes(id.into()).eq_slice(bytes)
    }

    #[inline]
    pub fn utf8_eq_wtf8(&self, utf8: AstUtf8, wtf8: AstWtf8) -> bool {
        self.logical_bytes(utf8.into())
            .eq_slice(self.get_wtf8(wtf8).as_bytes())
    }

    #[inline]
    pub fn wtf8_eq(&self, left: AstWtf8, right: AstWtf8) -> bool {
        self.get_wtf8(left) == self.get_wtf8(right)
    }

    #[inline]
    pub fn wtf8_eq_bytes(&self, id: AstWtf8, bytes: &[u8]) -> bool {
        self.get_wtf8(id).as_bytes() == bytes
    }

    #[inline]
    pub fn utf8_contains_byte(&self, id: AstUtf8, byte: u8) -> bool {
        let bytes = self.logical_bytes(id.into());
        bytes.prefix.contains(&byte) || bytes.suffix == Some(byte)
    }

    #[inline]
    pub fn utf8_first_byte(&self, id: AstUtf8) -> Option<u8> {
        let bytes = self.logical_bytes(id.into());
        bytes.prefix.first().copied().or(bytes.suffix)
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &Wtf8 {
        let id = StringId::from(id);
        debug_assert!(!id.is_generated());
        // SAFETY: `AstWtf8` is guaranteed to reference valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(self.get_flat_bytes(id)) }
    }

    #[inline]
    pub fn as_utf8(&self, id: AstWtf8) -> Option<AstUtf8> {
        self.get_wtf8(id).as_str().map(|_| AstUtf8 {
            start: id.start,
            end: id.end,
        })
    }

    #[inline]
    fn get_flat_bytes(&self, id: StringId) -> &[u8] {
        debug_assert!(!id.is_generated());
        self.get_storage_bytes(id)
    }

    #[inline]
    fn logical_bytes(&self, id: StringId) -> LogicalBytes<'_> {
        LogicalBytes {
            prefix: self.get_storage_bytes(id),
            suffix: id.suffix().map(|suffix| HEX_DIGITS[usize::from(suffix)]),
        }
    }

    #[inline]
    fn get_storage_bytes(&self, id: StringId) -> &[u8] {
        let start = id.start_offset() as usize;
        let end = id.end_offset() as usize;
        debug_assert!(end >= start);
        if id.is_extra() {
            &self.extra[start..end]
        } else {
            &self.source.as_bytes()[start..end]
        }
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        assert!(self.source.get(start as usize..end as usize).is_some());
        StringId::flat_source(start, end).into()
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8 {
        assert!(self.source.get(start as usize..end as usize).is_some());
        StringId::flat_source(start, end).into()
    }

    #[inline]
    pub fn add_utf8(&mut self, text: &str) -> AstInternedUtf8 {
        self.intern_utf8_bytes(text.as_bytes()).into()
    }

    #[inline]
    pub fn add_wtf8(&mut self, text: &Wtf8) -> AstWtf8 {
        let parts = LogicalBytes {
            prefix: text.as_bytes(),
            suffix: None,
        };
        if let Some(id) = Self::find_in(&self.wtf8_dedup, self.source, &self.extra, parts) {
            return id.into();
        }
        let id = self.append_extra(text.as_bytes());
        Self::insert_in(&mut self.wtf8_dedup, parts.hash(), id);
        id.into()
    }

    pub fn add_wtf8_prefixed_utf8(&mut self, prefix: &[u8], text: AstUtf8) -> AstWtf8 {
        let text = self.logical_bytes(text.into());
        let mut bytes = Vec::with_capacity(prefix.len() + text.len());
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(text.prefix);
        if let Some(suffix) = text.suffix {
            bytes.push(suffix);
        }
        // SAFETY: `prefix` is supplied by an internal caller and `AstUtf8`
        // is valid UTF-8, so their concatenation is valid WTF-8.
        let text = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        self.add_wtf8(text)
    }

    /// Converts UTF-8 to the shared WTF-8 storage representation. Flat
    /// handles reuse their range; compact handles are materialized first.
    pub fn utf8_to_wtf8(&mut self, text: AstUtf8) -> AstWtf8 {
        let id = StringId::from(text);
        if !id.is_generated() {
            return id.into();
        }
        self.add_wtf8_prefixed_utf8(&[], text)
    }

    /// Canonicalizes a UTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_utf8_range(&mut self, text: AstUtf8) -> AstInternedUtf8 {
        let id = StringId::from(text);
        let parts = self.logical_bytes(id);
        if let Some(existing) = Self::find_in(&self.utf8_dedup, self.source, &self.extra, parts) {
            return existing.into();
        }
        let hash = parts.hash();
        Self::insert_in(&mut self.utf8_dedup, hash, id);
        id.into()
    }

    /// Canonicalizes a WTF-8 range already stored in this pool.
    #[inline]
    pub fn intern_wtf8_range(&mut self, text: AstWtf8) -> AstWtf8 {
        let id = StringId::from(text);
        let parts = LogicalBytes {
            prefix: self.get_flat_bytes(id),
            suffix: None,
        };
        if let Some(existing) = Self::find_in(&self.wtf8_dedup, self.source, &self.extra, parts) {
            return existing.into();
        }
        let hash = parts.hash();
        Self::insert_in(&mut self.wtf8_dedup, hash, id);
        id.into()
    }

    /// Builds or reuses the physical prefix for one block of 16 generated names.
    /// Returns `None` when the prefix cannot fit in the compact 29-bit range.
    pub fn generated_name_prefix(&mut self, base: AstInternedUtf8, block: u64) -> Option<AstUtf8> {
        let base = StringId::from(base);
        if block == 0 && base.can_be_compact_prefix() {
            return Some(base.into());
        }

        let mut bytes = Vec::with_capacity(self.logical_bytes(base).len() + 16);
        let logical_base = self.logical_bytes(base);
        bytes.extend_from_slice(logical_base.prefix);
        if let Some(suffix) = logical_base.suffix {
            bytes.push(suffix);
        }
        if block != 0 {
            append_hex(&mut bytes, block);
        }
        self.compact_prefix_for_bytes(&bytes).map(Into::into)
    }

    /// Interns a generated name using a cached physical prefix. If no compact
    /// prefix is available, stores the complete name using the flat fallback.
    pub fn intern_generated_name(
        &mut self,
        base: AstInternedUtf8,
        prefix: Option<AstUtf8>,
        suffix: u64,
    ) -> AstInternedUtf8 {
        let nibble = (suffix & 0xf) as u8;
        if let Some(prefix) = prefix {
            let id = StringId::generated(prefix.into(), nibble);
            let parts = self.logical_bytes(id);
            if let Some(existing) = Self::find_in(&self.utf8_dedup, self.source, &self.extra, parts)
            {
                return existing.into();
            }
            let hash = parts.hash();
            Self::insert_in(&mut self.utf8_dedup, hash, id);
            return id.into();
        }

        let base = StringId::from(base);
        let mut bytes = Vec::with_capacity(self.logical_bytes(base).len() + 16);
        let logical_base = self.logical_bytes(base);
        bytes.extend_from_slice(logical_base.prefix);
        if let Some(base_suffix) = logical_base.suffix {
            bytes.push(base_suffix);
        }
        append_hex(&mut bytes, suffix);
        self.intern_utf8_bytes(&bytes).into()
    }

    fn compact_prefix_for_bytes(&mut self, bytes: &[u8]) -> Option<StringId> {
        let parts = LogicalBytes {
            prefix: bytes,
            suffix: None,
        };
        if let Some(existing) = Self::find_in(&self.utf8_dedup, self.source, &self.extra, parts)
            && existing.can_be_compact_prefix()
        {
            return Some(existing);
        }

        compact_extra_range(self.extra.len(), bytes.len())?;
        let id = self.append_extra(bytes);
        if Self::find_in(&self.utf8_dedup, self.source, &self.extra, parts).is_none() {
            Self::insert_in(&mut self.utf8_dedup, parts.hash(), id);
        }
        Some(id)
    }

    fn intern_utf8_bytes(&mut self, text: &[u8]) -> StringId {
        if text.is_empty() {
            return StringId::EMPTY;
        }
        let parts = LogicalBytes {
            prefix: text,
            suffix: None,
        };
        if let Some(id) = Self::find_in(&self.utf8_dedup, self.source, &self.extra, parts) {
            return id;
        }
        let id = self.append_extra(text);
        Self::insert_in(&mut self.utf8_dedup, parts.hash(), id);
        id
    }

    fn append_extra(&mut self, text: &[u8]) -> StringId {
        if text.is_empty() {
            return StringId::EMPTY;
        }
        let start = self.extra.len();
        let end = start
            .checked_add(text.len())
            .expect("string pool offset overflow");
        assert!(
            end <= FLAT_OFFSET_MASK as usize,
            "extra strings exceed 2 GiB string-handle range"
        );
        self.extra.extend_from_slice(text);
        StringId::flat_extra(start as u32, end as u32)
    }

    fn find_in(
        dedup: &ArenaHashMap<'a, InternEntry, ()>,
        source: &str,
        extra: &[u8],
        parts: LogicalBytes<'_>,
    ) -> Option<StringId> {
        let content_hash = parts.hash();
        let table_hash = dedup.hasher().hash_one(content_hash);
        dedup
            .raw_entry()
            .from_hash(table_hash, |entry| {
                entry.hash == content_hash && logical_bytes_from(source, extra, entry.id).eq(parts)
            })
            .map(|(entry, ())| entry.id)
    }

    #[inline]
    fn insert_in(dedup: &mut ArenaHashMap<'a, InternEntry, ()>, hash: u64, id: StringId) {
        dedup.insert(InternEntry { hash, id }, ());
    }
}

#[inline]
fn compact_extra_range(start: usize, len: usize) -> Option<(u32, u32)> {
    let end = start.checked_add(len)?;
    if end > COMPACT_OFFSET_MASK as usize {
        return None;
    }
    Some((start as u32, end as u32))
}

#[inline]
fn logical_bytes_from<'a>(source: &'a str, extra: &'a [u8], id: StringId) -> LogicalBytes<'a> {
    let start = id.start_offset() as usize;
    let end = id.end_offset() as usize;
    let prefix = if id.is_extra() {
        &extra[start..end]
    } else {
        &source.as_bytes()[start..end]
    };
    LogicalBytes {
        prefix,
        suffix: id.suffix().map(|suffix| HEX_DIGITS[usize::from(suffix)]),
    }
}

fn append_hex(output: &mut Vec<u8>, mut value: u64) {
    let mut digits = [0_u8; 16];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = HEX_DIGITS[(value & 0xf) as usize];
        value >>= 4;
        if value == 0 {
            break;
        }
    }
    output.extend_from_slice(&digits[start..]);
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use swc_next_allocator::Allocator;

    use super::{
        AstInternedUtf8, AstUtf8, AstWtf8, COMPACT_OFFSET_MASK, FLAT_OFFSET_MASK, InternEntry,
        LogicalBytes, StringId, StringPool, compact_extra_range, hash_bytes, hash_segmented_bytes,
    };

    #[test]
    fn string_handles_remain_eight_bytes() {
        assert_eq!(size_of::<AstUtf8>(), 8);
        assert_eq!(size_of::<AstInternedUtf8>(), 8);
        assert_eq!(size_of::<AstWtf8>(), 8);
    }

    #[test]
    fn codec_round_trips_flat_and_generated_boundaries() {
        let source = StringId::try_flat_source(0, FLAT_OFFSET_MASK).unwrap();
        assert!(!source.is_extra());
        assert_eq!(source.start_offset(), 0);
        assert_eq!(source.end_offset(), FLAT_OFFSET_MASK);

        let extra = StringId::try_flat_extra(0, FLAT_OFFSET_MASK).unwrap();
        assert!(extra.is_extra());
        assert_eq!(extra.start_offset(), 0);
        assert_eq!(extra.end_offset(), FLAT_OFFSET_MASK);

        let source_prefix = StringId::try_flat_source(0, COMPACT_OFFSET_MASK).unwrap();
        let extra_prefix = StringId::try_flat_extra(0, COMPACT_OFFSET_MASK).unwrap();
        for suffix in 0..16 {
            let source_name = StringId::try_generated(source_prefix, suffix).unwrap();
            assert!(!source_name.is_extra());
            assert_eq!(source_name.start_offset(), 0);
            assert_eq!(source_name.end_offset(), COMPACT_OFFSET_MASK);
            assert_eq!(source_name.suffix(), Some(suffix));

            let extra_name = StringId::try_generated(extra_prefix, suffix).unwrap();
            assert!(extra_name.is_extra());
            assert_eq!(extra_name.start_offset(), 0);
            assert_eq!(extra_name.end_offset(), COMPACT_OFFSET_MASK);
            assert_eq!(extra_name.suffix(), Some(suffix));
        }

        assert!(StringId::try_flat_source(1, 0).is_none());
        assert!(StringId::try_flat_source(0, FLAT_OFFSET_MASK + 1).is_none());
        assert!(StringId::try_flat_extra(1, 0).is_none());
        assert!(StringId::try_flat_extra(0, FLAT_OFFSET_MASK + 1).is_none());
        assert!(
            StringId::try_generated(
                StringId::flat_source(COMPACT_OFFSET_MASK + 1, COMPACT_OFFSET_MASK + 1),
                0,
            )
            .is_none()
        );
        assert!(StringId::try_generated(source_prefix, 16).is_none());
    }

    #[test]
    fn compact_extra_boundary_selects_flat_fallback_without_allocating() {
        assert_eq!(
            compact_extra_range(COMPACT_OFFSET_MASK as usize - 1, 1),
            Some((COMPACT_OFFSET_MASK - 1, COMPACT_OFFSET_MASK))
        );
        assert_eq!(compact_extra_range(COMPACT_OFFSET_MASK as usize, 1), None);
        assert_eq!(compact_extra_range(usize::MAX, 1), None);
    }

    #[test]
    fn segmented_hash_matches_flat_hash() {
        let bytes: Vec<u8> = (0..96).map(|index| b'a' + index % 26).collect();
        for prefix_len in 0..bytes.len() {
            let prefix = &bytes[..prefix_len];
            let suffix = b'f';
            let mut flat = prefix.to_vec();
            flat.push(suffix);
            assert_eq!(
                hash_segmented_bytes(prefix, suffix),
                hash_bytes(&flat),
                "prefix length {prefix_len}"
            );
        }
    }

    #[test]
    fn interns_existing_ranges_without_moving_the_first_value() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "name name");
        let first = strings.source_slice(0, 4);
        let second = strings.source_slice(5, 9);

        let canonical = strings.intern_utf8_range(first);
        assert_eq!(canonical.start, 0);
        assert_eq!(canonical.end, 4);
        assert_eq!(strings.extra.len(), 0);
        assert_eq!(strings.intern_utf8_range(second), canonical);
        assert_eq!(strings.extra.len(), 0);
        assert_eq!(strings.utf8_dedup.len(), 1);
    }

    #[test]
    fn added_strings_store_bytes_once() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "");
        let first = strings.add_utf8("generated");
        let second = strings.add_utf8("generated");

        assert_eq!(first, second);
        assert_eq!(strings.extra.as_slice(), b"generated");
        assert_eq!(strings.utf8_dedup.len(), 1);
    }

    #[test]
    fn flat_extra_uses_independent_offsets() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "source");
        let added = strings.add_utf8("extra");
        let id = StringId::from(added);

        assert!(id.is_extra());
        assert_eq!(id.start_offset(), 0);
        assert_eq!(id.end_offset(), 5);
        assert_eq!(strings.get_flat_utf8(added.into()), "extra");
    }

    #[test]
    fn generated_names_share_one_prefix_per_block() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "");
        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let extra_after_prefix = strings.extra.len();
        let ten = strings.intern_generated_name(base, Some(prefix), 0x10);
        let one_f = strings.intern_generated_name(base, Some(prefix), 0x1f);

        assert!(strings.utf8_eq_bytes(ten.into(), b"_temp10"));
        assert!(strings.utf8_eq_bytes(one_f.into(), b"_temp1f"));
        assert_eq!(strings.extra.len(), extra_after_prefix);
        assert_eq!(ten.len(), 7);
    }

    #[test]
    fn generated_name_collides_with_existing_flat_name() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "_temp10");
        let existing = strings.intern_utf8_range(strings.source_slice(0, 7));
        let base = strings.add_utf8("_temp");
        let prefix = strings.generated_name_prefix(base, 1).unwrap();
        let generated = strings.intern_generated_name(base, Some(prefix), 0x10);

        assert_eq!(generated, existing);
    }

    #[test]
    fn hash_collision_still_compares_logical_content() {
        let allocator = Allocator::new();
        let mut strings = StringPool::new_in(&allocator, "a b");
        let a = StringId::from(strings.source_slice(0, 1));
        let b = LogicalBytes {
            prefix: b"b",
            suffix: None,
        };
        strings.utf8_dedup.insert(
            InternEntry {
                hash: b.hash(),
                id: a,
            },
            (),
        );

        let canonical_b = strings.intern_utf8_range(strings.source_slice(2, 3));
        assert!(strings.utf8_eq_bytes(canonical_b.into(), b"b"));
        assert_eq!(strings.utf8_dedup.len(), 2);
    }
}
