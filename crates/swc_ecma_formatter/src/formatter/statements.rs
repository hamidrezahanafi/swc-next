//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{Ast, BlockStatement, Directive, GetSpan, Span, Stmt, StmtData};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail};
use super::expressions::{ExpressionContext, ExpressionPosition};
use super::literals::StringContext;
use crate::{FormatError, Semicolons, document::Writer};

pub(super) enum StatementSequenceKind {
    Program { has_source_items: bool },
    Block,
}

#[derive(Clone, Copy)]
pub(super) enum StatementItem {
    Directive(Directive),
    Statement(Stmt),
}

impl StatementItem {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Directive(node) => node.span(ast),
            Self::Statement(node) => node.span(ast),
        }
    }
}

#[derive(Clone, Copy)]
enum StatementBoundary {
    SourceEnd(u32),
    BeforeTerminator { end: u32, terminator: u32 },
}

impl StatementBoundary {
    fn end(self) -> u32 {
        match self {
            Self::SourceEnd(end) | Self::BeforeTerminator { end, .. } => end,
        }
    }

    fn comment_gap_end(self, comment_end: u32, next_start: u32) -> u32 {
        match self {
            Self::BeforeTerminator { terminator, .. } if comment_end <= terminator => terminator,
            _ => next_start,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_statement_sequence(
        &mut self,
        statements: impl Iterator<Item = StatementItem>,
        span: Span,
        previous_end: Option<u32>,
        kind: StatementSequenceKind,
        writer: &mut Writer<'ast>,
    ) -> Result<bool, FormatError> {
        let ast = self.ast;
        let mut previous = previous_end.map(StatementBoundary::SourceEnd);
        let mut statements = statements
            .filter(|item| match *item {
                StatementItem::Directive(_) => true,
                StatementItem::Statement(statement) => {
                    !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_))
                }
            })
            .peekable();
        while let Some(statement) = statements.next() {
            let statement_span = statement.span(self.ast);
            let leading = self.comments.between(Span::new(
                previous.map_or(span.start, StatementBoundary::end),
                statement_span.start,
            ))?;
            let leading = self.comments.take(leading.len());
            if let Some(boundary) = previous {
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                writer.hard_line();
                if self.line_breaks(boundary.end(), next_start) == 2 {
                    writer.hard_line();
                }
            }
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                let gap_end = previous.map_or(statement_span.start, |boundary| {
                    boundary.comment_gap_end(last.span.end, statement_span.start)
                });
                self.comment_gap(last.span.end, gap_end, writer);
            }
            let boundary = match statement {
                StatementItem::Directive(directive) => {
                    let expression = directive.expression(ast);
                    self.format_string_literal(expression, StringContext::Directive, writer);
                    self.finish_statement(statement_span, expression.span(ast).end, None, writer)?
                }
                StatementItem::Statement(statement) => self.format_statement(statement, writer)?,
            };
            let following = statements.peek().map(|next| next.span(self.ast).start);
            let end = self.format_trailing_comments(
                Span::new(boundary.end(), following.unwrap_or(span.end)),
                following.is_some(),
                writer,
            )?;
            previous = Some(match boundary {
                StatementBoundary::SourceEnd(_) => StatementBoundary::SourceEnd(end),
                StatementBoundary::BeforeTerminator { terminator, .. } => {
                    StatementBoundary::BeforeTerminator { end, terminator }
                }
            });
        }
        let dangling = self.comments.between(Span::new(
            previous.map_or(span.start, StatementBoundary::end),
            span.end,
        ))?;
        let dangling = self.comments.take(dangling.len());
        if let Some(first) = dangling.first()
            && let Some(boundary) = previous
        {
            self.comment_gap(boundary.end(), first.span.start, writer);
        }
        let layout = match (previous, kind) {
            (None, StatementSequenceKind::Block) => CommentRunLayout::Lines,
            (
                None,
                StatementSequenceKind::Program {
                    has_source_items: true,
                },
            ) => CommentRunLayout::Lines,
            _ => CommentRunLayout::Source,
        };
        self.format_comment_run(dangling, layout, writer)?;
        Ok(previous.is_some() || !dangling.is_empty())
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let (content_end, tail) = match self.ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => {
                let expression = node.expression(self.ast);
                let tail = self.format_expression(
                    expression,
                    ExpressionContext::at(ExpressionPosition::Statement),
                    writer,
                )?;
                (expression.span(self.ast).end, tail)
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                (node.span(self.ast).start + "debugger".len() as u32, None)
            }
            StmtData::BlockStatement(block) => {
                self.format_block(block, writer)?;
                return Ok(StatementBoundary::SourceEnd(block.span(self.ast).end));
            }
            _ => return Err(self.unsupported("statement", statement)),
        };
        self.finish_statement(statement.span(self.ast), content_end, tail, writer)
    }

    fn finish_statement(
        &mut self,
        span: Span,
        content_end: u32,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        if self.options.semicolons == Semicolons::Always {
            writer.text(";");
        }
        self.format_expression_tail(tail, writer)?;
        let comments = self.comments.between(Span::new(content_end, span.end))?;
        let mut inline_end = content_end;
        let mut count = 0;
        for comment in comments {
            if self.line_breaks(inline_end, comment.span.start) != 0 {
                break;
            }
            inline_end = comment.span.end;
            count += 1;
        }
        // Own-line comments before a terminator belong to the sequence boundary.
        let boundary = if count == comments.len() {
            StatementBoundary::SourceEnd(span.end)
        } else {
            StatementBoundary::BeforeTerminator {
                end: inline_end,
                terminator: span.end - 1,
            }
        };
        let comments = self.comments.take(count);
        if let Some(first) = comments.first() {
            self.comment_gap(content_end, first.span.start, writer);
        }
        self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
        Ok(boundary)
    }

    fn format_block(
        &mut self,
        block: BlockStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let block_span = block.span(self.ast);
        let body_span = Span::new(block_span.start + 1, block_span.end - 1);
        let body = block.body(self.ast);
        writer.text("{");
        let has_statements = body.iter().any(|id| {
            let statement = self.ast.get_node_in_sub_range(id);
            !matches!(self.ast.stmt_data(statement), StmtData::EmptyStatement(_))
        });
        if !has_statements && self.comments.between(body_span)?.is_empty() {
            writer.hard_line();
        } else {
            writer.indent(|writer| {
                writer.hard_line();
                self.format_statement_sequence(
                    body.iter()
                        .map(|id| StatementItem::Statement(self.ast.get_node_in_sub_range(id))),
                    body_span,
                    None,
                    StatementSequenceKind::Block,
                    writer,
                )
            })?;
            writer.hard_line();
        }
        writer.text("}");
        Ok(())
    }
}
