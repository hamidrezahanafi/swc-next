//! Array slots, comment boundaries, and grouped delimiter layout.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, CommentKind, ExprData, GetSpan, Span, UnaryOperator,
};

use super::Formatter;
use crate::formatter::comments::CommentRunLayout;
use crate::{FormatError, TrailingCommas, document::Writer};

struct ArrayItem {
    holes: usize,
    argument: Option<Argument>,
}

#[derive(Clone, Copy)]
enum ArrayLayout {
    Lines,
    Fill,
}

#[derive(Clone, Copy)]
enum Separator {
    Line,
    EmptyLine,
    OwnLineComment,
}

struct ItemBoundary {
    end: u32,
    separator: Separator,
}

impl ArrayLayout {
    fn entry<'a, T>(self, writer: &mut Writer<'a>, write: impl FnOnce(&mut Writer<'a>) -> T) -> T {
        match self {
            Self::Lines => write(writer),
            Self::Fill => writer.fill_entry(write),
        }
    }

    fn separator(self, separator: Separator, writer: &mut Writer<'_>) {
        self.entry(writer, |writer| match (self, separator) {
            (Self::Fill, Separator::EmptyLine) => {
                writer.hard_line();
                writer.hard_line();
            }
            (Self::Fill, Separator::OwnLineComment) => writer.hard_line(),
            (_, Separator::EmptyLine) => {
                writer.line();
                writer.soft_line();
            }
            _ => writer.line(),
        });
    }
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
        let content_span = Span::new(span.start + 1, span.end - 1);
        if elements.is_empty() && self.comments.between(content_span)?.is_empty() {
            writer.text("[]");
            return Ok(());
        }
        let expand = elements.len() > 1 && elements.iter().all(|id| {
            let Some(argument) = self.ast.get_node_in_sub_range(id) else { return false };
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else { return false };
            matches!(self.ast.expr_data(expression), ExprData::ArrayExpression(inner) if inner.elements(self.ast).len() > 1)
        });
        let numeric = self.numeric_array_first(array);
        let layout = if let Some(first) = numeric
            && !self.comments.between(content_span)?.iter().any(|comment| {
                comment.span.start >= first.span(self.ast).end
                    && comment.kind == CommentKind::Line
                    && !self.source_position_starts_line(comment.span.start)
            }) {
            ArrayLayout::Fill
        } else {
            ArrayLayout::Lines
        };
        writer.group(expand, |writer| {
            writer.text("[");
            writer.indent(|writer| {
                writer.soft_line();
                match layout {
                    ArrayLayout::Lines => {
                        self.format_array_items(array, content_span, layout, writer)
                    }
                    ArrayLayout::Fill => writer.fill(|writer| {
                        self.format_array_items(array, content_span, layout, writer)
                    }),
                }
            })?;
            writer.soft_line();
            writer.text("]");
            Ok(())
        })
    }

    fn numeric_array_first(&self, array: ArrayExpression) -> Option<Argument> {
        let mut elements = array
            .elements(self.ast)
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id));
        let first = elements.next().flatten()?;
        for argument in std::iter::once(Some(first)).chain(elements) {
            let argument = argument?;
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
                return None;
            };
            let (_, data) = self.unparenthesized_expression(expression);
            let numeric = match data {
                ExprData::NumericLiteral(_) => true,
                ExprData::UnaryExpression(unary)
                    if matches!(
                        unary.operator(self.ast),
                        UnaryOperator::Positive | UnaryOperator::Negate
                    ) =>
                {
                    let (_, argument) = self.unparenthesized_expression(unary.argument(self.ast));
                    matches!(argument, ExprData::NumericLiteral(_))
                        && self.comments.in_range(unary.span(self.ast)).is_empty()
                }
                _ => false,
            };
            if !numeric {
                return None;
            }
        }
        Some(first)
    }

    fn format_array_items(
        &mut self,
        array: ArrayExpression,
        span: Span,
        layout: ArrayLayout,
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
        let mut separator = Separator::Line;
        while let Some(item) = items.next() {
            for _ in 0..item.holes {
                if has_slot {
                    layout.separator(separator, writer);
                    separator = Separator::Line;
                }
                writer.text(",");
                has_slot = true;
            }
            let Some(argument) = item.argument else {
                continue;
            };
            if has_slot {
                layout.separator(separator, writer);
            }
            let following = items.peek().and_then(|item| item.argument);
            let boundary = layout.entry(writer, |writer| {
                self.format_array_item(
                    argument,
                    following,
                    items.peek().is_some(),
                    Span::new(previous_end, span.end),
                    writer,
                )
            })?;
            previous_end = boundary.end;
            separator = boundary.separator;
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

    fn format_array_item(
        &mut self,
        argument: Argument,
        following: Option<Argument>,
        has_more: bool,
        span: Span,
        writer: &mut Writer<'ast>,
    ) -> Result<ItemBoundary, FormatError> {
        let ast = self.ast;
        let argument_span = argument.span(ast);
        let leading = self
            .comments
            .between(Span::new(span.start, argument_span.start))?;
        let leading = self.comments.take(leading.len());
        self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
        if let Some(last) = leading.last() {
            self.comment_gap(last.span.end, argument_span.start, writer);
        }
        let tail = self.format_argument(argument, writer)?;
        let content_end = tail
            .as_ref()
            .map_or(argument_span.end, |tail| tail.source_end);
        let gap = Span::new(
            content_end,
            following.map_or(span.end, |node| node.span(ast).start),
        );
        let comments = self.comments.between(gap)?;
        let comma = self.find_comma(gap, comments);
        let mut inline_end = content_end;
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
            if following.is_none() || comma.is_none_or(|position| comment.span.start < position) {
                trailing_count = index + 1;
            }
        }
        let trailing = self.comments.take(trailing_count);
        let trailing_end = trailing
            .last()
            .map_or(content_end, |comment| comment.span.end);
        self.format_inline_trailing_comments(trailing, writer)?;
        let mut separator = if comments[trailing_count..]
            .iter()
            .any(|comment| comment.kind == CommentKind::Line)
        {
            Separator::OwnLineComment
        } else {
            Separator::Line
        };
        if has_more {
            writer.text(",");
            if let Some(comma) = comma {
                let after_separator = Span::new(comma + 1, gap.end);
                let next_start = comments
                    .get(trailing_count)
                    .map_or(gap.end, |comment| comment.span.start);
                let start = after_separator.start.max(trailing_end);
                if start < next_start && self.next_line_is_empty(start, next_start) {
                    separator = Separator::EmptyLine;
                }
            }
        } else if self.options.trailing_commas != TrailingCommas::None {
            writer.if_break(",");
        }
        let mut end = trailing_end;
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
                end = last.span.end;
            }
        }
        Ok(ItemBoundary { end, separator })
    }
}
