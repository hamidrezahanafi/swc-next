//! Actual await/yield operands and the lexical new.target primary expression.

use swc_next_ecma_ast::{AwaitExpression, Expr, GetSpan, MetaProperty, Span, YieldExpression};

use super::{ExpressionContext, ExpressionPosition, Formatter, TailOwner};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail, NodeHead};
use crate::formatter::diagnostics::FlatSeparator;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_await(
        &mut self,
        expression: AwaitExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        writer.text("await");
        self.format_unrestricted_keyword_argument(
            expression.span(self.ast).start + 5,
            expression.argument(self.ast),
            ExpressionPosition::AwaitArgument,
            writer,
        )
    }

    pub(super) fn format_yield(
        &mut self,
        expression: YieldExpression,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let keyword_end = expression.span(self.ast).start + 5;
        let Some(argument) = expression.argument(self.ast) else {
            writer.text("yield");
            return Ok(None);
        };
        if expression.delegate(self.ast) {
            let gap = Span::new(keyword_end, argument.span(self.ast).start);
            let comments = self.comments.in_range(gap);
            let Some(star) = self.find_token(gap, comments, "*") else {
                return Err(self.unsupported("delegated yield star boundary", expression));
            };
            writer.text("yield");
            self.format_source_token_gap(keyword_end, star, false, writer)?;
            writer.text("*");
            self.format_unrestricted_keyword_argument(
                star + 1,
                argument,
                ExpressionPosition::YieldArgument,
                writer,
            )
        } else {
            let prepared = self.prepare_yield_argument(keyword_end, argument)?;
            writer.text("yield");
            self.format_restricted_argument(prepared, writer)
        }
    }

    fn format_unrestricted_keyword_argument(
        &mut self,
        keyword_end: u32,
        argument: Expr,
        position: ExpressionPosition,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let span = argument.span(self.ast);
        let leading = self.comments.between(Span::new(keyword_end, span.start))?;
        let mode = self.content_mode(leading);
        let first = leading
            .first()
            .map_or(span.start, |comment| comment.span.start);
        self.line_constraint(keyword_end, first)
            .write(writer, FlatSeparator::Space, Writer::space);
        self.comments.claim(leading)?;
        self.format_expression_with_leading_comments(
            argument,
            NodeHead {
                comments: leading,
                content_start: span.start,
                layout: CommentRunLayout::FlatLeading(span.start),
                gap_end: None,
            },
            ExpressionContext::at(position)
                .with_content_mode(mode)
                .with_tail_owner(TailOwner::ParentOperation),
            writer,
        )
    }

    pub(super) fn format_meta_property(
        &mut self,
        expression: MetaProperty,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let meta = expression.meta(ast);
        let property = expression.property(ast);
        if ast.get_utf8(meta.name(ast)) != "new" || ast.get_utf8(property.name(ast)) != "target" {
            return Err(self.unsupported("meta property", expression));
        }
        let gap = Span::new(meta.span(ast).end, property.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(dot) = self.find_token(gap, comments, ".") else {
            return Err(self.unsupported("new.target dot boundary", expression));
        };
        writer.text(ast.get_utf8(meta.name(ast)));
        self.format_source_token_gap(gap.start, dot, false, writer)?;
        writer.text(".");
        self.format_source_token_gap(dot + 1, gap.end, false, writer)?;
        writer.text(ast.get_utf8(property.name(ast)));
        Ok(())
    }
}
