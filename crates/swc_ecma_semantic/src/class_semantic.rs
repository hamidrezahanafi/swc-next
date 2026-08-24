use swc_next_allocator::{Allocator, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{Ast, ClassId, PropertyKey};

use crate::UsedNameState;

/// Semantic information collected for class transforms.
///
/// This data is intentionally separate from scopes and symbols: class
/// transforms may add private elements without introducing bindings in the
/// surrounding lexical scope.
pub struct ClassSemantic<'a> {
    allocator: &'a Allocator,
    enabled: bool,
    classes: ArenaVec<'a, Option<ClassInfo<'a>>>,
    class_stack: ArenaVec<'a, ClassId>,
}

impl std::fmt::Debug for ClassSemantic<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClassSemantic")
            .field("enabled", &self.enabled)
            .field("classes", &self.classes)
            .finish_non_exhaustive()
    }
}

/// Information associated with one class.
#[derive(Debug)]
pub struct ClassInfo<'a> {
    used_names: UsedNameState<'a>,
}

impl<'a> ClassSemantic<'a> {
    pub(crate) fn new(allocator: &'a Allocator, enabled: bool, owner_count: usize) -> Self {
        let mut classes = ArenaVec::with_capacity_in(owner_count, allocator);
        for _ in 0..owner_count {
            classes.push(None);
        }
        Self {
            allocator,
            enabled,
            classes,
            class_stack: ArenaVec::new_in(allocator),
        }
    }

    /// Whether class information was requested for this semantic analysis.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the information collected for `class`.
    ///
    /// This returns `None` when collection was disabled or the node is not a
    /// class from the analyzed tree.
    pub fn get(&self, ast: &Ast<'_>, class: ClassId) -> Option<&ClassInfo<'a>> {
        self.classes
            .get(ast.class_index(class))
            .and_then(Option::as_ref)
    }

    /// Allocates and reserves the next private name for `class`.
    ///
    /// The returned name does not include the leading `#`. Reserving and
    /// returning the name in one operation keeps generated class elements and
    /// semantic state synchronized.
    pub fn next_private_name(&mut self, ast: &Ast<'_>, class: ClassId) -> &str {
        assert!(
            self.enabled,
            "class semantic collection must be enabled before allocating names"
        );
        let info = self
            .classes
            .get_mut(ast.class_index(class))
            .and_then(Option::as_mut)
            .expect("class semantic entry must exist for an analyzed class");
        info.used_names.next_name("_", 2)
    }

    pub(crate) fn enter_class(&mut self, ast: &Ast<'_>, class: ClassId) {
        if !self.enabled {
            return;
        }
        let index = ast.class_index(class);
        debug_assert!(index < self.classes.len());
        let entry = &mut self.classes[index];
        debug_assert!(entry.is_none());
        *entry = Some(ClassInfo {
            used_names: UsedNameState::new(self.allocator),
        });
        self.class_stack.push(class);
    }

    pub(crate) fn leave_class(&mut self, class: ClassId) {
        if !self.enabled {
            return;
        }
        debug_assert_eq!(self.class_stack.pop(), Some(class));
    }

    pub(crate) fn record_private_key(&mut self, ast: &Ast<'_>, key: PropertyKey) {
        if !self.enabled {
            return;
        }
        let Some(class) = self.class_stack.last().copied() else {
            return;
        };
        let Some(private) = key.as_private_identifier(ast) else {
            return;
        };
        let name = ast.get_utf8(private.name(ast)).as_bytes();
        let info = self
            .classes
            .get_mut(ast.class_index(class))
            .and_then(Option::as_mut)
            .expect("current class semantic entry must exist");
        info.used_names.observe(name);
    }

    pub(crate) fn finish_collection(&self) {
        debug_assert!(self.class_stack.is_empty());
    }
}

impl ClassInfo<'_> {
    /// Returns whether a private name, without its leading `#`, is used.
    pub fn has_name(&self, name: &[u8]) -> bool {
        self.used_names.contains(name)
    }

    /// Iterates the used private names, without their leading `#`.
    pub fn used_names(&self) -> impl Iterator<Item = &[u8]> {
        self.used_names.names()
    }
}
