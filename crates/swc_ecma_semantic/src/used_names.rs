use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

#[derive(Debug, Default, Clone, Copy)]
struct NameState {
    occurrences: u32,
    next_suffix: u64,
    reserved: bool,
}

impl NameState {
    fn is_used(self) -> bool {
        self.occurrences != 0 || self.reserved
    }
}

/// Tracks names used in one collision domain.
///
/// Source occurrences are reference-counted so transforms that replace AST
/// nodes can keep the state synchronized with [`observe`](Self::observe) and
/// [`unobserve`](Self::unobserve). Generated names are permanently reserved,
/// and each base name retains its next suffix so repeated allocation does not
/// rescan earlier candidates.
pub struct UsedNameState<'a> {
    allocator: &'a Allocator,
    names: ArenaHashMap<'a, ArenaVec<'a, u8>, NameState>,
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
            allocator,
            names: ArenaHashMap::new_in(allocator),
        }
    }

    /// Records one source occurrence of `name`.
    pub fn observe(&mut self, name: &[u8]) {
        let state = self.state_mut(name);
        state.occurrences = state
            .occurrences
            .checked_add(1)
            .expect("used name occurrence count overflow");
    }

    /// Removes one previously observed source occurrence of `name`.
    pub fn unobserve(&mut self, name: &[u8]) {
        let state = self
            .names
            .get_mut(name)
            .expect("cannot remove a name that was not observed");
        state.occurrences = state
            .occurrences
            .checked_sub(1)
            .expect("cannot remove a name that was not observed");
    }

    /// Permanently reserves `name` for generated syntax.
    ///
    /// Returns `true` when the name was unused before this call.
    pub fn reserve(&mut self, name: &[u8]) -> bool {
        let state = self.state_mut(name);
        let was_unused = !state.is_used();
        state.reserved = true;
        was_unused
    }

    /// Releases a name previously reserved for generated syntax.
    pub fn unreserve(&mut self, name: &[u8]) {
        let state = self
            .names
            .get_mut(name)
            .expect("cannot release a name that was not reserved");
        assert!(
            state.reserved,
            "cannot release a name that was not reserved"
        );
        state.reserved = false;
    }

    /// Returns whether `name` is observed or reserved.
    pub fn contains(&self, name: &[u8]) -> bool {
        self.names.get(name).is_some_and(|state| state.is_used())
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name("_temp", 2)` produces `_temp2`.
    pub fn next_name(&mut self, base: &str, min_suffix: u64) -> &str {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        if !self.contains(base.as_bytes()) {
            self.reserve(base.as_bytes());
            self.state_mut(base.as_bytes()).next_suffix = min_suffix;
            return self.stored_name(base.as_bytes());
        }

        let mut suffix = self
            .names
            .get(base.as_bytes())
            .map_or(min_suffix, |state| state.next_suffix.max(min_suffix));
        loop {
            let candidate = format!("{base}{suffix}");
            if !self.contains(candidate.as_bytes()) {
                self.reserve(candidate.as_bytes());
                self.state_mut(base.as_bytes()).next_suffix = suffix
                    .checked_add(1)
                    .expect("generated name suffix overflow");
                return self.stored_name(candidate.as_bytes());
            }
            suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
        }
    }

    /// Iterates all currently observed or reserved names.
    pub fn names(&self) -> impl Iterator<Item = &[u8]> {
        self.names
            .iter()
            .filter(|(_, state)| state.is_used())
            .map(|(name, _)| name.as_slice())
    }

    fn state_mut(&mut self, name: &[u8]) -> &mut NameState {
        if !self.names.contains_key(name) {
            let mut owned = ArenaVec::with_capacity_in(name.len(), self.allocator);
            owned.extend_from_slice(name);
            self.names.insert(owned, NameState::default());
        }
        self.names
            .get_mut(name)
            .expect("used name state must exist after insertion")
    }

    fn stored_name(&self, name: &[u8]) -> &str {
        let (stored, _) = self
            .names
            .get_key_value(name)
            .expect("used name must exist after reservation");
        std::str::from_utf8(stored).expect("generated names must be UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;

    use super::UsedNameState;

    #[test]
    fn observes_removes_and_allocates_names() {
        let allocator = Allocator::new();
        let mut names = UsedNameState::new(&allocator);

        names.observe(b"_temp");
        names.observe(b"_temp");
        names.observe(b"_temp2");
        assert_eq!(names.next_name("_temp", 2), "_temp3");
        assert_eq!(names.next_name("_temp", 2), "_temp4");
        assert_eq!(names.names().count(), 4);

        names.unobserve(b"_temp");
        assert!(names.contains(b"_temp"));
        names.unobserve(b"_temp");
        assert!(!names.contains(b"_temp"));
        assert!(!names.reserve(b"_temp3"));
        names.unreserve(b"_temp3");
        assert!(!names.contains(b"_temp3"));
    }
}
