//! Basic statement bodies and their enclosing source boundaries.

use swc_next_ecma_ast::{
    Ast, BlockStatement, Comment, CommentKind, DeclData, Directive, FunctionBody, GetSpan, Span,
    Stmt, StmtData,
};

use super::Formatter;
use super::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::{ExpressionContext, ExpressionPosition, ExpressionStart};
use super::literals::StringContext;
use super::source::is_source_whitespace;
use super::suppression::{BoundaryPunctuation, ContentMode, SuppressionPosition};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum StatementSequenceKind {
    Program { has_source_items: bool },
    Block,
    Function(Option<IncomingStatement>),
}

#[derive(Clone, Copy)]
pub(super) struct IncomingStatement {
    pub(super) span: Span,
    pub(super) mode: ContentMode,
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

enum BodySource {
    Block(BlockStatement),
    Function(FunctionBody, Option<IncomingStatement>),
}

struct StatementAnnotationTail<'ast> {
    comments: &'ast [Comment],
    end: u32,
    suppression_target: Option<Span>,
}

struct PreparedStatement<'ast> {
    item: StatementItem,
    span: Span,
    mode: ContentMode,
    annotation_tail: Option<StatementAnnotationTail<'ast>>,
}

impl StatementItem {
    fn is_nonempty(self, ast: &Ast<'_>) -> bool {
        !matches!(self, Self::Statement(statement) if matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
    }

    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Directive(node) => node.span(ast),
            Self::Statement(node) => node.span(ast),
        }
    }
}

#[derive(Clone, Copy)]
enum StatementPunctuation {
    Canonical(Semicolons),
    Source,
}

#[derive(Clone, Copy)]
enum StatementBoundary {
    Semicolon { end: u32, terminator: u32 },
    SourceEnd { end: u32, has_terminator: bool },
}

impl StatementBoundary {
    fn end(self) -> u32 {
        match self {
            Self::Semicolon { end, .. } | Self::SourceEnd { end, .. } => end,
        }
    }

    fn has_terminator(self) -> bool {
        match self {
            Self::Semicolon { .. } => true,
            Self::SourceEnd { has_terminator, .. } => has_terminator,
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_statement_sequence(
        &mut self,
        mut statements: std::iter::Peekable<impl Iterator<Item = StatementItem>>,
        span: Span,
        previous_end: Option<u32>,
        kind: StatementSequenceKind,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementSequence, FormatError> {
        let ast = self.ast;
        let incoming = match kind {
            StatementSequenceKind::Function(incoming) => incoming,
            _ => None,
        };
        let mut previous = previous_end.map(|end| StatementBoundary::SourceEnd {
            end,
            has_terminator: false,
        });
        let mut first_statement = true;
        let mut pending = statements.next().map(|item| {
            self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
        });
        while let Some(prepared) = pending {
            pending = statements.next().map(|item| {
                self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
            });
            let PreparedStatement {
                item: statement,
                span: statement_span,
                mode: source_mode,
                annotation_tail,
            } = prepared;
            if !statement.is_nonempty(ast) {
                continue;
            }
            let leading = self.comments.between(Span::new(
                previous.map_or(span.start, StatementBoundary::end),
                statement_span.start,
            ))?;
            let leading_mode = self.content_mode(leading);
            let source_following = pending.as_ref().map(|next| next.span.start);
            while let Some(next) = pending.as_ref() {
                if next.item.is_nonempty(ast) {
                    break;
                }
                pending = statements.next().map(|item| {
                    self.prepare_sequence_statement(item, statements.peek().copied(), incoming)
                });
            }
            let following = pending.as_ref().map(|next| next.span.start);
            let punctuation = if annotation_tail.is_some()
                && self
                    .ast
                    .source()
                    .as_bytes()
                    .get(statement_span.end as usize - 1)
                    == Some(&b';')
            {
                StatementPunctuation::Source
            } else {
                StatementPunctuation::Canonical(self.options.semicolons)
            };
            let mode = if source_mode == ContentMode::Verbatim
                || leading_mode == ContentMode::Verbatim
                || annotation_tail
                    .as_ref()
                    .is_some_and(|tail| tail.suppression_target == Some(statement_span))
            {
                ContentMode::Verbatim
            } else if !self.has_suppression {
                ContentMode::Formatted
            } else {
                let content_end = match statement {
                    StatementItem::Directive(directive) => directive.expression(ast).span(ast).end,
                    StatementItem::Statement(node) => match ast.stmt_data(node) {
                        StmtData::ExpressionStatement(expression) => {
                            expression.expression(ast).span(ast).end
                        }
                        StmtData::ReturnStatement(node) => node
                            .argument(ast)
                            .map_or(statement_span.start + "return".len() as u32, |argument| {
                                argument.span(ast).end
                            }),
                        StmtData::ThrowStatement(node) => node.argument(ast).span(ast).end,
                        StmtData::DebuggerStatement(_) => {
                            statement_span.start + "debugger".len() as u32
                        }
                        _ => statement_span.end,
                    },
                };
                self.trailing_content_mode(
                    Span::new(content_end, source_following.unwrap_or(span.end)),
                    BoundaryPunctuation::Semicolon,
                    source_following.is_none(),
                )
            };
            let leading = self.comments.take(leading.len());
            let mut expression_start = ExpressionStart::StatementPending;
            if first_statement
                && matches!(kind, StatementSequenceKind::Program { .. })
                && self.has_file_directives
                && leading
                    .iter()
                    .any(|&comment| self.comment_is_file_directive(comment))
                && leading
                    .iter()
                    .any(|&comment| self.comment_has_target_annotation(comment))
            {
                // Program start already separates the first statement. Keep its header intact.
                expression_start = ExpressionStart::StatementGuarded;
            }
            if let Some(boundary) = previous {
                if boundary.has_terminator() {
                    expression_start = ExpressionStart::StatementGuarded;
                }
                let next_start = leading
                    .first()
                    .map_or(statement_span.start, |comment| comment.span.start);
                let constraint = self.line_constraint(boundary.end(), next_start);
                if constraint == LineConstraint::SameLine
                    && self.options.semicolons == Semicolons::AsNeeded
                    && !boundary.has_terminator()
                    && statement.is_nonempty(ast)
                {
                    writer.text(";");
                    expression_start = ExpressionStart::StatementGuarded;
                }
                let separator = if matches!(boundary, StatementBoundary::Semicolon { end, terminator } if end == terminator + 1)
                {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                constraint.write(writer, separator, Writer::hard_line);
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
                gap_end: leading.last().map(|_| statement_span.start),
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
                    self.finish_statement(
                        statement_span,
                        expression.span(ast).end,
                        None,
                        punctuation,
                        writer,
                    )?
                }
                StatementItem::Statement(statement) => self.format_statement(
                    statement,
                    expression_start,
                    mode,
                    head,
                    punctuation,
                    writer,
                )?,
            };
            first_statement = false;
            let trailing_start = if let Some(tail) = annotation_tail {
                if let Some(next) = pending.as_mut()
                    && tail.suppression_target == Some(next.span)
                {
                    next.mode = ContentMode::Verbatim;
                }
                self.comments.claim(tail.comments)?;
                self.format_comment_run(
                    tail.comments,
                    CommentRunLayout::SourceAfter(statement_span.end),
                    writer,
                )?;
                tail.end
            } else {
                boundary.end()
            };
            let end = self.format_trailing_comments(
                Span::new(trailing_start, following.unwrap_or(span.end)),
                statement_span.end,
                following.is_some(),
                writer,
            )?;
            previous = Some(match boundary {
                StatementBoundary::Semicolon { terminator, .. } => {
                    StatementBoundary::Semicolon { end, terminator }
                }
                StatementBoundary::SourceEnd { has_terminator, .. } => {
                    StatementBoundary::SourceEnd {
                        end,
                        has_terminator,
                    }
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
            (None, StatementSequenceKind::Block | StatementSequenceKind::Function(_)) => {
                CommentRunLayout::Lines
            }
            (
                None,
                StatementSequenceKind::Program {
                    has_source_items: true,
                    ..
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

    fn prepare_sequence_statement(
        &self,
        item: StatementItem,
        following: Option<StatementItem>,
        incoming: Option<IncomingStatement>,
    ) -> PreparedStatement<'ast> {
        let span = item.span(self.ast);
        let annotation_tail = if (self.has_target_annotations || self.has_suppression)
            && item.is_nonempty(self.ast)
        {
            self.prepare_statement_annotation_tail(span, following)
        } else {
            None
        };
        PreparedStatement {
            item,
            span,
            mode: incoming
                .filter(|target| target.span == span)
                .map_or(ContentMode::Formatted, |target| target.mode),
            annotation_tail,
        }
    }

    fn prepare_statement_annotation_tail(
        &self,
        statement_span: Span,
        following: Option<StatementItem>,
    ) -> Option<StatementAnnotationTail<'ast>> {
        let following = following?;
        let target = if following.is_nonempty(self.ast) {
            if self
                .ast
                .source()
                .as_bytes()
                .get(statement_span.end as usize - 1)
                != Some(&b';')
            {
                return None;
            }
            None
        } else {
            if !self.has_suppression {
                return None;
            }
            Some(statement_span)
        };
        let span = Span::new(statement_span.end, following.span(self.ast).start);
        let comments = self.comments.in_range(span);
        let mut end = span.start;
        let mut count = 0;
        for &comment in comments {
            let gap = self.source(Span::new(end, comment.span.start));
            if target.is_some() && self.line_breaks(end, comment.span.start) != 0
                || !gap.chars().all(is_source_whitespace)
            {
                break;
            }
            end = comment.span.end;
            count += 1;
            if target.is_some() && comment.kind == CommentKind::Line {
                break;
            }
        }
        let comments = &comments[..count];
        let previous_ignore = target.is_some()
            || matches!(
                self.suppression_position(comments, span.end),
                Some(SuppressionPosition::EndOfLine)
            );
        if target.is_none() {
            // Block annotations belong to the following head, including its ASI guard.
            if comments
                .iter()
                .all(|comment| comment.kind == CommentKind::Block)
                && !previous_ignore
            {
                return None;
            }
            let optimizer = comments
                .iter()
                .any(|&comment| self.comment_is_optimizer_annotation(comment));
            let cast = matches!(following, StatementItem::Statement(next)
                if matches!(self.ast.stmt_data(next), StmtData::ExpressionStatement(expression)
                    if self.parenthesis_has_type_cast(expression.expression(self.ast).span(self.ast).start)));
            if !optimizer && !cast && !previous_ignore {
                return None;
            }
        }

        let binding = self.comment_run_binding(comments.iter().copied(), |comment| {
            let semantics = self.comment_semantics(comment);
            if semantics.suppression && previous_ignore {
                CommentBinding::BoundTo(statement_span)
            } else if semantics.has_target_annotation() {
                CommentBinding::SourceBound
            } else {
                CommentBinding::Unbound
            }
        });
        if binding.allows_relocation(None) {
            return None;
        }
        let suppression_target = if self.content_mode(comments) == ContentMode::Verbatim {
            Some(if previous_ignore {
                statement_span
            } else {
                following.span(self.ast)
            })
        } else {
            None
        };
        Some(StatementAnnotationTail {
            comments,
            end,
            suppression_target,
        })
    }

    fn format_statement(
        &mut self,
        statement: Stmt,
        start: ExpressionStart,
        mode: ContentMode,
        head: NodeHead<'ast>,
        punctuation: StatementPunctuation,
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
                punctuation,
                writer,
            );
        }
        self.format_node_head(head, start, writer)?;
        match data {
            StmtData::EmptyStatement(node) => {
                writer.text(";");
                Ok(StatementBoundary::Semicolon {
                    end: node.span(self.ast).end,
                    terminator: node.span(self.ast).end - 1,
                })
            }
            StmtData::DebuggerStatement(node) => {
                writer.text("debugger");
                self.finish_statement(
                    statement.span(self.ast),
                    node.span(self.ast).start + "debugger".len() as u32,
                    None,
                    punctuation,
                    writer,
                )
            }
            StmtData::ReturnStatement(node) if mode == ContentMode::Formatted => {
                let span = node.span(self.ast);
                let keyword_end = span.start + "return".len() as u32;
                let Some(argument) = node.argument(self.ast) else {
                    writer.text("return");
                    return self.finish_statement(span, keyword_end, None, punctuation, writer);
                };
                let boundary = self.prepare_restricted_argument(keyword_end, argument)?;
                writer.text("return");
                let tail = self.format_restricted_argument(boundary, writer)?;
                self.finish_statement(span, argument.span(self.ast).end, tail, punctuation, writer)
            }
            StmtData::ThrowStatement(node) if mode == ContentMode::Formatted => {
                let span = node.span(self.ast);
                let argument = node.argument(self.ast);
                let boundary =
                    self.prepare_restricted_argument(span.start + "throw".len() as u32, argument)?;
                writer.text("throw");
                let tail = self.format_restricted_argument(boundary, writer)?;
                self.finish_statement(span, argument.span(self.ast).end, tail, punctuation, writer)
            }
            StmtData::Declaration(declaration) if mode == ContentMode::Formatted => {
                let DeclData::Function(function) = self.ast.decl_data(declaration) else {
                    return Err(self.unsupported("declaration", declaration));
                };
                self.format_function(function, writer)?;
                Ok(StatementBoundary::SourceEnd {
                    end: function.span(self.ast).end,
                    has_terminator: false,
                })
            }
            StmtData::BlockStatement(block) => {
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(block.span(self.ast), writer)?;
                } else {
                    self.format_block(block, writer)?;
                }
                Ok(StatementBoundary::SourceEnd {
                    end: block.span(self.ast).end,
                    has_terminator: false,
                })
            }
            _ if mode == ContentMode::Verbatim => {
                let span = statement.span(self.ast);
                self.format_verbatim(span, writer)?;
                Ok(StatementBoundary::SourceEnd {
                    end: span.end,
                    has_terminator: self.source(span).ends_with(';'),
                })
            }
            _ => Err(self.unsupported("statement", statement)),
        }
    }

    fn finish_statement(
        &mut self,
        span: Span,
        content_end: u32,
        tail: Option<ExpressionTail<'ast>>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<StatementBoundary, FormatError> {
        let source_terminator =
            self.ast.source().as_bytes().get(span.end as usize - 1) == Some(&b';');
        if source_terminator && matches!(punctuation, StatementPunctuation::Source) {
            let tail_start = tail.as_ref().map_or(content_end, |tail| tail.source_end);
            let source_comments = self.comments.in_range(Span::new(tail_start, span.end));
            let printed_end = tail
                .as_ref()
                .and_then(|tail| tail.comments.last())
                .map_or(content_end, |comment| comment.span.end);
            self.format_expression_tail(tail, writer)?;
            let comments = self.comments.between(Span::new(content_end, span.end))?;
            self.comments.claim(comments)?;
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(printed_end), writer)?;
            let last = comments.last().or_else(|| source_comments.last());
            let end = last.map_or(content_end, |comment| comment.span.end);
            self.line_constraint(end, span.end - 1)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if self.line_breaks(end, span.end - 1) != 0
                        || last.is_some_and(|comment| comment.kind == CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                });
            writer.text(";");
            return Ok(if self.line_breaks(content_end, span.end - 1) != 0 {
                StatementBoundary::Semicolon {
                    end: span.end,
                    terminator: span.end - 1,
                }
            } else {
                StatementBoundary::SourceEnd {
                    end: span.end,
                    has_terminator: true,
                }
            });
        }
        if matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::Always)
        ) {
            writer.text(";");
        }
        self.format_expression_tail(tail, writer)?;
        Ok(StatementBoundary::SourceEnd {
            end: content_end,
            has_terminator: matches!(
                punctuation,
                StatementPunctuation::Canonical(Semicolons::Always)
            ),
        })
    }

    fn format_block(
        &mut self,
        block: BlockStatement,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_braced_body(BodySource::Block(block), ContentMode::Formatted, writer)
    }

    pub(super) fn format_function_body(
        &mut self,
        body: FunctionBody,
        mode: ContentMode,
        incoming: Option<IncomingStatement>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_braced_body(BodySource::Function(body, incoming), mode, writer)
    }

    fn format_braced_body(
        &mut self,
        source: BodySource,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let (span, directives, body, kind) = match source {
            BodySource::Block(block) => (
                block.span(ast),
                None,
                block.body(ast),
                StatementSequenceKind::Block,
            ),
            BodySource::Function(function, incoming) => (
                function.span(ast),
                Some(function.directives(ast)),
                function.body(ast),
                StatementSequenceKind::Function(incoming),
            ),
        };
        let function = matches!(kind, StatementSequenceKind::Function(_));
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        let content = Span::new(span.start + 1, span.end - 1);
        let has_items = directives.is_some_and(|directives| !directives.is_empty())
            || body
                .iter()
                .any(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)).is_nonempty(ast));
        let directives = directives
            .into_iter()
            .flat_map(|directives| directives.iter())
            .map(|id| StatementItem::Directive(ast.get_node_in_sub_range(id)));
        let body = body
            .iter()
            .map(|id| StatementItem::Statement(ast.get_node_in_sub_range(id)));
        let items = directives.chain(body).peekable();
        let empty = if has_items {
            false
        } else {
            let dangling = self.comments.in_range(content);
            if function && self.content_mode(dangling) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            dangling.is_empty()
        };
        let opening = self.opening_line_constraint(content);
        writer.text("{");
        if empty {
            opening.write(writer, FlatSeparator::Empty, |writer| {
                if !function {
                    writer.hard_line();
                }
            });
        } else {
            let sequence = writer.indent(|writer| {
                opening.write(writer, FlatSeparator::Space, Writer::hard_line);
                self.format_statement_sequence(items, content, None, kind, writer)
            })?;
            self.line_constraint(sequence.end, content.end).write(
                writer,
                FlatSeparator::Space,
                Writer::hard_line,
            );
        }
        writer.text("}");
        Ok(())
    }
}
