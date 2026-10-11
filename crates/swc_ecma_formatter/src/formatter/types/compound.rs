//! Source-ordered type members carry a prepared operator gap and actual ignore owner.

use swc_next_ecma_ast::{Comment, GetSpan, Span, SubRange, TsType, TsTypeData};

use super::context::TypeContext;
use super::{Formatter, PreparedType, PreparedTypeParentheses, TypeBoundaryLayout, TypeLayout};
use crate::formatter::comments::{CommentBinding, CommentBoundary, CommentRunLayout};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{
    FormatError,
    document::{Fragment, Writer},
};

#[derive(Clone, Copy)]
pub(super) enum CompoundTypeKind {
    Union,
    Intersection,
}

impl CompoundTypeKind {
    fn token(self) -> &'static str {
        match self {
            Self::Union => "|",
            Self::Intersection => "&",
        }
    }
}

struct CompoundHead<'ast> {
    current: PreparedType,
    leading: Span,
    first_start: u32,
    leading_comments: Option<&'ast [Comment]>,
    single: bool,
}

pub(in crate::formatter) struct PreparedUnion<'ast> {
    data: TsTypeData,
    leading: UnionLeading<'ast>,
    links: Vec<UnionLink<'ast>>,
    last: PreparedType,
    opening: LineConstraint,
    hug: bool,
}

pub(super) struct PreparedIntersection<'ast> {
    data: TsTypeData,
    leading: IntersectionPrefix<'ast>,
    links: Vec<IntersectionLink<'ast>>,
    last: IntersectionMember<'ast>,
}

impl<'ast> PreparedIntersection<'ast> {
    pub(super) fn movable_head_start(&self) -> Option<u32> {
        match &self.leading {
            IntersectionPrefix::Leading { boundary, .. } => boundary.first_comment_start(),
            _ => None,
        }
    }

    fn first_member(&self) -> &IntersectionMember<'ast> {
        self.links.first().map_or(&self.last, |link| &link.left)
    }

    fn first_member_mut(&mut self) -> &mut IntersectionMember<'ast> {
        match self.links.first_mut() {
            Some(link) => &mut link.left,
            None => &mut self.last,
        }
    }
}

enum IntersectionMember<'ast> {
    Type(PreparedType),
    Prepared(Box<PreparedTypeContent<'ast>>),
}

struct IntersectionLink<'ast> {
    left: IntersectionMember<'ast>,
    boundary: IntersectionBoundary<'ast>,
    continuation: IntersectionContinuation,
}

#[derive(Clone, Copy)]
enum IntersectionContinuation {
    Inline { indent: bool },
    Line,
}

enum IntersectionBoundary<'ast> {
    Source(Span),
    Bound {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
        leading_own_line: bool,
    },
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
        member_start: u32,
        leading_own_line: bool,
    },
}

enum IntersectionPrefix<'ast> {
    Empty,
    Source(Span),
    Leading {
        boundary: CommentBoundary<'ast>,
        member_start: u32,
    },
    BoundMember {
        boundary: CommentBoundary<'ast>,
        continuation: IntersectionContinuation,
    },
    Member {
        boundary: CommentBoundary<'ast>,
        member_start: u32,
        leading_own_line: bool,
        continuation: IntersectionContinuation,
    },
}

struct UnionLink<'ast> {
    left: PreparedType,
    boundary: TypeBoundaryLayout<'ast>,
}

enum UnionLeading<'ast> {
    Empty,
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: UnionMemberPrefix<'ast>,
        retain_operator: bool,
    },
}

enum UnionMemberPrefix<'ast> {
    Boundary(CommentBoundary<'ast>),
    Leading {
        boundary: CommentBoundary<'ast>,
        member_start: u32,
    },
}

impl UnionMemberPrefix<'_> {
    fn boundary(&self) -> &CommentBoundary<'_> {
        match self {
            Self::Boundary(boundary) | Self::Leading { boundary, .. } => boundary,
        }
    }
}

#[derive(Clone, Copy)]
enum UnionHug {
    NoObject,
    Object,
    Unavailable,
}

impl UnionHug {
    fn include(&mut self, member: PreparedType) {
        if member.source_owner.is_some() {
            *self = Self::Unavailable;
            return;
        }
        match member.data {
            TsTypeData::TsNullKeyword(_) | TsTypeData::TsVoidKeyword(_) => {}
            TsTypeData::TsTypeLiteral(_) | TsTypeData::TsTypeReference(_)
                if matches!(self, Self::NoObject) =>
            {
                *self = Self::Object;
            }
            _ => *self = Self::Unavailable,
        }
    }

    fn include_boundary(&mut self, boundary: &TypeBoundaryLayout<'_>) {
        if !matches!(boundary, TypeBoundaryLayout::Formatted { before, after }
            if before.is_empty() && after.is_empty())
        {
            *self = Self::Unavailable;
        }
    }
}

impl PreparedUnion<'_> {
    pub(super) fn alias_layout(&self) -> TypeLayout {
        if self.hug {
            TypeLayout::Root
        } else {
            TypeLayout::AliasRhs
        }
    }
}

pub(super) struct SingleCompoundHead<'ast> {
    data: TsTypeData,
    current: PreparedType,
    leading: Span,
    first_start: u32,
    comments: &'ast [Comment],
    kind: CompoundTypeKind,
    movable: bool,
}

impl SingleCompoundHead<'_> {
    pub(super) fn start(&self) -> u32 {
        self.comments
            .first()
            .map_or(self.first_start, |comment| comment.span.start)
    }

    pub(super) fn movable(&self) -> bool {
        self.movable
    }
}

/// A resolved first transferable packet on the actual leading spine.
#[derive(Clone, Copy)]
pub(super) struct MovableTypeHead {
    owner: TsTypeData,
}

pub(super) enum PreparedTypeContent<'ast> {
    Type(PreparedType),
    Union(PreparedUnion<'ast>),
    Intersection(PreparedIntersection<'ast>),
    Postfix(super::postfix::PreparedPostfixType<'ast>),
    Conditional(super::conditional::PreparedConditionalType<'ast>),
    Parentheses(PreparedTypeParentheses<'ast>),
    Compound(SingleCompoundHead<'ast>),
}

impl PreparedTypeContent<'_> {
    pub(super) fn alias_layout(&self) -> TypeLayout {
        match self {
            Self::Union(union) => union.alias_layout(),
            Self::Type(node)
                if node.source_owner.is_none()
                    && matches!(node.data, TsTypeData::TsUnionType(_)) =>
            {
                TypeLayout::AliasRhs
            }
            Self::Compound(single) if matches!(single.kind, CompoundTypeKind::Union) => {
                TypeLayout::AliasRhs
            }
            Self::Type(_)
            | Self::Intersection(_)
            | Self::Postfix(_)
            | Self::Conditional(_)
            | Self::Parentheses(_)
            | Self::Compound(_) => TypeLayout::Root,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    /// Check only the transferable leading spine before preparing optional head state.
    pub(super) fn type_movable_head(
        &self,
        node: PreparedType,
        mut context: TypeContext,
    ) -> Option<MovableTypeHead> {
        if node.source_owner.is_some() {
            return None;
        }
        let ast = self.ast;
        let mut current = node.node;
        loop {
            let data = ast.ts_type_data(current);
            let types = match data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let child = parentheses.type_annotation(ast);
                    let span = parentheses.span(ast);
                    let child_span = child.span(ast);
                    let leading = Span::new(span.start + 1, child_span.start);
                    let before = self.comments.in_range(leading);
                    if !self
                        .comments
                        .in_range(Span::new(child_span.end, span.end - 1))
                        .is_empty()
                        || self.diagnostic_range_is_sensitive(span)
                        || !matches!(
                            self.comment_binding(before, span, Some(child_span)),
                            CommentBinding::Unbound
                        )
                    {
                        return None;
                    }
                    if !before.is_empty() {
                        if context.preserves_union_head(child, ast) {
                            return None;
                        }
                        return Some(MovableTypeHead { owner: data });
                    }
                    current = child;
                    continue;
                }
                TsTypeData::TsConditionalType(conditional) => {
                    current = conditional.check_type(ast);
                    context = TypeContext::ConditionalCheck;
                    continue;
                }
                TsTypeData::TsArrayType(array) => {
                    current = array.element_type(ast);
                    context = TypeContext::PostfixOperand;
                    continue;
                }
                TsTypeData::TsIndexedAccessType(index) => {
                    current = index.object_type(ast);
                    context = TypeContext::PostfixOperand;
                    continue;
                }
                TsTypeData::TsUnionType(union) if union.types(ast).len() == 1 => union.types(ast),
                TsTypeData::TsIntersectionType(intersection) => intersection.types(ast),
                _ => return None,
            };
            let first = types.iter().next()?;
            let first = ast.get_node_in_sub_range(first);
            let first_span = first.span(ast);
            let leading = Span::new(current.span(ast).start, first_span.start);
            let comments = self.comments.in_range(leading);
            if !matches!(
                self.comment_binding(comments, leading, Some(first_span)),
                CommentBinding::Unbound
            ) {
                return None;
            }
            if !comments.is_empty() {
                return Some(MovableTypeHead { owner: data });
            }
            if types.len() > 1 {
                context = TypeContext::CompoundMember;
            }
            current = first;
        }
    }

    /// Keep a resolved unprepared frontier alongside already prepared head packets.
    pub(super) fn prepared_content_head(
        &self,
        content: &PreparedTypeContent<'ast>,
        context: TypeContext,
    ) -> (bool, Option<MovableTypeHead>) {
        let frontier = match content {
            PreparedTypeContent::Type(node) => self.type_movable_head(*node, context),
            PreparedTypeContent::Intersection(intersection) => {
                self.intersection_movable_head(intersection)
            }
            PreparedTypeContent::Compound(single)
                if single.movable && single.comments.is_empty() =>
            {
                self.type_movable_head(single.current, context)
            }
            _ => None,
        };
        let prepared_packet = match content {
            PreparedTypeContent::Parentheses(parentheses) => {
                parentheses.movable
                    && !context.preserves_union_head(parentheses.child.node, self.ast)
            }
            PreparedTypeContent::Compound(single) => single.movable && !single.comments.is_empty(),
            _ => false,
        };
        (prepared_packet || frontier.is_some(), frontier)
    }

    pub(super) fn prepared_content_accepts_caller_head(
        &self,
        content: &PreparedTypeContent<'ast>,
    ) -> bool {
        match content {
            PreparedTypeContent::Type(node) => {
                node.source_owner.is_none()
                    && matches!(
                        node.data,
                        TsTypeData::TsParenthesizedType(_)
                            | TsTypeData::TsUnionType(_)
                            | TsTypeData::TsIntersectionType(_)
                            | TsTypeData::TsArrayType(_)
                            | TsTypeData::TsIndexedAccessType(_)
                            | TsTypeData::TsConditionalType(_)
                    )
            }
            PreparedTypeContent::Union(_)
            | PreparedTypeContent::Intersection(_)
            | PreparedTypeContent::Postfix(_)
            | PreparedTypeContent::Conditional(_) => true,
            PreparedTypeContent::Parentheses(parentheses) => parentheses.movable,
            PreparedTypeContent::Compound(single) => single.movable,
        }
    }

    pub(super) fn format_compound_type(
        &mut self,
        node: PreparedType,
        types: SubRange<TsType>,
        kind: CompoundTypeKind,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let mut members = types.iter().map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = members.next() else {
            return Err(self.unsupported("empty compound type", node.node));
        };
        let first_span = first.span(ast);
        let leading = Span::new(node.node.span(ast).start, first_span.start);
        let mut current = self.prepare_type(first, ContentMode::Formatted);
        current.source_owner = node.source_owner;
        if types.len() == 1 {
            let single =
                self.prepare_single_compound_head(node, current, leading, first_span, kind)?;
            let content = self.format_single_compound_head(single, context, writer)?;
            return self.format_prepared_type_content(content, context, layout, writer);
        }
        if matches!(kind, CompoundTypeKind::Union) {
            let union =
                self.prepare_union_members(node, current, leading, first_span.start, members)?;
            return self.format_prepared_union(union, context, layout, None, writer);
        }
        let intersection =
            self.prepare_intersection_members(node, current, leading, first_span.start, members)?;
        self.format_prepared_intersection(intersection, context, layout, writer)
    }

    pub(super) fn prepare_intersection(
        &self,
        node: PreparedType,
        types: SubRange<TsType>,
    ) -> Result<PreparedIntersection<'ast>, FormatError> {
        let ast = self.ast;
        let mut members = types.iter().map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = members.next() else {
            return Err(self.unsupported("empty intersection type", node.node));
        };
        let first_start = first.span(ast).start;
        let mut first = self.prepare_type(first, ContentMode::Formatted);
        first.source_owner = node.source_owner;
        self.prepare_intersection_members(
            node,
            first,
            Span::new(node.node.span(ast).start, first_start),
            first_start,
            members,
        )
    }

    pub(super) fn intersection_movable_head(
        &self,
        intersection: &PreparedIntersection<'ast>,
    ) -> Option<MovableTypeHead> {
        if matches!(intersection.leading, IntersectionPrefix::Source(_)) {
            return None;
        }
        if intersection.movable_head_start().is_some() {
            return Some(MovableTypeHead {
                owner: intersection.data,
            });
        }
        let IntersectionMember::Type(first) = intersection.first_member() else {
            return None;
        };
        self.type_movable_head(*first, TypeContext::CompoundMember)
    }

    pub(super) fn format_intersection_head(
        &mut self,
        intersection: PreparedIntersection<'ast>,
        head: Option<MovableTypeHead>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedIntersection<'ast>, FormatError> {
        let Some(head) = head else {
            return Ok(intersection);
        };
        let first_start = match intersection.first_member() {
            IntersectionMember::Type(first) => first.node.span(self.ast).start,
            IntersectionMember::Prepared(_) => return Ok(intersection),
        };
        self.format_intersection_head_with_pending(
            intersection,
            &[],
            first_start,
            Some(head),
            writer,
        )
    }

    fn format_intersection_head_with_pending(
        &mut self,
        mut intersection: PreparedIntersection<'ast>,
        mut pending: &'ast [Comment],
        mut pending_next_start: u32,
        head: Option<MovableTypeHead>,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedIntersection<'ast>, FormatError> {
        if matches!(intersection.leading, IntersectionPrefix::Source(_)) {
            self.comments.claim(pending)?;
            self.format_comment_run(
                pending,
                CommentRunLayout::GroupedLeading(pending_next_start),
                writer,
            )?;
            return Ok(intersection);
        }
        if let IntersectionPrefix::Leading {
            boundary,
            member_start,
        } = std::mem::replace(&mut intersection.leading, IntersectionPrefix::Empty)
            && let Some(first_start) = boundary.first_comment_start()
        {
            self.comments.claim(pending)?;
            self.format_comment_run(
                pending,
                CommentRunLayout::GroupedLeading(first_start),
                writer,
            )?;
            pending = boundary.comments();
            pending_next_start = member_start;
        }
        let consumed_head = head.is_some_and(|head| head.owner == intersection.data);
        let member = intersection.first_member_mut();
        let IntersectionMember::Type(first) = member else {
            self.comments.claim(pending)?;
            self.format_comment_run(
                pending,
                CommentRunLayout::GroupedLeading(pending_next_start),
                writer,
            )?;
            return Ok(intersection);
        };
        let content = self.format_movable_type_head(
            *first,
            pending,
            pending_next_start,
            if consumed_head {
                self.type_movable_head(*first, TypeContext::CompoundMember)
            } else {
                head
            },
            TypeContext::CompoundMember,
            writer,
        )?;
        *member = match content {
            PreparedTypeContent::Type(node) => IntersectionMember::Type(node),
            content => IntersectionMember::Prepared(Box::new(content)),
        };
        Ok(intersection)
    }

    pub(super) fn format_prepared_intersection(
        &mut self,
        intersection: PreparedIntersection<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        let head = self.intersection_movable_head(&intersection);
        self.format_prepared_intersection_with_head(intersection, head, context, layout, writer)
    }

    pub(super) fn format_prepared_intersection_with_head(
        &mut self,
        intersection: PreparedIntersection<'ast>,
        head: Option<MovableTypeHead>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        let intersection = self.format_intersection_head(intersection, head, writer)?;
        self.format_type_in_context(
            intersection.data,
            context,
            layout,
            writer,
            |formatter, _, writer| formatter.format_intersection_members(intersection, writer),
        )
    }

    fn prepare_intersection_members(
        &self,
        node: PreparedType,
        mut current: PreparedType,
        leading: Span,
        first_start: u32,
        members: impl Iterator<Item = TsType>,
    ) -> Result<PreparedIntersection<'ast>, FormatError> {
        let leading = if leading.start == leading.end {
            IntersectionPrefix::Empty
        } else {
            let comments = self.comments.in_range(leading);
            if matches!(
                self.comment_binding(comments, leading, Some(current.node.span(self.ast))),
                CommentBinding::Unbound
            ) {
                if self.find_token(leading, comments, "&").is_none() {
                    return Err(FormatError::unsupported(
                        "intersection leading operator",
                        None,
                        leading,
                    ));
                }
                IntersectionPrefix::Leading {
                    boundary: self.prepare_comment_boundary(comments, leading.start, first_start),
                    member_start: first_start,
                }
            } else {
                self.select_type_head_comments(&mut current, comments, first_start);
                IntersectionPrefix::Source(leading)
            }
        };
        let mut current_object = self.intersection_member_is_object(current);
        let mut inline_indented = false;
        let mut links = Vec::with_capacity(members.size_hint().0);
        for (index, next) in members.enumerate() {
            let mut next = self.prepare_type(next, ContentMode::Formatted);
            let next_object = self.intersection_member_is_object(next);
            let gap = Span::new(
                current.node.span(self.ast).end,
                next.node.span(self.ast).start,
            );
            let boundary = self.prepare_intersection_boundary(gap, &mut current, &mut next)?;
            let own_line = matches!(
                &boundary,
                IntersectionBoundary::Bound {
                    leading_own_line: true,
                    ..
                } | IntersectionBoundary::Formatted {
                    leading_own_line: true,
                    ..
                }
            );
            let continuation = if current_object && next_object {
                IntersectionContinuation::Inline {
                    indent: inline_indented,
                }
            } else if !current_object && !next_object || own_line {
                IntersectionContinuation::Line
            } else {
                inline_indented = index > 0;
                IntersectionContinuation::Inline {
                    indent: inline_indented,
                }
            };
            links.push(IntersectionLink {
                left: IntersectionMember::Type(current),
                boundary,
                continuation,
            });
            current = next;
            current_object = next_object;
        }
        Ok(PreparedIntersection {
            data: node.data,
            leading,
            links,
            last: IntersectionMember::Type(current),
        })
    }

    fn prepare_intersection_boundary(
        &self,
        gap: Span,
        left: &mut PreparedType,
        right: &mut PreparedType,
    ) -> Result<IntersectionBoundary<'ast>, FormatError> {
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "&") else {
            return Err(FormatError::unsupported(
                "intersection type operator",
                None,
                gap,
            ));
        };
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let (before, after) = comments.split_at(split);
        if self.select_compound_boundary_suppression(operator, before, after, left, right) {
            return Ok(IntersectionBoundary::Source(gap));
        }
        if !matches!(
            self.comment_binding(comments, gap, Some(right.node.span(self.ast))),
            CommentBinding::Unbound
        ) {
            let leading_own_line = after
                .last()
                .is_some_and(|comment| self.line_breaks(comment.span.end, gap.end) != 0);
            return Ok(IntersectionBoundary::Bound {
                before: self.prepare_comment_boundary(before, gap.start, operator),
                after: self.prepare_comment_boundary(after, operator + 1, gap.end),
                leading_own_line,
            });
        }
        let Some((trailing_count, leading_own_line)) =
            self.intersection_comment_attachments(comments, operator, gap.end)
        else {
            // Relocating interleaved owners would change comment identity order.
            return Ok(IntersectionBoundary::Source(gap));
        };
        let (trailing, leading) = comments.split_at(trailing_count);
        Ok(IntersectionBoundary::Formatted {
            before: self.prepare_comment_boundary(trailing, gap.start, gap.end),
            after: self.prepare_comment_boundary(leading, operator + 1, gap.end),
            member_start: gap.end,
            leading_own_line,
        })
    }

    /// Classify bounded physical runs once, then carry the two real owners forward.
    fn intersection_comment_attachments(
        &self,
        comments: &[Comment],
        operator: u32,
        member_start: u32,
    ) -> Option<(usize, bool)> {
        let mut trailing_count = 0;
        let mut leading = false;
        let mut leading_own_line = false;
        let mut start = 0;
        while let Some(first) = comments.get(start) {
            let before_operator = first.span.end <= operator;
            let mut end = start + 1;
            let mut last = first;
            while let Some(next) = comments.get(end) {
                if (next.span.end <= operator) != before_operator
                    || !self.source_comment_gap_is_run(*last, *next)
                {
                    break;
                }
                last = next;
                end += 1;
            }
            let following = comments
                .get(end)
                .map_or(member_start, |next| next.span.start);
            let following = if before_operator && following > operator {
                operator
            } else {
                following
            };
            let ends_line = self.line_breaks(last.span.end, following) != 0;
            let run_leading =
                self.source_line_starts_line(first.span.start) || !before_operator && !ends_line;
            if run_leading {
                leading = true;
                leading_own_line |= ends_line;
            } else if leading {
                return None;
            } else {
                trailing_count = end;
            }
            start = end;
        }
        Some((trailing_count, leading_own_line))
    }

    fn intersection_member_is_object(&self, node: PreparedType) -> bool {
        let mut data = node.data;
        loop {
            let child = match data {
                TsTypeData::TsTypeLiteral(_) | TsTypeData::TsMappedType(_) => return true,
                TsTypeData::TsParenthesizedType(parentheses) => {
                    parentheses.type_annotation(self.ast)
                }
                TsTypeData::TsUnionType(union) if union.types(self.ast).len() == 1 => {
                    let Some(first) = union.types(self.ast).first() else {
                        return false;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                TsTypeData::TsIntersectionType(intersection)
                    if intersection.types(self.ast).len() == 1 =>
                {
                    let Some(first) = intersection.types(self.ast).first() else {
                        return false;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                _ => return false,
            };
            data = self.ast.ts_type_data(child);
        }
    }

    fn format_intersection_members(
        &mut self,
        intersection: PreparedIntersection<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        writer.group(false, |writer| {
            let mut prefix = intersection.leading;
            for link in intersection.links {
                self.format_intersection_member(link.left, prefix, writer)?;
                prefix =
                    self.format_intersection_separator(link.boundary, link.continuation, writer)?;
            }
            self.format_intersection_member(intersection.last, prefix, writer)?;
            Ok(None)
        })
    }

    fn format_intersection_member(
        &mut self,
        node: IntersectionMember<'ast>,
        prefix: IntersectionPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let indent = matches!(
            &prefix,
            IntersectionPrefix::Member {
                continuation: IntersectionContinuation::Line
                    | IntersectionContinuation::Inline { indent: true },
                ..
            } | IntersectionPrefix::BoundMember {
                continuation: IntersectionContinuation::Line
                    | IntersectionContinuation::Inline { indent: true },
                ..
            }
        );
        let content =
            |writer: &mut Writer<'ast>| {
                match prefix {
                    IntersectionPrefix::Empty => {}
                    IntersectionPrefix::Source(span) => self.format_verbatim(span, writer)?,
                    IntersectionPrefix::Leading {
                        boundary,
                        member_start,
                    } => {
                        self.format_comment_boundary_with_layout(
                            &boundary,
                            CommentRunLayout::GroupedLeading(member_start),
                            writer,
                        )?;
                    }
                    IntersectionPrefix::BoundMember {
                        boundary,
                        continuation,
                    } => {
                        self.format_comment_boundary(&boundary, writer)?;
                        boundary
                            .gap()
                            .write(writer, FlatSeparator::Space, |writer| match continuation {
                                IntersectionContinuation::Line => writer.line(),
                                IntersectionContinuation::Inline { .. } => writer.space(),
                            });
                    }
                    IntersectionPrefix::Member {
                        boundary,
                        member_start,
                        leading_own_line,
                        continuation,
                    } => {
                        if leading_own_line {
                            writer.line();
                        } else {
                            match continuation {
                                IntersectionContinuation::Line => writer.line(),
                                IntersectionContinuation::Inline { .. } => writer.space(),
                            }
                        }
                        self.format_comment_boundary_with_layout(
                            &boundary,
                            CommentRunLayout::GroupedLeading(member_start),
                            writer,
                        )?;
                    }
                }
                let tail = match node {
                    IntersectionMember::Type(node) => {
                        self.format_prepared_type(node, TypeContext::CompoundMember, writer)?
                    }
                    IntersectionMember::Prepared(content) => self.format_prepared_type_content(
                        *content,
                        TypeContext::CompoundMember,
                        TypeLayout::Root,
                        writer,
                    )?,
                };
                self.format_expression_tail(tail, writer)
            };
        if indent {
            writer.indent(content)
        } else {
            content(writer)
        }
    }

    fn format_intersection_separator(
        &mut self,
        boundary: IntersectionBoundary<'ast>,
        continuation: IntersectionContinuation,
        writer: &mut Writer<'ast>,
    ) -> Result<IntersectionPrefix<'ast>, FormatError> {
        let (before, after, member_start, leading_own_line) = match boundary {
            IntersectionBoundary::Source(span) => return Ok(IntersectionPrefix::Source(span)),
            IntersectionBoundary::Bound { before, after, .. } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("&");
                return Ok(IntersectionPrefix::BoundMember {
                    boundary: after,
                    continuation,
                });
            }
            IntersectionBoundary::Formatted {
                before,
                after,
                member_start,
                leading_own_line,
            } => (before, after, member_start, leading_own_line),
        };
        self.format_comment_boundary(&before, writer)?;
        // A trailing Line remains a suffix of the left member until its next break.
        writer.space();
        writer.text("&");
        Ok(IntersectionPrefix::Member {
            boundary: after,
            member_start,
            leading_own_line,
            continuation,
        })
    }

    pub(super) fn prepare_union(
        &self,
        node: PreparedType,
        types: SubRange<TsType>,
    ) -> Result<PreparedUnion<'ast>, FormatError> {
        let ast = self.ast;
        let mut members = types.iter().map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = members.next() else {
            return Err(self.unsupported("empty union type", node.node));
        };
        let first_start = first.span(ast).start;
        let mut first = self.prepare_type(first, ContentMode::Formatted);
        first.source_owner = node.source_owner;
        self.prepare_union_members(
            node,
            first,
            Span::new(node.node.span(ast).start, first_start),
            first_start,
            members,
        )
    }

    fn prepare_union_members(
        &self,
        node: PreparedType,
        mut current: PreparedType,
        leading: Span,
        first_start: u32,
        members: impl Iterator<Item = TsType>,
    ) -> Result<PreparedUnion<'ast>, FormatError> {
        let leading = if leading.start == leading.end {
            UnionLeading::Empty
        } else {
            let comments = self.comments.in_range(leading);
            if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
                self.select_type_head_comments(&mut current, comments, first_start);
                UnionLeading::Source(leading)
            } else {
                let Some(operator) = self.find_token(leading, comments, "|") else {
                    return Err(FormatError::unsupported(
                        "union leading operator",
                        None,
                        leading,
                    ));
                };
                let split = comments.partition_point(|comment| comment.span.end <= operator);
                let (before, after) = comments.split_at(split);
                let after = self.prepare_comment_boundary(after, operator + 1, first_start);
                let after = if !after.is_empty()
                    && matches!(
                        self.comment_boundary_binding(&after, current.node.span(self.ast)),
                        CommentBinding::Unbound
                    ) {
                    UnionMemberPrefix::Leading {
                        boundary: after,
                        member_start: first_start,
                    }
                } else {
                    UnionMemberPrefix::Boundary(after)
                };
                UnionLeading::Formatted {
                    before: self.prepare_comment_boundary(before, leading.start, operator),
                    after,
                    retain_operator: !comments.is_empty(),
                }
            }
        };
        let mut hug = match &leading {
            UnionLeading::Empty => UnionHug::NoObject,
            UnionLeading::Formatted { before, after, .. }
                if before.is_empty() && after.boundary().is_empty() =>
            {
                UnionHug::NoObject
            }
            UnionLeading::Source(_) | UnionLeading::Formatted { .. } => UnionHug::Unavailable,
        };
        let mut links = Vec::with_capacity(members.size_hint().0);
        for next in members {
            let mut next = self.prepare_type(next, ContentMode::Formatted);
            let gap = Span::new(
                current.node.span(self.ast).end,
                next.node.span(self.ast).start,
            );
            let boundary = self.prepare_compound_boundary(
                gap,
                CompoundTypeKind::Union,
                &mut current,
                &mut next,
            )?;
            hug.include(current);
            hug.include_boundary(&boundary);
            links.push(UnionLink {
                left: current,
                boundary,
            });
            current = next;
        }
        hug.include(current);
        let span = node.node.span(self.ast);
        let hug = matches!(hug, UnionHug::Object) && !self.diagnostic_range_is_sensitive(span);
        Ok(PreparedUnion {
            data: node.data,
            leading,
            links,
            last: current,
            opening: self.line_constraint(span.start, span.start),
            hug,
        })
    }

    pub(in crate::formatter) fn format_prepared_union(
        &mut self,
        union: PreparedUnion<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        head: Option<Fragment<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        self.format_type_in_context(
            union.data,
            context,
            layout,
            writer,
            |formatter, layout, writer| formatter.format_union_members(union, layout, head, writer),
        )
    }

    fn format_union_members(
        &mut self,
        union: PreparedUnion<'ast>,
        layout: TypeLayout,
        head: Option<Fragment<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        let PreparedUnion {
            leading,
            links,
            last,
            opening,
            hug,
            ..
        } = union;
        let content = |writer: &mut Writer<'ast>| {
            if let Some(head) = head {
                writer.write_fragment(head);
            }
            let content = |writer: &mut Writer<'ast>| {
                let leading = self.format_union_leading(leading, hug, writer)?;
                let mut prefix = leading;
                for link in links {
                    self.format_union_member(link.left, prefix, hug, writer)?;
                    prefix = self.format_union_separator(link.boundary, hug, writer)?;
                }
                self.format_union_member(last, prefix, hug, writer)?;
                Ok(None)
            };
            if hug {
                content(writer)
            } else {
                writer.group(false, content)
            }
        };
        if hug {
            return content(writer);
        }
        match layout {
            TypeLayout::AliasRhs | TypeLayout::CallerIndented => content(writer),
            TypeLayout::ConditionalConsequent | TypeLayout::ConditionalAlternate => {
                writer.align(2, content)
            }
            TypeLayout::Root | TypeLayout::SignatureParameter | TypeLayout::OpeningProvided => {
                writer.group(false, |writer| {
                    writer.indent(|writer| {
                        if layout != TypeLayout::OpeningProvided {
                            opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                        }
                        content(writer)
                    })
                })
            }
        }
    }

    fn format_union_leading(
        &mut self,
        leading: UnionLeading<'ast>,
        hug: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<UnionMemberPrefix<'ast>>, FormatError> {
        match leading {
            UnionLeading::Source(span) => {
                self.format_verbatim(span, writer)?;
                Ok(None)
            }
            UnionLeading::Empty => {
                if !hug {
                    writer.if_break("| ");
                }
                Ok(None)
            }
            UnionLeading::Formatted {
                before,
                after,
                retain_operator,
            } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write(writer, FlatSeparator::Empty, |_| {});
                if retain_operator {
                    writer.text("|");
                } else if !hug {
                    writer.if_break("| ");
                }
                if after.boundary().is_empty() {
                    after.boundary().gap().write(
                        writer,
                        if retain_operator {
                            FlatSeparator::Space
                        } else {
                            FlatSeparator::Empty
                        },
                        |writer| {
                            if retain_operator {
                                writer.space();
                            }
                        },
                    );
                    Ok(None)
                } else {
                    Ok(Some(after))
                }
            }
        }
    }

    fn format_union_member(
        &mut self,
        node: PreparedType,
        prefix: Option<UnionMemberPrefix<'ast>>,
        hug: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let content = |writer: &mut Writer<'ast>| {
            if let Some(prefix) = prefix {
                match prefix {
                    UnionMemberPrefix::Boundary(boundary) => {
                        self.format_comment_boundary(&boundary, writer)?;
                        boundary
                            .gap()
                            .write(writer, FlatSeparator::Space, Writer::space);
                    }
                    UnionMemberPrefix::Leading {
                        boundary,
                        member_start,
                    } => {
                        writer.space();
                        self.format_comment_boundary_with_layout(
                            &boundary,
                            CommentRunLayout::GroupedLeading(member_start),
                            writer,
                        )?;
                    }
                }
            }
            let tail = self.format_prepared_type(node, TypeContext::CompoundMember, writer)?;
            self.format_expression_tail(tail, writer)
        };
        if hug {
            content(writer)
        } else {
            writer.align(2, content)
        }
    }

    fn format_union_separator(
        &mut self,
        boundary: TypeBoundaryLayout<'ast>,
        hug: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<UnionMemberPrefix<'ast>>, FormatError> {
        let (before, after) = match boundary {
            TypeBoundaryLayout::Source(span) => {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(&before, writer)?;
        before.gap().write(writer, FlatSeparator::Space, |writer| {
            if hug {
                writer.space();
            } else {
                writer.line();
            }
        });
        writer.text("|");
        Ok(Some(UnionMemberPrefix::Boundary(after)))
    }

    pub(super) fn prepare_single_compound(
        &self,
        node: PreparedType,
    ) -> Result<SingleCompoundHead<'ast>, FormatError> {
        let (types, kind) = match node.data {
            TsTypeData::TsUnionType(union) => (union.types(self.ast), CompoundTypeKind::Union),
            TsTypeData::TsIntersectionType(intersection) => {
                (intersection.types(self.ast), CompoundTypeKind::Intersection)
            }
            _ => unreachable!("single compound dispatch retains its actual compound owner"),
        };
        let Some(first) = types.iter().next() else {
            return Err(self.unsupported("empty compound type", node.node));
        };
        let first = self.ast.get_node_in_sub_range(first);
        let span = first.span(self.ast);
        let mut current = self.prepare_type(first, ContentMode::Formatted);
        current.source_owner = node.source_owner;
        self.prepare_single_compound_head(
            node,
            current,
            Span::new(node.node.span(self.ast).start, span.start),
            span,
            kind,
        )
    }

    fn prepare_single_compound_head(
        &self,
        node: PreparedType,
        current: PreparedType,
        leading: Span,
        first_span: Span,
        kind: CompoundTypeKind,
    ) -> Result<SingleCompoundHead<'ast>, FormatError> {
        let comments = self.comments.in_range(leading);
        let movable = node.source_owner.is_none()
            && matches!(
                self.comment_binding(comments, leading, Some(first_span)),
                CommentBinding::Unbound
            );
        if movable
            && leading.start != leading.end
            && self.find_token(leading, comments, kind.token()).is_none()
        {
            return Err(FormatError::unsupported(
                "single compound leading operator",
                None,
                leading,
            ));
        }
        Ok(SingleCompoundHead {
            data: node.data,
            current,
            leading,
            first_start: first_span.start,
            comments,
            kind,
            movable,
        })
    }

    pub(super) fn format_single_compound_head(
        &mut self,
        single: SingleCompoundHead<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedTypeContent<'ast>, FormatError> {
        if !single.movable {
            return Ok(PreparedTypeContent::Compound(single));
        }
        self.format_movable_type_head(
            single.current,
            single.comments,
            single.first_start,
            self.type_movable_head(single.current, context),
            context,
            writer,
        )
    }

    pub(super) fn format_prepared_type_content_head(
        &mut self,
        content: PreparedTypeContent<'ast>,
        pending: &'ast [Comment],
        next_start: u32,
        head: Option<MovableTypeHead>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedTypeContent<'ast>, FormatError> {
        match content {
            PreparedTypeContent::Type(node) => {
                self.format_movable_type_head(node, pending, next_start, head, context, writer)
            }
            PreparedTypeContent::Intersection(intersection) => self
                .format_intersection_head_with_pending(
                    intersection,
                    pending,
                    next_start,
                    head,
                    writer,
                )
                .map(PreparedTypeContent::Intersection),
            PreparedTypeContent::Parentheses(parentheses)
                if parentheses.movable
                    && !context.preserves_union_head(parentheses.child.node, self.ast) =>
            {
                let comments = parentheses.leading_comments;
                if let Some(first) = comments.first() {
                    self.comments.claim(pending)?;
                    self.format_comment_run(
                        pending,
                        CommentRunLayout::GroupedLeading(first.span.start),
                        writer,
                    )?;
                    self.format_movable_type_head(
                        parentheses.child,
                        comments,
                        parentheses.child_span.start,
                        self.type_movable_head(parentheses.child, context),
                        context,
                        writer,
                    )
                } else {
                    self.format_movable_type_head(
                        parentheses.child,
                        pending,
                        next_start,
                        self.type_movable_head(parentheses.child, context),
                        context,
                        writer,
                    )
                }
            }
            content => {
                self.comments.claim(pending)?;
                self.format_comment_run(
                    pending,
                    CommentRunLayout::GroupedLeading(next_start),
                    writer,
                )?;
                Ok(content)
            }
        }
    }

    /// Delay each head packet until its actual next comment or content owner is known.
    pub(super) fn format_movable_type_head(
        &mut self,
        mut current: PreparedType,
        mut pending: &'ast [Comment],
        mut pending_next_start: u32,
        mut head: Option<MovableTypeHead>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<PreparedTypeContent<'ast>, FormatError> {
        let content = loop {
            if current.source_owner == Some(current.node) {
                break PreparedTypeContent::Type(current);
            }
            let (next, comments, next_start) = match current.data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let parentheses =
                        self.prepare_type_parentheses(parentheses, current.source_owner);
                    if parentheses.retained
                        && (!parentheses.movable
                            || (head.is_none_or(|head| head.owner != current.data)
                                && context.preserves_union_head(parentheses.child.node, self.ast)))
                    {
                        break PreparedTypeContent::Parentheses(parentheses);
                    }
                    (
                        parentheses.child,
                        parentheses.leading_comments,
                        parentheses.child_span.start,
                    )
                }
                TsTypeData::TsUnionType(union) if union.types(self.ast).len() == 1 => {
                    let single = self.prepare_single_compound(current)?;
                    if !single.movable {
                        break PreparedTypeContent::Compound(single);
                    }
                    (single.current, single.comments, single.first_start)
                }
                TsTypeData::TsIntersectionType(intersection)
                    if intersection.types(self.ast).len() == 1 =>
                {
                    let single = self.prepare_single_compound(current)?;
                    if !single.movable {
                        break PreparedTypeContent::Compound(single);
                    }
                    (single.current, single.comments, single.first_start)
                }
                TsTypeData::TsArrayType(node) => {
                    if head.is_none() {
                        break PreparedTypeContent::Type(current);
                    }
                    let postfix = self.format_postfix_head(
                        super::postfix::PostfixTypeOwner::Array(node),
                        pending,
                        pending_next_start,
                        head,
                        writer,
                    )?;
                    pending = &[];
                    break PreparedTypeContent::Postfix(postfix);
                }
                TsTypeData::TsIndexedAccessType(node) => {
                    if head.is_none() {
                        break PreparedTypeContent::Type(current);
                    }
                    let postfix = self.format_postfix_head(
                        super::postfix::PostfixTypeOwner::Indexed(node),
                        pending,
                        pending_next_start,
                        head,
                        writer,
                    )?;
                    pending = &[];
                    break PreparedTypeContent::Postfix(postfix);
                }
                TsTypeData::TsConditionalType(node) => {
                    if head.is_none() && pending.is_empty() {
                        break PreparedTypeContent::Type(current);
                    }
                    let conditional = self.format_conditional_head(
                        node,
                        pending,
                        pending_next_start,
                        head,
                        writer,
                    )?;
                    pending = &[];
                    break PreparedTypeContent::Conditional(conditional);
                }
                TsTypeData::TsUnionType(union) => {
                    break PreparedTypeContent::Union(
                        self.prepare_union(current, union.types(self.ast))?,
                    );
                }
                TsTypeData::TsIntersectionType(intersection) => {
                    let intersection =
                        self.prepare_intersection(current, intersection.types(self.ast))?;
                    let intersection = self.format_intersection_head_with_pending(
                        intersection,
                        pending,
                        pending_next_start,
                        head,
                        writer,
                    )?;
                    pending = &[];
                    break PreparedTypeContent::Intersection(intersection);
                }
                _ => break PreparedTypeContent::Type(current),
            };
            if let Some(first) = comments.first() {
                self.comments.claim(pending)?;
                self.format_comment_run(
                    pending,
                    CommentRunLayout::GroupedLeading(first.span.start),
                    writer,
                )?;
                pending = comments;
                pending_next_start = next_start;
            }
            if head.is_some_and(|head| head.owner == current.data) {
                head = self.type_movable_head(next, context);
            }
            current = next;
        };
        self.comments.claim(pending)?;
        self.format_comment_run(
            pending,
            CommentRunLayout::GroupedLeading(pending_next_start),
            writer,
        )?;
        Ok(content)
    }

    pub(super) fn format_prepared_type_content(
        &mut self,
        content: PreparedTypeContent<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        match content {
            PreparedTypeContent::Union(union) => {
                self.format_prepared_union(union, context, layout, None, writer)
            }
            PreparedTypeContent::Intersection(intersection) => {
                self.format_prepared_intersection(intersection, context, layout, writer)
            }
            PreparedTypeContent::Postfix(postfix) => {
                self.format_prepared_postfix_type(postfix, context, layout, writer)
            }
            PreparedTypeContent::Conditional(conditional) => {
                self.format_prepared_conditional_type(conditional, context, layout, writer)
            }
            PreparedTypeContent::Type(node) => {
                self.format_prepared_type_with_layout(node, context, layout, writer)
            }
            PreparedTypeContent::Parentheses(parentheses) => {
                self.format_prepared_type_parentheses(parentheses, context, layout, writer)
            }
            PreparedTypeContent::Compound(single) => self.format_type_in_context(
                single.data,
                context,
                layout,
                writer,
                |formatter, layout, writer| {
                    formatter.group_compound_members(
                        CompoundHead {
                            current: single.current,
                            leading: single.leading,
                            first_start: single.first_start,
                            leading_comments: Some(single.comments),
                            single: true,
                        },
                        std::iter::empty(),
                        single.kind,
                        layout,
                        writer,
                    )
                },
            ),
        }
    }

    fn group_compound_members(
        &mut self,
        head: CompoundHead<'ast>,
        members: impl Iterator<Item = TsType>,
        kind: CompoundTypeKind,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let CompoundHead {
            mut current,
            leading,
            first_start,
            leading_comments,
            single,
        } = head;
        writer.group(false, |writer| {
            let content = |writer: &mut Writer<'ast>| {
                let mut leading_operator = false;
                if leading.start != leading.end {
                    let comments =
                        leading_comments.unwrap_or_else(|| self.comments.in_range(leading));
                    if self.content_mode(comments) == ContentMode::Verbatim {
                        self.select_type_head_comments(&mut current, comments, first_start);
                        self.format_verbatim(leading, writer)?;
                        leading_operator = true;
                    } else if let Some(operator) = self.find_token(leading, comments, kind.token())
                    {
                        self.format_source_token_gap(leading.start, operator, false, writer)?;
                        // A retained leading operator keeps its comment slot inside
                        // the compound on both formatting passes.
                        leading_operator = single || !comments.is_empty();
                        if leading_operator {
                            writer.text(kind.token());
                        }
                        self.format_source_token_gap(
                            operator + 1,
                            first_start,
                            leading_operator,
                            writer,
                        )?;
                    }
                }
                if layout == TypeLayout::AliasRhs
                    && matches!(kind, CompoundTypeKind::Union)
                    && !leading_operator
                {
                    writer.if_break("| ");
                }
                for next in members {
                    let mut next = self.prepare_type(next, ContentMode::Formatted);
                    let gap = Span::new(current.node.span(ast).end, next.node.span(ast).start);
                    let boundary =
                        self.prepare_compound_boundary(gap, kind, &mut current, &mut next)?;
                    let tail = self.format_compound_member(current, kind, layout, writer)?;
                    self.format_expression_tail(tail, writer)?;
                    self.format_compound_boundary(boundary, kind, writer)?;
                    current = next;
                }
                let tail = self.format_compound_member(current, kind, layout, writer)?;
                self.format_expression_tail(tail, writer)?;
                Ok(None)
            };
            if layout == TypeLayout::AliasRhs {
                content(writer)
            } else {
                writer.indent(content)
            }
        })
    }

    fn format_compound_member(
        &mut self,
        node: PreparedType,
        kind: CompoundTypeKind,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        if layout == TypeLayout::AliasRhs && matches!(kind, CompoundTypeKind::Union) {
            writer.align(2, |writer| {
                self.format_prepared_type(node, TypeContext::CompoundMember, writer)
            })
        } else {
            self.format_prepared_type(node, TypeContext::CompoundMember, writer)
        }
    }

    fn prepare_compound_boundary(
        &self,
        gap: Span,
        kind: CompoundTypeKind,
        left: &mut PreparedType,
        right: &mut PreparedType,
    ) -> Result<TypeBoundaryLayout<'ast>, FormatError> {
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, kind.token()) else {
            return Err(FormatError::unsupported(
                "compound type operator",
                None,
                gap,
            ));
        };
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let (before, after) = comments.split_at(split);
        if self.select_compound_boundary_suppression(operator, before, after, left, right) {
            Ok(TypeBoundaryLayout::Source(gap))
        } else {
            Ok(TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator),
                after: self.prepare_comment_boundary(after, operator + 1, gap.end),
            })
        }
    }

    fn select_compound_boundary_suppression(
        &self,
        operator: u32,
        before: &[Comment],
        after: &[Comment],
        left: &mut PreparedType,
        right: &mut PreparedType,
    ) -> bool {
        if !self.has_suppression {
            return false;
        }
        let mut source = false;
        for (_, position) in self.suppressed_comment_runs(before, operator) {
            source = true;
            if matches!(position, SuppressionPosition::OwnLine) {
                self.select_type_head(right, position);
            } else {
                left.source_owner = Some(left.node);
            }
        }
        for (_, position) in self.suppressed_comment_runs(after, right.node.span(self.ast).start) {
            source = true;
            if matches!(position, SuppressionPosition::EndOfLine) {
                left.source_owner = Some(left.node);
            } else {
                self.select_type_head(right, position);
            }
        }
        source
    }

    fn format_compound_boundary(
        &mut self,
        boundary: TypeBoundaryLayout<'ast>,
        kind: CompoundTypeKind,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (before, after) = match boundary {
            TypeBoundaryLayout::Source(gap) => return self.format_verbatim(gap, writer),
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(&before, writer)?;
        before
            .gap()
            .write(writer, FlatSeparator::Space, |writer| match kind {
                CompoundTypeKind::Union => writer.line(),
                CompoundTypeKind::Intersection => writer.space(),
            });
        writer.text(kind.token());
        self.format_comment_boundary(&after, writer)?;
        after
            .gap()
            .write(writer, FlatSeparator::Space, |writer| match kind {
                CompoundTypeKind::Union => writer.space(),
                CompoundTypeKind::Intersection => writer.line(),
            });
        Ok(())
    }
}
