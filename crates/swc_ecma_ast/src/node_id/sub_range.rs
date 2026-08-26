use std::{marker::PhantomData, ops::Deref};

use crate::{Ast, AstEdge, AstId};

oxc_index::define_index_type! {
    pub struct NodeEdgeIndex = u32;
}

pub const EMPTY_NODE_RANGE: NodeRange = NodeRange {
    start: NodeEdgeIndex::from_raw(0),
    end: NodeEdgeIndex::from_raw(0),
};

#[derive(Debug, Clone, Copy, Hash)]
pub struct NodeRange {
    pub start: NodeEdgeIndex,
    pub end: NodeEdgeIndex,
}

impl NodeRange {
    pub(crate) unsafe fn cast_to_typed<T>(self) -> TypedNodeRange<T> {
        TypedNodeRange {
            inner: self,
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct TypedNodeRange<T> {
    pub(crate) inner: NodeRange,
    pub(crate) marker: PhantomData<T>,
}

impl<T> Clone for TypedNodeRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedNodeRange<T> {}

impl<T> TypedNodeRange<T> {
    pub fn empty() -> Self {
        Self {
            inner: EMPTY_NODE_RANGE,
            marker: PhantomData,
        }
    }

    pub fn len(self) -> usize {
        self.end.index() - self.start.index()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn remove_first(&mut self) -> NodeEdgeId<T> {
        assert!(!self.is_empty());
        let start = self.start;
        self.inner.start += 1;
        NodeEdgeId {
            inner: start,
            marker: PhantomData,
        }
    }

    pub fn get(self, index: usize) -> Option<NodeEdgeId<T>> {
        (index < self.len()).then(|| NodeEdgeId {
            inner: self.start + index,
            marker: PhantomData,
        })
    }

    pub fn first(self) -> Option<NodeEdgeId<T>> {
        self.get(0)
    }

    pub fn last(self) -> Option<NodeEdgeId<T>> {
        (!self.is_empty()).then(|| NodeEdgeId {
            inner: self.end - 1,
            marker: PhantomData,
        })
    }

    pub fn iter(self) -> TypedNodeRangeIterator<T> {
        TypedNodeRangeIterator {
            cur: self.start,
            end: self.end,
            marker: PhantomData,
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(
            at <= self.len(),
            "split point {at} is larger than length {}",
            self.len()
        );
        let at = self.start + at;
        let original_end = self.inner.end;
        self.inner.end = at;
        Self {
            inner: NodeRange {
                start: at,
                end: original_end,
            },
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn replace_slot(self, ast: &mut Ast<'_>, at: NodeEdgeId<T>, new: T)
    where
        T: AstEdge,
    {
        ast.node_edges[at.inner.index()] = new.to_ast_id();
    }

    #[inline]
    pub fn get_node(self, ast: &Ast<'_>, index: usize) -> Option<T>
    where
        T: AstEdge,
    {
        self.get(index).map(|id| ast.get_node_in_sub_range(id))
    }
}

impl<T> Deref for TypedNodeRange<T> {
    type Target = NodeRange;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct TypedNodeRangeIterator<T> {
    cur: NodeEdgeIndex,
    end: NodeEdgeIndex,
    marker: PhantomData<T>,
}

impl<T> Iterator for TypedNodeRangeIterator<T> {
    type Item = NodeEdgeId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }
        let next = self.cur;
        self.cur = NodeEdgeIndex::from_usize_unchecked(self.cur.index().wrapping_add(1));
        Some(NodeEdgeId {
            inner: next,
            marker: PhantomData,
        })
    }
}

impl<T> DoubleEndedIterator for TypedNodeRangeIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }
        self.end = NodeEdgeIndex::from_usize_unchecked(self.end.index().wrapping_sub(1));
        Some(NodeEdgeId {
            inner: self.end,
            marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeEdgeId<T> {
    inner: NodeEdgeIndex,
    marker: PhantomData<T>,
}

impl Ast<'_> {
    #[inline]
    pub fn replace_edge<T: AstEdge>(&mut self, id: NodeEdgeId<T>, old: AstId, new: AstId) {
        let edge = &mut self.node_edges[id.inner.index()];
        assert_eq!(
            *edge, old,
            "AST parent edge no longer points to the expected node"
        );
        *edge = new;
    }

    pub fn get_node_in_sub_range<T: AstEdge>(&self, id: NodeEdgeId<T>) -> T {
        let ast_id = unsafe { *self.node_edges.as_slice().get_unchecked(id.inner.index()) };
        unsafe { T::from_edge_unchecked(ast_id) }
    }
}

pub type SubRange = NodeRange;
pub type TypedSubRange<T> = TypedNodeRange<T>;
