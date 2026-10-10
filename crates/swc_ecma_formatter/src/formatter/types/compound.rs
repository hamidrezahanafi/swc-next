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

pub(super) enum SingleTypeContent<'ast> {
    Type(PreparedType),
    Union(PreparedUnion<'ast>),
    Parentheses(PreparedTypeParentheses),
    Compound(SingleCompoundHead<'ast>),
}

impl SingleTypeContent<'_> {
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
            Self::Type(_) | Self::Parentheses(_) | Self::Compound(_) => TypeLayout::Root,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
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
            let content = self.format_single_compound_head(single, writer)?;
            return self.format_single_type_content(content, context, layout, writer);
        }
        if matches!(kind, CompoundTypeKind::Union) {
            let union =
                self.prepare_union_members(node, current, leading, first_span.start, members)?;
            return self.format_prepared_union(union, context, layout, None, writer);
        }
        let leading_comments = None;
        self.format_type_in_context(
            node.data,
            context,
            layout,
            writer,
            |formatter, layout, writer| {
                let head = CompoundHead {
                    current,
                    leading,
                    first_start: first_span.start,
                    leading_comments,
                    single: types.len() == 1,
                };
                formatter.group_compound_members(head, members, kind, layout, writer)
            },
        )
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
        writer: &mut Writer<'ast>,
    ) -> Result<SingleTypeContent<'ast>, FormatError> {
        if !single.movable {
            return Ok(SingleTypeContent::Compound(single));
        }
        self.comments.claim(single.comments)?;
        self.format_comment_run(
            single.comments,
            CommentRunLayout::GroupedLeading(single.first_start),
            writer,
        )?;
        let mut current = single.current;
        loop {
            if current.source_owner == Some(current.node) {
                return Ok(SingleTypeContent::Type(current));
            }
            match current.data {
                TsTypeData::TsParenthesizedType(parentheses) => {
                    let (parentheses, before, after) = self
                        .prepare_type_parentheses_with_comments(parentheses, current.source_owner);
                    if parentheses.retained {
                        let movable = current.source_owner.is_none()
                            && after.is_empty()
                            && matches!(
                                self.comment_binding(
                                    before,
                                    parentheses.span,
                                    Some(parentheses.child_span)
                                ),
                                CommentBinding::Unbound
                            );
                        if !movable {
                            return Ok(SingleTypeContent::Parentheses(parentheses));
                        }
                        self.comments.claim(before)?;
                        self.format_comment_run(
                            before,
                            CommentRunLayout::GroupedLeading(parentheses.child_span.start),
                            writer,
                        )?;
                    }
                    current = parentheses.child;
                }
                TsTypeData::TsUnionType(union) if union.types(self.ast).len() == 1 => {
                    let single = self.prepare_single_compound(current)?;
                    return self.format_single_compound_head(single, writer);
                }
                TsTypeData::TsIntersectionType(intersection)
                    if intersection.types(self.ast).len() == 1 =>
                {
                    let single = self.prepare_single_compound(current)?;
                    return self.format_single_compound_head(single, writer);
                }
                TsTypeData::TsUnionType(union) => {
                    return self
                        .prepare_union(current, union.types(self.ast))
                        .map(SingleTypeContent::Union);
                }
                _ => return Ok(SingleTypeContent::Type(current)),
            }
        }
    }

    pub(super) fn format_single_type_content(
        &mut self,
        content: SingleTypeContent<'ast>,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<crate::formatter::comments::ExpressionTail<'ast>>, FormatError> {
        match content {
            SingleTypeContent::Union(union) => {
                self.format_prepared_union(union, context, layout, None, writer)
            }
            SingleTypeContent::Type(node) => {
                self.format_prepared_type_with_layout(node, context, layout, writer)
            }
            SingleTypeContent::Parentheses(parentheses) => {
                self.format_prepared_type_parentheses(parentheses, context, layout, writer)
            }
            SingleTypeContent::Compound(single) => self.format_type_in_context(
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
        let mut source = false;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, operator) {
                source = true;
                if matches!(position, SuppressionPosition::OwnLine) {
                    self.select_type_head(right, position);
                } else {
                    left.source_owner = Some(left.node);
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                source = true;
                if matches!(position, SuppressionPosition::EndOfLine) {
                    left.source_owner = Some(left.node);
                } else {
                    self.select_type_head(right, position);
                }
            }
        }
        if source {
            Ok(TypeBoundaryLayout::Source(gap))
        } else {
            Ok(TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator),
                after: self.prepare_comment_boundary(after, operator + 1, gap.end),
            })
        }
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
