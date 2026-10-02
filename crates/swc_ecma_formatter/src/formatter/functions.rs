//! Function headers and their actual parameter/body boundaries.

use swc_next_ecma_ast::{Comment, Function, FunctionBody, GetSpan, Span};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::diagnostics::FlatSeparator;
use super::suppression::ContentMode;
use crate::{FormatError, document::Writer};

pub(super) struct SourceFunctionBody<'a> {
    body: FunctionBody,
    span: Span,
    parameter_end: u32,
    comments: &'a [Comment],
    mode: ContentMode,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_function(
        &mut self,
        function: Function,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let body = self.format_function_head(function, writer)?;
        self.format_function_body_boundary(body, writer)
    }

    pub(super) fn format_function_head(
        &mut self,
        function: Function,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceFunctionBody<'ast>, FormatError> {
        let ast = self.ast;
        if function.type_parameters(ast).is_some()
            || function.this_param(ast).is_some()
            || function.return_type(ast).is_some()
        {
            return Err(self.unsupported("typed function", function));
        }
        let params = function.params(ast);
        let parameters = self.source_function_parameters(params)?;
        let parameter_span = params.span(ast);
        let body = function.body(ast);
        let body_span = body.span(ast);
        let after_parameters = self
            .comments
            .in_range(Span::new(parameter_span.end, body_span.start));
        let body_mode = self.content_mode(after_parameters);
        let start = function.span(ast).start;
        let name = function.id(ast);
        let head_end = name.map_or(parameter_span.start, |name| name.span(ast).start);
        let function_start = if function.r#async(ast) {
            let gap = Span::new(start + 5, head_end);
            let comments = self.comments.in_range(gap);
            let Some(token) = self.find_token(gap, comments, "function") else {
                return Err(self.unsupported("async function keyword boundary", function));
            };
            Some(token)
        } else {
            None
        };
        let keyword_end = function_start.unwrap_or(start) + 8;
        let star = if function.generator(ast) {
            let gap = Span::new(keyword_end, head_end);
            let comments = self.comments.in_range(gap);
            let Some(token) = self.find_token(gap, comments, "*") else {
                return Err(self.unsupported("generator star boundary", function));
            };
            Some(token)
        } else {
            None
        };
        if let Some(function_start) = function_start {
            writer.text("async");
            self.format_source_token_gap(start + 5, function_start, true, writer)?;
        }
        writer.text("function");
        let header_end = if let Some(star) = star {
            self.format_source_token_gap(keyword_end, star, false, writer)?;
            writer.text("*");
            star + 1
        } else {
            keyword_end
        };
        self.format_source_token_gap(header_end, head_end, true, writer)?;
        if let Some(name) = name {
            writer.text(ast.get_utf8(name.name(ast)));
            self.format_source_token_gap(name.span(ast).end, parameter_span.start, false, writer)?;
        }
        self.format_source_parameters(parameters, writer)?;
        Ok(SourceFunctionBody {
            body,
            span: body_span,
            parameter_end: parameter_span.end,
            comments: after_parameters,
            mode: body_mode,
        })
    }

    pub(super) fn format_function_body_boundary(
        &mut self,
        source: SourceFunctionBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceFunctionBody {
            body,
            span,
            parameter_end,
            comments,
            mode,
        } = source;
        self.comments.claim(comments)?;
        self.format_comment_run(
            comments,
            CommentRunLayout::SourceAfter(parameter_end),
            writer,
        )?;
        let end = comments
            .last()
            .map_or(parameter_end, |comment| comment.span.end);
        self.line_constraint(end, span.start)
            .write(writer, FlatSeparator::Space, Writer::space);
        self.format_function_body(body, mode, writer)
    }
}
