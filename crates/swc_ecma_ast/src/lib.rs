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

use std::mem::size_of;

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
    ExtraDataId, FunctionDomain, FunctionId, Id, IdDomain, NodeEdgeId, NodeEdgeIndex, NodeId,
    NodeMeta, NodeRange, StoreTag, SubRange, TypedAstId, TypedNodeRange, TypedSubRange,
};
pub use semantic::{ReferenceId, ScopeId, SymbolId};
pub use strings::{AstUtf8, AstWtf8};
pub use syntax::*;
pub use visit::{TraverseAction, TraverseCtx};

use swc_next_allocator::multi_index_vec;

use crate::strings::StringPool;

const HOLE_LIVE_PER_CHUNK: usize = 16;
const COMMON_HOLES_PER_CHUNK: usize = 0;
const EXPRESSION_HOLES_PER_CHUNK: usize = if cfg!(any(
    feature = "hole-profile-0",
    feature = "hole-profile-statement-only"
)) {
    0
} else if cfg!(feature = "hole-profile-hot-2") {
    2
} else {
    4
};
const STATEMENT_HOLES_PER_CHUNK: usize = if cfg!(any(
    feature = "hole-profile-0",
    feature = "hole-profile-expression-only"
)) {
    0
} else if cfg!(feature = "hole-profile-hot-2") {
    2
} else {
    4
};
const NAME_PATTERN_HOLES_PER_CHUNK: usize = if cfg!(any(
    feature = "hole-profile-0",
    feature = "hole-profile-expression-only",
    feature = "hole-profile-statement-only"
)) {
    0
} else if cfg!(feature = "hole-profile-hot-2") {
    2
} else {
    4
};
const CLASS_ELEMENT_HOLES_PER_CHUNK: usize = NAME_PATTERN_HOLES_PER_CHUNK;

/// AST context that stores everything about the flattening AST.
pub struct Ast<'a> {
    allocator: &'a Allocator,

    root: Option<AstId>,

    source_type: SourceType,

    lang: Lang,

    attached_comments: ArenaVec<'a, Comment>,

    /// Flattening AST nodes.
    pub(crate) nodes: NodeStore<'a, HOLE_LIVE_PER_CHUNK, COMMON_HOLES_PER_CHUNK>,

    /// Expression-only nodes, independently grown from other node kinds.
    pub(crate) expressions: NodeStore<'a, HOLE_LIVE_PER_CHUNK, EXPRESSION_HOLES_PER_CHUNK>,

    /// Statement/declaration nodes, independently grown from other node kinds.
    pub(crate) statements: NodeStore<'a, HOLE_LIVE_PER_CHUNK, STATEMENT_HOLES_PER_CHUNK>,

    /// Identifiers and binding/assignment patterns.
    pub(crate) name_patterns: NodeStore<'a, HOLE_LIVE_PER_CHUNK, NAME_PATTERN_HOLES_PER_CHUNK>,

    /// Class body elements, independently grown from large Class records.
    pub(crate) class_elements: NodeStore<'a, HOLE_LIVE_PER_CHUNK, CLASS_ELEMENT_HOLES_PER_CHUNK>,

    /// Cold TypeScript and JSX syntax.
    pub(crate) type_jsx: NodeStore<'a, HOLE_LIVE_PER_CHUNK, 0>,

    /// Program, function auxiliaries, directives, and remaining cold nodes.
    pub(crate) program_aux: NodeStore<'a, HOLE_LIVE_PER_CHUNK, 0>,

    /// Standalone class storage. A ClassId indexes this store directly.
    pub(crate) classes: ClassData<'a>,

    /// Standalone storage for Function payloads.
    pub(crate) functions: FunctionData<'a>,

    /// Four-byte untagged common-node child-list elements.
    pub(crate) node_edges: ArenaVec<'a, AstId>,

    /// Flattening AST fields.
    extra_data: ArenaVec<'a, ExtraData>,

    /// Source-backed and decoded AST strings.
    strings: StringPool<'a>,

    /// Builders append source nodes before transforms begin, then consume
    /// per-store holes after this flag is set.
    transforming: bool,
}

multi_index_vec! {
    /// Arena-backed structure-of-arrays storage for flattening AST nodes.
    pub struct NodeData<'a, NodeId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) kind => kind_mut => kind_unchecked => kind_mut_unchecked: NodeKind,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: NodePayload,
    }
}

/// Observable storage counters used by insertion/locality benchmarks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StorageMetrics {
    pub live_per_chunk: usize,
    pub holes_per_chunk: usize,
    pub live: usize,
    pub slots: usize,
    pub available_holes: usize,
    pub holes_created: usize,
    pub holes_used: usize,
    pub hole_lookups: usize,
    pub grow_count: usize,
    pub grow_copied_bytes: usize,
}

pub(crate) struct NodeStore<'a, const LIVE: usize, const HOLES: usize> {
    pub(crate) data: NodeData<'a>,
    free_holes: ArenaVec<'a, u32>,
    live_len: usize,
    tail_live: usize,
    holes_created: usize,
    holes_used: usize,
    hole_lookups: usize,
    grow_count: usize,
    grow_copied_bytes: usize,
}

impl<'a, const LIVE: usize, const HOLES: usize> NodeStore<'a, LIVE, HOLES> {
    const NODE_COPY_BYTES: usize =
        size_of::<Span>() + size_of::<NodeKind>() + size_of::<NodePayload>();

    fn new_in(allocator: &'a Allocator) -> Self {
        const { assert!(LIVE > 0, "hole policy requires at least one live slot") };
        Self {
            data: NodeData::new_in(allocator),
            free_holes: ArenaVec::new_in(allocator),
            live_len: 0,
            tail_live: 0,
            holes_created: 0,
            holes_used: 0,
            hole_lookups: 0,
            grow_count: 0,
            grow_copied_bytes: 0,
        }
    }

    #[inline]
    fn push_raw(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId {
        let old_capacity = self.data.capacity();
        let old_len = self.data.len();
        let id = self.data.push(span, kind, data);
        if self.data.capacity() != old_capacity {
            self.grow_count += 1;
            self.grow_copied_bytes += old_len * Self::NODE_COPY_BYTES;
        }
        id
    }

    fn append_holes_after_chunk(&mut self) {
        if HOLES == 0 || self.tail_live != LIVE {
            return;
        }
        for _ in 0..HOLES {
            let id = self.push_raw(DUMMY_SP, NodeKind::Hole, NodePayload(0));
            self.free_holes.push(id.index() as u32);
            self.holes_created += 1;
        }
        self.tail_live = 0;
    }

    #[inline]
    fn push(
        &mut self,
        transforming: bool,
        span: Span,
        kind: NodeKind,
        data: NodePayload,
    ) -> NodeId {
        if transforming && HOLES != 0 {
            self.hole_lookups += 1;
            if let Some(index) = self.free_holes.pop() {
                let id = NodeId::from_raw(index);
                *self.data.span_mut(id) = span;
                *self.data.kind_mut(id) = kind;
                *self.data.data_mut(id) = data;
                self.live_len += 1;
                self.holes_used += 1;
                return id;
            }
        }

        let id = self.push_raw(span, kind, data);
        self.live_len += 1;
        self.tail_live += 1;
        self.append_holes_after_chunk();
        id
    }

    fn retire(&mut self, id: NodeId) {
        debug_assert_ne!(*self.data.kind(id), NodeKind::Hole, "AST slot double free");
        if *self.data.kind(id) == NodeKind::Hole {
            return;
        }
        *self.data.span_mut(id) = DUMMY_SP;
        *self.data.kind_mut(id) = NodeKind::Hole;
        *self.data.data_mut(id) = NodePayload(0);
        self.free_holes.push(id.index() as u32);
        self.live_len -= 1;
    }

    fn reserve_live(&mut self, additional: usize) {
        let chunks = additional.div_ceil(LIVE);
        self.data
            .reserve(additional.saturating_add(chunks.saturating_mul(HOLES)));
    }

    fn truncate(&mut self, slots: usize) {
        self.data.truncate(slots);
        self.free_holes.retain(|index| *index < slots as u32);
        let kinds = unsafe { std::slice::from_raw_parts(self.data.kind.as_ptr(), slots) };
        self.live_len = kinds.iter().filter(|&&kind| kind != NodeKind::Hole).count();
        self.tail_live = self.live_len % LIVE;
    }

    #[inline]
    fn slot_len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn live_len(&self) -> usize {
        self.live_len
    }

    fn metrics(&self) -> StorageMetrics {
        StorageMetrics {
            live_per_chunk: LIVE,
            holes_per_chunk: HOLES,
            live: self.live_len,
            slots: self.data.len(),
            available_holes: self.free_holes.len(),
            holes_created: self.holes_created,
            holes_used: self.holes_used,
            hole_lookups: self.hole_lookups,
            grow_count: self.grow_count,
            grow_copied_bytes: self.grow_copied_bytes,
        }
    }
}

multi_index_vec! {
    /// Arena-backed standalone storage for large Class records.
    pub struct ClassData<'a, ClassId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: Class,
    }
}

multi_index_vec! {
    /// Arena-backed standalone storage for Function nodes.
    pub struct FunctionData<'a, FunctionId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
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
    #[inline]
    pub(crate) fn root_node_id(&self) -> AstId {
        self.root.expect("AST has no root")
    }

    /// Returns the AST's root program.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a program.
    #[inline]
    pub fn root_program(&self) -> Program {
        let root = self.root_node_id();
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
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_expr(),
            "AST root is not an expression"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Expr::from_ast_id_unchecked(root) }
    }

    /// Returns the AST's root statement.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a statement.
    #[inline]
    pub fn root_statement(&self) -> Stmt {
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_stmt(),
            "AST root is not a statement"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Stmt::from_ast_id_unchecked(root) }
    }

    #[inline]
    pub(crate) fn add_node<I: TypedAstId>(
        &mut self,
        span: Span,
        kind: NodeKind,
        data: NodePayload,
    ) -> I {
        let store = kind.store_tag();
        let local = match store {
            StoreTag::Common => self.nodes.push(self.transforming, span, kind, data),
            StoreTag::Expression => self.expressions.push(self.transforming, span, kind, data),
            StoreTag::Statement => self.statements.push(self.transforming, span, kind, data),
            StoreTag::NamePattern => self.name_patterns.push(self.transforming, span, kind, data),
            StoreTag::ClassElement => self
                .class_elements
                .push(self.transforming, span, kind, data),
            StoreTag::TypeJsx => self.type_jsx.push(self.transforming, span, kind, data),
            StoreTag::ProgramAux => self.program_aux.push(self.transforming, span, kind, data),
            _ => unreachable!(),
        }
        .index() as u32;
        unsafe { I::from_ast_id_unchecked(AstId::new(store, local)) }
    }

    #[inline]
    fn node_store(&self, store: StoreTag) -> &NodeData<'a> {
        match store {
            StoreTag::Common => &self.nodes.data,
            StoreTag::Expression => &self.expressions.data,
            StoreTag::Statement => &self.statements.data,
            StoreTag::NamePattern => &self.name_patterns.data,
            StoreTag::ClassElement => &self.class_elements.data,
            StoreTag::TypeJsx => &self.type_jsx.data,
            StoreTag::ProgramAux => &self.program_aux.data,
            _ => panic!("AST store does not contain NodePayload"),
        }
    }

    #[inline]
    fn node_store_mut(&mut self, store: StoreTag) -> &mut NodeData<'a> {
        match store {
            StoreTag::Common => &mut self.nodes.data,
            StoreTag::Expression => &mut self.expressions.data,
            StoreTag::Statement => &mut self.statements.data,
            StoreTag::NamePattern => &mut self.name_patterns.data,
            StoreTag::ClassElement => &mut self.class_elements.data,
            StoreTag::TypeJsx => &mut self.type_jsx.data,
            StoreTag::ProgramAux => &mut self.program_aux.data,
            _ => panic!("AST store does not contain NodePayload"),
        }
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
        let span = self.span(new_ast_id);
        let kind = self.node_kind(new_ast_id);
        let data = unsafe { *self.node_payload_unchecked(new_ast_id) };
        let old_store = self.node_store_mut(old_ast_id.store_tag());
        let old = old_ast_id.node_id();
        *old_store.span_mut(old) = span;
        *old_store.kind_mut(old) = kind;
        *old_store.data_mut(old) = data;
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
        assert_ne!(old_ast_id, new_ast_id);
        if old_ast_id.store_tag() != new_ast_id.store_tag()
            && new_ast_id.store_tag() == StoreTag::Function
        {
            self.retire_node(old_ast_id);
            return new;
        }
        let span = self.span(new_ast_id);
        let kind = self.node_kind(new_ast_id);
        let data = unsafe { *self.node_payload_unchecked(new_ast_id) };
        match old_ast_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => {
                let old_store = self.node_store_mut(old_ast_id.store_tag());
                let old = old_ast_id.node_id();
                *old_store.span_mut(old) = span;
                *old_store.kind_mut(old) = kind;
                *old_store.data_mut(old) = data;
            }
            StoreTag::Function => {
                let old = FunctionId::from_raw(old_ast_id.index() as u32);
                *self.functions.span_mut(old) = span;
                *self.functions.data_mut(old) = data;
            }
            _ => panic!("AST store does not support payload replacement"),
        }
        if new_ast_id.store_tag() == StoreTag::Function {
            let new_index = new_ast_id.index();
            assert_eq!(new_index + 1, self.functions.len());
            self.functions.truncate(new_index);
        } else {
            self.retire_node(new_ast_id);
        }
        unsafe { New::from_ast_id_unchecked(old_ast_id) }
    }

    fn retire_node(&mut self, id: AstId) {
        match id.store_tag() {
            StoreTag::Common => self.nodes.retire(id.node_id()),
            StoreTag::Expression => self.expressions.retire(id.node_id()),
            StoreTag::Statement => self.statements.retire(id.node_id()),
            StoreTag::NamePattern => self.name_patterns.retire(id.node_id()),
            StoreTag::ClassElement => self.class_elements.retire(id.node_id()),
            StoreTag::TypeJsx => self.type_jsx.retire(id.node_id()),
            StoreTag::ProgramAux => self.program_aux.retire(id.node_id()),
            _ => panic!("AST store does not support holes"),
        }
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
        self.nodes.live_len()
            + self.expressions.live_len()
            + self.statements.live_len()
            + self.name_patterns.live_len()
            + self.class_elements.live_len()
            + self.type_jsx.live_len()
            + self.program_aux.live_len()
            + self.classes.len()
            + self.functions.len()
    }

    #[inline]
    #[doc(hidden)]
    pub fn common_node_count(&self) -> usize {
        self.nodes.slot_len()
    }

    #[inline]
    #[doc(hidden)]
    pub fn store_node_count(&self, store: StoreTag) -> usize {
        match store {
            StoreTag::Common => self.nodes.slot_len(),
            StoreTag::Expression => self.expressions.slot_len(),
            StoreTag::Statement => self.statements.slot_len(),
            StoreTag::NamePattern => self.name_patterns.slot_len(),
            StoreTag::ClassElement => self.class_elements.slot_len(),
            StoreTag::TypeJsx => self.type_jsx.slot_len(),
            StoreTag::ProgramAux => self.program_aux.slot_len(),
            StoreTag::Class => self.classes.len(),
            StoreTag::Function => self.functions.len(),
        }
    }

    /// Returns insertion/locality counters for a node storage domain.
    #[doc(hidden)]
    pub fn storage_metrics(&self, store: StoreTag) -> StorageMetrics {
        match store {
            StoreTag::Common => self.nodes.metrics(),
            StoreTag::Expression => self.expressions.metrics(),
            StoreTag::Statement => self.statements.metrics(),
            StoreTag::NamePattern => self.name_patterns.metrics(),
            StoreTag::ClassElement => self.class_elements.metrics(),
            StoreTag::TypeJsx => self.type_jsx.metrics(),
            StoreTag::ProgramAux => self.program_aux.metrics(),
            StoreTag::Class => StorageMetrics {
                live_per_chunk: HOLE_LIVE_PER_CHUNK,
                live: self.classes.len(),
                slots: self.classes.len(),
                ..StorageMetrics::default()
            },
            StoreTag::Function => StorageMetrics {
                live_per_chunk: HOLE_LIVE_PER_CHUNK,
                live: self.functions.len(),
                slots: self.functions.len(),
                ..StorageMetrics::default()
            },
        }
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
        self.classes.len()
    }

    #[inline]
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    #[inline]
    pub fn class_index(&self, class_id: ClassId) -> usize {
        let index = class_id.index();
        assert!(index < self.classes.len());
        index
    }

    #[inline]
    pub(crate) fn class_data(&self, class_id: ClassId) -> &Class {
        self.classes
            .data(ClassId::from_raw(self.class_index(class_id) as u32))
    }

    #[inline]
    pub(crate) fn class_data_mut(&mut self, class_id: ClassId) -> &mut Class {
        let id = ClassId::from_raw(self.class_index(class_id) as u32);
        self.classes.data_mut(id)
    }

    /// Returns the concrete type stored at an AST ID.
    #[inline]
    pub fn node_kind<I: Into<AstId>>(&self, node_id: I) -> NodeKind {
        let node_id = node_id.into();
        match node_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => *self.node_store(node_id.store_tag()).kind(node_id.node_id()),
            StoreTag::Class => NodeKind::Class,
            StoreTag::Function => NodeKind::Function,
        }
    }

    /// Returns the source span stored for an AST ID.
    #[inline]
    pub fn span<I: Into<AstId>>(&self, node_id: I) -> Span {
        let node_id = node_id.into();
        match node_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => *self.node_store(node_id.store_tag()).span(node_id.node_id()),
            StoreTag::Class => *self.classes.span(ClassId::from_raw(node_id.index() as u32)),
            StoreTag::Function => *self
                .functions
                .span(FunctionId::from_raw(node_id.index() as u32)),
        }
    }

    #[inline]
    pub fn set_span(&mut self, node_id: AstId, span: Span) {
        match node_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => {
                *self
                    .node_store_mut(node_id.store_tag())
                    .span_mut(node_id.node_id()) = span;
            }
            StoreTag::Class => {
                *self
                    .classes
                    .span_mut(ClassId::from_raw(node_id.index() as u32)) = span;
            }
            StoreTag::Function => {
                *self
                    .functions
                    .span_mut(FunctionId::from_raw(node_id.index() as u32)) = span;
            }
        }
    }

    #[inline]
    pub(crate) unsafe fn node_payload_unchecked(&self, node_id: AstId) -> &NodePayload {
        match node_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => unsafe {
                self.node_store(node_id.store_tag())
                    .data_unchecked(node_id.node_id())
            },
            StoreTag::Function => unsafe {
                self.functions
                    .data_unchecked(FunctionId::from_raw(node_id.index() as u32))
            },
            _ => panic!("AST store does not contain NodePayload"),
        }
    }

    #[inline]
    pub(crate) unsafe fn node_payload_mut_unchecked(&mut self, node_id: AstId) -> &mut NodePayload {
        match node_id.store_tag() {
            StoreTag::Common
            | StoreTag::Expression
            | StoreTag::Statement
            | StoreTag::NamePattern
            | StoreTag::ClassElement
            | StoreTag::TypeJsx
            | StoreTag::ProgramAux => unsafe {
                self.node_store_mut(node_id.store_tag())
                    .data_mut_unchecked(node_id.node_id())
            },
            StoreTag::Function => unsafe {
                self.functions
                    .data_mut_unchecked(FunctionId::from_raw(node_id.index() as u32))
            },
            _ => panic!("AST store does not contain NodePayload"),
        }
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
            nodes: NodeStore::new_in(allocator),
            expressions: NodeStore::new_in(allocator),
            statements: NodeStore::new_in(allocator),
            name_patterns: NodeStore::new_in(allocator),
            class_elements: NodeStore::new_in(allocator),
            type_jsx: NodeStore::new_in(allocator),
            program_aux: NodeStore::new_in(allocator),
            classes: ClassData::new_in(allocator),
            functions: FunctionData::new_in(allocator),
            node_edges: ArenaVec::new_in(allocator),
            extra_data: ArenaVec::with_capacity_in(0, allocator),
            strings: StringPool::new_in(allocator, source),
            transforming: false,
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

    /// Sets a statement as the AST root.
    #[inline]
    pub fn set_root_statement(&mut self, root: Stmt) {
        self.root = Some(root.node_id());
    }

    /// Reserves storage for parser output without changing logical lengths.
    #[inline]
    pub fn reserve(&mut self, additional_nodes: usize, additional_extra_data: usize) {
        let expression_nodes = additional_nodes * 40 / 100;
        let statement_nodes = additional_nodes * 20 / 100;
        let name_pattern_nodes = additional_nodes * 20 / 100;
        let class_element_nodes = additional_nodes * 5 / 100;
        let type_jsx_nodes = additional_nodes * 10 / 100;
        let assigned = expression_nodes
            + statement_nodes
            + name_pattern_nodes
            + class_element_nodes
            + type_jsx_nodes;
        self.expressions.reserve_live(expression_nodes);
        self.statements.reserve_live(statement_nodes);
        self.name_patterns.reserve_live(name_pattern_nodes);
        self.class_elements.reserve_live(class_element_nodes);
        self.type_jsx.reserve_live(type_jsx_nodes);
        self.program_aux.reserve_live(additional_nodes - assigned);
        self.node_edges.reserve(additional_extra_data);
        self.extra_data.reserve(additional_extra_data);
    }

    /// Switch builders from source-order append mode to hole-consuming
    /// transform insertion mode.
    #[inline]
    pub fn begin_transform(&mut self) {
        self.transforming = true;
    }

    /// Reserves dedicated class storage without changing its logical length.
    #[inline]
    pub fn reserve_classes(&mut self, additional_classes: usize) {
        self.classes.reserve(additional_classes);
    }

    #[inline]
    pub fn reserve_functions(&mut self, additional_functions: usize) {
        self.functions.reserve(additional_functions);
    }

    /// Rewinds node and out-of-line field storage to an earlier checkpoint.
    #[inline]
    pub fn truncate(
        &mut self,
        nodes_len: usize,
        expressions_len: usize,
        statements_len: usize,
        name_patterns_len: usize,
        class_elements_len: usize,
        type_jsx_len: usize,
        program_aux_len: usize,
        classes_len: usize,
        functions_len: usize,
        node_edges_len: usize,
        extra_data_len: usize,
    ) {
        assert!(nodes_len <= self.nodes.slot_len());
        assert!(expressions_len <= self.expressions.slot_len());
        assert!(statements_len <= self.statements.slot_len());
        assert!(name_patterns_len <= self.name_patterns.slot_len());
        assert!(class_elements_len <= self.class_elements.slot_len());
        assert!(type_jsx_len <= self.type_jsx.slot_len());
        assert!(program_aux_len <= self.program_aux.slot_len());
        assert!(classes_len <= self.classes.len());
        assert!(functions_len <= self.functions.len());
        assert!(node_edges_len <= self.node_edges.len());
        assert!(extra_data_len <= self.extra_data.len());
        self.nodes.truncate(nodes_len);
        self.expressions.truncate(expressions_len);
        self.statements.truncate(statements_len);
        self.name_patterns.truncate(name_patterns_len);
        self.class_elements.truncate(class_elements_len);
        self.type_jsx.truncate(type_jsx_len);
        self.program_aux.truncate(program_aux_len);
        self.classes.truncate(classes_len);
        self.functions.truncate(functions_len);
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
mod hole_tests {
    use super::*;

    fn push_source<const HOLES: usize>(store: &mut NodeStore<'_, 16, HOLES>, count: usize) {
        for _ in 0..count {
            store.push(false, DUMMY_SP, NodeKind::Program, NodePayload(0));
        }
    }

    #[test]
    fn zero_hole_policy_has_dense_slots() {
        let allocator = Allocator::new();
        for live in [0, 1, 15, 16, 17, 31, 32] {
            let mut store = NodeStore::<16, 0>::new_in(&allocator);
            push_source(&mut store, live);
            assert_eq!(store.live_len(), live);
            assert_eq!(store.slot_len(), live);
            assert_eq!(store.metrics().available_holes, 0);
        }
    }

    #[test]
    fn four_hole_policy_inserts_and_reuses_slots() {
        let allocator = Allocator::new();
        let mut store = NodeStore::<16, 4>::new_in(&allocator);
        push_source(&mut store, 16);
        assert_eq!(store.live_len(), 16);
        assert_eq!(store.slot_len(), 20);
        assert_eq!(store.metrics().available_holes, 4);

        for _ in 0..4 {
            store.push(true, DUMMY_SP, NodeKind::Program, NodePayload(0));
        }
        assert_eq!(store.live_len(), 20);
        assert_eq!(store.slot_len(), 20);
        assert_eq!(store.metrics().available_holes, 0);

        store.push(true, DUMMY_SP, NodeKind::Program, NodePayload(0));
        assert_eq!(store.live_len(), 21);
        assert_eq!(store.slot_len(), 21);
    }

    #[test]
    fn non_default_policy_and_checkpoint_truncate() {
        let allocator = Allocator::new();
        let mut store = NodeStore::<16, 2>::new_in(&allocator);
        push_source(&mut store, 16);
        assert_eq!(store.slot_len(), 18);
        let checkpoint = store.slot_len();

        push_source(&mut store, 16);
        assert_eq!(store.live_len(), 32);
        assert_eq!(store.slot_len(), 36);
        store.truncate(checkpoint);
        assert_eq!(store.live_len(), 16);
        assert_eq!(store.slot_len(), 18);
        assert_eq!(store.metrics().available_holes, 2);
    }

    #[test]
    fn generated_node_kinds_select_disjoint_stores() {
        assert_eq!(NodeKind::CallExpression.store_tag(), StoreTag::Expression);
        assert_eq!(
            NodeKind::ExpressionStatement.store_tag(),
            StoreTag::Statement
        );
        assert_eq!(
            NodeKind::BindingIdentifier.store_tag(),
            StoreTag::NamePattern
        );
        assert_eq!(
            NodeKind::PropertyDefinition.store_tag(),
            StoreTag::ClassElement
        );
        assert_eq!(NodeKind::TsTypeAnnotation.store_tag(), StoreTag::TypeJsx);
        assert_eq!(NodeKind::Program.store_tag(), StoreTag::ProgramAux);
        assert_eq!(NodeKind::Class.store_tag(), StoreTag::Class);
        assert_eq!(NodeKind::Function.store_tag(), StoreTag::Function);
    }
}
