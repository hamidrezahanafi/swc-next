//! Assignment and update owners recover target delimiters from their grammar gaps.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTargetData, Comment, Expr, GetSpan,
    SimpleAssignmentTarget, SimpleAssignmentTargetData, Span, UpdateExpression, UpdateOperator,
};

use super::parentheses::SourceParentheses;
use super::{ExpressionContext, Formatter, StatementStart, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::source::is_source_whitespace;
use crate::formatter::suppression::{BoundaryPunctuation, ContentMode};
use crate::{FormatError, Semicolons, document::Writer};

#[derive(Clone, Copy)]
enum TargetOperator {
    Assignment(AssignmentOperator),
    Update(UpdateOperator),
}

impl TargetOperator {
    fn as_str(self) -> &'static str {
        match self {
            Self::Assignment(operator) => operator.as_str(),
            Self::Update(operator) => operator.as_str(),
        }
    }
}

pub(super) struct SourceTarget {
    target: SimpleAssignmentTarget,
    operator: TargetOperator,
    pub(super) operator_span: Span,
    pub(super) span: Span,
    layers: Vec<TargetLayer>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetLayerMode {
    Remove,
    Retain,
    Verbatim,
}

struct TargetLayer {
    parentheses: SourceParentheses,
    mode: TargetLayerMode,
}

impl SourceTarget {
    pub(super) fn operator(&self) -> &'static str {
        self.operator.as_str()
    }
}

/// Only walks the owner's gaps; complete target and comment bodies are excluded.
struct TargetGap<'a> {
    source: &'a str,
    comments: &'a [Comment],
    position: u32,
    end: u32,
}

impl TargetGap<'_> {
    fn skip_trivia(&mut self) {
        loop {
            if let Some((comment, rest)) = self.comments.split_first()
                && comment.span.start == self.position
            {
                self.position = comment.span.end;
                self.comments = rest;
                continue;
            }
            let remaining = &self.source[self.position as usize..self.end as usize];
            let Some(character) = remaining.chars().next() else {
                break;
            };
            if !is_source_whitespace(character) {
                break;
            }
            self.position += character.len_utf8() as u32;
        }
    }

    fn consume(&mut self, token: &str) -> Option<Span> {
        self.skip_trivia();
        let remaining = &self.source[self.position as usize..self.end as usize];
        if !remaining.starts_with(token) {
            return None;
        }
        let start = self.position;
        self.position += token.len() as u32;
        Some(Span::new(start, self.position))
    }

    fn finished(&mut self) -> bool {
        self.skip_trivia();
        self.position == self.end
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn assignment_source_target(
        &self,
        assignment: AssignmentExpression,
    ) -> Result<SourceTarget, FormatError> {
        let left = assignment.left(self.ast);
        let AssignmentTargetData::SimpleAssignmentTarget(target) =
            self.ast.assignment_target_data(left)
        else {
            return Err(self.unsupported("assignment target", left));
        };
        let span = target.span(self.ast);
        self.prepare_source_target(
            target,
            TargetOperator::Assignment(assignment.operator(self.ast)),
            Span::new(assignment.span(self.ast).start, span.start),
            Span::new(span.end, assignment.right(self.ast).span(self.ast).start),
            None,
        )
    }

    pub(super) fn update_source_target(
        &self,
        update: UpdateExpression,
    ) -> Result<SourceTarget, FormatError> {
        let target = update.argument(self.ast);
        let span = update.span(self.ast);
        let target_span = target.span(self.ast);
        let operator = TargetOperator::Update(update.operator(self.ast));
        let length = operator.as_str().len() as u32;
        let (prefix, suffix, operator_span) = if update.prefix(self.ast) {
            (
                Span::new(span.start + length, target_span.start),
                Span::new(target_span.end, span.end),
                Span::new(span.start, span.start + length),
            )
        } else {
            (
                Span::new(span.start, target_span.start),
                Span::new(target_span.end, span.end - length),
                Span::new(span.end - length, span.end),
            )
        };
        self.prepare_source_target(target, operator, prefix, suffix, Some(operator_span))
    }

    fn prepare_source_target(
        &self,
        target: SimpleAssignmentTarget,
        operator: TargetOperator,
        prefix: Span,
        suffix: Span,
        known_operator: Option<Span>,
    ) -> Result<SourceTarget, FormatError> {
        let gap = |span: Span| TargetGap {
            source: self.ast.source(),
            comments: self.comments.in_range(span),
            position: span.start,
            end: span.end,
        };
        let invalid = || {
            FormatError::unsupported(
                "target source boundary",
                None,
                Span::new(prefix.start, suffix.end),
            )
        };
        let mut leading = gap(prefix);
        let mut layers = Vec::new();
        while let Some(opening) = leading.consume("(") {
            layers.push(TargetLayer {
                parentheses: SourceParentheses {
                    outer: opening,
                    inner: target.span(self.ast),
                },
                mode: TargetLayerMode::Remove,
            });
        }
        if !leading.finished() {
            return Err(invalid());
        }
        let mut trailing = gap(suffix);
        let mut inner = target.span(self.ast);
        for layer in layers.iter_mut().rev() {
            let closing = trailing.consume(")").ok_or_else(invalid)?;
            layer.parentheses.outer.end = closing.end;
            layer.parentheses.inner = inner;
            layer.mode = self.target_layer_mode(layer.parentheses);
            inner = layer.parentheses.outer;
        }
        let operator_span = match known_operator {
            Some(span) if self.source(span) == operator.as_str() => span,
            Some(_) => return Err(invalid()),
            None => trailing.consume(operator.as_str()).ok_or_else(invalid)?,
        };
        if !trailing.finished() {
            return Err(invalid());
        }
        // Target parentheses affect diagnostic classification and JavaScript
        // property initialization inference, so preserve their outer source pair.
        if let Some(outer) = layers.first_mut()
            && outer.mode == TargetLayerMode::Remove
        {
            outer.mode = TargetLayerMode::Retain;
        }
        Ok(SourceTarget {
            target,
            operator,
            operator_span,
            span: inner,
            layers,
        })
    }

    fn target_layer_mode(&self, boundary: SourceParentheses) -> TargetLayerMode {
        if self.has_suppression {
            let leading = self
                .comments
                .in_range(Span::new(boundary.outer.start, boundary.inner.start));
            let trailing = Span::new(boundary.inner.end, boundary.outer.end - 1);
            if self.content_mode(leading) == ContentMode::Verbatim
                || self.trailing_content_mode(trailing, BoundaryPunctuation::None, true)
                    == ContentMode::Verbatim
            {
                return TargetLayerMode::Verbatim;
            }
        }
        if self.parentheses_are_removable(boundary) {
            TargetLayerMode::Remove
        } else {
            TargetLayerMode::Retain
        }
    }

    pub(super) fn format_source_target(
        &mut self,
        target: &SourceTarget,
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let verbatim = context.content_mode == ContentMode::Verbatim && !target.layers.is_empty();
        if !target.layers.is_empty()
            && context.statement_start != StatementStart::None
            && self.options.semicolons == Semicolons::AsNeeded
        {
            // Prepare the statement guard before any removed-layer comments.
            writer.text(";");
            context.statement_start = StatementStart::None;
        }
        if verbatim {
            self.format_verbatim(target.span, writer)?;
            return Ok(None);
        }
        self.format_target_layers(target.target, &target.layers, context, writer)
    }

    fn format_target_layers(
        &mut self,
        target: SimpleAssignmentTarget,
        mut layers: &[TargetLayer],
        mut context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let target_end = target.span(self.ast).end;
        let outer_end = layers
            .first()
            .map_or(target_end, |layer| layer.parentheses.outer.end);
        let (content_end, tail) = loop {
            let Some((layer, remaining)) = layers.split_first() else {
                break (
                    target_end,
                    self.format_simple_assignment_target(target, context, writer)?,
                );
            };
            let boundary = layer.parentheses;
            match layer.mode {
                TargetLayerMode::Verbatim => {
                    self.format_verbatim(boundary.outer, writer)?;
                    break (boundary.outer.end, None);
                }
                TargetLayerMode::Retain => {
                    let tail = self.format_retained_target_parentheses(
                        boundary,
                        context,
                        writer,
                        |formatter, context, writer| {
                            formatter.format_target_layers(target, remaining, context, writer)
                        },
                    )?;
                    break (boundary.outer.end, tail);
                }
                TargetLayerMode::Remove => {
                    let leading = self
                        .comments
                        .between(Span::new(boundary.outer.start, boundary.inner.start))?;
                    self.comments.claim(leading)?;
                    context.statement_start = self.format_expression_leading_comments(
                        leading,
                        boundary.inner.start,
                        CommentRunLayout::FlatLeading(boundary.inner.start),
                        context.statement_start,
                        writer,
                    )?;
                    layers = remaining;
                }
            }
        };
        if content_end == outer_end {
            return Ok(tail);
        }
        // One deferred tail belongs to the whole run of removed wrappers.
        self.capture_expression_tail(Span::new(content_end, outer_end), tail)
    }

    pub(super) fn format_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let expression = match self.ast.simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::IdentifierReference(identifier) => {
                Expr::IdentifierReference(identifier)
            }
            SimpleAssignmentTargetData::MemberExpression(member) => Expr::MemberExpression(member),
            _ => return Err(self.unsupported("assignment target", target)),
        };
        self.format_expression(
            expression,
            context.with_tail_owner(TailOwner::ParentOperation),
            writer,
        )
    }
}
