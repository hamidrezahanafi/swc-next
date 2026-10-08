//! Real for-header owners and the independent punctuation between them.

use swc_next_ecma_ast::{
    AssignmentTargetData, BindingPattern, Comment, Expr, ForInStatement, ForOfStatement,
    ForStatement, ForStatementInitData, ForStatementLeftData, GetSpan, Span, Stmt,
};

use super::{
    ControlledBodyEnd, ControlledBodyRole, Formatter, PrintedStatement, StatementPunctuation,
};
use crate::formatter::comments::{CommentRunLayout, NodeHead};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::targets::SourceTarget;
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition, TailOwner};
use crate::formatter::suppression::{ContentMode, DelimitedFollowing, SuppressionPosition};
use crate::formatter::variables::VariableContext;
use crate::{FormatError, document::Writer};

pub(in crate::formatter::statements) enum IterationStatement {
    In(ForInStatement),
    Of(ForOfStatement),
}

struct HeaderNode {
    span: Span,
    data: HeaderData,
}

enum HeaderData {
    Expression {
        node: Expr,
        context: ExpressionContext,
    },
    Variable {
        node: swc_next_ecma_ast::VariableDeclaration,
        context: VariableContext,
    },
    Target {
        node: SourceTarget,
        context: ExpressionContext,
    },
    Pattern(BindingPattern),
}

#[derive(Clone, Copy)]
struct HeaderToken {
    span: Span,
    text: &'static str,
    space: bool,
}

struct HeaderSyntax {
    keyword_end: u32,
    closing: Span,
    tokens: [Option<HeaderToken>; 4],
    classic: bool,
}

#[derive(Clone, Copy)]
enum HeaderRelation {
    First,
    Items(Span),
    Body(Span),
}

struct HeaderGap<'ast> {
    span: Span,
    tokens: [Option<HeaderToken>; 3],
    leading: &'ast [Comment],
    source: bool,
    preceding: ContentMode,
    following: ContentMode,
}

enum HeaderGapOutput<'ast> {
    Canonical {
        start: u32,
        leading: &'ast [Comment],
    },
    Source {
        following: LineConstraint,
    },
}

struct PreparedHeaderItem<'ast> {
    node: HeaderNode,
    before: HeaderGap<'ast>,
    mode: ContentMode,
}

struct PreparedHeader<'ast> {
    items: [Option<PreparedHeaderItem<'ast>>; 3],
    trailing: HeaderGap<'ast>,
    closing: Span,
    classic: bool,
    empty: bool,
    body_mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter::statements) fn format_for_statement(
        &mut self,
        node: ForStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let body = node.body(ast);
        let body_start = body.span(ast).start;
        let init = node.init(ast).map(|init| {
            let span = init.span(ast);
            let data = match ast.for_statement_init_data(init) {
                ForStatementInitData::VariableDeclaration(node) => HeaderData::Variable {
                    node,
                    context: VariableContext::ForInitializer,
                },
                ForStatementInitData::Expr(node) => HeaderData::Expression {
                    node,
                    context: self.for_initializer_context(node),
                },
            };
            HeaderNode { span, data }
        });
        let test = node
            .test(ast)
            .map(|node| self.for_expression_node(node, ExpressionPosition::ForHead));
        let update = node
            .update(ast)
            .map(|node| self.for_expression_node(node, ExpressionPosition::ForHead));
        let keyword_end = node.span(ast).start + 3;
        let token = |span: Span, text: &'static str| {
            self.find_token(span, self.comments.in_range(span), text)
                .map(|start| HeaderToken {
                    span: Span::new(start, start + 1),
                    text,
                    space: false,
                })
                .ok_or_else(|| self.unsupported("for header boundary", node))
        };
        let first = token(
            Span::new(
                init.as_ref().map_or(keyword_end, |node| node.span.end),
                test.as_ref()
                    .or(update.as_ref())
                    .map_or(body_start, |node| node.span.start),
            ),
            ";",
        )?;
        let second = token(
            Span::new(
                test.as_ref().map_or(first.span.end, |node| node.span.end),
                update.as_ref().map_or(body_start, |node| node.span.start),
            ),
            ";",
        )?;
        let closing = token(
            Span::new(
                update
                    .as_ref()
                    .map_or(second.span.end, |node| node.span.end),
                body_start,
            ),
            ")",
        )?;
        let opening = token(
            Span::new(
                keyword_end,
                init.as_ref()
                    .map_or(first.span.start, |node| node.span.start),
            ),
            "(",
        )?;
        let header = self.prepare_for_header(
            HeaderSyntax {
                keyword_end,
                closing: closing.span,
                tokens: [Some(opening), Some(first), Some(second), None],
                classic: true,
            },
            [init, test, update],
            body,
        );
        self.format_for_with_body(header, body, node.span(ast).end, punctuation, writer)
    }

    pub(in crate::formatter::statements) fn format_iteration_statement(
        &mut self,
        statement: IterationStatement,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let (span, left, right, body, of, awaited) = match statement {
            IterationStatement::In(node) => (
                node.span(ast),
                node.left(ast),
                node.right(ast),
                node.body(ast),
                false,
                false,
            ),
            IterationStatement::Of(node) => (
                node.span(ast),
                node.left(ast),
                node.right(ast),
                node.body(ast),
                true,
                node.r#await(ast),
            ),
        };
        let keyword_end = span.start + 3;
        let left_span = left.span(ast);
        let right_span = right.span(ast);
        let opening_gap = Span::new(keyword_end, left_span.start);
        let opening_start = self
            .find_token(opening_gap, self.comments.in_range(opening_gap), "(")
            .ok_or_else(|| self.unsupported("iteration opening boundary", left))?;
        let opening = HeaderToken {
            span: Span::new(opening_start, opening_start + 1),
            text: "(",
            space: true,
        };
        let operator_gap = Span::new(left_span.end, right_span.start);
        let operator_text = if of { "of" } else { "in" };
        let operator_start = self
            .find_token(
                operator_gap,
                self.comments.in_range(operator_gap),
                operator_text,
            )
            .ok_or_else(|| self.unsupported("iteration operator boundary", left))?;
        let operator = HeaderToken {
            span: Span::new(operator_start, operator_start + 2),
            text: operator_text,
            space: true,
        };
        let closing_gap = Span::new(right_span.end, body.span(ast).start);
        let closing_start = self
            .find_token(closing_gap, self.comments.in_range(closing_gap), ")")
            .ok_or_else(|| self.unsupported("iteration closing boundary", right))?;
        let closing = Span::new(closing_start, closing_start + 1);
        let await_token = if awaited {
            let start = self
                .find_token(opening_gap, self.comments.in_range(opening_gap), "await")
                .ok_or_else(|| self.unsupported("for-await keyword boundary", left))?;
            Some(HeaderToken {
                span: Span::new(start, start + 5),
                text: "await",
                space: true,
            })
        } else {
            None
        };
        let left = match ast.for_statement_left_data(left) {
            ForStatementLeftData::VariableDeclaration(node) => HeaderNode {
                span: left_span,
                data: HeaderData::Variable {
                    node,
                    context: VariableContext::Iteration,
                },
            },
            ForStatementLeftData::AssignmentTarget(target) => {
                match ast.assignment_target_data(target) {
                    AssignmentTargetData::SimpleAssignmentTarget(target) => {
                        let node = self.iteration_source_target(
                            target,
                            Span::new(opening.span.end, left_span.start),
                            operator.span,
                            of,
                        )?;
                        let context = self.iteration_left_context(&node, of, awaited)?;
                        HeaderNode {
                            span: node.span(),
                            data: HeaderData::Target { node, context },
                        }
                    }
                    AssignmentTargetData::ArrayAssignmentTarget(node) => HeaderNode {
                        span: left_span,
                        data: HeaderData::Pattern(BindingPattern::ArrayPattern(node)),
                    },
                    AssignmentTargetData::ObjectAssignmentTarget(node) => HeaderNode {
                        span: left_span,
                        data: HeaderData::Pattern(BindingPattern::ObjectPattern(node)),
                    },
                }
            }
        };
        let right = self.for_expression_node(
            right,
            if of {
                ExpressionPosition::Nested
            } else {
                ExpressionPosition::ForHead
            },
        );
        let header = self.prepare_for_header(
            HeaderSyntax {
                keyword_end,
                closing,
                tokens: [await_token, Some(opening), Some(operator), None],
                classic: false,
            },
            [Some(left), Some(right), None],
            body,
        );
        self.format_for_with_body(header, body, span.end, punctuation, writer)
    }

    fn for_expression_node(&self, node: Expr, position: ExpressionPosition) -> HeaderNode {
        HeaderNode {
            span: node.span(self.ast),
            data: HeaderData::Expression {
                node,
                context: ExpressionContext::at(position),
            },
        }
    }

    fn iteration_left_context(
        &self,
        target: &SourceTarget,
        of: bool,
        awaited: bool,
    ) -> Result<ExpressionContext, FormatError> {
        let mut context = ExpressionContext::at(ExpressionPosition::Nested);
        if target.has_source_parentheses() {
            return Ok(context);
        }
        let Some(target) = target.simple() else {
            return Ok(context);
        };
        let mut expression = target.expression();
        let root_data = self.expression_content(expression).data;
        let bare = matches!(
            root_data,
            swc_next_ecma_ast::ExprData::IdentifierReference(_)
        );
        let ordinary = matches!(
            root_data,
            swc_next_ecma_ast::ExprData::IdentifierReference(_)
                | swc_next_ecma_ast::ExprData::MemberExpression(_)
        );
        let mut typed_head = false;
        let mut data = root_data;
        loop {
            typed_head |= matches!(
                data,
                swc_next_ecma_ast::ExprData::TsAsExpression(_)
                    | swc_next_ecma_ast::ExprData::TsSatisfiesExpression(_)
                    | swc_next_ecma_ast::ExprData::TsNonNullExpression(_)
            );
            if self.expression_is_parenthesized(data, context)
                || matches!(
                    data,
                    swc_next_ecma_ast::ExprData::ParenthesizedExpression(_)
                        | swc_next_ecma_ast::ExprData::TsTypeAssertion(_)
                )
                || context.is_verbatim()
            {
                return Ok(ExpressionContext::at(ExpressionPosition::Nested));
            }
            if let swc_next_ecma_ast::ExprData::IdentifierReference(node) = data {
                let name = self.ast.get_utf8(node.name(self.ast));
                let needs =
                    name == "let" && (of || !bare) || name == "async" && of && !awaited && bare;
                return Ok(if ordinary && !typed_head {
                    ExpressionContext::at(ExpressionPosition::ForLeft { parentheses: needs })
                } else {
                    ExpressionContext::at(ExpressionPosition::Nested)
                        .with_lexical_head(needs.then_some(node))
                });
            }
            let Some((child, child_context)) = self.expression_head_operand(data, context)? else {
                return Ok(ExpressionContext::at(ExpressionPosition::Nested));
            };
            expression = child;
            context = child_context;
            data = self.expression_content(expression).data;
        }
    }

    fn prepare_for_header(
        &self,
        syntax: HeaderSyntax,
        nodes: [Option<HeaderNode>; 3],
        body: Stmt,
    ) -> PreparedHeader<'ast> {
        let mut nodes = nodes.into_iter().flatten().peekable();
        let first_start = nodes
            .peek()
            .map_or(syntax.closing.start, |node| node.span.start);
        let empty = nodes.peek().is_none();
        let initial_relation = if empty {
            HeaderRelation::Body(syntax.closing)
        } else {
            HeaderRelation::First
        };
        let mut before = self.prepare_for_gap(
            Span::new(syntax.keyword_end, first_start),
            initial_relation,
            body.span(self.ast).start,
            &syntax.tokens,
        );
        let mut items = [None, None, None];
        for entry in &mut items {
            let Some(node) = nodes.next() else {
                break;
            };
            let (end, relation) = if let Some(next) = nodes.peek() {
                let separator = syntax
                    .tokens
                    .iter()
                    .flatten()
                    .rfind(|token| {
                        token.span.start >= node.span.end && token.span.end <= next.span.start
                    })
                    .map_or(Span::new(node.span.end, node.span.end), |token| token.span);
                (next.span.start, HeaderRelation::Items(separator))
            } else {
                (syntax.closing.start, HeaderRelation::Body(syntax.closing))
            };
            let after = self.prepare_for_gap(
                Span::new(node.span.end, end),
                relation,
                body.span(self.ast).start,
                &syntax.tokens,
            );
            let mode = if before.following == ContentMode::Verbatim
                || after.preceding == ContentMode::Verbatim
            {
                ContentMode::Verbatim
            } else {
                ContentMode::Formatted
            };
            *entry = Some(PreparedHeaderItem { node, before, mode });
            before = after;
        }
        let body_mode = before.following;
        PreparedHeader {
            items,
            trailing: before,
            closing: syntax.closing,
            classic: syntax.classic,
            empty,
            body_mode,
        }
    }

    fn prepare_for_gap(
        &self,
        span: Span,
        relation: HeaderRelation,
        body_start: u32,
        syntax: &[Option<HeaderToken>; 4],
    ) -> HeaderGap<'ast> {
        let mut tokens = [None, None, None];
        for (entry, token) in tokens.iter_mut().zip(
            syntax
                .iter()
                .flatten()
                .filter(|token| token.span.start >= span.start && token.span.end <= span.end),
        ) {
            *entry = Some(*token);
        }
        let leading_start = tokens
            .iter()
            .flatten()
            .last()
            .map_or(span.start, |token| token.span.end);
        let leading = self.comments.in_range(Span::new(leading_start, span.end));
        let mut gap = HeaderGap {
            span,
            tokens,
            leading,
            source: false,
            preceding: ContentMode::Formatted,
            following: ContentMode::Formatted,
        };
        if !self.has_suppression {
            return gap;
        }
        let end = if matches!(relation, HeaderRelation::Body(_)) {
            body_start
        } else {
            span.end
        };
        let comments = self.comments.in_range(Span::new(span.start, end));
        for (run, position) in self.suppressed_comment_runs(comments, end) {
            let Some(last) = run.last() else {
                continue;
            };
            gap.source |= run.iter().any(|comment| {
                comment.span.end <= span.end
                    && crate::formatter::suppression::is_suppression_comment(self.ast, *comment)
            });
            let preceding = match relation {
                HeaderRelation::First => false,
                HeaderRelation::Items(separator) => match position {
                    SuppressionPosition::OwnLine => false,
                    SuppressionPosition::EndOfLine => true,
                    SuppressionPosition::Inline => last.span.end <= separator.start,
                },
                HeaderRelation::Body(closing) => {
                    last.span.end <= closing.start
                        && !matches!(position, SuppressionPosition::OwnLine)
                }
            };
            if preceding {
                gap.preceding = ContentMode::Verbatim;
            } else {
                gap.following = ContentMode::Verbatim;
            }
        }
        gap
    }

    fn format_for_with_body(
        &mut self,
        header: PreparedHeader<'ast>,
        statement: Stmt,
        end: u32,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let body = self
            .prepare_controlled_body(
                statement,
                header.closing.end,
                ControlledBodyEnd::Enclosing(end),
                ControlledBodyRole::Body,
                header.body_mode,
            )
            .with_enclosing_punctuation(punctuation);
        writer.group(false, |writer| {
            self.format_for_header(header, writer)?;
            self.format_controlled_body(body, writer)
        })
    }

    fn format_for_header(
        &mut self,
        header: PreparedHeader<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let PreparedHeader {
            items,
            trailing,
            closing,
            classic,
            empty,
            ..
        } = header;
        writer.text("for");
        let write = |writer: &mut Writer<'ast>| {
            for (index, item) in items.into_iter().flatten().enumerate() {
                let output = self.format_for_gap(item.before, writer)?;
                let first = index == 0;
                let flat = if first {
                    FlatSeparator::Empty
                } else {
                    FlatSeparator::Space
                };
                let default = |writer: &mut Writer<'ast>| {
                    if classic {
                        if first {
                            writer.soft_line();
                        } else {
                            writer.line();
                        }
                    } else if !first {
                        writer.space();
                    }
                };
                let comments = match output {
                    HeaderGapOutput::Canonical { start, leading } => {
                        let first_comment = leading
                            .first()
                            .map_or(item.node.span.start, |comment| comment.span.start);
                        self.line_constraint(start, first_comment)
                            .write(writer, flat, default);
                        leading
                    }
                    HeaderGapOutput::Source { following } => {
                        following.write(writer, flat, default);
                        &[]
                    }
                };
                let head = NodeHead {
                    comments,
                    content_start: item.node.span.start,
                    layout: CommentRunLayout::GroupedLeading(item.node.span.start),
                    gap_end: None,
                };
                self.format_for_header_item(item.node, item.mode, head, writer)?;
            }
            self.format_for_gap(trailing, writer)
        };
        let tail = if classic && !empty {
            writer.indent(write)?
        } else {
            write(writer)?
        };
        match tail {
            HeaderGapOutput::Source { following } => {
                following.write(writer, FlatSeparator::Empty, |writer| {
                    if classic && !empty {
                        writer.soft_line();
                    }
                });
            }
            HeaderGapOutput::Canonical { start, leading } => {
                self.comments.claim(leading)?;
                self.format_comment_run(leading, CommentRunLayout::SourceAfter(start), writer)?;
                let last = leading.last().map_or(start, |comment| comment.span.end);
                self.line_constraint(last, closing.start).write(
                    writer,
                    FlatSeparator::Empty,
                    |writer| {
                        if leading.last().is_some_and(|comment| {
                            comment.kind == swc_next_ecma_ast::CommentKind::Line
                        }) {
                            writer.hard_line();
                        } else if classic && !empty {
                            writer.soft_line();
                        }
                    },
                );
            }
        }
        writer.text(")");
        Ok(())
    }

    fn format_for_gap(
        &mut self,
        gap: HeaderGap<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<HeaderGapOutput<'ast>, FormatError> {
        if gap.source {
            let source =
                self.format_delimited_source_gap(gap.span, DelimitedFollowing::Node, writer)?;
            return Ok(HeaderGapOutput::Source {
                following: source.following,
            });
        }
        let mut start = gap.span.start;
        for token in gap.tokens.into_iter().flatten() {
            self.format_source_token_gap(
                start,
                token.span.start,
                token.space || token.text == "(",
                writer,
            )?;
            writer.text(token.text);
            start = token.span.end;
        }
        Ok(HeaderGapOutput::Canonical {
            start,
            leading: gap.leading,
        })
    }

    fn format_for_header_item(
        &mut self,
        node: HeaderNode,
        mode: ContentMode,
        head: NodeHead<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.comments.claim(head.comments)?;
        let tail = match node.data {
            HeaderData::Expression { node, context } => self
                .format_expression_with_leading_comments(
                    node,
                    head,
                    context
                        .with_content_mode(mode)
                        .with_tail_owner(TailOwner::ParentOperation),
                    writer,
                )?,
            HeaderData::Variable {
                node: declaration,
                context,
            } => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                if mode == ContentMode::Verbatim {
                    self.format_verbatim(node.span, writer)?;
                    None
                } else {
                    self.format_variable_declaration_with_context(declaration, context, writer)?
                        .1
                }
            }
            HeaderData::Target {
                node: target,
                context,
            } => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                self.format_source_target(&target, context.with_content_mode(mode), writer)?
            }
            HeaderData::Pattern(pattern) => {
                self.format_comment_run(head.comments, head.layout, writer)?;
                self.format_declaration_binding(pattern, mode, writer)?
            }
        };
        self.format_expression_tail(tail, writer)
    }
}
