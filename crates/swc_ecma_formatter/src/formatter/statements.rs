//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{Ast, BlockStatement, Directive, GetSpan, Span, Stmt, StmtData};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::{ExpressionContext, ExpressionPosition, ExpressionStart};
use super::literals::StringContext;
use super::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, Semicolons, document::Writer};

pub(super) enum StatementSequenceKind {
    Program { has_source_items: bool },
    Block,
}

pub(super) struct StatementSequence {
    pub(super) has_content: bool,
    end: u32,
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
    ) -> Result<StatementSequence, FormatError> {
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
            let leading_mode = self.content_mode(leading);
            let following = statements.peek().map(|next| next.span(self.ast).start);
            let mode = if leading_mode == ContentMode::Verbatim || !self.has_suppression {
                leading_mode
            } else {
                let content_end = match statement {
                    StatementItem::Directive(directive) => directive.expression(ast).span(ast).end,
                    StatementItem::Statement(node) => match ast.stmt_data(node) {
                        StmtData::ExpressionStatement(expression) => {
                            expression.expression(ast).span(ast).end
                        }
                        StmtData::DebuggerStatement(_) => {
                            statement_span.start + "debugger".len() as u32
                        }
                        _ => statement_span.end,
                    },
                };
                self.trailing_content_mode(
                    Span::new(content_end, following.unwrap_or(span.end)),
                    BoundaryPunctuation::Semicolon,
                    following.is_none(),
                )
            };
            let leading = self.comments.take(leading.len());
            let mut expression_start = ExpressionStart::StatementPending;
            if let Some(boundary) = previous {
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                let constraint = self.line_constraint(boundary.end(), next_start);
                if constraint == LineConstraint::SameLine
                    && self.options.semicolons == Semicolons::AsNeeded
                {
                    writer.text(";");
                    expression_start = ExpressionStart::None;
                }
                constraint.write(writer, FlatSeparator::Space, Writer::hard_line);
                if constraint == LineConstraint::Unconstrained
                    && self.line_breaks(boundary.end(), next_start) == 2
                {
                    writer.hard_line();
                }
            }
            let head = NodeHead {
                comments: leading,
                content_start: statement_span.start,
                layout: CommentRunLayout::Source,
                gap_end: leading.last().map(|last| {
                    previous.map_or(statement_span.start, |boundary| {
                        boundary.comment_gap_end(last.span.end, statement_span.start)
                    })
                }),
            };
            let boundary = match statement {
                StatementItem::Directive(directive) => {
                    self.format_node_head(head, expression_start, writer)?;
                    let expression = directive.expression(ast);
                    if mode == ContentMode::Verbatim {
                        self.format_verbatim(expression.span(ast), writer)?;
                    } else {
                        self.format_string_literal(expression, StringContext::Directive, writer);
                    }
                    self.finish_statement(statement_span, expression.span(ast).end, None, writer)?
                }
                StatementItem::Statement(statement) => {
                    self.format_statement(statement, expression_start, mode, head, writer)?
                }
            };
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
        Ok(StatementSequence {
            has_content: previous.is_some() || !dangling.is_empty(),
            end: dangling.last().map_or_else(
                || previous.map_or(span.start, StatementBoundary::end),
                |comment| comment.span.end,
            ),
        })
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        start: ExpressionStart,
        mode: ContentMode,
        head: NodeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let data = self.ast.stmt_data(statement);
        if let StmtData::ExpressionStatement(node) = data {
            let expression = node.expression(self.ast);
            let tail = self.format_expression_with_leading_comments(
                expression,
                head,
                ExpressionContext::at(ExpressionPosition::Statement)
                    .with_expression_start(start)
                    .with_content_mode(mode),
                writer,
            )?;
            return self.finish_statement(
                statement.span(self.ast),
                expression.span(self.ast).end,
                tail,
                writer,
            );
        }
        self.format_node_head(head, start, writer)?;
        match data {
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                self.finish_statement(
                    statement.span(self.ast),
                    node.span(self.ast).start + "debugger".len() as u32,
                    None,
                    writer,
                )
            }
            StmtData::BlockStatement(block) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(block.span(self.ast), writer)?;
                } else {
                    self.format_block(block, writer)?;
                }
                Ok(StatementBoundary::SourceEnd(block.span(self.ast).end))
            }
            _ if mode == ContentMode::Verbatim => {
                let span = statement.span(self.ast);
                self.format_verbatim(span, writer)?;
                Ok(StatementBoundary::SourceEnd(span.end))
            }
            _ => Err(self.unsupported("statement", statement)),
        }
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
        let terminator_on_next_line = self.ast.source().as_bytes().get(span.end as usize - 1)
            == Some(&b';')
            && self.line_breaks(inline_end, span.end - 1) != 0;
        let boundary = if count == comments.len() && !terminator_on_next_line {
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
        let opening = self.opening_line_constraint(body_span);
        writer.text("{");
        let has_statements = body.iter().any(|id| {
            let statement = self.ast.get_node_in_sub_range(id);
            !matches!(self.ast.stmt_data(statement), StmtData::EmptyStatement(_))
        });
        if !has_statements && self.comments.between(body_span)?.is_empty() {
            opening.write(writer, FlatSeparator::Empty, Writer::hard_line);
        } else {
            let sequence = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
                self.format_statement_sequence(
                    body.iter()
                        .map(|id| StatementItem::Statement(self.ast.get_node_in_sub_range(id))),
                    body_span,
                    None,
                    StatementSequenceKind::Block,
                    writer,
                )
            })?;
            self.line_constraint(sequence.end, body_span.end).write(
                writer,
                FlatSeparator::Space,
                Writer::hard_line,
            );
        }
        writer.text("}");
        Ok(())
    }
}
