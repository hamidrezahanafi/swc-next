use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use oxc_index::Idx;

use crate::Span;

const MAX_INDEX: u32 = u32::MAX - 1;

pub trait IdDomain: 'static {}

pub enum CommonDomain {}

impl IdDomain for CommonDomain {}

pub enum ClassDomain {}

impl IdDomain for ClassDomain {}

/// A statically typed, untagged AST index.
///
/// NodeId addresses any common node. ClassId addresses a common Class owner
/// node whose payload points into the dense Vec<Class>. The marker prevents
/// accidentally indexing the wrong storage without spending bits on a tag.
#[repr(transparent)]
pub struct Id<D: IdDomain> {
    index: u32,
    marker: PhantomData<fn(D) -> D>,
}

impl<D: IdDomain> Id<D> {
    #[inline]
    pub const fn from_raw(index: u32) -> Self {
        assert!(index <= MAX_INDEX, "AST store index exceeds u32 range");
        Self {
            index,
            marker: PhantomData,
        }
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.index
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.index as usize
    }

    #[inline]
    pub const fn node_id(self) -> Self {
        self
    }

    #[inline]
    pub const fn from_raw_unchecked(index: u32) -> Self {
        Self {
            index,
            marker: PhantomData,
        }
    }
}

impl Id<CommonDomain> {
    #[inline]
    pub const fn ast_id(self) -> AstId {
        AstId(self.index)
    }
}

impl Id<ClassDomain> {
    #[inline]
    pub const fn ast_id(self) -> AstId {
        AstId(self.index)
    }
}

impl<D: IdDomain> Copy for Id<D> {}
impl<D: IdDomain> Clone for Id<D> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<D: IdDomain> PartialEq for Id<D> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<D: IdDomain> Eq for Id<D> {}
impl<D: IdDomain> PartialOrd for Id<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<D: IdDomain> Ord for Id<D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}
impl<D: IdDomain> Hash for Id<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<D: IdDomain> fmt::Debug for Id<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple(std::any::type_name::<D>())
            .field(&self.index)
            .finish()
    }
}

impl<D: IdDomain> Idx for Id<D> {
    const MAX: usize = MAX_INDEX as usize;

    #[inline]
    unsafe fn from_usize_unchecked(index: usize) -> Self {
        debug_assert!(index <= Self::MAX);
        Self {
            index: index as u32,
            marker: PhantomData,
        }
    }

    #[inline]
    fn index(self) -> usize {
        self.index as usize
    }
}

pub type NodeId = Id<CommonDomain>;
pub type ClassId = Id<ClassDomain>;

/// An untagged ID into the common AST node pool.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AstId(u32);

impl AstId {
    pub const NONE: Self = Self(u32::MAX);

    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }

    #[inline]
    pub const fn node_id(self) -> NodeId {
        assert!(!self.is_none(), "sentinel has no common node ID");
        NodeId::from_raw_unchecked(self.0)
    }
}

impl fmt::Debug for AstId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            formatter.write_str("AstId::NONE")
        } else {
            formatter.debug_tuple("AstId").field(&self.0).finish()
        }
    }
}

impl From<NodeId> for AstId {
    #[inline]
    fn from(id: NodeId) -> Self {
        id.ast_id()
    }
}

pub trait TypedAstId: Copy {
    fn ast_id(self) -> AstId;

    /// # Safety
    ///
    /// ast_id must identify a common node accepted by Self.
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self;
}

impl TypedAstId for NodeId {
    #[inline]
    fn ast_id(self) -> AstId {
        self.ast_id()
    }

    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        ast_id.node_id()
    }
}

impl TypedAstId for ClassId {
    #[inline]
    fn ast_id(self) -> AstId {
        self.ast_id()
    }

    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        ClassId::from_raw_unchecked(ast_id.raw())
    }
}

impl TypedAstId for AstId {
    #[inline]
    fn ast_id(self) -> AstId {
        self
    }

    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        ast_id
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeMeta<I> {
    pub span: Span,
    marker: PhantomData<fn(I) -> I>,
}

impl<I> NodeMeta<I> {
    #[inline]
    pub const fn new(span: Span) -> Self {
        Self {
            span,
            marker: PhantomData,
        }
    }
}

pub trait AstEdge: Copy {
    fn to_ast_id(self) -> AstId;

    /// # Safety
    ///
    /// ast_id must identify a common node accepted by Self, or be the
    /// sentinel when Self is an optional edge.
    unsafe fn from_edge_unchecked(ast_id: AstId) -> Self;
}

impl<T: CommonAstNode> AstEdge for T {
    #[inline]
    fn to_ast_id(self) -> AstId {
        CommonAstNode::ast_id(self)
    }

    #[inline]
    unsafe fn from_edge_unchecked(ast_id: AstId) -> Self {
        unsafe { T::from_ast_id_unchecked(ast_id) }
    }
}

impl<T: CommonAstNode> AstEdge for Option<T> {
    #[inline]
    fn to_ast_id(self) -> AstId {
        self.map_or(AstId::NONE, CommonAstNode::ast_id)
    }

    #[inline]
    unsafe fn from_edge_unchecked(ast_id: AstId) -> Self {
        if ast_id.is_none() {
            None
        } else {
            Some(unsafe { T::from_ast_id_unchecked(ast_id) })
        }
    }
}

/// A typed handle into any AST-owned store.
pub trait AstNode: Copy {
    type Id: Copy;

    fn id(self) -> Self::Id;

    #[inline]
    fn node_id(self) -> Self::Id {
        self.id()
    }

    /// # Safety
    ///
    /// id must identify the concrete node type represented by Self.
    #[inline]
    unsafe fn from_id_unchecked(id: Self::Id) -> Self
    where
        Self: Sized,
    {
        const {
            assert!(std::mem::size_of::<Self>() == std::mem::size_of::<Self::Id>());
        }
        unsafe { std::mem::transmute_copy(&id) }
    }

    /// # Safety
    ///
    /// id must identify the concrete node type represented by Self.
    #[inline]
    unsafe fn from_node_id_unchecked(id: Self::Id) -> Self
    where
        Self: Sized,
    {
        unsafe { Self::from_id_unchecked(id) }
    }
}

/// A typed handle backed by the common node pool.
pub trait CommonAstNode: AstNode {
    fn ast_id(self) -> AstId;

    /// # Safety
    ///
    /// ast_id must identify the concrete common node type represented by Self.
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_ids_are_untagged_and_distinct() {
        assert_eq!(std::mem::size_of::<AstId>(), 4);
        assert_eq!(std::mem::size_of::<NodeId>(), 4);
        assert_eq!(std::mem::size_of::<ClassId>(), 4);
        assert_eq!(
            std::mem::size_of::<NodeMeta<NodeId>>(),
            std::mem::size_of::<Span>()
        );

        let common = NodeId::from_raw(17);
        let class = ClassId::from_raw(17);
        assert_eq!(common.raw(), class.raw());
        assert_eq!(common.ast_id().raw(), 17);
    }

    #[test]
    fn sentinel_is_not_a_valid_store_index() {
        let max = NodeId::from_raw(MAX_INDEX).ast_id();
        assert_eq!(max.raw(), MAX_INDEX);
        assert!(!max.is_none());
        assert!(AstId::NONE.is_none());
    }
}
