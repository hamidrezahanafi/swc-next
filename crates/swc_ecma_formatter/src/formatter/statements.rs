//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{BlockStatement, GetSpan, Span, Stmt, StmtData, SubRange};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::expressions::ExpressionPosition;
use crate::{FormatError, Semicolons, document::Writer};

pub(super) enum StatementSequenceKind {
    Program,
    Block,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_statement_sequence(
        &mut self,
        statements: SubRange<Stmt>,
        span: Span,
        mut previous_end: Option<u32>,
        kind: StatementSequenceKind,
        writer: &mut Writer<'ast>,
    ) -> Result<bool, FormatError> {
        let ast = self.ast;
        let has_source_statements = !statements.is_empty();
        let mut statements = statements
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .filter(|statement| !matches!(ast.stmt_data(*statement), StmtData::EmptyStatement(_)))
            .peekable();
        while let Some(statement) = statements.next() {
            let statement_span = statement.span(self.ast);
            let leading = self.comments.between(Span::new(
                previous_end.unwrap_or(span.start),
                statement_span.start,
            ))?;
            let leading = self.comments.take(leading.len());
            if let Some(end) = previous_end {
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                writer.hard_line();
                if self.line_breaks(end, next_start) == 2 {
                    writer.hard_line();
                }
            }
            self.format_comment_run(leading, CommentRunLayout::Source, writer)?;
            if let Some(last) = leading.last() {
                self.comment_gap(last.span.end, statement_span.start, writer);
            }
            self.format_statement(statement, writer)?;
            let following = statements.peek().map(|next| next.span(self.ast).start);
            previous_end = Some(self.format_trailing_comments(
                Span::new(statement_span.end, following.unwrap_or(span.end)),
                following.is_some(),
                writer,
            )?);
        }
        let dangling = self
            .comments
            .between(Span::new(previous_end.unwrap_or(span.start), span.end))?;
        let dangling = self.comments.take(dangling.len());
        if let Some(first) = dangling.first()
            && let Some(end) = previous_end
        {
            self.comment_gap(end, first.span.start, writer);
        }
        let layout = match (previous_end, kind) {
            (None, StatementSequenceKind::Block) => CommentRunLayout::Lines,
            (None, StatementSequenceKind::Program) if has_source_statements => {
                CommentRunLayout::Lines
            }
            _ => CommentRunLayout::Source,
        };
        self.format_comment_run(dangling, layout, writer)?;
        Ok(previous_end.is_some() || !dangling.is_empty())
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let content_end = match self.ast.stmt_data(statement) {
            StmtData::ExpressionStatement(node) => {
                let expression = node.expression(self.ast);
                self.format_expression(expression, ExpressionPosition::Statement, writer)?;
                expression.span(self.ast).end
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                node.span(self.ast).start + "debugger".len() as u32
            }
            StmtData::BlockStatement(block) => return self.format_block(block, writer),
            _ => return Err(self.unsupported("statement", statement)),
        };
        if self.options.semicolons == Semicolons::Always {
            writer.text(";");
        }
        let comments = self
            .comments
            .between(Span::new(content_end, statement.span(self.ast).end))?;
        let comments = self.comments.take(comments.len());
        if let Some(first) = comments.first() {
            self.comment_gap(content_end, first.span.start, writer);
        }
        self.format_comment_run(comments, CommentRunLayout::Source, writer)
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
                    body,
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
