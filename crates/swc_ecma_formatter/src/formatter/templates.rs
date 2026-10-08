//! Borrowed template text and physical quasi/slot delimiter geometry.

use swc_next_ecma_ast::{
    ExtraDataValue, GetSpan, Span, SubRange, TemplateElement, TemplateLiteral,
};

use super::Formatter;
use crate::{FormatError, document::Writer};

struct TemplateQuasi<'ast> {
    span: Span,
    raw: &'ast str,
    tail: bool,
}

/// The actual interpolation interior, independent of normalized raw byte length.
#[derive(Clone, Copy)]
pub(super) struct TemplateSlot {
    pub(super) content: Span,
    pub(super) interior: Span,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn template_quasi(&self, node: TemplateElement) -> TemplateQuasi<'ast> {
        TemplateQuasi {
            span: node.span(self.ast),
            raw: self.ast.get_utf8(node.raw(self.ast)),
            tail: node.tail(self.ast),
        }
    }

    fn template_token(&self, span: Span, token: &str) -> bool {
        self.ast
            .source()
            .get(span.start as usize..span.end as usize)
            == Some(token)
    }

    pub(super) fn format_zero_slot_template(
        &mut self,
        template: TemplateLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if !template.expressions(self.ast).is_empty() {
            return Err(self.unsupported("template literal expression slots", template));
        }
        self.format_template_without_slots(
            template.span(self.ast),
            template.quasis(self.ast),
            writer,
        )
    }

    fn format_template_without_slots(
        &self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Some(first) = quasis.first().filter(|_| quasis.len() == 1) else {
            return Err(FormatError::unsupported("template quasi count", None, span));
        };
        let quasi = self.template_quasi(self.ast.get_node_in_sub_range(first));
        if quasi.span.start > quasi.span.end
            || !quasi.tail
            || span.start >= span.end
            || quasi.span.start != span.start + 1
            || quasi.span.end != span.end - 1
            || !self.template_token(Span::new(span.start, quasi.span.start), "`")
            || !self.template_token(Span::new(quasi.span.end, span.end), "`")
        {
            return Err(FormatError::unsupported(
                "template quasi geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        writer.literal_text(quasi.raw);
        writer.text("`");
        Ok(())
    }

    pub(super) fn format_template_shell<S>(
        &mut self,
        span: Span,
        quasis: SubRange<TemplateElement>,
        slots: SubRange<S>,
        mut format_slot: impl FnMut(
            &mut Self,
            S,
            TemplateSlot,
            &mut Writer<'ast>,
        ) -> Result<(), FormatError>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError>
    where
        S: GetSpan + ExtraDataValue,
    {
        if slots.is_empty() {
            return self.format_template_without_slots(span, quasis, writer);
        }
        if quasis.len() != slots.len() + 1 {
            return Err(FormatError::unsupported("template quasi count", None, span));
        }
        let ast = self.ast;
        let mut quasis = quasis.iter().map(|id| ast.get_node_in_sub_range(id));
        let Some(first) = quasis.next() else {
            return Err(FormatError::unsupported("template quasi count", None, span));
        };
        let mut current = self.template_quasi(first);
        if span.end as usize > ast.source().len()
            || span.start >= span.end
            || current.span.start != span.start + 1
            || !self.template_token(Span::new(span.start, current.span.start), "`")
        {
            return Err(FormatError::unsupported(
                "template opening geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        writer.literal_text(current.raw);
        for slot in slots.iter().map(|id| ast.get_node_in_sub_range(id)) {
            let Some(next) = quasis.next() else {
                return Err(FormatError::unsupported("template quasi count", None, span));
            };
            let next = self.template_quasi(next);
            let slot_span = slot.span(ast);
            if next.span.end >= span.end
                || next.span.start > next.span.end
                || current.span.start > current.span.end
                || current.tail
                || current.span.end > slot_span.start
                || slot_span.start > slot_span.end
                || slot_span.end >= next.span.start
                || !self.template_token(Span::new(current.span.end, current.span.end + 2), "${")
                || !self.template_token(Span::new(next.span.start - 1, next.span.start), "}")
                || current.span.end + 2 > slot_span.start
            {
                return Err(FormatError::unsupported(
                    "template slot geometry",
                    None,
                    span,
                ));
            }
            writer.text("${");
            format_slot(
                self,
                slot,
                TemplateSlot {
                    content: slot_span,
                    interior: Span::new(current.span.end + 2, next.span.start - 1),
                },
                writer,
            )?;
            writer.text("}");
            writer.literal_text(next.raw);
            current = next;
        }
        if !current.tail
            || current.span.end != span.end - 1
            || !self.template_token(Span::new(current.span.end, span.end), "`")
        {
            return Err(FormatError::unsupported(
                "template closing geometry",
                None,
                span,
            ));
        }
        writer.text("`");
        Ok(())
    }
}
