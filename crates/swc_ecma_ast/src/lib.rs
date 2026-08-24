mod assert_layout;
mod ast;
mod comment;
mod derive;
mod diagnostic;
pub mod ffi;
mod node_id;
mod semantic;
mod strings;
mod syntax;
mod visit;

mod generated {
    mod ast_builder;
    mod ast_clone_in;
    pub(crate) mod ast_extra_compact;
    pub mod ast_node;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use oxc_index::Idx;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

pub use ast::*;
pub use comment::*;
pub use derive::*;
pub use diagnostic::*;
pub use generated::ast_node::{NodeKind, NodeKindData};
pub use generated::ast_visitor::*;
pub use node_id::{
    AstEdge, AstId, AstNode, ClassDomain, ClassId, CommonAstNode, CommonDomain, ExtraDataCompact,
    ExtraDataId, Id, IdDomain, NodeEdgeId, NodeEdgeIndex, NodeId, NodeMeta, NodeRange, SubRange,
    TypedAstId, TypedNodeRange, TypedSubRange,
};
pub use semantic::{ReferenceId, ScopeId, SymbolId};
pub use strings::{AstUtf8, AstWtf8};
pub use syntax::*;
pub use visit::{TraverseAction, TraverseCtx};

use swc_next_allocator::multi_index_vec;

use crate::strings::StringPool;

/// AST context that stores everything about the flattening AST.
pub struct Ast<'a> {
    allocator: &'a Allocator,

    root: Option<AstId>,

    source_type: SourceType,

    lang: Lang,

    attached_comments: ArenaVec<'a, Comment>,

    /// Flattening AST nodes.
    pub(crate) nodes: NodeData<'a>,

    /// Dense object storage for classes. Common Class owner nodes store this
    /// vector's index in their payload.
    pub(crate) classes: ArenaVec<'a, Class>,

    /// Cold optional fields for classes, indexed in lockstep with `classes`.
    pub(crate) class_optional: ArenaVec<'a, ClassOptional>,

    /// Four-byte untagged common-node child-list elements.
    pub(crate) node_edges: ArenaVec<'a, AstId>,

    /// Flattening AST fields.
    extra_data: ArenaVec<'a, ExtraData>,

    /// Source-backed and decoded AST strings.
    strings: StringPool<'a>,
}

multi_index_vec! {
    /// Arena-backed structure-of-arrays storage for flattening AST nodes.
    pub struct NodeData<'a, NodeId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) kind => kind_mut => kind_unchecked => kind_mut_unchecked: NodeKind,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: NodePayload,
    }
}

/// Compact 128-bit payload for a node.
///
/// Fully inlined nodes use all 128 bits. Nodes with out-of-line fields use the
/// low 96 bits for inline data and the high 32 bits for the start index in
/// [Ast::extra_data].
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NodePayload(pub(crate) u128);

#[derive(Debug, Clone, Copy)]
pub struct ExtraDataRange {
    pub start: ExtraDataId,
    pub end: ExtraDataId,
}

/// Compact values stored in the flattening AST's extra-data table.
#[derive(Clone, Copy)]
pub union ExtraData {
    ast_id: AstId,

    sub_range: SubRange,

    /// Any scalar enum or other value representable within eight bytes.
    other: u64,
}

impl ExtraData {
    /// Construct a fully initialized extra-data word containing a node ID.
    ///
    /// `ExtraData` is copied byte-for-byte across the JavaScript FFI. Writing
    /// the four-byte union field directly would leave the high four bytes
    /// uninitialized.
    #[inline]
    pub(crate) fn from_ast_id(ast_id: AstId) -> Self {
        let mut data = Self { other: 0 };
        data.ast_id = ast_id;
        data
    }
}

impl<'a> Ast<'a> {
    /// Returns the AST's root program.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a program.
    #[inline]
    pub fn root_program(&self) -> Program {
        let root = self.root.expect("AST has no root");
        assert_eq!(
            self.node_kind(root),
            NodeKind::Program,
            "AST root is not a program"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Program::from_ast_id_unchecked(root) }
    }

    /// Returns the AST's root expression.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not an expression.
    #[inline]
    pub fn root_expression(&self) -> Expr {
        let root = self.root.expect("AST has no root");
        assert!(
            self.node_kind(root).is_expr(),
            "AST root is not an expression"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Expr::from_ast_id_unchecked(root) }
    }

    #[inline]
    pub(crate) fn add_node(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId {
        self.nodes.push(span, kind, data)
    }

    /// Replace a common owner node in place while preserving its AST ID.
    ///
    /// # Safety
    ///
    /// Replacing the data can invalidate concrete typed handles and semantic
    /// side tables. The caller must ensure all surviving handles accept the
    /// replacement and invalidate semantics when the structure changes.
    #[inline]
    pub unsafe fn replace_in_place_same_domain(
        &mut self,
        old_ast_id: AstId,
        new_ast_id: AstId,
    ) -> AstId {
        let old = old_ast_id.node_id();
        let new = new_ast_id.node_id();
        let span = *self.nodes.span(new);
        let kind = *self.nodes.kind(new);
        let data = *self.nodes.data(new);
        *self.nodes.span_mut(old) = span;
        *self.nodes.kind_mut(old) = kind;
        *self.nodes.data_mut(old) = data;
        old_ast_id
    }

    /// Replace `old` with a newly built `new` node, discard the temporary node,
    /// and return a typed handle that preserves `old`'s node ID.
    ///
    /// # Safety
    ///
    /// Replacing the data may change the concrete node kind stored at `old`,
    /// invalidating typed handles and AST edges that refer to it. The caller
    /// must ensure that every typed handle and every AST edge used after this
    /// call accepts `New`'s concrete kind. In particular, a handle for `old`'s
    /// previous concrete kind must never be used to access node-specific data
    /// after the replacement.
    ///
    /// `new` must be an unattached temporary node at the end of the node table.
    /// No handle or AST edge may refer to `new` after this call, and the moved
    /// payload must not contain a reference to `new`, because its slot is
    /// removed before this function returns.
    #[inline]
    pub unsafe fn replace_data<Old: CommonAstNode, New: CommonAstNode>(
        &mut self,
        old: Old,
        new: New,
    ) -> New {
        let old_ast_id = old.ast_id();
        let new_ast_id = new.ast_id();
        let old = old_ast_id.node_id();
        let new = new_ast_id.node_id();
        assert_ne!(old, new);
        assert_eq!(new.index() + 1, self.nodes.len());

        let span = *self.nodes.span(new);
        let kind = *self.nodes.kind(new);
        let data = *self.nodes.data(new);
        *self.nodes.span_mut(old) = span;
        *self.nodes.kind_mut(old) = kind;
        *self.nodes.data_mut(old) = data;
        self.nodes.truncate(new.index());

        unsafe { New::from_ast_id_unchecked(old.ast_id()) }
    }

    #[inline]
    fn add_extra(&mut self, extra: ExtraData) -> ExtraDataId {
        let id = self.next_extra_data_id();
        self.extra_data.push(extra);
        id
    }

    #[inline]
    fn next_extra_data_id(&self) -> ExtraDataId {
        let index = self.extra_data.len();
        assert!(
            index <= ExtraDataId::MAX,
            "extra data index exceeds ExtraDataId range"
        );
        ExtraDataId::from_usize_unchecked(index)
    }

    #[inline]
    pub fn add_typed_sub_range<N: AstEdge, I: IntoIterator<Item = N>>(
        &mut self,
        iter: I,
    ) -> TypedNodeRange<N> {
        let start = NodeEdgeIndex::from_usize_unchecked(self.node_edges.len());
        self.node_edges
            .extend(iter.into_iter().map(AstEdge::to_ast_id));
        let end = NodeEdgeIndex::from_usize_unchecked(self.node_edges.len());
        unsafe { NodeRange { start, end }.cast_to_typed() }
    }

    /// Appends already-encoded AST IDs as a typed child range.
    ///
    /// # Safety
    ///
    /// Every non-sentinel ID in `ast_ids` must identify a node accepted by
    /// `N`; sentinel IDs are only valid when `N` is an optional AST edge.
    #[inline]
    pub unsafe fn add_typed_ast_id_sub_range<N>(&mut self, ast_ids: &[AstId]) -> TypedNodeRange<N> {
        let start = NodeEdgeIndex::from_usize_unchecked(self.node_edges.len());
        self.node_edges.extend_from_slice_copy(ast_ids);
        let end = NodeEdgeIndex::from_usize_unchecked(self.node_edges.len());
        unsafe { NodeRange { start, end }.cast_to_typed() }
    }

    /// Append pre-encoded extra data as a single contiguous range.
    pub fn add_extra_data_sub_range(&mut self, data: &[ExtraData]) -> ExtraDataRange {
        let start = self.next_extra_data_id();
        let end = start.index() + data.len();
        self.extra_data.extend_from_slice_copy(data);
        ExtraDataRange {
            start,
            end: ExtraDataId::from_usize_unchecked(end),
        }
    }

    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Creates a map grouping attached comments by attachment offset.
    pub fn create_comments_map(&self) -> ArenaHashMap<'a, u32, ArenaVec<'a, Comment>> {
        let mut attached_comments =
            ArenaHashMap::with_capacity_in(self.attached_comments.len(), self.allocator);
        let allocator = self.allocator;
        for &comment in &self.attached_comments {
            let comments = attached_comments.entry(comment.attached_to);
            let comments = comments.or_insert_with(|| ArenaVec::new_in(allocator));
            comments.push(comment);
        }
        attached_comments
    }

    /// Returns attached comments in source order.
    pub fn comments(&self) -> &[Comment] {
        &self.attached_comments
    }

    /// Retargets comment attachment offsets while preserving source order.
    ///
    /// The callback receives each comment's position and current attachment
    /// offset. Returning `Some` replaces the offset, while `None` leaves it
    /// unchanged.
    pub fn retarget_comments(
        &mut self,
        mut retarget: impl FnMut(CommentPosition, u32) -> Option<u32>,
    ) {
        for comment in &mut self.attached_comments {
            if let Some(attached_to) = retarget(comment.position, comment.attached_to) {
                comment.attached_to = attached_to;
            }
        }
    }

    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[inline]
    pub fn lang(&self) -> Lang {
        self.lang
    }

    #[inline]
    pub fn is_ts(&self) -> bool {
        self.lang.is_typescript()
    }

    #[inline]
    pub fn is_jsx(&self) -> bool {
        self.lang.is_jsx()
    }

    #[inline]
    pub fn is_module(&self) -> bool {
        self.source_type == SourceType::Module
    }

    #[inline]
    pub fn source(&self) -> &'a str {
        self.strings.source
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8 {
        self.strings.source_slice(start, end)
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Actually this function should be unsafe because it does not check if the offsets are valid.
    /// So external crates should be better not use this function directly.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8 {
        self.strings.source_slice_wtf8(start, end)
    }

    #[inline]
    pub fn add_utf8(&mut self, value: &str) -> AstUtf8 {
        self.strings.add_utf8(value)
    }

    #[inline]
    pub fn add_wtf8(&mut self, value: &swc_next_allocator::wtf8::Wtf8) -> AstWtf8 {
        self.strings.add_wtf8(value)
    }

    /// Stores a canonical UTF-8 range for strings whose identity is used by
    /// semantic analysis.
    #[inline]
    pub fn intern_utf8(&mut self, value: &str) -> AstUtf8 {
        self.strings.intern_utf8(value)
    }

    /// Canonicalizes an existing UTF-8 string range.
    #[inline]
    pub fn intern_utf8_slice(&mut self, value: AstUtf8) -> AstUtf8 {
        self.strings.intern_utf8_slice(value)
    }

    #[inline]
    pub fn get_utf8(&self, id: AstUtf8) -> &str {
        self.strings.get_utf8(id)
    }

    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8) -> &swc_next_allocator::wtf8::Wtf8 {
        self.strings.get_wtf8(id)
    }

    #[inline]
    pub fn empty_utf8(&self) -> AstUtf8 {
        AstUtf8::EMPTY
    }

    #[inline]
    pub fn empty_wtf8(&self) -> AstWtf8 {
        AstWtf8::EMPTY
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of entries in the out-of-line field table.
    #[inline]
    pub fn extra_data_count(&self) -> usize {
        self.extra_data.len()
    }

    #[inline]
    pub fn node_edge_count(&self) -> usize {
        self.node_edges.len()
    }

    #[inline]
    pub fn class_count(&self) -> usize {
        debug_assert_eq!(self.classes.len(), self.class_optional.len());
        self.classes.len()
    }

    /// Size of one cold class side-table record.
    #[doc(hidden)]
    #[inline]
    pub const fn class_optional_record_size() -> usize {
        std::mem::size_of::<ClassOptional>()
    }

    /// Logical bytes occupied by the hot and cold class records.
    #[doc(hidden)]
    #[inline]
    pub fn class_storage_bytes(&self) -> usize {
        self.classes.len() * std::mem::size_of::<Class>()
            + self.class_optional.len() * std::mem::size_of::<ClassOptional>()
    }

    #[inline]
    pub fn class_index(&self, class_id: ClassId) -> usize {
        let node_id = class_id.ast_id().node_id();
        assert_eq!(*self.nodes.kind(node_id), NodeKind::Class);
        let index = self.nodes.data(node_id).0 as u32 as usize;
        assert!(index < self.classes.len());
        index
    }

    #[inline]
    pub(crate) fn class_data(&self, class_id: ClassId) -> &Class {
        &self.classes[self.class_index(class_id)]
    }

    #[inline]
    pub(crate) fn class_data_mut(&mut self, class_id: ClassId) -> &mut Class {
        let index = self.class_index(class_id);
        &mut self.classes[index]
    }

    #[inline]
    pub(crate) fn class_optional_data(&self, class_id: ClassId) -> &ClassOptional {
        &self.class_optional[self.class_index(class_id)]
    }

    #[inline]
    pub(crate) fn class_optional_data_mut(&mut self, class_id: ClassId) -> &mut ClassOptional {
        let index = self.class_index(class_id);
        &mut self.class_optional[index]
    }

    #[inline]
    pub(crate) const fn class_payload(
        storage_index: u32,
        class_type: ClassType,
        r#abstract: bool,
        declare: bool,
    ) -> NodePayload {
        NodePayload(
            storage_index as u128
                | ((class_type as u128) << 32)
                | ((r#abstract as u128) << 40)
                | ((declare as u128) << 41),
        )
    }

    #[inline]
    pub(crate) fn class_type(&self, class_id: ClassId) -> ClassType {
        let payload = self.nodes.data(class_id.ast_id().node_id()).0;
        match (payload >> 32) as u8 {
            0 => ClassType::ClassDeclaration,
            1 => ClassType::ClassExpression,
            _ => unreachable!("invalid ClassType in class owner payload"),
        }
    }

    #[inline]
    pub(crate) fn set_class_type(&mut self, class_id: ClassId, value: ClassType) {
        let payload = self.nodes.data_mut(class_id.ast_id().node_id());
        payload.0 = (payload.0 & !((u8::MAX as u128) << 32)) | ((value as u128) << 32);
    }

    #[inline]
    pub(crate) fn class_abstract(&self, class_id: ClassId) -> bool {
        (self.nodes.data(class_id.ast_id().node_id()).0 >> 40) & 1 != 0
    }

    #[inline]
    pub(crate) fn set_class_abstract(&mut self, class_id: ClassId, value: bool) {
        let payload = self.nodes.data_mut(class_id.ast_id().node_id());
        payload.0 = (payload.0 & !(1 << 40)) | ((value as u128) << 40);
    }

    #[inline]
    pub(crate) fn class_declare(&self, class_id: ClassId) -> bool {
        (self.nodes.data(class_id.ast_id().node_id()).0 >> 41) & 1 != 0
    }

    #[inline]
    pub(crate) fn set_class_declare(&mut self, class_id: ClassId, value: bool) {
        let payload = self.nodes.data_mut(class_id.ast_id().node_id());
        payload.0 = (payload.0 & !(1 << 41)) | ((value as u128) << 41);
    }

    /// Returns the concrete type stored at an AST ID.
    #[inline]
    pub fn node_kind<I: Into<AstId>>(&self, node_id: I) -> NodeKind {
        *self.nodes.kind(node_id.into().node_id())
    }

    /// Returns the source span stored for an AST ID.
    #[inline]
    pub fn span<I: Into<AstId>>(&self, node_id: I) -> Span {
        *self.nodes.span(node_id.into().node_id())
    }

    #[inline]
    pub fn set_span(&mut self, node_id: AstId, span: Span) {
        *self.nodes.span_mut(node_id.node_id()) = span;
    }
}

#[doc(hidden)]
impl<'a> Ast<'a> {
    /// Create a flattening AST backed by `source` and `allocator`.
    pub fn new(source: &'a str, allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            root: None,
            source_type: SourceType::Module,
            lang: Lang::Js,
            attached_comments: ArenaVec::new_in(allocator),
            nodes: NodeData::new_in(allocator),
            classes: ArenaVec::new_in(allocator),
            class_optional: ArenaVec::new_in(allocator),
            node_edges: ArenaVec::new_in(allocator),
            extra_data: ArenaVec::with_capacity_in(0, allocator),
            strings: StringPool::new_in(allocator, source),
        }
    }

    #[inline]
    pub fn set_source_metadata(&mut self, source_type: SourceType, lang: Lang) {
        self.source_type = source_type;
        self.lang = lang;
    }

    /// Replaces attached comments while preserving source order.
    pub fn set_attached_comments(&mut self, comments: ArenaVec<'a, Comment>) {
        self.attached_comments = comments;
    }

    /// Sets a program as the AST root.
    #[inline]
    pub fn set_root_program(&mut self, root: Program) {
        self.root = Some(root.ast_id());
    }

    /// Sets an expression as the AST root.
    #[inline]
    pub fn set_root_expression(&mut self, root: Expr) {
        self.root = Some(root.ast_id());
    }

    /// Reserves storage for parser output without changing logical lengths.
    #[inline]
    pub fn reserve(&mut self, additional_nodes: usize, additional_extra_data: usize) {
        self.nodes.reserve(additional_nodes);
        self.node_edges.reserve(additional_extra_data);
        self.extra_data.reserve(additional_extra_data);
    }

    /// Reserves dedicated class storage without changing its logical length.
    #[inline]
    pub fn reserve_classes(&mut self, additional_classes: usize) {
        self.classes.reserve(additional_classes);
        self.class_optional.reserve(additional_classes);
    }

    /// Rewinds node and out-of-line field storage to an earlier checkpoint.
    #[inline]
    pub fn truncate(
        &mut self,
        nodes_len: usize,
        classes_len: usize,
        node_edges_len: usize,
        extra_data_len: usize,
    ) {
        assert!(nodes_len <= self.nodes.len());
        assert!(classes_len <= self.classes.len());
        assert!(node_edges_len <= self.node_edges.len());
        assert!(extra_data_len <= self.extra_data.len());
        self.nodes.truncate(nodes_len);
        self.classes.truncate(classes_len);
        self.class_optional.truncate(classes_len);
        self.node_edges.truncate(node_edges_len);
        self.extra_data.truncate(extra_data_len);
    }
}

pub const DUMMY_SP: Span = Span { start: 0, end: 0 };

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn is_dummy(self) -> bool {
        self.start == 0 && self.end == 0
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_hot_and_optional_records_stay_in_lockstep() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("C", &allocator);
        let decorators = ast.add_typed_sub_range(std::iter::empty::<Decorator>());
        let implements = ast.add_typed_sub_range(std::iter::empty::<TsClassImplements>());
        let body = ast.add_typed_sub_range(std::iter::empty::<ClassElement>());
        let name = ast.source_slice(0, 1);
        let id = ast.binding_identifier(Span::new(0, 1), name, decorators, None, false);
        let class = ast.class(
            Span::new(0, 1),
            ClassType::ClassExpression,
            decorators,
            Some(id),
            None,
            None,
            None,
            implements,
            Span::new(1, 1),
            body,
            true,
            false,
        );

        assert_eq!(ast.class_count(), 1);
        assert_eq!(ast.class_storage_bytes(), 48);
        assert_eq!(class.class_type(&ast), ClassType::ClassExpression);
        assert_eq!(class.id(&ast), Some(id));
        assert!(class.r#abstract(&ast));
        assert!(!class.declare(&ast));

        class.set_class_type(&mut ast, ClassType::ClassDeclaration);
        class.set_id(&mut ast, None);
        class.set_abstract(&mut ast, false);
        class.set_declare(&mut ast, true);
        assert_eq!(class.class_type(&ast), ClassType::ClassDeclaration);
        assert_eq!(class.id(&ast), None);
        assert!(!class.r#abstract(&ast));
        assert!(class.declare(&ast));

        ast.truncate(0, 0, 0, 0);
        assert_eq!(ast.class_count(), 0);
        assert_eq!(ast.class_storage_bytes(), 0);
    }
}
