//! Source-ordered type members carry a prepared operator gap and actual ignore owner.

use swc_next_ecma_ast::{Comment, GetSpan, Span, SubRange, TsType, TsTypeData};

use super::context::TypeContext;
use super::{Formatter, PreparedType, PreparedTypeParentheses, TypeBoundaryLayout, TypeLayout};
use crate::formatter::comments::{CommentBinding, CommentRunLayout};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

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
    Parentheses(PreparedTypeParentheses),
    Compound(SingleCompoundHead<'ast>),
}

impl SingleTypeContent<'_> {
    pub(super) fn alias_layout(&self) -> TypeLayout {
        match self {
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
