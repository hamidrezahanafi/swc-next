use std::{marker::PhantomData, ops::Deref};

use crate::{Ast, node_id::ListDataCompact};

pub const EMPTY_SUB_RANGE: SubRange = SubRange {
    start: ListDataId::from_raw(0),
    end: ListDataId::from_raw(0),
};

oxc_index::define_index_type! {
    pub struct ListDataId = u32;
}

/// [SubRange] is a cheap way to represent an undetermined range of nodes in an arena, just like a [std::vec::Vec].
///
/// It only contains the start and end of the range in the AST list-data table.
#[derive(Debug, Clone, Copy, Hash)]
pub struct SubRange {
    pub start: ListDataId,
    pub end: ListDataId,
}

impl SubRange {
    /// # Safety:
    /// 1. The caller should make sure that the nodes in the range are all of type `T`.
    pub(crate) unsafe fn cast_to_typed<T>(self) -> TypedSubRange<T> {
        TypedSubRange {
            inner: self,
            _phantom: PhantomData,
        }
    }
}

/// A typed sub range of nodes in an arena.
///
/// See [SubRange]
#[derive(Debug)]
pub struct TypedSubRange<T> {
    pub(crate) inner: SubRange,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> Clone for TypedSubRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedSubRange<T> {}

impl<T> TypedSubRange<T> {
    pub fn empty() -> Self {
        Self {
            inner: EMPTY_SUB_RANGE,
            _phantom: PhantomData,
        }
    }

    pub fn len(self) -> usize {
        self.end.index() - self.start.index()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Similar to `Vec::remove(0)`
    pub fn remove_first(&mut self) -> NodeListDataId<T> {
        assert!(!self.is_empty());
        let start = self.start;
        self.inner.start += 1;
        NodeListDataId {
            inner: start,
            _phantom: PhantomData,
        }
    }

    pub fn get(self, index: usize) -> Option<NodeListDataId<T>> {
        if index < self.len() {
            let id = self.start + index;
            return Some(NodeListDataId {
                inner: id,
                _phantom: PhantomData,
            });
        }
        None
    }

    pub fn first(self) -> Option<NodeListDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeListDataId {
            inner: self.start,
            _phantom: PhantomData,
        })
    }

    pub fn last(self) -> Option<NodeListDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeListDataId {
            inner: self.end - 1,
            _phantom: PhantomData,
        })
    }

    pub fn iter(self) -> TypedSubRangeIterator<T> {
        TypedSubRangeIterator {
            cur: self.start,
            end: self.end,
            _phantom: PhantomData,
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        if at > self.len() {
            panic!(
                "Split point {} is larger than the length {}",
                at,
                self.len()
            );
        }

        let at = self.start + at;
        let original_end = self.inner.end;

        self.inner.end = at;
        TypedSubRange {
            inner: SubRange {
                start: at,
                end: original_end,
            },
            _phantom: PhantomData,
        }
    }
}

impl<T> TypedSubRange<T> {
    #[inline]
    pub fn replace_slot<'a>(self, ast: &mut Ast<'a>, at: NodeListDataId<T>, new: T)
    where
        T: ListDataCompact,
    {
        ast.list_data[at.inner.index()] = new.to_list_data();
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a>, index: usize) -> Option<T>
    where
        T: ListDataCompact,
    {
        self.get(index).map(|id| ast.get_node_in_sub_range(id))
    }
}

impl<T> Deref for TypedSubRange<T> {
    type Target = SubRange;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct TypedSubRangeIterator<T> {
    cur: ListDataId,
    end: ListDataId,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for TypedSubRangeIterator<T> {
    type Item = NodeListDataId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        let next = self.cur;
        debug_assert!(next < self.end);

        self.cur = ListDataId::from_usize_unchecked(self.cur.index().wrapping_add(1));
        debug_assert!(self.cur <= self.end);

        Some(NodeListDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

impl<T> DoubleEndedIterator for TypedSubRangeIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        self.end = ListDataId::from_usize_unchecked(self.end.index().wrapping_sub(1));
        debug_assert!(self.cur <= self.end);

        let next = self.end;
        debug_assert!(next >= self.cur);

        Some(NodeListDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

/// This is a typed index into the AST list-data table.
/// It's type safe and can be used to access nodes in the AST context.
#[derive(Debug, Clone, Copy)]
pub struct NodeListDataId<T> {
    inner: ListDataId,
    _phantom: PhantomData<T>,
}

impl<'a> Ast<'a> {
    pub fn get_node_in_sub_range<T: ListDataCompact>(&self, id: NodeListDataId<T>) -> T {
        // Safety: `NodeListDataId<T>` should always be valid and typed for `T`.
        unsafe { T::from_list_data(*self.list_data.as_slice().get_unchecked(id.inner.index())) }
    }
}
