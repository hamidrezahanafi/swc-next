//! Source-ordered type members carry a prepared operator gap and actual ignore owner.

use swc_next_ecma_ast::{GetSpan, Span, SubRange, TsType};

use super::context::TypeContext;
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeLayout};
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

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_compound_type(
        &mut self,
        node: PreparedType,
        types: SubRange<TsType>,
        kind: CompoundTypeKind,
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
        writer.group(false, |writer| {
            let content = |writer: &mut Writer<'ast>| {
                let mut leading_operator = false;
                if leading.start != leading.end {
                    let comments = self.comments.in_range(leading);
                    if self.content_mode(comments) == ContentMode::Verbatim {
                        self.select_type_head_comments(&mut current, comments, first_span.start);
                        self.format_verbatim(leading, writer)?;
                        leading_operator = true;
                    } else if let Some(operator) = self.find_token(leading, comments, kind.token())
                    {
                        self.format_source_token_gap(leading.start, operator, false, writer)?;
                        // A retained leading operator keeps its comment slot inside
                        // the compound on both formatting passes.
                        leading_operator = types.len() == 1 || !comments.is_empty();
                        if leading_operator {
                            writer.text(kind.token());
                        }
                        self.format_source_token_gap(
                            operator + 1,
                            first_span.start,
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
