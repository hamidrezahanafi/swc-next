//! Array slots, comment boundaries, and grouped delimiter layout.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrayExpression, CommentKind, ExprData, GetSpan, Span, UnaryOperator,
};

use super::Formatter;
use super::arguments::{ArgumentLayout, FollowingArgument, Separator};
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, TrailingCommas, document::Writer};

struct ArgumentItem {
    holes: usize,
    argument: Option<Argument>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrayChildKind {
    Array,
    Object,
}

fn next_item(elements: &mut impl Iterator<Item = Option<Argument>>) -> Option<ArgumentItem> {
    let mut holes = 0;
    for element in elements.by_ref() {
        if let Some(argument) = element {
            return Some(ArgumentItem {
                holes,
                argument: Some(argument),
            });
        }
        holes += 1;
    }
    (holes != 0).then_some(ArgumentItem {
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
        if self.has_suppression
            && elements
                .iter()
                .all(|id| self.ast.get_node_in_sub_range(id).is_none())
            && self.content_mode(self.comments.in_range(content_span)) == ContentMode::Verbatim
        {
            return self.format_verbatim(span, writer);
        }
        if elements.is_empty() && self.comments.between(content_span)?.is_empty() {
            writer.text("[]");
            return Ok(());
        }
        let expand = self.array_should_expand(array);
        let numeric = self.numeric_array_first(array);
        let opening = self.opening_line_constraint(content_span);
        let layout = if let Some(first) = numeric
            && !self.comments.between(content_span)?.iter().any(|comment| {
                comment.span.start >= first.span(self.ast).end
                    && comment.kind == CommentKind::Line
                    && !self.source_position_starts_line(comment.span.start)
            }) {
            ArgumentLayout::Fill
        } else {
            ArgumentLayout::Lines
        };
        writer.group(expand, |writer| {
            writer.text("[");
            let end = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Empty, Writer::soft_line);
                match layout {
                    ArgumentLayout::Lines => {
                        self.format_array_items(array, content_span, layout, writer)
                    }
                    ArgumentLayout::Fill => writer.fill(|writer| {
                        self.format_array_items(array, content_span, layout, writer)
                    }),
                }
            })?;
            self.line_constraint(end, content_span.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            writer.text("]");
            Ok(())
        })
    }

    fn array_should_expand(&self, array: ArrayExpression) -> bool {
        let elements = array.elements(self.ast);
        if elements.len() < 2 {
            return false;
        }
        let mut kinds = elements.iter().map(|id| {
            let argument = self.ast.get_node_in_sub_range(id)?;
            let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
                return None;
            };
            match self.expression_content(expression).data {
                ExprData::ArrayExpression(child) if child.elements(self.ast).len() > 1 => {
                    Some(ArrayChildKind::Array)
                }
                ExprData::ObjectExpression(child) if child.properties(self.ast).len() > 1 => {
                    Some(ArrayChildKind::Object)
                }
                _ => None,
            }
        });
        let Some(Some(kind)) = kinds.next() else {
            return false;
        };
        kinds.all(|next| next == Some(kind))
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
            let data = self.expression_content(expression).data;
            let numeric = match data {
                ExprData::NumericLiteral(_) => true,
                ExprData::UnaryExpression(unary)
                    if matches!(
                        unary.operator(self.ast),
                        UnaryOperator::Positive | UnaryOperator::Negate
                    ) =>
                {
                    let argument = self.expression_content(unary.argument(self.ast)).data;
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
        layout: ArgumentLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<u32, FormatError> {
        let ast = self.ast;
        let mut elements = array
            .elements(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let mut items = std::iter::from_fn(|| next_item(&mut elements)).peekable();
        let mut previous_end = span.start;
        let mut has_slot = false;
        let mut separator = Separator::Line;
        let constrained = self.diagnostic_range_is_sensitive(span);
        while let Some(item) = items.next() {
            for _ in 0..item.holes {
                if constrained {
                    let end = item
                        .argument
                        .map_or(span.end, |argument| argument.span(ast).start);
                    let gap = Span::new(previous_end, end);
                    let comments = self.comments.between(gap)?;
                    let Some(comma) = self.find_comma(gap, comments) else {
                        return Err(FormatError::unsupported(
                            "array elision boundary",
                            None,
                            gap,
                        ));
                    };
                    let count = comments.partition_point(|comment| comment.span.end <= comma);
                    let comments = self.comments.take(count);
                    match (comments.first(), has_slot) {
                        (Some(first), has_slot) => {
                            if has_slot {
                                self.comment_gap(previous_end, first.span.start, writer);
                            }
                            self.format_comment_run(
                                comments,
                                CommentRunLayout::Leading(comma),
                                writer,
                            )?;
                        }
                        (None, true) => layout.separator(
                            separator,
                            self.line_constraint(previous_end, comma),
                            writer,
                        ),
                        (None, false) => self.line_constraint(previous_end, comma).write(
                            writer,
                            FlatSeparator::Empty,
                            |_| {},
                        ),
                    }
                    writer.text(",");
                    previous_end = comma + 1;
                    separator = Separator::Line;
                    has_slot = true;
                    continue;
                }
                if has_slot {
                    layout.separator(separator, LineConstraint::Unconstrained, writer);
                    separator = Separator::Line;
                }
                writer.text(",");
                has_slot = true;
            }
            let Some(argument) = item.argument else {
                continue;
            };
            if has_slot {
                let next_start = self
                    .comments
                    .between(Span::new(previous_end, argument.span(ast).start))?
                    .first()
                    .map_or(argument.span(ast).start, |comment| comment.span.start);
                layout.separator(
                    separator,
                    self.line_constraint(previous_end, next_start),
                    writer,
                );
            }
            let boundary = layout.entry(writer, |writer| {
                self.format_argument_item(
                    argument,
                    match items.peek() {
                        Some(item) if item.holes != 0 => FollowingArgument::Elision(item.argument),
                        Some(item) => item
                            .argument
                            .map_or(FollowingArgument::End, FollowingArgument::Argument),
                        None => FollowingArgument::End,
                    },
                    Span::new(previous_end, span.end),
                    self.options.trailing_commas != TrailingCommas::None,
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
        self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
        Ok(dangling
            .last()
            .map_or(previous_end, |comment| comment.span.end))
    }
}
