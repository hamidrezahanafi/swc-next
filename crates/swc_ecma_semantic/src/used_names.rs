use swc_next_allocator::{
    Allocator, hash_map::HashMap as ArenaHashMap, hash_set::HashSet as ArenaHashSet,
};
use swc_next_ecma_ast::{Ast, AstInternedUtf8, GeneratedUtf8State};

/// Tracks interned names used in one collision domain.
///
/// Names use their canonical AST handles, avoiding a second owned copy in
/// semantic state. Suffix cursors are stored separately and only allocated for
/// bases that actually generate colliding names.
pub struct UsedNameState<'a> {
    names: ArenaHashSet<'a, AstInternedUtf8>,
    next_suffixes: ArenaHashMap<'a, AstInternedUtf8, u64>,
}

/// Reusable state for allocating a series of names with the same base.
///
/// The base is interned and looked up once. Subsequent allocations keep their
/// suffix cursor and cached physical prefix here while [`UsedNameState`]
/// remains the source of truth for collisions.
pub struct GeneratedNameState {
    strings: GeneratedUtf8State,
    next_suffix: u64,
    return_base: bool,
}

impl std::fmt::Debug for UsedNameState<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UsedNameState")
            .field("names", &self.names)
            .field("next_suffixes", &self.next_suffixes)
            .finish()
    }
}

impl<'a> UsedNameState<'a> {
    /// Creates an empty name state backed by `allocator`.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            names: ArenaHashSet::new_in(allocator),
            next_suffixes: ArenaHashMap::new_in(allocator),
        }
    }

    /// Records an interned name and initializes its suffix cursor.
    pub fn observe(&mut self, name: AstInternedUtf8) {
        self.names.insert(name);
    }

    /// Returns whether an interned name is observed or reserved.
    pub fn contains(&self, name: AstInternedUtf8) -> bool {
        self.names.contains(&name)
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name(ast, "_temp", 2)` produces
    /// `_temp2`.
    pub fn next_name(&mut self, ast: &mut Ast<'_>, base: &str, min_suffix: u64) -> AstInternedUtf8 {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let mut strings = ast.generated_utf8_state(base);
        let base_name = Ast::generated_utf8_base(&strings);
        if self.names.insert(base_name) {
            return base_name;
        }
        let mut suffix = self
            .next_suffixes
            .get(&base_name)
            .copied()
            .unwrap_or(min_suffix)
            .max(min_suffix);

        loop {
            let next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            let candidate = ast.intern_generated_name_from_state(&mut strings, suffix);
            if self.names.insert(candidate) {
                self.next_suffixes.insert(base_name, next_suffix);
                return candidate;
            }
            suffix = next_suffix;
        }
    }

    /// Creates reusable allocation state for a series of names based on
    /// `base`.
    pub fn generated_name_state(
        &mut self,
        ast: &mut Ast<'_>,
        base: &str,
        min_suffix: u64,
    ) -> GeneratedNameState {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        let strings = ast.generated_utf8_state(base);
        let base_name = Ast::generated_utf8_base(&strings);
        let return_base = self.names.insert(base_name);
        GeneratedNameState {
            strings,
            next_suffix: min_suffix,
            return_base,
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
            return Ast::generated_utf8_base(&state.strings);
        }

        loop {
            let suffix = state.next_suffix;
            state.next_suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
            let candidate = ast.intern_generated_name_from_state(&mut state.strings, suffix);
            if self.names.insert(candidate) {
                return candidate;
            }
        }
    }

    /// Iterates all interned names currently observed or reserved.
    pub fn names(&self) -> impl Iterator<Item = AstInternedUtf8> + '_ {
        self.names.iter().copied()
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
        names.observe(ast.intern_utf8("_temp"));

        let second = names.next_name(&mut ast, "_temp", 2);
        let third = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn skips_interned_collisions() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        names.observe(ast.intern_utf8("_temp"));
        names.observe(ast.intern_utf8("_temp2"));

        let name = names.next_name(&mut ast, "_temp", 2);

        assert_eq!(name, ast.intern_utf8("_temp3"));
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

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn generated_name_states_share_collision_domain() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        names.observe(ast.intern_utf8("_temp"));

        let mut first_state = names.generated_name_state(&mut ast, "_temp", 2);
        let mut second_state = names.generated_name_state(&mut ast, "_temp", 2);
        let second = names.next_name_from_state(&mut ast, &mut first_state);
        let third = names.next_name_from_state(&mut ast, &mut second_state);

        assert_eq!(second, ast.intern_utf8("_temp2"));
        assert_eq!(third, ast.intern_utf8("_temp3"));
    }

    #[test]
    fn allocates_hexadecimal_suffix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        names.observe(ast.intern_utf8("_temp"));

        let mut state = names.generated_name_state(&mut ast, "_temp", 0xf);
        let f = names.next_name_from_state(&mut ast, &mut state);
        let ten = names.next_name_from_state(&mut ast, &mut state);
        let eleven = names.next_name_from_state(&mut ast, &mut state);

        assert_eq!(f, ast.intern_utf8("_tempf"));
        assert_eq!(ten, ast.intern_utf8("_temp10"));
        assert_eq!(eleven, ast.intern_utf8("_temp11"));
    }

    #[test]
    fn crosses_hexadecimal_prefix_blocks() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        names.observe(ast.intern_utf8("_temp"));

        let mut one_f_state = names.generated_name_state(&mut ast, "_temp", 0x1f);
        let one_f = names.next_name_from_state(&mut ast, &mut one_f_state);
        let twenty = names.next_name_from_state(&mut ast, &mut one_f_state);
        assert_eq!(one_f, ast.intern_utf8("_temp1f"));
        assert_eq!(twenty, ast.intern_utf8("_temp20"));

        names.observe(ast.intern_utf8("_other"));
        let mut ff_state = names.generated_name_state(&mut ast, "_other", 0xff);
        let ff = names.next_name_from_state(&mut ast, &mut ff_state);
        let hundred = names.next_name_from_state(&mut ast, &mut ff_state);
        assert_eq!(ff, ast.intern_utf8("_otherff"));
        assert_eq!(hundred, ast.intern_utf8("_other100"));
    }

    #[test]
    fn supports_unicode_long_and_hex_ending_bases() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("", &allocator);
        let mut names = UsedNameState::new(&allocator);
        let base = "变量abcdef0123456789abcdef0123456789f";
        names.observe(ast.intern_utf8(base));

        let generated = names.next_name(&mut ast, base, 0x10);
        assert_eq!(generated, ast.intern_utf8(&format!("{base}10")));
    }
}
