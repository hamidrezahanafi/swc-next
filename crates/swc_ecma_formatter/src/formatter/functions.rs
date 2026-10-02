//! Ordinary function headers and their actual parameter/body boundaries.

use swc_next_ecma_ast::{Function, GetSpan, Span};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::diagnostics::FlatSeparator;
use crate::{FormatError, document::Writer};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_function(
        &mut self,
        function: Function,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        if function.r#async(ast)
            || function.generator(ast)
            || function.type_parameters(ast).is_some()
            || function.this_param(ast).is_some()
            || function.return_type(ast).is_some()
        {
            return Err(self.unsupported("async, generator or typed function", function));
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
        writer.text("function");
        let keyword_end = function.span(ast).start + "function".len() as u32;
        let name = function.id(ast);
        let head_end = name.map_or(parameter_span.start, |name| name.span(ast).start);
        self.format_function_header_gap(keyword_end, head_end, true, writer)?;
        if let Some(name) = name {
            writer.text(ast.get_utf8(name.name(ast)));
            self.format_function_header_gap(
                name.span(ast).end,
                parameter_span.start,
                false,
                writer,
            )?;
        }
        self.format_source_parameters(parameters, writer)?;
        self.comments.claim(after_parameters)?;
        self.format_comment_run(
            after_parameters,
            CommentRunLayout::SourceAfter(parameter_span.end),
            writer,
        )?;
        let last = after_parameters
            .last()
            .map_or(parameter_span.end, |comment| comment.span.end);
        self.line_constraint(last, body_span.start).write(
            writer,
            FlatSeparator::Space,
            Writer::space,
        );
        self.format_function_body(body, body_mode, writer)
    }

    fn format_function_header_gap(
        &mut self,
        start: u32,
        end: u32,
        space: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let comments = self.comments.between(Span::new(start, end))?;
        self.comments.claim(comments)?;
        if space {
            let first = comments.first().map_or(end, |comment| comment.span.start);
            self.line_constraint(start, first)
                .write(writer, FlatSeparator::Space, Writer::space);
            self.format_comment_run(comments, CommentRunLayout::FlatLeading(end), writer)
        } else {
            self.format_comment_run(comments, CommentRunLayout::SourceAfter(start), writer)?;
            let last = comments.last().map_or(start, |comment| comment.span.end);
            self.line_constraint(last, end)
                .write(writer, FlatSeparator::Empty, |writer| {
                    if comments
                        .last()
                        .is_some_and(|comment| comment.kind == swc_next_ecma_ast::CommentKind::Line)
                    {
                        writer.hard_line();
                    }
                });
            Ok(())
        }
    }
}
