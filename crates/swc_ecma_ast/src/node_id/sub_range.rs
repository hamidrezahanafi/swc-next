use std::marker::PhantomData;

use crate::{
    Ast,
    node_id::{ExtraDataId, ExtraDataValue},
};

/// A typed range of values in the AST extra-data arena.
///
/// The invariant brand identifies the AST owning the range, independently of
/// the element type. Empty ranges are valid in every AST.
#[repr(C)]
#[derive(Debug, Hash)]
pub struct SubRange<'id, T> {
    pub(crate) start: ExtraDataId,
    pub(crate) end: ExtraDataId,
    _phantom: PhantomData<(T, generativity::Id<'id>)>,
}

impl<'id, T> Clone for SubRange<'id, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, T> Copy for SubRange<'id, T> {}

impl<'id, T> SubRange<'id, T> {
    #[inline]
    pub(crate) fn raw(self) -> super::RawSubRange {
        super::RawSubRange {
            start: self.start,
            end: self.end,
        }
    }

    #[inline]
    pub(crate) fn from_raw(range: super::RawSubRange) -> Self {
        Self::new(range.start, range.end)
    }

    pub(crate) fn new(start: ExtraDataId, end: ExtraDataId) -> Self {
        Self {
            start,
            end,
            _phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            start: ExtraDataId::from_raw(0),
            end: ExtraDataId::from_raw(0),
            _phantom: PhantomData,
        }
    }

    pub fn len(self) -> usize {
        self.end.index() - self.start.index()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Similar to `Vec::remove(0)`.
    pub fn remove_first(&mut self) -> NodeExtraDataId<'id, T> {
        assert!(!self.is_empty());
        let start = self.start;
        self.start += 1;
        NodeExtraDataId {
            inner: start,
            _phantom: PhantomData,
        }
    }

    pub fn get(self, index: usize) -> Option<NodeExtraDataId<'id, T>> {
        if index < self.len() {
            let id = self.start + index;
            return Some(NodeExtraDataId {
                inner: id,
                _phantom: PhantomData,
            });
        }
        None
    }

    pub fn first(self) -> Option<NodeExtraDataId<'id, T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.start,
            _phantom: PhantomData,
        })
    }

    pub fn last(self) -> Option<NodeExtraDataId<'id, T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.end - 1,
            _phantom: PhantomData,
        })
    }

    pub fn iter(self) -> SubRangeIterator<'id, T> {
        SubRangeIterator {
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
        let original_end = self.end;
        self.end = at;
        Self {
            start: at,
            end: original_end,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn replace_slot<'a>(self, ast: &mut Ast<'a, 'id>, at: NodeExtraDataId<'id, T>, new: T)
    where
        T: ExtraDataValue<'id>,
    {
        assert!(
            self.start <= at.inner && at.inner < self.end,
            "slot is outside this range"
        );
        // SAFETY: The slot and range belong to this AST and the slot stores `T`.
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(at.inner.index()) = new.into_extra_data();
        };
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a, 'id>, index: usize) -> Option<T>
    where
        T: ExtraDataValue<'id>,
    {
        self.get(index).map(|id| ast.get_node_in_sub_range(id))
    }
}

pub struct SubRangeIterator<'id, T> {
    cur: ExtraDataId,
    end: ExtraDataId,
    _phantom: PhantomData<(T, generativity::Id<'id>)>,
}

impl<'id, T> Iterator for SubRangeIterator<'id, T> {
    type Item = NodeExtraDataId<'id, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        let next = self.cur;
        debug_assert!(next < self.end);

        self.cur = ExtraDataId::from_usize_unchecked(self.cur.index().wrapping_add(1));
        debug_assert!(self.cur <= self.end);

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

impl<'id, T> DoubleEndedIterator for SubRangeIterator<'id, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            return None;
        }

        self.end = ExtraDataId::from_usize_unchecked(self.end.index().wrapping_sub(1));
        debug_assert!(self.cur <= self.end);

        let next = self.end;
        debug_assert!(next >= self.cur);

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

/// A typed index for a value inside a [`SubRange`].
#[derive(Debug, Clone, Copy)]
pub struct NodeExtraDataId<'id, T> {
    inner: ExtraDataId,
    _phantom: PhantomData<(T, generativity::Id<'id>)>,
}

impl<'id, 'a> Ast<'a, 'id> {
    pub fn get_node_in_sub_range<T: ExtraDataValue<'id>>(&self, id: NodeExtraDataId<'id, T>) -> T {
        // SAFETY: A typed sub-range index identifies a slot containing `T`.
        unsafe { T::from_extra_data(self.extra_data.as_slice().get_unchecked(id.inner.index())) }
    }
}
