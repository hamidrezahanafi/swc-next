use std::str;

use swc_next_allocator::{
    Allocator,
    hash_map::{Entry, HashMap as ArenaHashMap},
};
use swc_next_ecma_ast::{Ast, AstInternedUtf8};

/// Tracks interned names used in one collision domain.
///
/// Names use their canonical AST handles, avoiding a second owned copy in
/// semantic state. The map value is the next suffix to try for that name, so
/// repeated allocation does not rescan earlier candidates.
pub struct UsedNameState<'a> {
    names: ArenaHashMap<'a, AstInternedUtf8, u64>,
}

/// Reusable state for allocating a series of names with the same base.
///
/// The base is interned and looked up once. Subsequent allocations keep their
/// suffix cursor and candidate buffer here while [`UsedNameState`] remains the
/// source of truth for collisions.
pub struct GeneratedNameState {
    base_name: AstInternedUtf8,
    next_suffix: u64,
    return_base: bool,
    candidate: CandidateName,
}

impl std::fmt::Debug for UsedNameState<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UsedNameState")
            .field("names", &self.names)
            .finish()
    }
}

impl<'a> UsedNameState<'a> {
    /// Creates an empty name state backed by `allocator`.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            names: ArenaHashMap::new_in(allocator),
        }
    }

    /// Records an interned name and initializes its suffix cursor.
    pub fn observe(&mut self, name: AstInternedUtf8) {
        self.names.entry(name).or_insert(1);
    }

    /// Returns whether an interned name is observed or reserved.
    pub fn contains(&self, name: AstInternedUtf8) -> bool {
        self.names.contains_key(&name)
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name(ast, "_temp", 2)` produces
    /// `_temp2`.
    pub fn next_name(&mut self, ast: &mut Ast<'_>, base: &str, min_suffix: u64) -> AstInternedUtf8 {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let base_name = ast.intern_utf8(base);
        let mut suffix = match self.names.entry(base_name) {
            Entry::Occupied(entry) => (*entry.get()).max(min_suffix),
            Entry::Vacant(entry) => {
                entry.insert(min_suffix);
                return base_name;
            }
        };

        let mut candidate = CandidateName::new(base);
        loop {
            candidate.reset(base);
            candidate.append_u64(suffix);
            let next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            *self.names.get_mut(&base_name).expect("base name exists") = next_suffix;

            let candidate_name = ast.intern_utf8(candidate.as_str());
            match self.names.entry(candidate_name) {
                Entry::Occupied(_) => suffix = next_suffix,
                Entry::Vacant(entry) => {
                    entry.insert(min_suffix);
                    return candidate_name;
                }
            }
        }
    }

    /// Creates reusable allocation state for a series of names based on
    /// `base`.
    ///
    /// This is preferable to repeated [`Self::next_name`] calls when one
    /// transform allocates many names with the same base, because it avoids
    /// re-interning and looking up the base for every allocation.
    pub fn generated_name_state(
        &mut self,
        ast: &mut Ast<'_>,
        base: &str,
        min_suffix: u64,
    ) -> GeneratedNameState {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let base_name = ast.intern_utf8(base);
        let (next_suffix, return_base) = match self.names.entry(base_name) {
            Entry::Occupied(entry) => ((*entry.get()).max(min_suffix), false),
            Entry::Vacant(entry) => {
                entry.insert(min_suffix);
                (min_suffix, true)
            }
        };
        GeneratedNameState {
            base_name,
            next_suffix,
            return_base,
            candidate: CandidateName::new(base),
        }
    }

    /// Reserves and returns the next name from a reusable generated-name
    /// series.
    pub fn next_name_from_state(
        &mut self,
        ast: &mut Ast<'_>,
        state: &mut GeneratedNameState,
    ) -> AstInternedUtf8 {
        if state.return_base {
            state.return_base = false;
            return state.base_name;
        }

        loop {
            let suffix = state.next_suffix;
            state.next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            let base = ast.get_interned_utf8(state.base_name);
            state.candidate.reset(base);
            state.candidate.append_u64(suffix);

            let candidate_name = ast.intern_utf8(state.candidate.as_str());
            match self.names.entry(candidate_name) {
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(1);
                    return candidate_name;
                }
            }
        }
    }

    /// Iterates all interned names currently observed or reserved.
    pub fn names(&self) -> impl Iterator<Item = AstInternedUtf8> + '_ {
        self.names.keys().copied()
    }
}

const INLINE_NAME_CAPACITY: usize = 32;

enum CandidateName {
    Inline {
        bytes: [u8; INLINE_NAME_CAPACITY],
        len: u8,
    },
    Heap(Vec<u8>),
}

impl CandidateName {
    fn new(base: &str) -> Self {
        if base.len() + 20 <= INLINE_NAME_CAPACITY {
            Self::Inline {
                bytes: [0; INLINE_NAME_CAPACITY],
                len: 0,
            }
        } else {
            Self::Heap(Vec::with_capacity(base.len() + 20))
        }
    }

    fn reset(&mut self, base: &str) {
        match self {
            Self::Inline { bytes, len } => {
                bytes[..base.len()].copy_from_slice(base.as_bytes());
                *len = base.len() as u8;
            }
            Self::Heap(output) => {
                output.clear();
                output.extend_from_slice(base.as_bytes());
            }
        }
    }

    fn append_u64(&mut self, mut value: u64) {
        let mut digits = [0; 20];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.extend_from_slice(&digits[start..]);
    }

    fn extend_from_slice(&mut self, input: &[u8]) {
        match self {
            Self::Inline { bytes, len } => {
                let start = usize::from(*len);
                let end = start + input.len();
                bytes[start..end].copy_from_slice(input);
                *len = end as u8;
            }
            Self::Heap(output) => output.extend_from_slice(input),
        }
    }

    fn as_str(&self) -> &str {
        let bytes = match self {
            Self::Inline { bytes, len } => &bytes[..usize::from(*len)],
            Self::Heap(output) => output.as_slice(),
        };
        // SAFETY: The buffer consists of a UTF-8 `base` followed by ASCII digits.
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;

    use super::UsedNameState;

    #[test]
    fn allocates_interned_generated_names() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = ast.intern_utf8("_temp");
        names.observe(base);

        let second = names.next_name(&mut ast, "_temp", 2);
        let third = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(ast.get_interned_utf8(second), "_temp2");
        assert_eq!(ast.get_interned_utf8(third), "_temp3");
    }

    #[test]
    fn skips_interned_collisions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = ast.intern_utf8("_temp");
        names.observe(base);
        let collision = ast.intern_utf8("_temp2");
        names.observe(collision);

        let name = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(ast.get_interned_utf8(name), "_temp3");
    }

    #[test]
    fn reuses_generated_name_state() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = ast.intern_utf8("_temp");
        names.observe(base);

        let mut state = names.generated_name_state(&mut ast, "_temp", 2);
        let second = names.next_name_from_state(&mut ast, &mut state);
        let third = names.next_name_from_state(&mut ast, &mut state);

        assert_eq!(ast.get_interned_utf8(second), "_temp2");
        assert_eq!(ast.get_interned_utf8(third), "_temp3");
    }

    #[test]
    fn generated_name_states_share_collision_domain() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = ast.intern_utf8("_temp");
        names.observe(base);

        let mut first_state = names.generated_name_state(&mut ast, "_temp", 2);
        let mut second_state = names.generated_name_state(&mut ast, "_temp", 2);
        let second = names.next_name_from_state(&mut ast, &mut first_state);
        let third = names.next_name_from_state(&mut ast, &mut second_state);

        assert_eq!(ast.get_interned_utf8(second), "_temp2");
        assert_eq!(ast.get_interned_utf8(third), "_temp3");
    }
}
