//! Parent-owned conditions, controlled bodies, and their following keywords.

use swc_next_ecma_ast::{
    Comment, CommentKind, Expr, ExprData, GetSpan, IfStatement, Span, Stmt, StmtData, UnaryOperator,
};

use super::{
    Formatter, PrintedStatement, StatementBoundary, StatementItem, StatementPunctuation,
    StatementTailRelation,
};
use crate::formatter::comments::{CommentBinding, CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::{
    ExpressionContext, ExpressionPosition, ExpressionStart, TailOwner,
};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
enum ControlledBodyRole {
    Consequent,
    Alternate,
}

#[derive(Clone, Copy)]
enum ControlledBodyShape {
    Empty,
    Block,
    ElseIf,
    Single,
}

#[derive(Clone, Copy)]
enum ControlledBodyEnd {
    Keyword(Span),
    Enclosing(u32),
}

impl ControlledBodyEnd {
    fn limit(self) -> u32 {
        match self {
            Self::Keyword(keyword) => keyword.start,
            Self::Enclosing(end) => end,
        }
    }

    fn has_following(self) -> bool {
        matches!(self, Self::Keyword(_))
    }
}

struct ControlledBody<'ast> {
    statement: Stmt,
    span: Span,
    head_end: u32,
    leading: &'ast [Comment],
    shape: ControlledBodyShape,
    mode: ContentMode,
    punctuation: StatementPunctuation,
    end: ControlledBodyEnd,
    annotation_tail: Option<ControlledBodyTail<'ast>>,
}

/// The statement may consume a prefix while preserving an ignored source terminator.
struct ControlledBodyTail<'ast> {
    tail: ExpressionTail<'ast>,
    binding: CommentBinding,
}

impl ControlledBody<'_> {
    fn with_enclosing_punctuation(mut self, punctuation: StatementPunctuation) -> Self {
        if matches!(
            punctuation,
            StatementPunctuation::Canonical(Semicolons::Always)
                | StatementPunctuation::Source(_)
                | StatementPunctuation::SourcePacket
        ) {
            self.punctuation = punctuation;
        }
        self
    }
}

struct Condition<'ast> {
    expression: Expr,
    keyword_end: u32,
    opening: u32,
    closing: u32,
    before_opening: &'ast [Comment],
    leading: &'ast [Comment],
    mode: ContentMode,
    relocate_before_opening: bool,
    inline: bool,
}

struct ElseBoundary {
    keyword: Span,
    preceding_mode: ContentMode,
    enclosing_mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_if_statement(
        &mut self,
        node: IfStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let consequent = node.consequent(ast);
        let alternate = node.alternate(ast);
        let boundary = alternate
            .map(|alternate| self.prepare_else_boundary(consequent, alternate))
            .transpose()?;
        if boundary
            .as_ref()
            .is_some_and(|boundary| boundary.enclosing_mode == ContentMode::Verbatim)
        {
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::SourceEnd {
                end: span.end,
                has_terminator: self.source(span).ends_with(';'),
            }
            .into());
        }
        let condition = self.prepare_condition(
            node.test(ast),
            span.start + "if".len() as u32,
            consequent.span(ast).start,
        )?;
        let body = self.prepare_controlled_body(
            consequent,
            condition.closing + 1,
            boundary
                .as_ref()
                .map_or(ControlledBodyEnd::Enclosing(span.end), |boundary| {
                    ControlledBodyEnd::Keyword(boundary.keyword)
                }),
            ControlledBodyRole::Consequent,
            boundary
                .as_ref()
                .map_or(ContentMode::Formatted, |boundary| boundary.preceding_mode),
        );
        let body = if boundary.is_none() {
            body.with_enclosing_punctuation(punctuation)
        } else {
            body
        };
        let consequent_is_block = matches!(body.shape, ControlledBodyShape::Block);
        let printed = writer.group(false, |writer| {
            writer.text("if");
            self.format_condition(condition, writer)?;
            self.format_controlled_body(body, writer)
        })?;
        let (Some(alternate), Some(boundary)) = (alternate, boundary) else {
            return Ok(printed);
        };
        self.format_else_boundary(&boundary, printed, consequent_is_block, writer)?;
        let body = self
            .prepare_controlled_body(
                alternate,
                boundary.keyword.end,
                ControlledBodyEnd::Enclosing(span.end),
                ControlledBodyRole::Alternate,
                ContentMode::Formatted,
            )
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| self.format_controlled_body(body, writer))
    }

    fn prepare_condition(
        &self,
        expression: Expr,
        keyword_end: u32,
        body_start: u32,
    ) -> Result<Condition<'ast>, FormatError> {
        let span = expression.span(self.ast);
        let head = Span::new(keyword_end, span.start);
        let leading = self.comments.in_range(head);
        let Some(opening) = self.find_token(head, leading, "(") else {
            return Err(FormatError::unsupported(
                "condition opening boundary",
                None,
                head,
            ));
        };
        let tail = Span::new(span.end, body_start);
        let trailing = self.comments.in_range(tail);
        let Some(closing) = self.find_token(tail, trailing, ")") else {
            return Err(FormatError::unsupported(
                "condition closing boundary",
                None,
                tail,
            ));
        };
        let split = leading.partition_point(|comment| comment.span.end <= opening);
        let before_opening = &leading[..split];
        let relocate_before_opening = self
            .comment_binding(before_opening, head, Some(span))
            .allows_relocation(Some(span));
        let leading = if relocate_before_opening {
            leading
        } else {
            &leading[split..]
        };
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            self.trailing_content_mode(Span::new(span.end, closing), true)
        };
        let inline = before_opening.is_empty()
            && leading.is_empty()
            && trailing.is_empty()
            && self.condition_is_inline(expression);
        Ok(Condition {
            expression,
            keyword_end,
            opening,
            closing,
            before_opening,
            leading,
            mode,
            relocate_before_opening,
            inline,
        })
    }

    fn condition_is_inline(&self, expression: Expr) -> bool {
        if !self.comments.in_range(expression.span(self.ast)).is_empty() {
            return false;
        }
        let mut expression = self.expression_content(expression).expression;
        let mut negations = 0;
        while negations < 2 {
            let ExprData::UnaryExpression(unary) = self.ast.expr_data(expression) else {
                break;
            };
            if unary.operator(self.ast) != UnaryOperator::LogicalNot {
                break;
            }
            expression = self.expression_content(unary.argument(self.ast)).expression;
            negations += 1;
        }
        negations != 0
            && matches!(
                self.ast.expr_data(expression),
                ExprData::LogicalExpression(_)
            )
    }

    fn format_condition(
        &mut self,
        condition: Condition<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let Condition {
            expression,
            keyword_end,
            opening,
            closing,
            before_opening,
            leading,
            mode,
            relocate_before_opening,
            inline,
        } = condition;
        if relocate_before_opening {
            self.line_constraint(keyword_end, opening).write(
                writer,
                FlatSeparator::Space,
                Writer::space,
            );
        } else {
            let first = before_opening
                .first()
                .map_or(opening, |comment| comment.span.start);
            self.line_constraint(keyword_end, first).write(
                writer,
                FlatSeparator::Space,
                Writer::space,
            );
            self.comments.claim(before_opening)?;
            self.format_comment_run(
                before_opening,
                CommentRunLayout::FlatLeading(opening),
                writer,
            )?;
        }
        writer.text("(");
        let span = expression.span(self.ast);
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start.max(opening + 1));
        let opening_constraint = self.line_constraint(opening + 1, first);
        writer.group(false, |writer| {
            let mut write_contents = |writer: &mut Writer<'ast>| {
                if !inline {
                    opening_constraint.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
                self.comments.claim(leading)?;
                let tail = self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments: leading,
                        content_start: span.start,
                        layout: CommentRunLayout::GroupedLeading(span.start),
                        gap_end: None,
                    },
                    ExpressionContext::at(ExpressionPosition::ControlCondition)
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?;
                let tail = self.capture_expression_tail(Span::new(span.end, closing), tail)?;
                let last = tail.as_ref().and_then(|tail| tail.comments.last());
                let end = last.map_or(span.end, |comment| comment.span.end);
                self.format_expression_tail(tail, writer)?;
                Ok::<_, FormatError>((
                    end,
                    last.is_some_and(|comment| comment.kind == CommentKind::Line),
                ))
            };
            let (end, line_comment) = if inline {
                write_contents(writer)?
            } else {
                writer.indent(write_contents)?
            };
            if !inline {
                self.line_constraint(end, closing)
                    .write(writer, FlatSeparator::Empty, |writer| {
                        if line_comment {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                    });
            }
            writer.text(")");
            Ok(())
        })
    }

    fn prepare_controlled_body(
        &self,
        statement: Stmt,
        head_end: u32,
        end: ControlledBodyEnd,
        role: ControlledBodyRole,
        trailing_mode: ContentMode,
    ) -> ControlledBody<'ast> {
        let span = statement.span(self.ast);
        let leading = self.comments.in_range(Span::new(head_end, span.start));
        let data = self.ast.stmt_data(statement);
        let shape = match data {
            StmtData::EmptyStatement(_) => ControlledBodyShape::Empty,
            StmtData::BlockStatement(_) => ControlledBodyShape::Block,
            StmtData::IfStatement(_) if matches!(role, ControlledBodyRole::Alternate) => {
                ControlledBodyShape::ElseIf
            }
            _ => ControlledBodyShape::Single,
        };
        let needs_source_tail = self.has_suppression
            || self.has_target_annotations
            || !self.diagnostic_lines.is_empty();
        let needs_comment_tail =
            !matches!(shape, ControlledBodyShape::Block) && !self.ast.comments().is_empty();
        let content_end = if needs_source_tail || needs_comment_tail {
            match data {
                StmtData::ExpressionStatement(node) => node.expression(self.ast).span(self.ast).end,
                StmtData::ReturnStatement(node) => node
                    .argument(self.ast)
                    .map_or(span.start + "return".len() as u32, |argument| {
                        argument.span(self.ast).end
                    }),
                StmtData::ThrowStatement(node) => node.argument(self.ast).span(self.ast).end,
                StmtData::DebuggerStatement(_) => span.start + "debugger".len() as u32,
                StmtData::Declaration(declaration) => self.declaration_content_end(declaration),
                _ => span.end,
            }
        } else {
            span.end
        };
        let trailing_limit = end.limit();
        let leading_mode = self.content_mode(leading);
        let mode = if leading_mode == ContentMode::Verbatim {
            leading_mode
        } else {
            trailing_mode
        };
        let semicolons = if end.has_following()
            && self.options.semicolons == Semicolons::AsNeeded
            && self.line_constraint(content_end, trailing_limit) == LineConstraint::SameLine
        {
            // An expression body before `else` still needs a terminator on a covered source line.
            Semicolons::Always
        } else {
            self.options.semicolons
        };
        let punctuation = StatementPunctuation::Canonical(semicolons);
        let annotation_tail = if needs_comment_tail || self.has_suppression {
            self.prepare_controlled_body_tail(content_end, end, shape)
        } else {
            None
        };
        ControlledBody {
            statement,
            span,
            head_end,
            leading,
            shape,
            mode,
            punctuation,
            end,
            annotation_tail,
        }
    }

    /// A trailing source run keeps the same owner across a moved semicolon.
    /// Ordinary runs can follow the compound group; bound runs keep their
    /// original source boundary and the keyword consumes its own pending tail.
    fn prepare_controlled_body_tail(
        &self,
        content_end: u32,
        end: ControlledBodyEnd,
        shape: ControlledBodyShape,
    ) -> Option<ControlledBodyTail<'ast>> {
        let limit = match end {
            ControlledBodyEnd::Keyword(keyword) => keyword.start,
            ControlledBodyEnd::Enclosing(_) => self.ast.source().len() as u32,
        };
        let comments = self.comments.in_range(Span::new(content_end, limit));
        let mut previous = content_end;
        let mut count = 0;
        for &comment in comments {
            let gap = self.source(Span::new(previous, comment.span.start));
            if !gap.chars().all(|character| {
                character == ';'
                    || is_source_whitespace(character)
                        && !matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
            }) || self.line_breaks(comment.span.start, comment.span.end) != 0
            {
                break;
            }
            previous = comment.span.end;
            count += 1;
            if comment.kind == CommentKind::Line {
                break;
            }
        }
        let comments = &comments[..count];
        let last = comments.last()?;
        let binding = self.comment_binding(comments, Span::new(content_end, last.span.end), None);
        let comments = if binding.allows_relocation(None) {
            comments
        } else {
            let count = comments.partition_point(|comment| comment.span.start < end.limit());
            &comments[..count]
        };
        if comments.is_empty()
            || matches!(shape, ControlledBodyShape::Block)
                && self.content_mode(comments) != ContentMode::Verbatim
        {
            return None;
        }
        Some(ControlledBodyTail {
            tail: ExpressionTail {
                source_end: content_end,
                comments,
            },
            binding,
        })
    }

    fn format_controlled_body(
        &mut self,
        body: ControlledBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ControlledBody {
            statement,
            span,
            head_end,
            leading,
            shape,
            mode,
            punctuation,
            end,
            annotation_tail,
        } = body;
        let trailing_limit = end.limit();
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start);
        let constraint = self.line_constraint(head_end, first);
        let leading_line = leading.first().is_some_and(|comment| {
            self.line_breaks(head_end, comment.span.start) != 0
                || self.line_breaks(comment.span.start, comment.span.end) != 0
        });
        let adjacent = matches!(
            shape,
            ControlledBodyShape::Block | ControlledBodyShape::ElseIf
        );
        let empty = matches!(shape, ControlledBodyShape::Empty);
        let write_body = |writer: &mut Writer<'ast>| {
            let flat = if empty && leading.is_empty() {
                FlatSeparator::Empty
            } else {
                FlatSeparator::Space
            };
            constraint.write(writer, flat, |writer| {
                if leading_line {
                    writer.hard_line();
                } else if empty && leading.is_empty() {
                } else if adjacent {
                    writer.space();
                } else {
                    writer.line();
                }
            });
            self.comments.claim(leading)?;
            let mut printed = self.format_statement(
                statement,
                ExpressionStart::StatementGuarded,
                mode,
                NodeHead {
                    comments: leading,
                    content_start: span.start,
                    layout: if matches!(shape, ControlledBodyShape::Block) {
                        CommentRunLayout::Source
                    } else {
                        CommentRunLayout::Leading(span.start)
                    },
                    gap_end: if matches!(shape, ControlledBodyShape::Block) {
                        leading.last().map(|_| span.start)
                    } else {
                        None
                    },
                },
                punctuation,
                writer,
            )?;
            if let Some(tail) = annotation_tail
                && let Some(last) = tail.tail.comments.last()
                && last.span.end > printed.boundary.end()
            {
                printed.pending = self.capture_expression_tail(
                    Span::new(
                        tail.tail.source_end.max(printed.boundary.end()),
                        last.span.end,
                    ),
                    printed.pending,
                )?;
                if !tail.binding.allows_relocation(None) {
                    printed = self.flush_statement_tail(printed, writer)?.into();
                }
            }
            if end.has_following() {
                printed = self.flush_statement_tail(printed, writer)?.into();
            }
            if printed.pending.is_some() {
                return Ok(printed);
            }
            let end = self.format_trailing_comments(
                Span::new(printed.boundary.end(), trailing_limit),
                span.end,
                end.has_following(),
                writer,
            )?;
            printed.boundary = printed.boundary.with_end(end);
            Ok(printed)
        };
        if matches!(shape, ControlledBodyShape::Block) || adjacent && !leading_line || empty {
            write_body(writer)
        } else {
            writer.indent(write_body)
        }
    }

    fn prepare_else_boundary(
        &self,
        consequent: Stmt,
        alternate: Stmt,
    ) -> Result<ElseBoundary, FormatError> {
        let gap = Span::new(
            consequent.span(self.ast).end,
            alternate.span(self.ast).start,
        );
        let comments = self.comments.in_range(gap);
        let Some(start) = self.find_token(gap, comments, "else") else {
            return Err(FormatError::unsupported("else boundary", None, gap));
        };
        let (preceding_mode, enclosing_mode) = match self.prepare_statement_suppression_tail(
            StatementItem::Statement(consequent),
            start,
            StatementTailRelation::Keyword,
        ) {
            Some(tail) => (tail.modes.current, tail.modes.enclosing),
            None => (ContentMode::Formatted, ContentMode::Formatted),
        };
        Ok(ElseBoundary {
            keyword: Span::new(start, start + "else".len() as u32),
            preceding_mode,
            enclosing_mode,
        })
    }

    fn format_else_boundary(
        &mut self,
        boundary: &ElseBoundary,
        printed: PrintedStatement<'ast>,
        consequent_is_block: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let printed = self.flush_statement_tail(printed, writer)?;
        let span = Span::new(printed.end(), boundary.keyword.start);
        let comments = self.comments.between(span)?;
        let first = comments
            .first()
            .map_or(span.end, |comment| comment.span.start);
        let binding = self.comment_binding(comments, span, None);
        self.line_constraint(span.start, first)
            .write(writer, FlatSeparator::Space, |writer| {
                if !binding.allows_relocation(None) && !comments.is_empty() {
                    self.comment_gap(span.start, first, writer);
                } else if consequent_is_block {
                    if !comments.is_empty() && self.line_breaks(span.start, first) != 0 {
                        writer.hard_line();
                        if self.line_breaks(span.start, first) == 2 {
                            writer.hard_line();
                        }
                    } else {
                        writer.space();
                    }
                } else {
                    writer.hard_line();
                    if !comments.is_empty() && self.line_breaks(span.start, first) == 2 {
                        writer.hard_line();
                    }
                }
            });
        self.comments.claim(comments)?;
        self.format_comment_run(comments, CommentRunLayout::Source, writer)?;
        if let Some(last) = comments.last() {
            self.line_constraint(last.span.end, boundary.keyword.start)
                .write(writer, FlatSeparator::Space, |writer| {
                    if last.kind == CommentKind::Line
                        || self.line_breaks(last.span.end, boundary.keyword.start) != 0
                    {
                        writer.hard_line();
                    } else {
                        writer.space();
                    }
                });
        }
        writer.text("else");
        Ok(())
    }
}
