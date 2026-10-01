use std::marker::PhantomData;

use crate::{
    Ast,
    node_id::{ExtraDataId, ExtraDataValue},
};

/// A typed range of values in the AST extra-data arena.
///
/// It only contains the start and length of the range in the extra-data arena.
#[repr(C)]
#[derive(Debug, Hash)]
pub struct SubRange<T> {
    pub(crate) start: ExtraDataId,
    pub(crate) len: u32,
    _phantom: PhantomData<T>,
}

impl<T> Clone for SubRange<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SubRange<T> {}

impl<T> SubRange<T> {
    #[inline]
    pub(crate) const fn new(start: ExtraDataId, len: u32) -> Self {
        Self {
            start,
            len,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn empty() -> Self {
        Self {
            start: ExtraDataId::from_raw(0),
            len: 0,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get(self, index: usize) -> Option<NodeExtraDataId<T>> {
        if index < self.len() {
            let id = self.start + index;
            return Some(NodeExtraDataId {
                inner: id,
                _phantom: PhantomData,
            });
        }
        None
    }

    #[inline]
    pub fn first(self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.start,
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn last(self) -> Option<NodeExtraDataId<T>> {
        if self.is_empty() {
            return None;
        }

        Some(NodeExtraDataId {
            inner: self.start + (self.len() - 1),
            _phantom: PhantomData,
        })
    }

    #[inline]
    pub fn iter(self) -> SubRangeIterator<T> {
        SubRangeIterator {
            cur: self.start,
            len: self.len,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn replace_slot<'a>(self, ast: &mut Ast<'a>, at: NodeExtraDataId<T>, new: T)
    where
        T: ExtraDataValue,
    {
        // SAFETY: A typed sub-range index identifies a slot assigned to `T`.
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(at.inner.index()) = new.into_extra_data();
        };
    }

    #[inline]
    pub fn get_node<'a>(self, ast: &Ast<'a>, index: usize) -> Option<T>
    where
        T: ExtraDataValue,
    {
        self.get(index).map(|id| ast.get_node_in_sub_range(id))
    }
}

pub struct SubRangeIterator<T> {
    cur: ExtraDataId,
    len: u32,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for SubRangeIterator<T> {
    type Item = NodeExtraDataId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let next = self.cur;
        self.cur = ExtraDataId::from_usize_unchecked(self.cur.index().wrapping_add(1));
        self.len -= 1;

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

impl<T> DoubleEndedIterator for SubRangeIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let next = self.cur + self.len as usize;

        Some(NodeExtraDataId {
            inner: next,
            _phantom: PhantomData,
        })
    }
}

/// A typed index for a value inside a [`SubRange`].
#[derive(Debug, Clone, Copy)]
pub struct NodeExtraDataId<T> {
    inner: ExtraDataId,
    _phantom: PhantomData<T>,
}

impl<'a> Ast<'a> {
    pub fn get_node_in_sub_range<T: ExtraDataValue>(&self, id: NodeExtraDataId<T>) -> T {
        // SAFETY: A typed sub-range index identifies a slot containing `T`.
        unsafe { T::from_extra_data(self.extra_data.as_slice().get_unchecked(id.inner.index())) }
    }
}
