use std::{fmt, marker::PhantomData};

use oxc_index::nonmax::NonMaxU32;

use super::sealed::AstNode as SealedAstNode;

oxc_index::define_index_type! {
    #[repr(transparent)]
    pub(crate) struct RawNodeId = u32;
    MAX_INDEX = (u32::MAX - 1) as usize;
}

/// A node index belonging to the AST identified by `'id`.
/// Obtain IDs from typed nodes or [`crate::Ast::node_id_at`].
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId<'id> {
    index: NonMaxU32,
    brand: PhantomData<fn(&'id ()) -> &'id ()>,
}

impl<'id> NodeId<'id> {
    #[inline]
    pub const fn index(self) -> usize {
        self.index.get() as usize
    }

    #[inline]
    pub(crate) const fn from_raw(raw: RawNodeId) -> Self {
        Self::from_raw_index(raw.index() as u32)
    }

    #[inline]
    pub(crate) const fn from_raw_index(index: u32) -> Self {
        Self {
            index: match NonMaxU32::new(index) {
                Some(index) => index,
                None => panic!("invalid node index"),
            },
            brand: PhantomData,
        }
    }

    #[inline]
    pub(crate) const fn raw(self) -> RawNodeId {
        RawNodeId::from_raw(self.index.get())
    }
}

impl fmt::Debug for NodeId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeId").field(&self.index()).finish()
    }
}

/// A generated AST handle. Implementations are restricted to this crate.
pub trait AstNode<'id>: SealedAstNode {
    fn node_id(self) -> NodeId<'id>;

    /// Creates a typed wrapper after the caller has verified the node kind.
    ///
    /// # Safety
    ///
    /// `node_id` must identify a live node of the type represented by `Self`
    /// in the AST branded by `'id`.
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self
    where
        Self: Sized;
}
