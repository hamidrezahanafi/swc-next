//! Conditional type children retain their grammar roles and prepared token gaps.

use swc_next_ecma_ast::{GetSpan, Span, TsConditionalType};

use super::context::TypeContext;
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeLayout};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum ConditionalTypeSeparator {
    Extends,
    Question,
    Colon,
}

impl ConditionalTypeSeparator {
    fn token(self) -> &'static str {
        match self {
            Self::Extends => "extends",
            Self::Question => "?",
            Self::Colon => ":",
        }
    }
}

struct ConditionalTypeBoundary<'ast> {
    separator: ConditionalTypeSeparator,
    layout: TypeBoundaryLayout<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_conditional_type(
        &mut self,
        node: TsConditionalType,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let mut check = self.prepare_type(node.check_type(ast), ContentMode::Formatted);
        let mut extends = self.prepare_type(node.extends_type(ast), ContentMode::Formatted);
        let mut consequent = self.prepare_type(node.true_type(ast), ContentMode::Formatted);
        let mut alternate = self.prepare_type(node.false_type(ast), ContentMode::Formatted);
        let head = self.prepare_conditional_type_boundary(
            &mut check,
            &mut extends,
            ConditionalTypeSeparator::Extends,
        )?;
        let question = self.prepare_conditional_type_boundary(
            &mut extends,
            &mut consequent,
            ConditionalTypeSeparator::Question,
        )?;
        let colon = self.prepare_conditional_type_boundary(
            &mut consequent,
            &mut alternate,
            ConditionalTypeSeparator::Colon,
        )?;
        let content = |writer: &mut Writer<'ast>| {
            if layout == TypeLayout::ConditionalConsequent {
                writer.if_flat("(");
            }
            let tail = self.format_prepared_type(check, TypeContext::ConditionalCheck, writer)?;
            self.format_expression_tail(tail, writer)?;
            self.format_conditional_type_boundary(head, writer)?;
            let tail =
                self.format_prepared_type(extends, TypeContext::ConditionalExtends, writer)?;
            self.format_expression_tail(tail, writer)?;
            let tail = writer.indent(|writer| {
                self.format_conditional_type_boundary(question, writer)?;
                let tail = self.format_prepared_type_with_layout(
                    consequent,
                    TypeContext::Root,
                    TypeLayout::ConditionalConsequent,
                    writer,
                )?;
                self.format_expression_tail(tail, writer)?;
                self.format_conditional_type_boundary(colon, writer)?;
                self.format_prepared_type_with_layout(
                    alternate,
                    TypeContext::Root,
                    TypeLayout::ConditionalAlternate,
                    writer,
                )
            })?;
            if layout == TypeLayout::ConditionalConsequent {
                self.format_expression_tail(tail, writer)?;
                writer.if_flat(")");
                Ok(None)
            } else {
                Ok(tail)
            }
        };
        if matches!(
            layout,
            TypeLayout::ConditionalConsequent | TypeLayout::ConditionalAlternate
        ) {
            content(writer)
        } else {
            writer.group(false, content)
        }
    }

    fn prepare_conditional_type_boundary(
        &self,
        left: &mut PreparedType,
        right: &mut PreparedType,
        separator: ConditionalTypeSeparator,
    ) -> Result<ConditionalTypeBoundary<'ast>, FormatError> {
        let gap = Span::new(
            left.node.span(self.ast).end,
            right.node.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let token = separator.token();
        let Some(operator) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "conditional type separator",
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
                    right.source_owner = Some(right.node);
                } else {
                    left.source_owner = Some(left.node);
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                source = true;
                if matches!(position, SuppressionPosition::EndOfLine) {
                    left.source_owner = Some(left.node);
                } else {
                    right.source_owner = Some(right.node);
                }
            }
        }
        let layout = if source {
            TypeBoundaryLayout::Source(gap)
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator),
                after: self.prepare_comment_boundary(after, operator + token.len() as u32, gap.end),
            }
        };
        Ok(ConditionalTypeBoundary { separator, layout })
    }

    fn format_conditional_type_boundary(
        &mut self,
        boundary: ConditionalTypeBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (before, after) = match boundary.layout {
            TypeBoundaryLayout::Source(gap) => return self.format_verbatim(gap, writer),
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(&before, writer)?;
        before.gap().write(writer, FlatSeparator::Space, |writer| {
            match boundary.separator {
                // A conditional check and `extends` must remain on the same line.
                ConditionalTypeSeparator::Extends => writer.space(),
                ConditionalTypeSeparator::Question | ConditionalTypeSeparator::Colon => {
                    writer.line();
                }
            }
        });
        writer.text(boundary.separator.token());
        self.format_comment_boundary(&after, writer)?;
        after
            .gap()
            .write(writer, FlatSeparator::Space, Writer::space);
        Ok(())
    }
}
