//! Bodyless declarations consume real typed signatures and declaration endings.

use swc_next_ecma_ast::{BindingIdentifier, GetSpan, Span, TsFunction, TsTypeParameterDeclaration};

use super::declarations::AmbientDeclarationHead;
use super::{Formatter, PrintedStatement, StatementBoundary, StatementPunctuation};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::formatter::types::TypeGap;
use crate::formatter::types::callable::{
    CallableParameters, SignatureIncoming, SignatureReturn, SignatureReturnDelimiter,
    SignatureReturnEolTarget,
};
use crate::formatter::types::generics::{TypeListKind, TypeListPrefix, TypeParameterContext};
use crate::{FormatError, document::Writer};

struct FunctionName<'ast> {
    node: BindingIdentifier,
    mode: ContentMode,
    prefix: TypeGap<'ast>,
}

struct FunctionGenerics<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: Option<TypeListPrefix<'ast>>,
    following: TypeGap<'ast>,
}

struct BodylessSignature<'ast> {
    async_prefix: Option<TypeGap<'ast>>,
    star_prefix: Option<TypeGap<'ast>>,
    name: Option<FunctionName<'ast>>,
    prefix: Option<TypeGap<'ast>>,
    generics: Option<FunctionGenerics<'ast>>,
    parameters: CallableParameters,
    result: Option<SignatureReturn<'ast>>,
    content_end: u32,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_bodyless_function(
        &mut self,
        node: TsFunction,
        ambient: Option<AmbientDeclarationHead<'ast>>,
        punctuation: StatementPunctuation,
        writer: &mut Writer<'ast>,
    ) -> Result<PrintedStatement<'ast>, FormatError> {
        let ast = self.ast;
        let parameter_span = node.params(ast).span(ast);
        let parameters = self.callable_parameters(node.params(ast), node.this_param(ast));
        let has_parameters = parameters.has_parameters();
        let enclosing_prefix = self.has_suppression
            && node.id(ast).is_none()
            && node.type_parameters(ast).is_none()
            && node.return_type(ast).is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(node.span(ast).start, parameter_span.start)),
            ) == ContentMode::Verbatim;
        if self.has_suppression
            && !has_parameters
            && (enclosing_prefix
                || self.content_mode(self.comments.in_range(parameter_span))
                    == ContentMode::Verbatim)
        {
            let span = node.span(ast);
            self.format_verbatim(span, writer)?;
            return Ok(StatementBoundary::TypeDeclarationEnd {
                end: span.end,
                has_terminator: self.source(span).ends_with(';'),
            }
            .into());
        }
        let signature = self.prepare_bodyless_signature(node, ambient.as_ref(), parameters)?;
        if let Some(head) = ambient {
            self.format_ambient_declaration_prefix(&head, writer)?;
        }
        let content_end = signature.content_end;
        let tail = writer.group(false, |writer| {
            if let Some(gap) = signature.async_prefix {
                writer.text("async");
                self.format_type_gap(gap, FlatSeparator::Space, writer)?;
            }
            writer.text("function");
            if let Some(gap) = signature.star_prefix {
                self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
                writer.text("*");
            }
            let named = signature.name.is_some();
            if let Some(name) = signature.name {
                self.format_type_gap(name.prefix, FlatSeparator::Space, writer)?;
                if name.mode == ContentMode::Verbatim {
                    self.format_verbatim(name.node.span(ast), writer)?;
                } else {
                    writer.text(ast.get_utf8(name.node.name(ast)));
                }
            }
            if let Some(prefix) = signature.prefix {
                self.format_type_gap(
                    prefix,
                    if named {
                        FlatSeparator::Empty
                    } else {
                        FlatSeparator::Space
                    },
                    writer,
                )?;
            }
            if let Some(generic) = signature.generics {
                if let Some(prefix) = generic.prefix {
                    self.format_type_list_prefix(prefix, writer)?;
                }
                self.format_type_parameters(
                    generic.node,
                    TypeParameterContext::Callable,
                    generic.mode,
                    writer,
                )?;
                self.format_type_gap(generic.following, FlatSeparator::Empty, writer)?;
            }
            self.format_callable_parameters(signature.parameters, writer)?;
            match signature.result {
                Some(result) => self.format_signature_return(result, writer),
                None => Ok(None),
            }
        })?;
        let mut printed = self.finish_statement(content_end, tail, punctuation, writer)?;
        printed.boundary = printed.boundary.type_declaration();
        Ok(printed)
    }

    fn prepare_bodyless_signature(
        &self,
        node: TsFunction,
        ambient: Option<&AmbientDeclarationHead<'ast>>,
        mut parameters: CallableParameters,
    ) -> Result<BodylessSignature<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let parameter_span = node.params(ast).span(ast);
        let generic = node.type_parameters(ast);
        let signature_start = generic.map_or(parameter_span.start, |node| node.span(ast).start);
        let name = node.id(ast);
        let head_end = name.map_or(signature_start, |name| name.span(ast).start);
        let function_start = if let Some(head) = ambient {
            head.keyword.span.start
        } else if node.r#async(ast) {
            let search = Span::new(span.start + 5, head_end);
            self.find_token(search, self.comments.in_range(search), "function")
                .ok_or_else(|| self.unsupported("bodyless async function keyword", node))?
        } else {
            span.start
        };
        let keyword_end = function_start + 8;
        let async_prefix = node
            .r#async(ast)
            .then(|| self.prepare_type_gap(Span::new(span.start + 5, function_start), false));
        let star = if node.generator(ast) {
            let search = Span::new(keyword_end, head_end);
            Some(
                self.find_token(search, self.comments.in_range(search), "*")
                    .ok_or_else(|| self.unsupported("bodyless generator star", node))?,
            )
        } else {
            None
        };
        let star_prefix =
            star.map(|start| self.prepare_type_gap(Span::new(keyword_end, start), false));
        let head_start = star.map_or(keyword_end, |start| start + 1);
        let mut name = name.map(|name| {
            let gap = Span::new(head_start, name.span(ast).start);
            let suppressed = self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
            let mode =
                if suppressed || ambient.is_some_and(|head| head.keyword.suppresses_name(name)) {
                    ContentMode::Verbatim
                } else {
                    ContentMode::Formatted
                };
            FunctionName {
                node: name,
                mode,
                prefix: self.prepare_type_gap(gap, suppressed),
            }
        });
        let prefix_start = name
            .as_ref()
            .map_or(head_start, |name| name.node.span(ast).end);
        let mut incoming = None;
        let mut source_return = None;
        let mut generic_mode = ContentMode::Formatted;
        let prefix;
        let generic_prefix;
        if generic.is_some() && name.is_some() {
            let prepared = self.prepare_type_list_prefix(
                Span::new(prefix_start, signature_start),
                TypeListKind::Parameters,
            );
            if prepared.left_mode == ContentMode::Verbatim
                && let Some(name) = &mut name
            {
                name.mode = ContentMode::Verbatim;
            }
            generic_mode = prepared.right_mode;
            generic_prefix = Some(prepared);
            prefix = None;
        } else {
            let gap = Span::new(prefix_start, signature_start);
            let comments = self.comments.in_range(gap);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            if source {
                if generic.is_some() {
                    generic_mode = ContentMode::Verbatim;
                } else {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        let owner = self.signature_incoming(
                            &parameters,
                            node.return_type(ast),
                            &mut incoming,
                        )?;
                        if name.is_some()
                            && (matches!(position, SuppressionPosition::EndOfLine)
                                || matches!(
                                    owner,
                                    SignatureIncoming::Return(_) | SignatureIncoming::Empty
                                ) && matches!(position, SuppressionPosition::Inline))
                        {
                            if let Some(name) = &mut name {
                                name.mode = ContentMode::Verbatim;
                            }
                        } else {
                            match owner {
                                SignatureIncoming::Parameter(first) => {
                                    parameters.source_head = Some(first)
                                }
                                SignatureIncoming::Return(annotation) => {
                                    source_return = Some(annotation)
                                }
                                SignatureIncoming::Empty => {
                                    if let Some(name) = &mut name {
                                        name.mode = ContentMode::Verbatim;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            prefix = Some(self.prepare_type_gap(gap, source));
            generic_prefix = None;
        }
        let mut generics = generic
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameter_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        let owner = self.signature_incoming(
                            &parameters,
                            node.return_type(ast),
                            &mut incoming,
                        )?;
                        if matches!(position, SuppressionPosition::EndOfLine)
                            || matches!(
                                owner,
                                SignatureIncoming::Return(_) | SignatureIncoming::Empty
                            ) && matches!(position, SuppressionPosition::Inline)
                        {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            match owner {
                                SignatureIncoming::Parameter(first) => {
                                    parameters.source_head = Some(first)
                                }
                                SignatureIncoming::Return(annotation) => {
                                    source_return = Some(annotation)
                                }
                                SignatureIncoming::Empty => generic_mode = ContentMode::Verbatim,
                            }
                        }
                    }
                }
                Ok::<_, FormatError>(FunctionGenerics {
                    node: generic,
                    mode: generic_mode,
                    prefix: generic_prefix,
                    following: self.prepare_type_gap(gap, source),
                })
            })
            .transpose()?;
        let result = node.return_type(ast).map(|annotation| {
            let eol_target = if parameters.has_parameters() {
                SignatureReturnEolTarget::Parameter
            } else if let Some(generics) = &mut generics {
                SignatureReturnEolTarget::Header(&mut generics.mode)
            } else if let Some(name) = &mut name {
                SignatureReturnEolTarget::Header(&mut name.mode)
            } else {
                SignatureReturnEolTarget::Annotation
            };
            self.prepare_signature_return(
                parameter_span.end,
                eol_target,
                annotation,
                source_return,
                SignatureReturnDelimiter::Colon,
            )
        });
        let preceding = result
            .as_ref()
            .map_or(ContentMode::Formatted, |result| result.preceding_mode);
        self.prepare_callable_parameter_tail(&mut parameters, preceding)?;
        let content_end = node
            .return_type(ast)
            .map_or(parameter_span.end, |annotation| annotation.span(ast).end);
        Ok(BodylessSignature {
            async_prefix,
            star_prefix,
            name,
            prefix,
            generics,
            parameters,
            result,
            content_end,
        })
    }
}
