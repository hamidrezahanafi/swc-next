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
    pub mod ast_node;
    pub(crate) mod ast_payload;
    mod ast_property;
    pub(crate) mod ast_visitor;
}

use generated::ast_payload::NodePayload;
use node_id::{ExtraData, ExtraDataId, RawNodeId};
use oxc_index::Idx;
use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

pub use ast::*;
pub use comment::*;
pub use derive::*;
pub use diagnostic::*;
pub use generated::ast_node::*;
pub use generated::ast_visitor::*;
pub use generativity::{Guard as AstGuard, make_guard as make_ast_guard};
pub use node_id::{AstNode, ExtraDataValue, NodeId, SubRange};
pub use semantic::{ReferenceId, ScopeId, SymbolId};
pub use strings::{AstUtf8, AstWtf8};
pub use syntax::*;
pub use visit::{TraverseAction, TraverseCtx, TraverseMutCtx};

use swc_next_allocator::multi_index_vec;

use crate::strings::StringPool;

/// AST context that stores everything about the flattening AST.
///
/// `'a` is the source/allocator lifetime; `'id` uniquely brands the AST and its string pool.
/// Create a fresh guard for each AST, including ASTs sharing an allocator.
///
/// ```
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(guard);
/// let mut ast = Ast::new("source", &allocator, guard);
/// let text = ast.add_utf8("value");
/// assert_eq!(ast.get_utf8(text), "value");
/// ```
///
/// Strings from another pool cannot be read, even with the same allocator.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let right = Ast::new("", &allocator, right_guard);
/// let text = left.add_utf8("value");
/// right.get_utf8(text);
/// ```
///
/// Converting to WTF-8 preserves the pool identity.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let right = Ast::new("", &allocator, right_guard);
/// right.get_wtf8(left.add_utf8("value").into());
/// ```
///
/// Builders and setters also require strings from their AST's pool.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, DUMMY_SP, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// right.numeric_literal(DUMMY_SP, 1.0, Some(left.add_utf8("1")));
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, DUMMY_SP, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let node = right.numeric_literal(DUMMY_SP, 1.0, None);
/// node.set_raw(&mut right, Some(left.add_utf8("1")));
/// ```
///
/// Typed extra-data lists cannot transfer strings between pools either.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::{Ast, make_ast_guard};
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// right.add_sub_range([left.add_utf8("value")]);
/// ```
///
/// Node properties, raw IDs, enum downcasts, and root setters preserve ownership.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let node = left.numeric_literal(DUMMY_SP, 1.0, None);
/// node.value(&right);
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let node = left.numeric_literal(DUMMY_SP, 1.0, None);
/// right.span(node.node_id());
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let expr: Expr = Expr::NumericLiteral(left.numeric_literal(DUMMY_SP, 1.0, None));
/// expr.as_numeric_literal(&right);
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let expr = Expr::NumericLiteral(left.numeric_literal(DUMMY_SP, 1.0, None));
/// right.set_root_expression(expr);
/// ```
///
/// Child builders and setters reject foreign nodes.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let expr = Expr::NumericLiteral(left.numeric_literal(DUMMY_SP, 1.0, None));
/// right.parenthesized_expression(DUMMY_SP, expr);
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let expr = Expr::NumericLiteral(left.numeric_literal(DUMMY_SP, 1.0, None));
/// let own = Expr::NumericLiteral(right.numeric_literal(DUMMY_SP, 2.0, None));
/// let parent = right.parenthesized_expression(DUMMY_SP, own);
/// parent.set_expression(&mut right, expr);
/// ```
///
/// Even scalar lists and their slots are tied to the AST that allocated them.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let range = left.add_sub_range([1.0]);
/// range.get_node(&right, 0);
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let range = left.add_sub_range([1.0]);
/// right.get_node_in_sub_range(range.first().unwrap());
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let foreign = left.add_sub_range([1.0]);
/// let own = right.add_sub_range([2.0]);
/// own.replace_slot(&mut right, foreign.first().unwrap(), 3.0);
/// ```
///
/// Cloning and traversal require the same owner as well.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let node = left.numeric_literal(DUMMY_SP, 1.0, None);
/// node.clone_in(&mut right);
/// ```
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// struct Walker;
/// impl<'a, 'id> Traverse<'a, 'id> for Walker {}
/// let node = left.numeric_literal(DUMMY_SP, 1.0, None);
/// let mut ctx = TraverseCtx::new(&right, ());
/// node.traverse_with(&mut Walker, &mut ctx);
/// ```
///
/// Foreign nodes cannot be inserted into extra-data lists or returned as
/// mutable traversal replacements.
///
/// ```compile_fail
/// use swc_next_allocator::Allocator;
/// use swc_next_ecma_ast::*;
/// let allocator = Allocator::new();
/// make_ast_guard!(left_guard);
/// let mut left = Ast::new("", &allocator, left_guard);
/// make_ast_guard!(right_guard);
/// let mut right = Ast::new("", &allocator, right_guard);
/// let node = left.numeric_literal(DUMMY_SP, 1.0, None);
/// right.add_sub_range([node]);
/// ```
///
/// ```compile_fail
/// use swc_next_ecma_ast::*;
/// struct Replace<'foreign>(NumericLiteral<'foreign>);
/// impl<'a, 'id, 'foreign> TraverseMut<'a, 'id> for Replace<'foreign> {
///     fn enter_numeric_literal(
///         &mut self,
///         _node: NumericLiteral<'id>,
///         _ctx: &mut TraverseMutCtx<'_, 'a, 'id>,
///     ) -> TraverseAction<NumericLiteral<'id>> {
///         TraverseAction::ReplaceSkip(self.0)
///     }
/// }
/// ```
pub struct Ast<'a, 'id> {
    allocator: &'a Allocator,

    root: Option<RawNodeId>,

    source_type: SourceType,

    lang: Lang,

    attached_comments: ArenaVec<'a, Comment>,

    /// Flattening AST nodes.
    pub(crate) nodes: NodeData<'a>,

    /// Flattening AST fields.
    extra_data: ArenaVec<'a, ExtraData>,

    /// Source-backed and decoded AST strings.
    strings: StringPool<'a, 'id>,
}

multi_index_vec! {
    /// Arena-backed structure-of-arrays storage for flattening AST nodes.
    pub(crate) struct NodeData<'a, RawNodeId> {
        pub(crate) span => span_mut => span_unchecked => span_mut_unchecked: Span,
        pub(crate) kind => kind_mut => kind_unchecked => kind_mut_unchecked: NodeKind,
        pub(crate) data => data_mut => data_unchecked => data_mut_unchecked: NodePayload,
    }
}

impl<'a, 'id> Ast<'a, 'id> {
    #[inline]
    fn root_node_id(&self) -> NodeId<'id> {
        NodeId::from_raw(self.root.expect("AST has no root"))
    }

    /// Returns the AST's root program.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a program.
    #[inline]
    pub fn root_program(&self) -> Program<'id> {
        let root = self.root_node_id();
        assert_eq!(
            self.node_kind(root),
            NodeKind::Program,
            "AST root is not a program"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Program::from_node_id_unchecked(root) }
    }

    /// Returns the AST's root expression.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not an expression.
    #[inline]
    pub fn root_expression(&self) -> Expr<'id> {
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_expr(),
            "AST root is not an expression"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Expr::from_node_id_unchecked(root) }
    }

    /// Returns the AST's root statement.
    ///
    /// # Panics
    ///
    /// Panics if the AST has no root or its root is not a statement.
    #[inline]
    pub fn root_statement(&self) -> Stmt<'id> {
        let root = self.root_node_id();
        assert!(
            self.node_kind(root).is_stmt(),
            "AST root is not a statement"
        );
        // SAFETY: The node kind was checked above.
        unsafe { Stmt::from_node_id_unchecked(root) }
    }

    #[inline]
    fn add_node(&mut self, span: Span, kind: NodeKind, data: NodePayload) -> NodeId<'id> {
        NodeId::from_raw(self.nodes.push(span, kind, data))
    }

    /// Replace `old`'s node data with `new`'s while preserving `old`'s node ID.
    ///
    /// # Safety
    ///
    /// Replacing the data may change the concrete node kind stored at `old`,
    /// invalidating typed handles and AST edges that refer to it.
    /// The caller must ensure that every typed handle and every AST edge used
    /// after this call accepts the replacement kind.
    /// In particular, a handle for `old`'s previous concrete kind must never be
    /// used to access node-specific data after the replacement.
    #[inline]
    pub unsafe fn set_data(&mut self, old: NodeId<'id>, new: NodeId<'id>) {
        let span = *self.nodes.span(new.raw());
        let kind = *self.nodes.kind(new.raw());
        let data = *self.nodes.data(new.raw());
        *self.nodes.span_mut(old.raw()) = span;
        *self.nodes.kind_mut(old.raw()) = kind;
        *self.nodes.data_mut(old.raw()) = data;
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
    pub unsafe fn replace_data<Old: AstNode<'id>, New: AstNode<'id>>(
        &mut self,
        old: Old,
        new: New,
    ) -> New {
        let old = old.node_id();
        let new = new.node_id();
        assert_ne!(old, new);
        assert_eq!(new.index() + 1, self.nodes.len());

        let span = *self.nodes.span(new.raw());
        let kind = *self.nodes.kind(new.raw());
        let data = *self.nodes.data(new.raw());
        *self.nodes.span_mut(old.raw()) = span;
        *self.nodes.kind_mut(old.raw()) = kind;
        *self.nodes.data_mut(old.raw()) = data;
        self.nodes.truncate(new.index());

        unsafe { New::from_node_id_unchecked(old) }
    }

    #[inline]
    pub fn add_sub_range<N: ExtraDataValue<'id>, I: IntoIterator<Item = N>>(
        &mut self,
        iter: I,
    ) -> SubRange<'id, N> {
        let start = self.next_extra_data_id();
        for node in iter {
            let data = node.into_extra_data();
            self.extra_data.push(data);
        }
        SubRange::new(start, self.next_extra_data_id())
    }

    /// Append pre-encoded extra data and return its starting index.
    fn add_extra_data_slice(&mut self, data: &[ExtraData]) -> ExtraDataId {
        let start = self.next_extra_data_id();
        self.extra_data.extend_from_slice_copy(data);
        start
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
    /// Panics if the offsets do not bound a UTF-8 source slice.
    #[inline]
    pub fn source_slice(&self, start: u32, end: u32) -> AstUtf8<'id> {
        self.strings.source_slice(start, end)
    }

    /// Returns a slice of the source string corresponding to the given start and end offsets.
    /// Panics if the offsets do not bound a UTF-8 source slice.
    #[inline]
    pub fn source_slice_wtf8(&self, start: u32, end: u32) -> AstWtf8<'id> {
        self.strings.source_slice_wtf8(start, end)
    }

    #[inline]
    pub fn add_utf8(&mut self, value: &str) -> AstUtf8<'id> {
        self.strings.add_utf8(value)
    }

    #[inline]
    pub fn add_wtf8(&mut self, value: &swc_next_allocator::wtf8::Wtf8) -> AstWtf8<'id> {
        self.strings.add_wtf8(value)
    }

    /// Borrows immutable string bytes for the source/allocator lifetime.
    /// The reference remains valid when strings are added or the AST is dropped.
    #[inline]
    pub fn get_utf8(&self, id: AstUtf8<'id>) -> &'a str {
        self.strings.get_utf8(id)
    }

    /// Borrows immutable string bytes for the source/allocator lifetime.
    /// The reference remains valid when strings are added or the AST is dropped.
    #[inline]
    pub fn get_wtf8(&self, id: AstWtf8<'id>) -> &'a swc_next_allocator::wtf8::Wtf8 {
        self.strings.get_wtf8(id)
    }

    #[inline]
    pub fn empty_utf8(&self) -> AstUtf8<'id> {
        self.strings.source_slice(0, 0)
    }

    #[inline]
    pub fn empty_wtf8(&self) -> AstWtf8<'id> {
        self.strings.source_slice_wtf8(0, 0)
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns a branded ID if `index` identifies a live node in this AST.
    #[inline]
    pub fn node_id_at(&self, index: usize) -> Option<NodeId<'id>> {
        (index < self.node_count()).then(|| NodeId::from_raw_index(index as u32))
    }

    /// Iterates the current node IDs without borrowing the node storage.
    /// Appending nodes leaves these IDs valid; unsafe truncation may invalidate them.
    #[inline]
    pub fn node_ids(
        &self,
    ) -> impl ExactSizeIterator<Item = NodeId<'id>> + DoubleEndedIterator + use<'id> {
        (0..self.node_count()).map(|index| NodeId::from_raw_index(index as u32))
    }

    /// Returns the number of entries in the out-of-line field table.
    #[inline]
    pub fn extra_data_count(&self) -> usize {
        self.extra_data.len()
    }

    /// Returns the concrete type stored at `node_id`.
    #[inline]
    pub fn node_kind(&self, node_id: NodeId<'id>) -> NodeKind {
        *self.nodes.kind(node_id.raw())
    }

    /// Returns the source span stored for `node_id`.
    #[inline]
    pub fn span(&self, node_id: NodeId<'id>) -> Span {
        *self.nodes.span(node_id.raw())
    }
}

#[doc(hidden)]
impl<'a, 'id> Ast<'a, 'id> {
    /// Create a flattening AST backed by `source` and `allocator`.
    pub fn new(source: &'a str, allocator: &'a Allocator, guard: AstGuard<'id>) -> Self {
        Self::new_with_brand(source, allocator, guard.into())
    }

    /// Creates storage with a caller-provided AST brand.
    ///
    /// # Safety
    ///
    /// The lifetime must be a trusted invariant brand. The caller must ensure
    /// that no node, string, range, or slot handle from another AST with this
    /// brand is used here, or vice versa, including after this AST is returned.
    pub unsafe fn new_unchecked(source: &'a str, allocator: &'a Allocator) -> Self {
        // SAFETY: The caller guarantees that this is a trusted AST brand.
        Self::new_with_brand(source, allocator, unsafe { generativity::Id::new() })
    }

    fn new_with_brand(
        source: &'a str,
        allocator: &'a Allocator,
        brand: generativity::Id<'id>,
    ) -> Self {
        Self {
            allocator,
            root: None,
            source_type: SourceType::Module,
            lang: Lang::Js,
            attached_comments: ArenaVec::new_in(allocator),
            nodes: NodeData::new_in(allocator),
            extra_data: ArenaVec::with_capacity_in(0, allocator),
            strings: StringPool::new_in(allocator, source, brand),
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

    /// Removes all attached comments.
    pub fn clear_attached_comments(&mut self) {
        self.attached_comments.clear();
    }

    /// Sets a program as the AST root.
    #[inline]
    pub fn set_root_program(&mut self, root: Program<'id>) {
        self.root = Some(root.node_id().raw());
    }

    /// Sets an expression as the AST root.
    #[inline]
    pub fn set_root_expression(&mut self, root: Expr<'id>) {
        self.root = Some(root.node_id().raw());
    }

    /// Sets a statement as the AST root.
    #[inline]
    pub fn set_root_statement(&mut self, root: Stmt<'id>) {
        self.root = Some(root.node_id().raw());
    }

    /// Reserves storage for parser output without changing logical lengths.
    #[inline]
    pub fn reserve(&mut self, additional_nodes: usize, additional_extra_data: usize) {
        self.nodes.reserve(additional_nodes);
        self.extra_data.reserve(additional_extra_data);
    }

    /// Rewinds node and out-of-line field storage to an earlier checkpoint.
    ///
    /// # Safety
    ///
    /// The root and every retained node or extra-data value used after this call
    /// must remain valid, including their node references, sub-ranges, and
    /// out-of-line fields. Handles, sub-ranges, and extra-data indices referring
    /// to removed storage must not be used afterward, even if their indices are
    /// later reused.
    #[inline]
    pub unsafe fn truncate(&mut self, nodes_len: usize, extra_data_len: usize) {
        assert!(nodes_len <= self.nodes.len());
        assert!(extra_data_len <= self.extra_data.len());
        self.nodes.truncate(nodes_len);
        self.extra_data.truncate(extra_data_len);
    }
}

#[doc(hidden)]
impl<'id> Ast<'_, 'id> {
    #[inline]
    fn add_extra<T: ExtraDataValue<'id>>(&mut self, value: T) -> ExtraDataId {
        let id = self.next_extra_data_id();
        self.extra_data.push(value.into_extra_data());
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

impl<'id> From<FormalParameterPattern<'id>> for BindingPattern<'id> {
    fn from(pattern: FormalParameterPattern<'id>) -> Self {
        // SAFETY: all formal parameter binding variants are also BindingPattern variants.
        unsafe { Self::from_node_id_unchecked(pattern.node_id()) }
    }
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn owned_handles() {
        let allocator = Allocator::new();
        make_ast_guard!(guard);
        let mut ast = Ast::new("", &allocator, guard);
        let node = ast.numeric_literal(Span::new(0, 1), 1.0, None);
        let expr: Expr = Expr::NumericLiteral(node);
        ast.set_root_expression(expr);
        assert_eq!(node.value(&ast), 1.0);
        assert_eq!(ast.span(node.node_id()), Span::new(0, 1));
        assert_eq!(
            expr.as_numeric_literal(&ast).unwrap().node_id(),
            node.node_id()
        );
        assert_eq!(ast.root_expression().node_id(), node.node_id());
        assert_eq!(ast.node_id_at(0), Some(node.node_id()));
        assert_eq!(ast.node_id_at(1), None);

        let snapshot = ast.node_ids();
        let own = ast.numeric_literal(DUMMY_SP, 2.0, None);
        let parent = ast.parenthesized_expression(DUMMY_SP, expr);
        parent.set_expression(&mut ast, Expr::NumericLiteral(own));
        assert_eq!(parent.expression(&ast).node_id(), own.node_id());
        assert_eq!(snapshot.collect::<Vec<_>>(), vec![node.node_id()]);
        assert_eq!(ast.node_ids().len(), 3);

        let clone = node.clone_in(&mut ast);
        assert_ne!(clone.node_id(), node.node_id());
        assert_eq!(clone.value(&ast), 1.0);

        struct Walker;
        impl<'a, 'id> Traverse<'a, 'id> for Walker {}
        let mut ctx = TraverseCtx::new(&ast, ());
        node.traverse_with(&mut Walker, &mut ctx);

        let mut range = ast.add_sub_range([1.0, 2.0]);
        let tail = range.split_off(1);
        tail.replace_slot(&mut ast, tail.first().unwrap(), 3.0);
        assert_eq!(range.get_node(&ast, 0), Some(1.0));
        assert_eq!(tail.get_node(&ast, 0), Some(3.0));
        assert!(SubRange::<f64>::empty().get_node(&ast, 0).is_none());

        let nodes = ast.add_sub_range([node, own]);
        assert_eq!(nodes.get_node(&ast, 1).unwrap().node_id(), own.node_id());
    }

    #[test]
    #[should_panic(expected = "slot is outside this range")]
    fn list_slot_membership() {
        let allocator = Allocator::new();
        make_ast_guard!(guard);
        let mut ast = Ast::new("", &allocator, guard);
        let first = ast.add_sub_range([1.0]);
        let second = ast.add_sub_range([2.0]);
        first.replace_slot(&mut ast, second.first().unwrap(), 3.0);
    }
}
