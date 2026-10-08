//! Runtime expression slots and the distinct tagged-template operand edge.

use swc_next_ecma_ast::{Comment, Expr, GetSpan, Span, TaggedTemplateExpression, TemplateLiteral};

use super::{
    AccessKind, ExpressionContext, ExpressionPosition, ExpressionTypeArguments, TailOwner,
};
use crate::formatter::Formatter;
use crate::formatter::comments::{CommentBoundary, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::templates::TemplateSlot;
use crate::{FormatError, document::Writer};

struct TemplateTagHead<'ast> {
    tag: Expr,
    quasi: TemplateLiteral,
    types: Option<ExpressionTypeArguments<'ast>>,
    tag_mode: ContentMode,
    quasi_mode: ContentMode,
    gap: TemplateTagGap<'ast>,
}

enum TemplateTagGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

impl TemplateTagHead<'_> {
    fn context(&self, outer: ExpressionContext) -> ExpressionContext {
        outer.type_operand(
            ExpressionPosition::TemplateTag,
            self.tag_mode,
            TailOwner::ParentOperation,
        )
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_template_literal(
        &mut self,
        template: TemplateLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        self.format_template_shell(
            template.span(ast),
            template.quasis(ast),
            template.expressions(ast),
            Self::format_template_expression_slot,
            writer,
        )
    }

    fn format_template_expression_slot(
        &mut self,
        expression: Expr,
        slot: TemplateSlot,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let span = slot.content;
        let leading = self
            .comments
            .in_range(Span::new(slot.interior.start, span.start));
        let trailing = self
            .comments
            .in_range(Span::new(span.end, slot.interior.end));
        let leading_mode = self.content_mode(leading);
        let trailing_mode = self.content_mode(trailing);
        let mode =
            if leading_mode == ContentMode::Verbatim || trailing_mode == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
        let opening = if leading_mode == ContentMode::Verbatim {
            let start = leading
                .first()
                .map_or(span.start, |comment| comment.span.start);
            self.source_line_constraint(slot.interior.start, start)
        } else {
            self.line_constraint(slot.interior.start, span.start)
        };
        let after = self.prepare_comment_boundary(trailing, span.end, slot.interior.end);
        writer.group(false, |writer| {
            writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                self.comments.claim(leading)?;
                let tail = self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments: leading,
                        content_start: span.start,
                        layout: CommentRunLayout::Leading(span.start),
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::TemplateInterpolation)
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.format_comment_boundary(&after, writer)
            })?;
            after
                .gap()
                .write(writer, FlatSeparator::Empty, Writer::soft_line);
            Ok(())
        })
    }

    fn prepare_template_tag(
        &self,
        node: TaggedTemplateExpression,
    ) -> Result<TemplateTagHead<'ast>, FormatError> {
        let tag = node.tag(self.ast);
        let mut types = node
            .type_arguments(self.ast)
            .map(|types| self.prepare_expression_type_arguments(tag, types, AccessKind::Ordinary))
            .transpose()?;
        let end = types.as_ref().map_or_else(
            || tag.span(self.ast).end,
            |types| types.node.span(self.ast).end,
        );
        let quasi = node.quasi(self.ast);
        let start = quasi.span(self.ast).start;
        if end > start {
            return Err(self.unsupported("template tag boundary", node));
        }
        let gap = Span::new(end, start);
        let comments = self.comments.in_range(gap);
        let mut tag_mode = types
            .as_ref()
            .map_or(ContentMode::Formatted, |types| types.callee_mode);
        let mut quasi_mode = ContentMode::Formatted;
        if self.content_mode(comments) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(comments, start) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    tag_mode = ContentMode::Verbatim;
                    if let Some(types) = &mut types {
                        types.mode = ContentMode::Verbatim;
                    }
                } else {
                    quasi_mode = ContentMode::Verbatim;
                }
            }
        }
        let gap = if tag_mode == ContentMode::Verbatim || quasi_mode == ContentMode::Verbatim {
            if !self.template_tag_gap_is_trivia(gap, comments) {
                return Err(self.unsupported("template tag source boundary", node));
            }
            TemplateTagGap::Source(gap)
        } else {
            TemplateTagGap::Formatted(self.prepare_comment_boundary(comments, end, start))
        };
        Ok(TemplateTagHead {
            tag,
            quasi,
            types,
            tag_mode,
            quasi_mode,
            gap,
        })
    }

    fn template_tag_gap_is_trivia(&self, gap: Span, comments: &[Comment]) -> bool {
        let mut start = gap.start;
        for comment in comments {
            if !self
                .source(Span::new(start, comment.span.start))
                .chars()
                .all(is_source_whitespace)
            {
                return false;
            }
            start = comment.span.end;
        }
        self.source(Span::new(start, gap.end))
            .chars()
            .all(is_source_whitespace)
    }

    pub(super) fn template_tag_head_operand(
        &self,
        node: TaggedTemplateExpression,
        context: ExpressionContext,
    ) -> Result<(Expr, ExpressionContext), FormatError> {
        // Only suppression can change the real tag edge's content mode.
        if !self.has_suppression || context.content_mode == ContentMode::Verbatim {
            return Ok((
                node.tag(self.ast),
                context.type_operand(
                    ExpressionPosition::TemplateTag,
                    context.content_mode,
                    TailOwner::ParentOperation,
                ),
            ));
        }
        let head = self.prepare_template_tag(node)?;
        Ok((head.tag, head.context(context)))
    }

    pub(super) fn format_tagged_template(
        &mut self,
        node: TaggedTemplateExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let head = self.prepare_template_tag(node)?;
        let tail = self.format_expression(head.tag, head.context(context), writer)?;
        if let Some(types) = head.types {
            self.format_expression_type_arguments(types, tail, writer)?;
        } else {
            self.format_expression_tail(tail, writer)?;
        }
        match head.gap {
            TemplateTagGap::Source(gap) => self.format_verbatim(gap, writer)?,
            TemplateTagGap::Formatted(gap) => {
                self.format_comment_boundary(&gap, writer)?;
                gap.gap().write(writer, FlatSeparator::Empty, |_| {});
            }
        }
        let quasi = head.quasi;
        if head.quasi_mode == ContentMode::Verbatim {
            self.format_verbatim(quasi.span(self.ast), writer)?;
        } else {
            self.format_template_literal(quasi, writer)?;
        }
        Ok(None)
    }
}
