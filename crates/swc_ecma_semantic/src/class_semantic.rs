use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{Ast, AstUtf8, NodeId, PropertyKey};

use crate::UsedNameState;

/// Semantic information collected for class transforms.
///
/// This data is intentionally separate from scopes and symbols: class
/// transforms may add private elements without introducing bindings in the
/// surrounding lexical scope.
pub struct ClassSemantic<'a> {
    allocator: &'a Allocator,
    enabled: bool,
    classes: ArenaHashMap<'a, NodeId, ClassInfo<'a>>,
    class_stack: ArenaVec<'a, NodeId>,
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
    pub(crate) fn new(allocator: &'a Allocator, enabled: bool) -> Self {
        Self {
            allocator,
            enabled,
            classes: ArenaHashMap::new_in(allocator),
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
    pub fn get(&self, class: NodeId) -> Option<&ClassInfo<'a>> {
        self.classes.get(&class)
    }

    /// Allocates and reserves the next private name for `class`.
    ///
    /// The returned name does not include the leading `#`. Reserving and
    /// returning the name in one operation keeps generated class elements and
    /// semantic state synchronized.
    pub fn next_private_name(&mut self, ast: &mut Ast<'_>, class: NodeId) -> AstUtf8 {
        assert!(
            self.enabled,
            "class semantic collection must be enabled before allocating names"
        );
        let info = self
            .classes
            .get_mut(&class)
            .expect("class semantic entry must exist for an analyzed class");
        let name = info.used_names.next_name("_", 2);
        ast.add_utf8(name)
    }

    pub(crate) fn enter_class(&mut self, class: NodeId) {
        if !self.enabled {
            return;
        }
        let previous = self.classes.insert(
            class,
            ClassInfo {
                used_names: UsedNameState::new(self.allocator),
            },
        );
        debug_assert!(previous.is_none());
        self.class_stack.push(class);
    }

    pub(crate) fn leave_class(&mut self, class: NodeId) {
        if !self.enabled {
            return;
        }
        debug_assert_eq!(self.class_stack.pop(), Some(class));
    }

    pub(crate) fn record_private_key(&mut self, ast: &Ast, key: PropertyKey) {
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
            .get_mut(&class)
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
