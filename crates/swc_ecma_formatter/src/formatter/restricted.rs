//! Restricted operand boundaries prepared before their keyword is emitted.

use swc_next_ecma_ast::{
    AssignmentTargetData, Comment, CommentKind, Expr, ExprData, GetSpan,
    SimpleAssignmentTargetData, Span,
};

use super::Formatter;
use super::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::FlatSeparator;
use super::expressions::parentheses::SourceParentheses;
use super::expressions::{ExpressionContext, ExpressionPosition, TailOwner};
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

#[derive(Clone, Copy)]
enum ArgumentEnvelope {
    Inline,
    Breakable,
    Required,
}

#[derive(Clone, Copy)]
enum RestrictedGrammar {
    ReturnOrThrow,
    Yield,
}

/// The first lexical owner and its removable source-prefix comment run.
struct RestrictedHead<'ast> {
    expression: Expr,
    comments: &'ast [Comment],
}

pub(super) struct RestrictedArgument<'ast> {
    argument: Expr,
    head: RestrictedHead<'ast>,
    keyword_end: u32,
    mode: ContentMode,
    envelope: ArgumentEnvelope,
    grammar: RestrictedGrammar,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_restricted_argument(
        &self,
        keyword_end: u32,
        argument: Expr,
    ) -> Result<RestrictedArgument<'ast>, FormatError> {
        self.prepare_restricted_operand(keyword_end, argument, RestrictedGrammar::ReturnOrThrow)
    }

    pub(super) fn prepare_yield_argument(
        &self,
        keyword_end: u32,
        argument: Expr,
    ) -> Result<RestrictedArgument<'ast>, FormatError> {
        self.prepare_restricted_operand(keyword_end, argument, RestrictedGrammar::Yield)
    }

    fn prepare_restricted_operand(
        &self,
        keyword_end: u32,
        argument: Expr,
        grammar: RestrictedGrammar,
    ) -> Result<RestrictedArgument<'ast>, FormatError> {
        let span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(keyword_end, span.start))?;
        let mode = self.content_mode(leading);
        let protected_opening = self.diagnostic_line_is_sensitive(span.start);
        let mut head = RestrictedHead {
            expression: argument,
            comments: leading,
        };
        let envelope = if protected_opening || mode == ContentMode::Verbatim {
            ArgumentEnvelope::Inline
        } else {
            let lexical_head = self.restricted_lexical_head(argument);
            let leading_requires_envelope = leading.iter().any(|comment| {
                comment.kind == CommentKind::Line
                    || self.line_breaks(comment.span.start, comment.span.end) != 0
            });
            if let Some(expression) = lexical_head {
                head = RestrictedHead {
                    expression,
                    comments: self
                        .comments
                        .between(Span::new(keyword_end, expression.span(self.ast).start))?,
                };
            }
            if leading_requires_envelope || lexical_head.is_some() {
                ArgumentEnvelope::Required
            } else {
                match self.expression_content(argument).data {
                    ExprData::BinaryExpression(_) | ExprData::LogicalExpression(_) => {
                        ArgumentEnvelope::Breakable
                    }
                    ExprData::SequenceExpression(_)
                        if matches!(grammar, RestrictedGrammar::ReturnOrThrow) =>
                    {
                        ArgumentEnvelope::Breakable
                    }
                    _ => ArgumentEnvelope::Inline,
                }
            }
        };
        Ok(RestrictedArgument {
            argument,
            head,
            keyword_end,
            mode,
            envelope,
            grammar,
        })
    }

    pub(super) fn format_restricted_argument(
        &mut self,
        prepared: RestrictedArgument<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let RestrictedArgument {
            argument,
            head,
            keyword_end,
            mode,
            envelope,
            grammar,
        } = prepared;
        let content_start = head.expression.span(self.ast).start;
        let first = head
            .comments
            .first()
            .map_or(content_start, |comment| comment.span.start);
        self.line_constraint(keyword_end, first)
            .write(writer, FlatSeparator::Space, Writer::space);
        let position = match grammar {
            RestrictedGrammar::ReturnOrThrow => ExpressionPosition::RestrictedArgument,
            RestrictedGrammar::Yield => ExpressionPosition::YieldArgument,
        };
        let write_argument = |formatter: &mut Self, writer: &mut Writer<'ast>| {
            formatter.comments.claim(head.comments)?;
            formatter.format_expression_with_leading_comments(
                argument,
                NodeHead {
                    comments: head.comments,
                    content_start,
                    layout: CommentRunLayout::FlatLeading(content_start),
                    gap_end: None,
                },
                ExpressionContext::at(position)
                    .with_content_mode(mode)
                    .with_tail_owner(TailOwner::ParentOperation),
                writer,
            )
        };
        match envelope {
            ArgumentEnvelope::Inline => write_argument(self, writer),
            ArgumentEnvelope::Breakable | ArgumentEnvelope::Required => {
                writer.group(matches!(envelope, ArgumentEnvelope::Required), |writer| {
                    if matches!(envelope, ArgumentEnvelope::Required) {
                        writer.text("(");
                    } else {
                        writer.if_break("(");
                    }
                    writer.indent(|writer| {
                        if matches!(envelope, ArgumentEnvelope::Required) {
                            writer.hard_line();
                        } else {
                            writer.soft_line();
                        }
                        let tail = write_argument(self, writer)?;
                        self.format_expression_tail(tail, writer)
                    })?;
                    if matches!(envelope, ArgumentEnvelope::Required) {
                        writer.hard_line();
                        writer.text(")");
                    } else {
                        writer.soft_line();
                        writer.if_break(")");
                    }
                    Ok(None)
                })
            }
        }
    }

    fn restricted_lexical_head(&self, mut expression: Expr) -> Option<Expr> {
        if self.comments.in_range(expression.span(self.ast)).is_empty() {
            return None;
        }
        let mut requires_envelope = false;
        loop {
            let data = self.ast.expr_data(expression);
            expression = match data {
                ExprData::ParenthesizedExpression(parenthesized) => {
                    let child = parenthesized.expression(self.ast);
                    let boundary = SourceParentheses {
                        outer: parenthesized.span(self.ast),
                        inner: child.span(self.ast),
                    };
                    if !self.parentheses_are_removable(boundary) {
                        break;
                    }
                    let leading = self
                        .comments
                        .in_range(Span::new(boundary.outer.start, boundary.inner.start));
                    if leading.iter().any(|comment| {
                        comment.kind == CommentKind::Line
                            || self.line_breaks(comment.span.start, comment.span.end) != 0
                            || self.line_breaks(boundary.outer.start, comment.span.start) != 0
                    }) {
                        requires_envelope = true;
                    }
                    child
                }
                ExprData::MemberExpression(member) => member.object(self.ast),
                ExprData::CallExpression(call) => call.callee(self.ast),
                ExprData::TaggedTemplateExpression(tagged) => tagged.tag(self.ast),
                ExprData::ConditionalExpression(conditional) => conditional.test(self.ast),
                ExprData::SequenceExpression(sequence) => {
                    let Some(first) = sequence.expressions(self.ast).iter().next() else {
                        break;
                    };
                    self.ast.get_node_in_sub_range(first)
                }
                ExprData::AssignmentExpression(assignment) => {
                    let AssignmentTargetData::SimpleAssignmentTarget(target) =
                        self.ast.assignment_target_data(assignment.left(self.ast))
                    else {
                        break;
                    };
                    let SimpleAssignmentTargetData::MemberExpression(member) =
                        self.ast.simple_assignment_target_data(target)
                    else {
                        break;
                    };
                    member.object(self.ast)
                }
                ExprData::UpdateExpression(update) if !update.prefix(self.ast) => {
                    let SimpleAssignmentTargetData::MemberExpression(member) = self
                        .ast
                        .simple_assignment_target_data(update.argument(self.ast))
                    else {
                        break;
                    };
                    member.object(self.ast)
                }
                ExprData::BinaryExpression(binary) => binary.left(self.ast),
                ExprData::LogicalExpression(logical) => logical.left(self.ast),
                _ => break,
            };
        }
        requires_envelope.then_some(expression)
    }
}
