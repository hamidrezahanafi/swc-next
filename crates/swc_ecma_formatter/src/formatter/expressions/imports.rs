//! Dynamic imports and their real source/options expression slots.

use swc_next_ecma_ast::{ExprData, GetSpan, ImportExpression, ImportPhase, Span};

use super::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
use super::arrows::ArrowLayout;
use super::{ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::diagnostics::FlatSeparator;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_import_expression(
        &mut self,
        node: ImportExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let keyword_end = span.start + 6;
        let source = node.source(ast);
        let source_span = source.span(ast);
        let options = node.options(ast);
        let opening_gap = Span::new(keyword_end, source_span.start);
        let comments = self.comments.in_range(opening_gap);
        let opening = self
            .find_token(opening_gap, comments, "(")
            .ok_or_else(|| self.unsupported("import opening boundary", node))?;
        let prefix_count = comments.partition_point(|comment| comment.span.end <= opening);
        let prefix = &comments[..prefix_count];
        let prefix_span = Span::new(keyword_end, opening);
        let target = self.has_suppression.then_some(source_span);
        let binding = self.comment_binding(prefix, Span::new(keyword_end, opening + 1), target);
        let move_prefix = binding.allows_relocation(target);
        let modifier = if node.sync(ast) {
            Some("sync")
        } else {
            match node.phase(ast) {
                Some(ImportPhase::Source) => Some("source"),
                Some(ImportPhase::Defer) => Some("defer"),
                None => None,
            }
        };
        let modifier_tokens = if !move_prefix {
            modifier
                .map(|name| {
                    let dot = self
                        .find_token(prefix_span, prefix, ".")
                        .ok_or_else(|| self.unsupported("import phase dot boundary", node))?;
                    let start = self
                        .find_token(Span::new(dot + 1, opening), prefix, name)
                        .ok_or_else(|| self.unsupported("import phase keyword boundary", node))?;
                    Ok::<_, FormatError>((dot, Span::new(start, start + name.len() as u32)))
                })
                .transpose()?
        } else {
            None
        };
        writer.text("import");
        if let Some(name) = modifier {
            if let Some((dot, token)) = modifier_tokens {
                self.format_source_token_gap(keyword_end, dot, false, writer)?;
                writer.text(".");
                self.format_source_token_gap(dot + 1, token.start, false, writer)?;
                writer.text(name);
                self.format_source_token_gap(token.end, opening, false, writer)?;
            } else {
                writer.text(".");
                writer.text(name);
            }
        } else if !move_prefix {
            self.format_source_token_gap(keyword_end, opening, false, writer)?;
        }
        let content = Span::new(
            if move_prefix && !prefix.is_empty() {
                keyword_end
            } else {
                opening + 1
            },
            span.end - 1,
        );
        let expand = self.next_line_is_empty(opening + 1, source_span.start)
            || options.is_some_and(|option| {
                self.next_line_is_empty(source_span.end, option.span(ast).start)
            });
        let hugged = options.is_none()
            && matches!(
                self.expression_content(source).data,
                ExprData::StringLiteral(_)
            )
            && self.comments.in_range(content).is_empty()
            && !self.diagnostic_range_is_sensitive(content);
        writer.group(!hugged && expand, |writer| {
            writer.text("(");
            let mut write_items = |writer: &mut Writer<'ast>| {
                if !hugged {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut expressions = [Some(source), options].into_iter().flatten().peekable();
                let mut previous: Option<ItemBoundary> = None;
                while let Some(expression) = expressions.next() {
                    let item_span = expression.span(ast);
                    let end = previous.map_or(content.start, |boundary| boundary.end);
                    if let Some(boundary) = previous {
                        let next = self
                            .comments
                            .between(Span::new(end, item_span.start))?
                            .first()
                            .map_or(item_span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            boundary.separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = expressions.peek().map_or(FollowingItem::End, |next| {
                        FollowingItem::Node(next.span(ast))
                    });
                    let mut item = self.prepare_delimited_item(
                        item_span,
                        following,
                        Span::new(end, content.end),
                        false,
                        writer,
                    )?;
                    if let Some(boundary) = previous {
                        item.set_leading_mode(boundary.following_mode);
                    }
                    let tail = self.format_expression(
                        expression,
                        ExpressionContext::at(ExpressionPosition::Nested)
                            .with_content_mode(item.mode)
                            .with_arrow_layout(ArrowLayout::Argument),
                        writer,
                    )?;
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                let end = previous.map_or(content.start, |boundary| boundary.end);
                self.format_argument_dangling(Span::new(end, content.end), writer)
            };
            let end = if hugged {
                write_items(writer)?
            } else {
                writer.indent(write_items)?
            };
            if !hugged {
                self.line_constraint(end, content.end).write(
                    writer,
                    FlatSeparator::Empty,
                    Writer::soft_line,
                );
            }
            writer.text(")");
            Ok(None)
        })
    }
}
