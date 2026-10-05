//! Function headers and their actual parameter/body boundaries.

mod methods;
pub(super) use methods::{MethodOwner, SourceMethodHeader};

use swc_next_ecma_ast::{
    BindingIdentifier, Comment, CommentKind, Function, FunctionBody, GetSpan, PropertyKey, Span,
    StmtData,
};

use super::Formatter;
use super::comments::CommentRunLayout;
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::parameters::SourceParameters;
use super::statements::IncomingStatement;
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use crate::{FormatError, document::Writer};

pub(super) struct SourceFunctionSignature<'ast> {
    pub(super) span: Span,
    parameters: SourceParameters,
    body: SourceFunctionBody<'ast>,
    preceding: Option<SignaturePreceding>,
}

pub(super) enum FunctionSignatureOwner {
    Function,
    Method {
        key: PropertyKey,
        kind: MethodSignatureKind,
    },
}

#[derive(Clone, Copy)]
pub(super) enum MethodSignatureKind {
    Public,
    Private,
}

enum SignaturePreceding {
    Name(BindingIdentifier),
    Key(PropertyKey),
}

pub(super) enum SignatureBoundaryRole {
    Function { has_preceding: bool },
    PrivateMethod { has_preceding: bool },
    Arrow { has_parameters: bool },
}

pub(super) struct SignatureBodyModes {
    preceding: ContentMode,
    pub(super) body: ContentMode,
    pub(super) first_statement: Option<IncomingStatement>,
    pub(super) constraint: LineConstraint,
}

impl SourceFunctionSignature<'_> {
    pub(super) fn has_parameters(&self) -> bool {
        self.parameters.has_parameters()
    }

    pub(super) fn enclosing_mode(&self) -> ContentMode {
        self.parameters.enclosing_mode()
    }

    pub(super) fn name_mode(&self, name: BindingIdentifier) -> ContentMode {
        if matches!(self.preceding, Some(SignaturePreceding::Name(target)) if target == name) {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    pub(super) fn key_mode(&self, key: PropertyKey) -> ContentMode {
        if matches!(self.preceding, Some(SignaturePreceding::Key(target)) if target == key) {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        }
    }

    pub(super) fn set_leading_mode(&mut self, mode: ContentMode) {
        if self.parameters.has_parameters() {
            self.parameters.set_leading_mode(mode);
        } else if mode == ContentMode::Verbatim {
            self.body.mode = mode;
        }
    }
}

pub(super) struct FunctionNameModes {
    pub(super) name: ContentMode,
    pub(super) following: ContentMode,
}

pub(super) struct SourceFunctionBody<'a> {
    body: FunctionBody,
    parameter_end: u32,
    comments: &'a [Comment],
    mode: ContentMode,
    first_statement: Option<IncomingStatement>,
    constraint: LineConstraint,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_function(
        &mut self,
        function: Function,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let signature =
            self.source_function_signature(function, FunctionSignatureOwner::Function)?;
        if signature.enclosing_mode() == ContentMode::Verbatim {
            return self.format_verbatim(function.span(self.ast), writer);
        }
        let body = self.format_function_head(function, signature, writer)?;
        self.format_function_body_boundary(body, writer)
    }

    pub(super) fn format_function_head(
        &mut self,
        function: Function,
        mut signature: SourceFunctionSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceFunctionBody<'ast>, FormatError> {
        let ast = self.ast;
        let parameter_span = signature.span;
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
        let header_end = star.map_or(keyword_end, |star| star + 1);
        let mut name_modes = match name {
            Some(name) => {
                self.function_name_modes(Span::new(name.span(ast).end, parameter_span.start))
            }
            None => FunctionNameModes {
                name: ContentMode::Formatted,
                following: if self.has_suppression {
                    self.content_mode(
                        self.comments
                            .in_range(Span::new(start, parameter_span.start)),
                    )
                } else {
                    ContentMode::Formatted
                },
            },
        };
        if self.has_suppression
            && let Some(name) = name
            && self.content_mode(
                self.comments
                    .in_range(Span::new(start, name.span(ast).start)),
            ) == ContentMode::Verbatim
        {
            name_modes.name = ContentMode::Verbatim;
        }
        if let Some(name) = name
            && signature.name_mode(name) == ContentMode::Verbatim
        {
            name_modes.name = ContentMode::Verbatim;
        }
        signature.set_leading_mode(name_modes.following);
        if let Some(function_start) = function_start {
            writer.text("async");
            self.format_source_token_gap(start + 5, function_start, true, writer)?;
        }
        writer.text("function");
        if let Some(star) = star {
            self.format_source_token_gap(keyword_end, star, false, writer)?;
            writer.text("*");
        }
        self.format_source_token_gap(header_end, head_end, true, writer)?;
        if let Some(name) = name {
            let name_span = name.span(ast);
            if name_modes.name == ContentMode::Verbatim {
                self.format_verbatim(name_span, writer)?;
            } else {
                writer.text(ast.get_utf8(name.name(ast)));
            }
            self.format_source_token_gap(name_span.end, parameter_span.start, false, writer)?;
        }
        self.format_function_signature(signature, writer)
    }

    pub(super) fn function_name_modes(&self, span: Span) -> FunctionNameModes {
        let mut modes = FunctionNameModes {
            name: ContentMode::Formatted,
            following: ContentMode::Formatted,
        };
        if !self.has_suppression {
            return modes;
        }
        let comments = self.comments.in_range(span);
        for position in self.suppression_positions(comments, span.end) {
            match position {
                SuppressionPosition::OwnLine => modes.following = ContentMode::Verbatim,
                SuppressionPosition::Inline | SuppressionPosition::EndOfLine => {
                    modes.name = ContentMode::Verbatim;
                }
            }
        }
        modes
    }

    pub(super) fn source_function_signature(
        &self,
        function: Function,
        owner: FunctionSignatureOwner,
    ) -> Result<SourceFunctionSignature<'ast>, FormatError> {
        let ast = self.ast;
        if function.type_parameters(ast).is_some()
            || function.this_param(ast).is_some()
            || function.return_type(ast).is_some()
        {
            return Err(self.unsupported("typed function", function));
        }
        let params = function.params(ast);
        let body = function.body(ast);
        let private = matches!(
            owner,
            FunctionSignatureOwner::Method {
                kind: MethodSignatureKind::Private,
                ..
            }
        );
        let mut parameters = if private {
            self.source_private_method_parameters(params, body)?
        } else {
            self.source_function_parameters(params)?
        };
        let parameter_key_mode = parameters.key_mode();
        let parameter_body_mode = parameters.body_mode(body);
        let span = params.span(ast);
        let body_span = body.span(ast);
        let comments = self.comments.in_range(Span::new(span.end, body_span.start));
        let previous = if parameter_key_mode == ContentMode::Verbatim
            || self.content_mode(comments) == ContentMode::Verbatim
        {
            match owner {
                FunctionSignatureOwner::Function => function.id(ast).map(SignaturePreceding::Name),
                FunctionSignatureOwner::Method { key, .. } => Some(SignaturePreceding::Key(key)),
            }
        } else {
            None
        };
        let mut modes = self.signature_body_modes(
            body,
            comments,
            body_span.start,
            span.end,
            if private {
                SignatureBoundaryRole::PrivateMethod {
                    has_preceding: parameters.has_parameters() || previous.is_some(),
                }
            } else {
                SignatureBoundaryRole::Function {
                    has_preceding: parameters.has_parameters() || previous.is_some(),
                }
            },
        );
        if parameter_body_mode == ContentMode::Verbatim {
            modes.body = parameter_body_mode;
        }
        let preceding = if parameter_key_mode == ContentMode::Verbatim {
            previous
        } else if modes.preceding == ContentMode::Verbatim {
            if parameters.has_parameters() {
                parameters.suppress_last(ast)?;
                None
            } else {
                previous
            }
        } else {
            None
        };
        Ok(SourceFunctionSignature {
            span,
            parameters,
            body: SourceFunctionBody {
                body,
                parameter_end: span.end,
                comments,
                mode: modes.body,
                first_statement: modes.first_statement,
                constraint: modes.constraint,
            },
            preceding,
        })
    }

    pub(super) fn signature_body_modes(
        &self,
        body: FunctionBody,
        comments: &[Comment],
        body_start: u32,
        source_end: u32,
        role: SignatureBoundaryRole,
    ) -> SignatureBodyModes {
        let end = comments
            .last()
            .map_or(source_end, |comment| comment.span.end);
        let mut modes = SignatureBodyModes {
            preceding: ContentMode::Formatted,
            body: ContentMode::Formatted,
            first_statement: None,
            constraint: self.line_constraint(end, body_start),
        };
        if !self.has_suppression {
            return modes;
        }
        let mut needs_first_statement = false;
        for (run, position) in self.suppressed_comment_runs(comments, body_start) {
            for &comment in run {
                if !is_suppression_comment(self.ast, comment) {
                    continue;
                }
                match role {
                    SignatureBoundaryRole::PrivateMethod {
                        has_preceding: true,
                    } if matches!(position, SuppressionPosition::EndOfLine) => {
                        modes.preceding = ContentMode::Verbatim;
                    }
                    SignatureBoundaryRole::Function { .. } if comment.kind == CommentKind::Line => {
                        needs_first_statement = true;
                    }
                    SignatureBoundaryRole::Function {
                        has_preceding: true,
                    } if matches!(position, SuppressionPosition::EndOfLine) => {
                        modes.preceding = ContentMode::Verbatim;
                    }
                    SignatureBoundaryRole::Arrow {
                        has_parameters: true,
                    } if comment.kind == CommentKind::Line
                        || matches!(position, SuppressionPosition::EndOfLine) =>
                    {
                        needs_first_statement = true;
                    }
                    _ => modes.body = ContentMode::Verbatim,
                }
            }
        }
        if needs_first_statement {
            let ast = self.ast;
            modes.first_statement = body
                .body(ast)
                .iter()
                .map(|id| ast.get_node_in_sub_range(id))
                .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)))
                .map(|statement| IncomingStatement {
                    span: statement.span(ast),
                    mode: ContentMode::Verbatim,
                });
        }
        if modes.preceding == ContentMode::Verbatim
            || modes.body == ContentMode::Verbatim
            || modes.first_statement.is_some()
        {
            modes.constraint = self.source_line_constraint(end, body_start);
        }
        modes
    }

    pub(super) fn format_function_signature(
        &mut self,
        signature: SourceFunctionSignature<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<SourceFunctionBody<'ast>, FormatError> {
        self.format_source_parameters(signature.parameters, writer)?;
        Ok(signature.body)
    }

    pub(super) fn format_function_body_boundary(
        &mut self,
        source: SourceFunctionBody<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceFunctionBody {
            body,
            parameter_end,
            comments,
            mode,
            first_statement,
            constraint,
        } = source;
        self.comments.claim(comments)?;
        self.format_comment_run(
            comments,
            CommentRunLayout::SourceAfter(parameter_end),
            writer,
        )?;
        if comments
            .last()
            .is_some_and(|comment| comment.kind == CommentKind::Line)
        {
            writer.hard_line();
        } else {
            constraint.write(writer, FlatSeparator::Space, Writer::space);
        }
        self.format_function_body(body, mode, first_statement, writer)
    }
}
