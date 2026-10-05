//! Try blocks, catch bindings, and finally boundaries retain distinct owners.

use super::{CompoundBlock, CompoundGap, Formatter};
use crate::formatter::comments::CommentRunLayout;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::statements::{EmptyBlock, StatementBoundary};
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{BindingPattern, CatchClause, CommentKind, GetSpan, Span, TryStatement};

struct CatchBinding<'ast> {
    pattern: BindingPattern,
    opening: u32,
    closing: u32,
    head: CompoundGap<'ast>,
    tail: CompoundGap<'ast>,
    mode: ContentMode,
}

enum PreparedCatch<'ast> {
    Source(CatchClause),
    Formatted(CatchHead<'ast>),
}

struct FinallyBoundary<'ast> {
    keyword: Span,
    before: CompoundGap<'ast>,
    after: CompoundGap<'ast>,
}

struct CatchHead<'ast> {
    binding: Option<CatchBinding<'ast>>,
    body_head: CompoundGap<'ast>,
    body: CompoundBlock,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_try_statement(
        &mut self,
        node: TryStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let mut body = CompoundBlock::new(node.block(ast), EmptyBlock::Expanded);
        let head = self.compound_gap(Span::new(span.start + 3, body.node.span(ast).start));
        self.prepare_block_head(head, &mut body);
        let handler = node.handler(ast);
        let mut finalizer = node
            .finalizer(ast)
            .map(|body| CompoundBlock::new(body, EmptyBlock::Expanded));
        let catch_gap = handler.map(|handler| {
            self.compound_gap(Span::new(body.node.span(ast).end, handler.span(ast).start))
        });
        let mut handler_mode = ContentMode::Formatted;
        let mut handler_first = ContentMode::Formatted;
        if let Some(gap) = catch_gap
            && gap.has_suppression()
        {
            for position in self.suppression_positions(gap.comments, gap.span.end) {
                match position {
                    SuppressionPosition::Inline => handler_mode = ContentMode::Verbatim,
                    SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                        handler_first = ContentMode::Verbatim
                    }
                }
            }
        }
        let finally_head = if let Some(finalizer) = finalizer.as_ref() {
            let start = handler.map_or(body.node.span(ast).end, |handler| handler.span(ast).end);
            let limit = Span::new(start, finalizer.node.span(ast).start);
            let comments = self.comments.in_range(limit);
            let Some(keyword) = self.find_token(limit, comments, "finally") else {
                return Err(FormatError::unsupported(
                    "finally keyword boundary",
                    None,
                    limit,
                ));
            };
            let keyword = Span::new(keyword, keyword + 7);
            Some(FinallyBoundary {
                keyword,
                before: self.compound_gap(Span::new(start, keyword.start)),
                after: self.compound_gap(Span::new(keyword.end, limit.end)),
            })
        } else {
            None
        };
        if let (Some(boundary), Some(finalizer)) = (finally_head.as_ref(), finalizer.as_mut()) {
            if boundary.before.has_suppression() {
                for position in
                    self.suppression_positions(boundary.before.comments, boundary.before.span.end)
                {
                    match position {
                        SuppressionPosition::Inline => {
                            if handler.is_some() {
                                handler_mode = ContentMode::Verbatim;
                            } else {
                                body.mode = ContentMode::Verbatim;
                            }
                        }
                        SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                            self.protect_block_first(finalizer)
                        }
                    }
                }
            }
            self.prepare_block_head(boundary.after, finalizer);
        }
        let handler = handler
            .map(|handler| {
                self.prepare_catch(handler, handler_mode, handler_first, finalizer.is_some())
            })
            .transpose()?;
        writer.text("try");
        self.format_compound_gap(head, FlatSeparator::Space, Writer::space, writer)?;
        self.format_compound_block_part(body, writer)?;
        if let (Some(gap), Some(handler)) = (catch_gap, handler) {
            self.format_compound_gap(gap, FlatSeparator::Space, Writer::space, writer)?;
            self.format_catch(handler, writer)?;
        }
        if let (Some(boundary), Some(finalizer)) = (finally_head, finalizer) {
            self.format_compound_gap(boundary.before, FlatSeparator::Space, Writer::space, writer)?;
            writer.text(self.source(boundary.keyword));
            self.format_compound_gap(boundary.after, FlatSeparator::Space, Writer::space, writer)?;
            self.format_compound_block_part(finalizer, writer)?;
        }
        Ok(StatementBoundary::SourceEnd {
            end: span.end,
            has_terminator: false,
        })
    }

    fn prepare_catch(
        &self,
        node: CatchClause,
        mode: ContentMode,
        incoming_first: ContentMode,
        has_finalizer: bool,
    ) -> Result<PreparedCatch<'ast>, FormatError> {
        if mode == ContentMode::Verbatim {
            return Ok(PreparedCatch::Source(node));
        }
        let ast = self.ast;
        if node.type_annotation(ast).is_some() {
            return Err(self.unsupported("catch parameter type", node));
        }
        let keyword_end = node.span(ast).start + 5;
        let mut body = CompoundBlock::new(
            node.body(ast),
            if has_finalizer {
                EmptyBlock::Expanded
            } else {
                EmptyBlock::Compact
            },
        );
        let body_start = body.node.span(ast).start;
        let binding = if let Some(pattern) = node.param(ast) {
            let span = pattern.span(ast);
            let head = self.compound_gap(Span::new(keyword_end, span.start));
            let tail = self.compound_gap(Span::new(span.end, body_start));
            let Some(opening) = self.find_token(head.span, head.comments, "(") else {
                return Err(FormatError::unsupported(
                    "catch opening boundary",
                    None,
                    head.span,
                ));
            };
            let Some(closing) = self.find_token(tail.span, tail.comments, ")") else {
                return Err(FormatError::unsupported(
                    "catch closing boundary",
                    None,
                    tail.span,
                ));
            };
            let mut mode = ContentMode::Formatted;
            if self.has_suppression {
                let before_close = self.comments.in_range(Span::new(span.end, closing));
                if head.has_suppression()
                    || self.content_mode(before_close) == ContentMode::Verbatim
                {
                    mode = ContentMode::Verbatim;
                }
                let after_close = self.comments.in_range(Span::new(closing + 1, body_start));
                for position in self.suppression_positions(after_close, body_start) {
                    match position {
                        SuppressionPosition::Inline => body.mode = ContentMode::Verbatim,
                        SuppressionPosition::EndOfLine | SuppressionPosition::OwnLine => {
                            mode = ContentMode::Verbatim
                        }
                    }
                }
            }
            Some(CatchBinding {
                pattern,
                opening,
                closing,
                head,
                tail,
                mode,
            })
        } else {
            None
        };
        let body_head = self.compound_gap(Span::new(
            binding
                .as_ref()
                .map_or(keyword_end, |binding| binding.closing + 1),
            body_start,
        ));
        if binding.is_none() {
            self.prepare_block_head(body_head, &mut body);
        }
        if incoming_first == ContentMode::Verbatim {
            self.protect_block_first(&mut body);
        }
        Ok(PreparedCatch::Formatted(CatchHead {
            binding,
            body_head,
            body,
        }))
    }

    fn format_catch(
        &mut self,
        handler: PreparedCatch<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let handler = match handler {
            PreparedCatch::Source(node) => {
                return self.format_verbatim(node.span(self.ast), writer);
            }
            PreparedCatch::Formatted(handler) => handler,
        };
        writer.text("catch");
        if let Some(binding) = handler.binding {
            writer.group(false, |writer| {
                if binding.head.has_suppression() {
                    self.format_compound_gap(
                        binding.head,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                        writer,
                    )?;
                } else {
                    let split = binding
                        .head
                        .comments
                        .partition_point(|comment| comment.span.end <= binding.opening);
                    let before = &binding.head.comments[..split];
                    let leading = &binding.head.comments[split..];
                    self.line_constraint(
                        binding.head.span.start,
                        before
                            .first()
                            .map_or(binding.opening, |comment| comment.span.start),
                    )
                    .write(writer, FlatSeparator::Space, Writer::space);
                    self.comments.claim(before)?;
                    self.format_comment_run(
                        before,
                        CommentRunLayout::FlatLeading(binding.opening),
                        writer,
                    )?;
                    writer.text("(");
                    self.comments.claim(leading)?;
                    self.format_comment_run(
                        leading,
                        CommentRunLayout::GroupedLeading(binding.head.span.end),
                        writer,
                    )?;
                }
                let tail =
                    self.format_declaration_binding(binding.pattern, binding.mode, writer)?;
                if binding.tail.has_suppression() {
                    self.format_expression_tail(tail, writer)?;
                    self.format_compound_gap(
                        binding.tail,
                        FlatSeparator::Space,
                        Writer::space,
                        writer,
                    )?;
                } else {
                    let span = binding.pattern.span(self.ast);
                    let tail =
                        self.capture_expression_tail(Span::new(span.end, binding.closing), tail)?;
                    let last = tail.as_ref().and_then(|tail| tail.comments.last());
                    let end = last.map_or(span.end, |comment| comment.span.end);
                    self.format_expression_tail(tail, writer)?;
                    self.line_constraint(end, binding.closing).write(
                        writer,
                        FlatSeparator::Empty,
                        |writer| {
                            if last.is_some_and(|comment| comment.kind == CommentKind::Line) {
                                writer.hard_line();
                            }
                        },
                    );
                    writer.text(")");
                    self.format_compound_gap(
                        handler.body_head,
                        FlatSeparator::Space,
                        Writer::space,
                        writer,
                    )?;
                }
                Ok::<_, FormatError>(())
            })?;
        } else {
            self.format_compound_gap(
                handler.body_head,
                FlatSeparator::Space,
                Writer::space,
                writer,
            )?;
        }
        self.format_compound_block_part(handler.body, writer)
    }
}
