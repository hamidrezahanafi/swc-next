//! Variable declarations and their real keyword, binding and initializer owners.

use swc_next_ecma_ast::{GetSpan, Span, VariableDeclaration, VariableDeclarator, VariableKind};

use super::Formatter;
use super::bindings::BindingInitializer;
use super::comments::ExpressionTail;
use super::diagnostics::FlatSeparator;
use super::expressions::ExpressionContext;
use super::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use super::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_variable_declaration(
        &mut self,
        declaration: VariableDeclaration,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        let ast = self.ast;
        if declaration.declare(ast) {
            return Err(self.unsupported("ambient variable declaration", declaration));
        }
        let mut nodes = declaration.declarators(ast);
        let Some(first) = nodes.first().map(|id| ast.get_node_in_sub_range(id)) else {
            return Err(self.unsupported("empty variable declaration", declaration));
        };
        let remaining = nodes.split_off(1);
        let (end, has_value) = remaining.iter().fold(
            (first.span(ast).end, first.init(ast).is_some()),
            |(_, has_value), id| {
                let declarator = ast.get_node_in_sub_range(id);
                (
                    declarator.span(ast).end,
                    has_value || declarator.init(ast).is_some(),
                )
            },
        );
        let mut declarators = std::iter::once(first)
            .chain(remaining.iter().map(|id| ast.get_node_in_sub_range(id)))
            .peekable();
        let start = declaration.span(ast).start;
        let kind = declaration.kind(ast);
        let keyword_end = if kind == VariableKind::AwaitUsing {
            let gap = Span::new(start + 5, first.span(ast).start);
            let using = self
                .find_token(gap, self.comments.in_range(gap), "using")
                .ok_or_else(|| self.unsupported("await using keyword boundary", declaration))?;
            writer.text("await");
            self.format_source_token_gap(start + 5, using, true, writer)?;
            writer.text("using");
            using + 5
        } else {
            writer.text(kind.as_str());
            start + kind.as_str().len() as u32
        };
        let comments = self
            .comments
            .in_range(Span::new(keyword_end, first.span(ast).start));
        let first_start = comments
            .first()
            .map_or(first.span(ast).start, |comment| comment.span.start);
        let indent_first = !remaining.is_empty() || !comments.is_empty();
        self.line_constraint(keyword_end, first_start).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        writer.group(false, |writer| {
            let mut previous_end = keyword_end;
            let mut separator = Separator::Line;
            let mut following_mode = None;
            let mut first_item = true;
            while let Some(declarator) = declarators.next() {
                let span = declarator.span(ast);
                let following = declarators.peek().map_or(FollowingItem::End, |node| {
                    FollowingItem::Node(node.span(ast))
                });
                let mut write = |writer: &mut Writer<'ast>| {
                    if !first_item {
                        let next = self
                            .comments
                            .in_range(Span::new(previous_end, span.start))
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        if has_value && !matches!(separator, Separator::Source(_)) {
                            self.line_constraint(previous_end, next).write(
                                writer,
                                FlatSeparator::Space,
                                Writer::hard_line,
                            );
                        } else {
                            ArgumentLayout::Lines.separator(
                                separator,
                                self.line_constraint(previous_end, next),
                                writer,
                            );
                        }
                    }
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(previous_end, end),
                        false,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    let tail = self.format_variable_declarator(declarator, item.mode, writer)?;
                    self.finish_delimited_item(item, tail, writer)
                };
                let boundary = if !first_item || indent_first {
                    writer.indent(write)?
                } else {
                    write(writer)?
                };
                previous_end = boundary.end;
                separator = boundary.separator;
                following_mode = Some(boundary.following_mode);
                first_item = false;
            }
            Ok((end, None))
        })
    }

    fn format_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            self.format_verbatim(declarator.span(ast), writer)?;
            return Ok(None);
        }
        if declarator.type_annotation(ast).is_some() || declarator.definite(ast) {
            return Err(self.unsupported("typed variable binding", declarator));
        }
        let left = declarator.id(ast);
        let Some(right) = declarator.init(ast) else {
            return self.format_declaration_binding(left, ContentMode::Formatted, writer);
        };
        let left_end = left.span(ast).end;
        let right_start = right.span(ast).start;
        let gap = Span::new(left_end, right_start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("variable initializer boundary", declarator));
        };
        let content = self.expression_content(right);
        let mut boundary =
            self.assignment_boundary(left_end, &content, Span::new(operator, operator + 1))?;
        let mut left_mode = ContentMode::Formatted;
        if self.has_suppression {
            let before =
                &comments[..comments.partition_point(|comment| comment.span.end <= operator)];
            for position in self.suppression_positions(before, operator) {
                match position {
                    SuppressionPosition::Inline => left_mode = ContentMode::Verbatim,
                    SuppressionPosition::OwnLine | SuppressionPosition::EndOfLine => {
                        boundary.mode = ContentMode::Verbatim;
                        if let Some(comment) = boundary.before.last() {
                            boundary.before_operator =
                                self.source_line_constraint(comment.span.end, operator);
                        }
                    }
                }
            }
        }
        let break_after = boundary.own_line
            || !boundary.operator_line.is_empty()
            || self.assignment_right_breaks_after_operator(content.data);
        self.format_binding_initializer(
            BindingInitializer {
                left,
                right,
                boundary,
                left_mode,
                break_after,
                context: ExpressionContext::variable_initializer(),
            },
            writer,
        )
    }
}
