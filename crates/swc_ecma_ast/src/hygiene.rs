use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

/// A compact identifier for a chain of hygiene marks.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SyntaxContext(u32);

impl SyntaxContext {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn from_u32(raw: u32) -> Self {
        Self(raw)
    }
}

/// A unique hygiene mark.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mark(u32);

impl Mark {
    #[inline]
    pub const fn root() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn from_u32(raw: u32) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug)]
struct MarkData {
    parent: Mark,
}

#[derive(Clone, Copy, Debug)]
struct SyntaxContextData {
    outer_mark: Mark,
    previous: SyntaxContext,
}

/// Per-AST hygiene storage.
///
/// Unlike SWC's process-global hygiene tables, this storage is owned by an
/// [`Ast`](crate::Ast), isolating marks between concurrently built trees.
pub(crate) struct Hygiene<'a> {
    marks: ArenaVec<'a, MarkData>,
    syntax_contexts: ArenaVec<'a, SyntaxContextData>,
    markings: ArenaHashMap<'a, (SyntaxContext, Mark), SyntaxContext>,
    node_contexts: ArenaVec<'a, SyntaxContext>,
}

impl<'a> Hygiene<'a> {
    pub(crate) fn new_in(allocator: &'a Allocator) -> Self {
        let mut marks = ArenaVec::new_in(allocator);
        marks.push(MarkData {
            parent: Mark::root(),
        });

        let mut syntax_contexts = ArenaVec::new_in(allocator);
        syntax_contexts.push(SyntaxContextData {
            outer_mark: Mark::root(),
            previous: SyntaxContext::empty(),
        });

        Self {
            marks,
            syntax_contexts,
            markings: ArenaHashMap::new_in(allocator),
            node_contexts: ArenaVec::new_in(allocator),
        }
    }

    #[inline]
    pub(crate) fn truncate_nodes(&mut self, len: usize) {
        if len < self.node_contexts.len() {
            self.node_contexts.truncate(len);
        }
    }

    #[inline]
    pub(crate) fn node_context(&self, node: crate::NodeId) -> SyntaxContext {
        self.node_contexts
            .get(node.index())
            .copied()
            .unwrap_or_default()
    }

    #[inline]
    pub(crate) fn set_node_context(&mut self, node: crate::NodeId, context: SyntaxContext) {
        let index = node.index();
        if index >= self.node_contexts.len() {
            if context == SyntaxContext::empty() {
                return;
            }
            self.node_contexts.resize(index + 1, SyntaxContext::empty());
        }
        self.node_contexts[index] = context;
    }

    pub(crate) fn fresh_mark(&mut self, parent: Mark) -> Mark {
        assert!(parent.0 < self.marks.len() as u32);
        assert!(self.marks.len() < u32::MAX as usize);
        let mark = Mark(self.marks.len() as u32);
        self.marks.push(MarkData { parent });
        mark
    }

    pub(crate) fn is_descendant_of(&self, mut mark: Mark, ancestor: Mark) -> bool {
        while mark != ancestor {
            if mark == Mark::root() {
                return false;
            }
            mark = self.mark_parent(mark);
        }
        true
    }

    #[inline]
    pub(crate) fn mark_parent(&self, mark: Mark) -> Mark {
        self.marks[mark.0 as usize].parent
    }

    pub(crate) fn apply_mark(&mut self, context: SyntaxContext, mark: Mark) -> SyntaxContext {
        assert_ne!(mark, Mark::root());
        assert!((context.0 as usize) < self.syntax_contexts.len());
        assert!((mark.0 as usize) < self.marks.len());

        if let Some(&marked) = self.markings.get(&(context, mark)) {
            return marked;
        }

        assert!(self.syntax_contexts.len() < u32::MAX as usize);
        let marked = SyntaxContext(self.syntax_contexts.len() as u32);
        self.syntax_contexts.push(SyntaxContextData {
            outer_mark: mark,
            previous: context,
        });
        self.markings.insert((context, mark), marked);
        marked
    }

    #[inline]
    pub(crate) fn outer_mark(&self, context: SyntaxContext) -> Mark {
        self.syntax_contexts[context.0 as usize].outer_mark
    }

    #[inline]
    pub(crate) fn remove_mark(&self, context: &mut SyntaxContext) -> Mark {
        let data = self.syntax_contexts[context.0 as usize];
        *context = data.previous;
        data.outer_mark
    }
}
