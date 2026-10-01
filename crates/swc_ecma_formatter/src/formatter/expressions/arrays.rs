//! Array slots, comment boundaries, and grouped delimiter layout.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, CommentKind, ExprData, GetSpan, Span,
};

use super::Formatter;
use crate::formatter::comments::CommentRunLayout;
use crate::{FormatError, TrailingCommas, document::Writer};

struct ArrayItem {
    holes: usize,
    argument: Option<Argument>,
}

fn next_item(elements: &mut impl Iterator<Item = Option<Argument>>) -> Option<ArrayItem> {
    let mut holes = 0;
    for element in elements.by_ref() {
        if let Some(argument) = element {
            return Some(ArrayItem {
                holes,
                argument: Some(argument),
            });
        }
        holes += 1;
    }
    (holes != 0).then_some(ArrayItem {
        holes,
        argument: None,
    })
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_array(
        &mut self,
        array: ArrayExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let elements = array.elements(self.ast);
        let span = array.span(self.ast);
        if elements.is_empty()
            && self
                .comments
                .between(Span::new(span.start + 1, span.end - 1))?
                .is_empty()
        {
            writer.text("[]");
            return Ok(());
        }
        let expand = elements.len() > 1 && elements.iter().all(|id| {
            let Some(argument) = self.ast.get_node_in_sub_range(id) else { return false };
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else { return false };
            matches!(self.ast.expr_data(expression), ExprData::ArrayExpression(inner) if inner.elements(self.ast).len() > 1)
        });
        writer.group(expand, |writer| {
            writer.text("[");
            writer.indent(|writer| {
                writer.soft_line();
                self.format_array_items(array, Span::new(span.start + 1, span.end - 1), writer)
            })?;
            writer.soft_line();
            writer.text("]");
            Ok(())
        })
    }

    fn format_array_items(
        &mut self,
        array: ArrayExpression,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let mut elements = array
            .elements(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let mut items = std::iter::from_fn(|| next_item(&mut elements)).peekable();
        let mut previous_end = span.start;
        let mut has_slot = false;
        while let Some(item) = items.next() {
            for _ in 0..item.holes {
                if has_slot {
                    writer.line();
                }
                writer.text(",");
                has_slot = true;
            }
            let Some(argument) = item.argument else {
                continue;
            };
            if has_slot {
                writer.line();
            }
            let argument_span = argument.span(ast);
            let leading = self
                .comments
                .between(Span::new(previous_end, argument_span.start))?;
            let leading = self.comments.take(leading.len());
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                self.comment_gap(last.span.end, argument_span.start, writer);
            }
            self.format_argument(argument, writer)?;
            let following = items.peek().and_then(|item| item.argument);
            let gap = Span::new(
                argument_span.end,
                following.map_or(span.end, |node| node.span(ast).start),
            );
            let comments = self.comments.between(gap)?;
            let comma = self.find_comma(gap, comments);
            let mut inline_end = argument_span.end;
            let mut trailing_count = 0;
            for (index, comment) in comments.iter().enumerate() {
                if self.line_breaks(inline_end, comment.span.start) != 0 {
                    break;
                }
                inline_end = comment.span.end;
                if comment.kind == CommentKind::Line {
                    trailing_count = index + 1;
                    break;
                }
                if following.is_none() || comma.is_none_or(|position| comment.span.start < position)
                {
                    trailing_count = index + 1;
                }
            }
            let trailing = self.comments.take(trailing_count);
            let trailing_end = trailing
                .last()
                .map_or(argument_span.end, |comment| comment.span.end);
            self.format_inline_trailing_comments(trailing, writer)?;
            if items.peek().is_some() {
                writer.text(",");
                if let Some(comma) = comma {
                    let after_separator = Span::new(comma + 1, gap.end);
                    let next_start = comments
                        .get(trailing_count)
                        .map_or(gap.end, |comment| comment.span.start);
                    let start = after_separator.start.max(trailing_end);
                    if start < next_start && self.next_line_is_empty(start, next_start) {
                        writer.soft_line();
                    }
                }
            } else if self.options.trailing_commas != TrailingCommas::None {
                writer.if_break(",");
            }
            previous_end = trailing_end;
            if following.is_none() {
                let own_line = self.comments.between(Span::new(trailing_end, span.end))?;
                let own_line = self.comments.take(own_line.len());
                if let Some(first) = own_line.first() {
                    writer.hard_line();
                    if self.line_breaks(trailing_end, first.span.start) == 2 {
                        writer.hard_line();
                    }
                    self.format_comment_run(own_line, CommentRunLayout::Lines, writer)?;
                }
                if let Some(last) = own_line.last() {
                    previous_end = last.span.end;
                }
            }
            has_slot = true;
        }
        let dangling = self.comments.between(Span::new(previous_end, span.end))?;
        let dangling = self.comments.take(dangling.len());
        if dangling
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            writer.break_parent();
        }
        self.format_comment_run(dangling, CommentRunLayout::Lines, writer)
    }
}
