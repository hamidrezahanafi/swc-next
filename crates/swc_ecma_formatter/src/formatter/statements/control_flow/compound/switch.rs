//! Case heads, statement sequences, and source packets between real case owners.

use super::{CompoundBlock, CompoundGap, Formatter, GapLayout};
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition, TailOwner};
use crate::formatter::statements::{
    CaseIncoming, EmptyBlock, IncomingStatement, StatementBoundary, StatementItem,
    StatementSequenceKind,
};
use crate::formatter::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use crate::{FormatError, document::Writer};
use swc_next_ecma_ast::{
    CommentKind, Expr, GetSpan, Span, Stmt, StmtData, SwitchCase, SwitchStatement,
};

struct PreparedCase<'ast> {
    node: SwitchCase,
    mode: ContentMode,
    test: Option<Expr>,
    test_mode: ContentMode,
    colon: u32,
    leading: CompoundGap<'ast>,
    body_head: CompoundGap<'ast>,
    incoming: Option<CaseIncoming>,
    trailing: CompoundGap<'ast>,
    following_mode: ContentMode,
    first: Option<Stmt>,
    adjacent_block: bool,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_switch_statement(
        &mut self,
        node: SwitchStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let mut cases = node
            .cases(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let first = cases.peek().copied();
        let discriminant = node.discriminant(ast);
        let disc_span = discriminant.span(ast);
        let limit = Span::new(
            disc_span.end,
            first.map_or(span.end - 1, |node| node.span(ast).start),
        );
        let comments = self.comments.in_range(limit);
        let Some(opening) = self.find_token(limit, comments, "{") else {
            return Err(FormatError::unsupported(
                "switch body opening boundary",
                None,
                limit,
            ));
        };
        let body = Span::new(opening + 1, span.end - 1);
        let first_gap = self.compound_gap(Span::new(
            body.start,
            first.map_or(body.end, |node| node.span(ast).start),
        ));
        if first.is_none() && first_gap.has_suppression() {
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::SourceEnd {
                end: span.end,
                has_terminator: false,
            });
        }
        let keyword_end = span.start + 6;
        let mut condition = self.prepare_condition(discriminant, keyword_end, opening)?;
        let leading = self.compound_gap(Span::new(keyword_end, disc_span.start));
        let condition_tail = self.compound_gap(Span::new(disc_span.end, opening));
        let mut first_mode = ContentMode::Formatted;
        condition.mode = self.content_mode(leading.comments);
        if condition_tail.has_suppression() {
            for position in self.suppression_positions(
                condition_tail.comments,
                first.map_or(opening, |node| node.span(ast).start),
            ) {
                match position {
                    SuppressionPosition::OwnLine if first.is_some() => {
                        first_mode = ContentMode::Verbatim
                    }
                    _ => condition.mode = ContentMode::Verbatim,
                }
            }
        }
        if first_gap.has_suppression() {
            for position in self.suppression_positions(first_gap.comments, first_gap.span.end) {
                match position {
                    SuppressionPosition::EndOfLine => condition.mode = ContentMode::Verbatim,
                    SuppressionPosition::Inline | SuppressionPosition::OwnLine => {
                        first_mode = ContentMode::Verbatim
                    }
                }
            }
        }
        writer.group(false, |writer| {
            writer.text("switch");
            if leading.has_suppression() || condition_tail.has_suppression() {
                self.format_compound_gap(
                    CompoundGap {
                        layout: GapLayout::Source,
                        ..leading
                    },
                    FlatSeparator::Empty,
                    Writer::soft_line,
                    writer,
                )?;
                self.format_expression_with_leading_comments(
                    discriminant,
                    NodeHead {
                        comments: &[],
                        content_start: disc_span.start,
                        layout: CommentRunLayout::Source,
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::ControlCondition)
                        .with_content_mode(condition.mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?;
                self.format_compound_gap(
                    CompoundGap {
                        layout: GapLayout::Source,
                        ..condition_tail
                    },
                    FlatSeparator::Space,
                    Writer::space,
                    writer,
                )?;
            } else {
                let closing = condition.closing;
                self.format_condition(condition, writer)?;
                self.format_compound_gap(
                    self.compound_gap(Span::new(closing + 1, opening)),
                    FlatSeparator::Space,
                    Writer::space,
                    writer,
                )?;
            }
            writer.text("{");
            Ok::<_, FormatError>(())
        })?;
        if first.is_none() {
            self.format_compound_gap(first_gap, FlatSeparator::Empty, Writer::hard_line, writer)?;
        } else {
            writer
                .indent(|writer| {
                    self.format_compound_gap(
                        first_gap,
                        FlatSeparator::Space,
                        Writer::hard_line,
                        writer,
                    )?;
                    let mut pending = cases
                        .next()
                        .map(|node| self.prepare_case(node, cases.peek().copied(), body.end))
                        .transpose()?;
                    if let Some(first) = pending.as_mut()
                        && first_mode == ContentMode::Verbatim
                    {
                        first.mode = first_mode;
                    }
                    while let Some(current) = pending {
                        pending = cases
                            .next()
                            .map(|node| self.prepare_case(node, cases.peek().copied(), body.end))
                            .transpose()?;
                        if let Some(next) = pending.as_mut()
                            && current.following_mode == ContentMode::Verbatim
                        {
                            next.mode = ContentMode::Verbatim;
                        }
                        let trailing = current.trailing;
                        if current.mode == ContentMode::Verbatim {
                            self.format_verbatim(current.node.span(ast), writer)?;
                        } else {
                            self.format_case(current, writer)?;
                        }
                        if pending.is_some() {
                            self.format_compound_gap(
                                trailing,
                                FlatSeparator::Space,
                                Writer::hard_line,
                                writer,
                            )?;
                        } else {
                            return Ok::<_, FormatError>(trailing);
                        }
                    }
                    Ok(self.compound_gap(Span::new(body.end, body.end)))
                })
                .and_then(|trailing| {
                    self.format_compound_gap(
                        trailing,
                        FlatSeparator::Space,
                        Writer::hard_line,
                        writer,
                    )
                })?;
        }
        writer.text("}");
        Ok(StatementBoundary::SourceEnd {
            end: span.end,
            has_terminator: false,
        })
    }

    fn protect_case_first(
        &self,
        first: Option<Stmt>,
        incoming: &mut Option<CaseIncoming>,
        nested_block: bool,
    ) {
        if matches!(incoming, Some(CaseIncoming::Statement(_))) {
            return;
        }
        if let Some(CaseIncoming::Block { block, .. }) = *incoming {
            if !nested_block {
                *incoming = Some(CaseIncoming::Statement(IncomingStatement {
                    span: block.span(self.ast),
                    mode: ContentMode::Verbatim,
                }));
            }
            return;
        }
        let Some(first) = first else {
            return;
        };
        if nested_block && let StmtData::BlockStatement(block) = self.ast.stmt_data(first) {
            let mut body = CompoundBlock::new(block, EmptyBlock::Expanded);
            self.protect_block_first(&mut body);
            if let Some(inner) = body.incoming {
                if !matches!(incoming, Some(CaseIncoming::Statement(target)) if target.span == first.span(self.ast))
                {
                    *incoming = Some(CaseIncoming::Block {
                        block,
                        incoming: inner,
                    });
                }
                return;
            }
        }
        *incoming = Some(CaseIncoming::Statement(IncomingStatement {
            span: first.span(self.ast),
            mode: ContentMode::Verbatim,
        }));
    }

    fn prepare_case(
        &self,
        node: SwitchCase,
        following: Option<SwitchCase>,
        end: u32,
    ) -> Result<PreparedCase<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let test = node.test(ast);
        let keyword_end = span.start + if test.is_some() { 4 } else { 7 };
        let mut statements = node
            .consequent(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id));
        let mut empty_prefix = None;
        let first = loop {
            let Some(statement) = statements.next() else {
                break None;
            };
            match ast.stmt_data(statement) {
                StmtData::EmptyStatement(empty) => empty_prefix = Some(empty),
                _ => break Some(statement),
            }
        };
        let adjacent_block = first
            .is_some_and(|first| matches!(ast.stmt_data(first), StmtData::BlockStatement(_)))
            && statements.all(|node| !StatementItem::Statement(node).is_nonempty(ast));
        let head_end = first.map_or(span.end, |node| node.span(ast).start);
        let tail_start = test.map_or(keyword_end, |test| test.span(ast).end);
        let colon_gap = Span::new(tail_start, head_end);
        let comments = self.comments.in_range(colon_gap);
        let Some(colon) = self.find_token(colon_gap, comments, ":") else {
            return Err(FormatError::unsupported(
                "case colon boundary",
                None,
                colon_gap,
            ));
        };
        let leading = self.compound_gap(Span::new(
            keyword_end,
            test.map_or(colon, |test| test.span(ast).start),
        ));
        let body_head = self.compound_gap(Span::new(tail_start, head_end));
        let trailing = self.compound_gap(Span::new(
            span.end,
            following.map_or(end, |node| node.span(ast).start),
        ));
        let mut prepared = PreparedCase {
            node,
            mode: ContentMode::Formatted,
            test,
            test_mode: self.content_mode(leading.comments),
            colon,
            leading,
            body_head,
            incoming: None,
            trailing,
            following_mode: ContentMode::Formatted,
            first,
            adjacent_block,
        };
        if body_head.has_suppression() {
            let before = self.comments.in_range(Span::new(tail_start, colon));
            let after = self.comments.in_range(Span::new(colon + 1, head_end));
            if test.is_some() {
                for position in self.suppression_positions(before, head_end) {
                    if matches!(position, SuppressionPosition::OwnLine) && first.is_some() {
                        self.protect_case_first(first, &mut prepared.incoming, false);
                    } else {
                        prepared.test_mode = ContentMode::Verbatim;
                    }
                }
                for (run, position) in self.suppressed_comment_runs(after, head_end) {
                    let before_empty = empty_prefix.is_some_and(|empty| {
                        run.last()
                            .is_some_and(|comment| comment.span.end <= empty.span(ast).start)
                    });
                    if matches!(position, SuppressionPosition::EndOfLine)
                        || first.is_none()
                        || before_empty && matches!(position, SuppressionPosition::Inline)
                    {
                        prepared.test_mode = ContentMode::Verbatim;
                    } else {
                        self.protect_case_first(first, &mut prepared.incoming, false);
                    }
                }
            } else {
                let default_comments = self.comments.in_range(Span::new(keyword_end, head_end));
                for (run, position) in self.suppressed_comment_runs(default_comments, head_end) {
                    if empty_prefix.is_some() {
                        self.protect_case_first(first, &mut prepared.incoming, false);
                        continue;
                    }
                    match position {
                        SuppressionPosition::EndOfLine => {
                            for &comment in run {
                                if !is_suppression_comment(ast, comment) {
                                    continue;
                                }
                                if comment.kind == CommentKind::Line
                                    && first.is_some_and(|first| {
                                        matches!(ast.stmt_data(first), StmtData::BlockStatement(_))
                                    })
                                {
                                    self.protect_case_first(first, &mut prepared.incoming, true);
                                } else {
                                    prepared.mode = ContentMode::Verbatim;
                                }
                            }
                        }
                        SuppressionPosition::Inline | SuppressionPosition::OwnLine => {
                            self.protect_case_first(first, &mut prepared.incoming, false)
                        }
                    }
                }
            }
        }
        if trailing.has_suppression() {
            for position in self.suppression_positions(trailing.comments, trailing.span.end) {
                match position {
                    SuppressionPosition::EndOfLine => prepared.mode = ContentMode::Verbatim,
                    _ if following.is_none() => prepared.mode = ContentMode::Verbatim,
                    _ => prepared.following_mode = ContentMode::Verbatim,
                }
            }
        }
        Ok(prepared)
    }

    fn format_case(
        &mut self,
        node: PreparedCase<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        if let Some(test) = node.test {
            writer.text("case");
            if node.leading.has_suppression() {
                self.format_compound_gap(
                    node.leading,
                    FlatSeparator::Space,
                    Writer::space,
                    writer,
                )?;
            } else {
                writer.space();
                self.comments.claim(node.leading.comments)?;
            }
            let tail = self.format_expression_with_leading_comments(
                test,
                NodeHead {
                    comments: if node.leading.has_suppression() {
                        &[]
                    } else {
                        node.leading.comments
                    },
                    content_start: test.span(ast).start,
                    layout: CommentRunLayout::FlatLeading(test.span(ast).start),
                    gap_end: None,
                },
                ExpressionContext::at(ExpressionPosition::Nested)
                    .with_content_mode(node.test_mode)
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )?;
            if node.body_head.has_suppression() {
                self.format_expression_tail(tail, writer)?;
            } else {
                let tail =
                    self.capture_expression_tail(Span::new(test.span(ast).end, node.colon), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(test.span(ast).end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                self.line_constraint(end, node.colon).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if last.is_some_and(|comment| comment.kind == CommentKind::Line) {
                            writer.hard_line();
                        }
                    },
                );
                writer.text(":");
            }
        } else {
            writer.text("default");
            if !node.body_head.has_suppression() {
                self.comments.claim(node.leading.comments)?;
                self.format_comment_run(
                    node.leading.comments,
                    CommentRunLayout::SourceAfter(node.leading.span.start),
                    writer,
                )?;
                let last = node.leading.comments.last();
                let end = last.map_or(node.leading.span.start, |comment| comment.span.end);
                self.line_constraint(end, node.colon).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if last.is_some_and(|comment| comment.kind == CommentKind::Line) {
                            writer.hard_line();
                        }
                    },
                );
                writer.text(":");
            }
        }
        let statements = node.node.consequent(ast);
        let first = node.first;
        let content_start = if node.body_head.has_suppression() {
            first.map_or(node.node.span(ast).end, |first| first.span(ast).start)
        } else {
            node.colon + 1
        };
        let body_span = Span::new(content_start, node.node.span(ast).end);
        let block = node.adjacent_block;
        let terminator = if node.trailing.has_suppression()
            && node.following_mode == ContentMode::Verbatim
            && self.line_breaks(node.trailing.span.start, node.trailing.span.end) == 0
        {
            statements.last().and_then(|id| {
                self.statement_source_terminator(ast.get_node_in_sub_range(id).span(ast))
            })
        } else {
            None
        };
        let mut write_body = |writer: &mut Writer<'ast>| {
            if node.body_head.has_suppression() {
                self.format_compound_gap(
                    node.body_head,
                    FlatSeparator::Space,
                    if block {
                        Writer::space
                    } else {
                        Writer::hard_line
                    },
                    writer,
                )?;
            } else if first.is_some() {
                self.opening_line_constraint(body_span).write(
                    writer,
                    FlatSeparator::Space,
                    if block {
                        Writer::space
                    } else {
                        Writer::hard_line
                    },
                );
            }
            let items = statements
                .iter()
                .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)))
                .peekable();
            self.format_statement_sequence(
                items,
                body_span,
                None,
                StatementSequenceKind::Case {
                    incoming: node.incoming,
                    terminator,
                },
                writer,
            )?;
            Ok::<_, FormatError>(())
        };
        if block {
            write_body(writer)
        } else {
            writer.indent(write_body)
        }
    }
}
