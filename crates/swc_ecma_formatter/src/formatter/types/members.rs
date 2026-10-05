//! Real type members share a body while retaining their key, signature, and tail owners.

use swc_next_ecma_ast::{
    FormalParameters, GetSpan, PropertyKey, Span, SubRange, TsCallSignatureDeclaration,
    TsConstructSignatureDeclaration, TsIndexSignature, TsIndexSignatureParameter,
    TsMethodSignature, TsMethodSignatureKind, TsPropertySignature, TsThisParameter,
    TsTypeAnnotation, TsTypeElement, TsTypeElementData, TsTypeLiteral, TsTypeParameterDeclaration,
};

use super::callable::{
    AnnotationContent, CallableParameter, CallableParameters, SignatureReturn,
    SignatureReturnDelimiter,
};
use super::generics::TypeParameterContext;
use super::{Formatter, PreparedType, TypeBoundaryLayout, TypeContext, TypeGap};
use crate::formatter::comments::{CommentBoundary, CommentGap, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::expressions::objects::PropertyKeyContext;
use crate::formatter::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
};
use crate::{FormatError, Semicolons, document::Writer};

struct MemberKey<'ast> {
    node: PropertyKey,
    computed: Option<Span>,
    end: u32,
    mode: ContentMode,
    prefix: Option<(&'static str, TypeGap<'ast>)>,
}

struct PropertyMember<'ast> {
    key: MemberKey<'ast>,
    optional: bool,
    annotation: Option<(PreparedType, TypeBoundaryLayout<'ast>)>,
    ending: Option<TypeGap<'ast>>,
}

#[derive(Clone, Copy)]
enum MemberSignatureOwner {
    Method(TsMethodSignature),
    Call(TsCallSignatureDeclaration),
    Construct(TsConstructSignatureDeclaration),
}

struct MemberSignatureParts {
    parameters: FormalParameters,
    receiver: Option<TsThisParameter>,
    generics: Option<TsTypeParameterDeclaration>,
    result: Option<TsTypeAnnotation>,
}

impl MemberSignatureOwner {
    fn parts(self, ast: &swc_next_ecma_ast::Ast<'_>) -> MemberSignatureParts {
        match self {
            Self::Method(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: node.this_param(ast),
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
            Self::Call(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: node.this_param(ast),
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
            Self::Construct(node) => MemberSignatureParts {
                parameters: node.params(ast),
                receiver: None,
                generics: node.type_parameters(ast),
                result: node.return_type(ast),
            },
        }
    }
}

#[derive(Clone, Copy)]
enum SignatureIncoming {
    Parameter(CallableParameter),
    Return(TsTypeAnnotation),
    Empty,
}

struct SignatureMember<'ast> {
    key: Option<MemberKey<'ast>>,
    construct: bool,
    optional: bool,
    prefix: TypeGap<'ast>,
    generics: Option<(TsTypeParameterDeclaration, ContentMode, TypeGap<'ast>)>,
    parameters: CallableParameters,
    result: Option<SignatureReturn<'ast>>,
}

struct IndexMember<'ast> {
    node: TsIndexSignature,
    opening: u32,
    closing: u32,
    prefix: Option<TypeGap<'ast>>,
    closing_gap: TypeGap<'ast>,
    source_head: Option<TsIndexSignatureParameter>,
    source_tail: Option<TsIndexSignatureParameter>,
    result: SignatureReturn<'ast>,
}

enum MemberContent<'ast> {
    Source(Span),
    Property(PropertyMember<'ast>),
    Signature(SignatureMember<'ast>),
    Index(IndexMember<'ast>),
}

#[derive(Clone, Copy)]
struct MemberHandle {
    node: TsTypeElement,
    data: TsTypeElementData,
}

impl MemberHandle {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        self.node.span(ast)
    }
}

#[derive(Clone, Copy)]
pub(super) enum TypeMemberBodyKind {
    Literal,
    Interface,
}

#[derive(Clone, Copy)]
pub(super) struct IncomingTypeMember {
    pub(super) node: TsTypeElement,
    pub(super) mode: ContentMode,
}

#[derive(Clone, Copy)]
enum MemberGapRole {
    Opening,
    Between,
}

struct PreparedMember<'ast> {
    content_end: u32,
    separator: Option<Span>,
    content: MemberContent<'ast>,
}

enum MemberGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

enum MemberClosing {
    Source(Span),
    Formatted(CommentGap),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_type_literal(
        &mut self,
        node: TsTypeLiteral,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_type_member_body(
            node.span(self.ast),
            node.members(self.ast),
            TypeMemberBodyKind::Literal,
            None,
            writer,
        )?;
        Ok(None)
    }

    pub(super) fn format_type_member_body(
        &mut self,
        span: Span,
        members: SubRange<TsTypeElement>,
        kind: TypeMemberBodyKind,
        incoming: Option<IncomingTypeMember>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let content = Span::new(span.start + 1, span.end - 1);
        let mut members = members
            .iter()
            .map(|id| {
                let node = ast.get_node_in_sub_range(id);
                MemberHandle {
                    node,
                    data: ast.ts_type_element_data(node),
                }
            })
            .peekable();
        let Some(first) = members.peek().copied() else {
            let comments = self.comments.in_range(content);
            if self.content_mode(comments) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            if comments.is_empty() {
                writer.text("{}");
                return Ok(());
            }
            let boundary = self.prepare_comment_boundary(comments, content.start, content.end);
            return writer.group(matches!(kind, TypeMemberBodyKind::Interface), |writer| {
                writer.text("{");
                writer.indent(|writer| self.format_comment_boundary(&boundary, writer))?;
                boundary.gap().write(
                    writer,
                    if self.options.bracket_spacing {
                        FlatSeparator::Space
                    } else {
                        FlatSeparator::Empty
                    },
                    |writer| {
                        if matches!(kind, TypeMemberBodyKind::Interface) {
                            writer.hard_line();
                        } else if self.options.bracket_spacing {
                            writer.line();
                        } else {
                            writer.soft_line();
                        }
                    },
                );
                writer.text("}");
                Ok(())
            });
        };
        let first_start = first.span(ast).start;
        let leading_span = Span::new(content.start, first_start);
        let leading_comments = self.comments.in_range(leading_span);
        let mut mode = incoming
            .filter(|member| member.node == first.node)
            .map_or_else(|| self.content_mode(leading_comments), |member| member.mode);
        let leading = self.prepare_member_gap(leading_span);
        let expanded = matches!(kind, TypeMemberBodyKind::Interface)
            || self.line_breaks(content.start, first_start) != 0;
        writer.group(expanded, |writer| {
            writer.text("{");
            let closing = writer.indent(|writer| {
                self.format_member_gap(leading, MemberGapRole::Opening, kind, writer)?;
                while let Some(node) = members.next() {
                    let span = node.span(ast);
                    let next = members.peek().copied();
                    let next_start = next.map_or(content.end, |node| node.span(ast).start);
                    let suppression = self.delimited_suppression(
                        span,
                        next_start,
                        if next.is_some() {
                            DelimitedFollowing::Node
                        } else {
                            DelimitedFollowing::End
                        },
                    );
                    if suppression
                        .as_ref()
                        .is_some_and(|source| source.preceding.is_some())
                    {
                        mode = ContentMode::Verbatim;
                    }
                    let prepared = self.prepare_type_member(node, mode)?;
                    let source = matches!(prepared.content, MemberContent::Source(_));
                    let gap_span = Span::new(prepared.content_end, next_start);
                    // An ignored member without a source separator relies on the
                    // original newline. Adding a separator would change its raw owner.
                    let gap = if source && prepared.separator.is_none() && next.is_some() {
                        MemberGap::Source(gap_span)
                    } else {
                        self.prepare_member_gap(gap_span)
                    };
                    let source_gap = matches!(gap, MemberGap::Source(_));
                    let required_separator = self.options.semicolons == Semicolons::AsNeeded
                        && self.type_member_requires_separator(node, next);
                    let tail = self.format_type_member(prepared, writer)?;
                    self.format_expression_tail(tail, writer)?;
                    if !source && !source_gap {
                        if required_separator
                            || next.is_some() && self.options.semicolons == Semicolons::Always
                        {
                            writer.text(";");
                        } else if self.options.semicolons == Semicolons::Always {
                            writer.if_break(";");
                        } else if next.is_some() {
                            writer.if_flat(";");
                        }
                    }
                    if next.is_none() {
                        return match gap {
                            MemberGap::Source(span) => Ok(MemberClosing::Source(span)),
                            MemberGap::Formatted(boundary) => {
                                self.format_comment_boundary(&boundary, writer)?;
                                Ok(MemberClosing::Formatted(boundary.gap()))
                            }
                        };
                    }
                    self.format_member_gap(gap, MemberGapRole::Between, kind, writer)?;
                    mode = suppression.map_or(ContentMode::Formatted, |source| source.following);
                }
                unreachable!("a nonempty type body retains its final member")
            })?;
            match closing {
                MemberClosing::Source(span) => self.format_verbatim(span, writer)?,
                MemberClosing::Formatted(gap) => gap.write(
                    writer,
                    if self.options.bracket_spacing {
                        FlatSeparator::Space
                    } else {
                        FlatSeparator::Empty
                    },
                    |writer| {
                        if matches!(kind, TypeMemberBodyKind::Interface) {
                            writer.hard_line();
                        } else if self.options.bracket_spacing {
                            writer.line();
                        } else {
                            writer.soft_line();
                        }
                    },
                ),
            }
            writer.text("}");
            Ok(())
        })
    }

    fn prepare_member_gap(&self, gap: Span) -> MemberGap<'ast> {
        let comments = self.comments.in_range(gap);
        if self.content_mode(comments) == ContentMode::Verbatim {
            MemberGap::Source(gap)
        } else {
            MemberGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
        }
    }

    fn format_member_gap(
        &mut self,
        gap: MemberGap<'ast>,
        role: MemberGapRole,
        kind: TypeMemberBodyKind,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let spaced = self.options.bracket_spacing || matches!(role, MemberGapRole::Between);
        let flat = if spaced {
            FlatSeparator::Space
        } else {
            FlatSeparator::Empty
        };
        match gap {
            MemberGap::Source(span) => self.format_verbatim(span, writer),
            MemberGap::Formatted(boundary) => {
                self.format_comment_boundary(&boundary, writer)?;
                boundary.gap().write(writer, flat, |writer| {
                    if matches!(kind, TypeMemberBodyKind::Interface) {
                        writer.hard_line();
                    } else if spaced {
                        writer.line();
                    } else {
                        writer.soft_line();
                    }
                });
                Ok(())
            }
        }
    }

    fn type_member_requires_separator(
        &self,
        node: MemberHandle,
        next: Option<MemberHandle>,
    ) -> bool {
        let TsTypeElementData::TsPropertySignature(property) = node.data else {
            return false;
        };
        if property.type_annotation(self.ast).is_some() || property.computed(self.ast) {
            return false;
        }
        if next.is_some_and(|next| {
            matches!(next.data, TsTypeElementData::TsCallSignatureDeclaration(_))
        }) {
            return true;
        }
        self.property_key_identifier(
            property.key(self.ast),
            PropertyKeyContext::Ordinary,
            ContentMode::Formatted,
        )
        .is_some_and(|name| matches!(name, "get" | "set" | "static"))
    }

    fn prepare_type_member(
        &self,
        handle: MemberHandle,
        mode: ContentMode,
    ) -> Result<PreparedMember<'ast>, FormatError> {
        let ast = self.ast;
        let node = handle.node;
        let span = node.span(ast);
        let separator = self
            .source(span)
            .as_bytes()
            .last()
            .filter(|&&byte| matches!(byte, b';' | b','))
            .map(|_| Span::new(span.end - 1, span.end));
        if mode == ContentMode::Verbatim {
            return Ok(PreparedMember {
                content_end: span.end,
                separator,
                content: MemberContent::Source(span),
            });
        }
        let data = handle.data;
        let (content, content_end) = match data {
            TsTypeElementData::TsPropertySignature(property) => {
                let key = self.prepare_member_key(
                    property.key(ast),
                    property.computed(ast),
                    span,
                    property
                        .type_annotation(ast)
                        .map_or(span.end, |annotation| annotation.span(ast).start),
                    property.readonly(ast).then_some("readonly"),
                )?;
                let end = if let Some(annotation) = property.type_annotation(ast) {
                    annotation.span(ast).end
                } else if property.optional(ast) {
                    let gap = Span::new(key.end, span.end);
                    let Some(optional) = self.find_token(gap, self.comments.in_range(gap), "?")
                    else {
                        return Err(self.unsupported("type property optional marker", property));
                    };
                    optional + 1
                } else {
                    key.end
                };
                let member = self.prepare_property_member(property, key, end)?;
                (MemberContent::Property(member), end)
            }
            TsTypeElementData::TsMethodSignature(method) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Method(method))?
            }
            TsTypeElementData::TsCallSignatureDeclaration(call) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Call(call))?
            }
            TsTypeElementData::TsConstructSignatureDeclaration(construct) => {
                self.prepare_signature_member(node, MemberSignatureOwner::Construct(construct))?
            }
            TsTypeElementData::TsIndexSignature(index) => {
                let end = index.type_annotation(ast).span(ast).end;
                (MemberContent::Index(self.prepare_index_member(index)?), end)
            }
        };
        let mut prepared = PreparedMember {
            content_end,
            separator,
            content,
        };
        if !self.has_suppression {
            return Ok(prepared);
        }
        let trailing = self.comments.in_range(Span::new(content_end, span.end));
        if self.content_mode(trailing) == ContentMode::Verbatim {
            match &mut prepared.content {
                MemberContent::Property(property) => match &mut property.annotation {
                    Some((node, _)) => node.source_owner = Some(node.node),
                    None => prepared.content = MemberContent::Source(span),
                },
                MemberContent::Signature(signature) => {
                    match signature.result.as_mut().map(|result| &mut result.content) {
                        Some(AnnotationContent::Type(node)) => node.source_owner = Some(node.node),
                        Some(AnnotationContent::Wrapper(_)) => {}
                        None => prepared.content = MemberContent::Source(span),
                    }
                }
                MemberContent::Index(index) => {
                    if let AnnotationContent::Type(node) = &mut index.result.content {
                        node.source_owner = Some(node.node);
                    }
                }
                MemberContent::Source(_) => {}
            }
            if matches!(prepared.content, MemberContent::Source(_)) {
                prepared.content_end = span.end;
            }
        }
        Ok(prepared)
    }

    fn prepare_member_key(
        &self,
        node: PropertyKey,
        computed: bool,
        span: Span,
        following_start: u32,
        keyword: Option<&'static str>,
    ) -> Result<MemberKey<'ast>, FormatError> {
        let key = node.span(self.ast);
        let computed = if computed {
            let before = Span::new(span.start, key.start);
            let after = Span::new(key.end, following_start);
            let Some(opening) = self.find_token(before, self.comments.in_range(before), "[") else {
                return Err(self.unsupported("type member computed opening", node));
            };
            let Some(closing) = self.find_token(after, self.comments.in_range(after), "]") else {
                return Err(self.unsupported("type member computed closing", node));
            };
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        let start = computed.map_or(key.start, |span| span.start);
        if keyword.is_none() && span.start != start {
            return Err(self.unsupported("type member modifier", node));
        }
        let prefix = keyword.map(|keyword| {
            let gap = Span::new(span.start + keyword.len() as u32, start);
            (
                keyword,
                self.prepare_type_gap(
                    gap,
                    self.has_suppression
                        && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim,
                ),
            )
        });
        let mut mode = ContentMode::Formatted;
        if self.has_suppression
            && let Some((_, TypeGap::Source(_))) = &prefix
        {
            mode = ContentMode::Verbatim;
        }
        Ok(MemberKey {
            node,
            computed,
            end: computed.map_or(key.end, |span| span.end),
            mode,
            prefix,
        })
    }

    fn prepare_property_member(
        &self,
        node: TsPropertySignature,
        mut key: MemberKey<'ast>,
        content_end: u32,
    ) -> Result<PropertyMember<'ast>, FormatError> {
        let ast = self.ast;
        let annotation = node
            .type_annotation(ast)
            .map(|annotation| {
                let mut type_node =
                    self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted);
                let gap = Span::new(key.end, type_node.node.span(ast).start);
                let comments = self.comments.in_range(gap);
                let Some(colon) = self.find_token(gap, comments, ":") else {
                    return Err(self.unsupported("type property colon", node));
                };
                let (before, after) = comments
                    .split_at(comments.partition_point(|comment| comment.span.end <= colon));
                if self.has_suppression {
                    let before = if key.computed.is_some() {
                        self.comments
                            .in_range(Span::new(key.node.span(ast).end, colon))
                    } else {
                        before
                    };
                    for (_, position) in self.suppressed_comment_runs(before, colon) {
                        if matches!(position, SuppressionPosition::OwnLine) {
                            self.select_type_head(&mut type_node, position);
                        } else {
                            key.mode = ContentMode::Verbatim;
                        }
                    }
                    self.select_type_head_comments(&mut type_node, after, gap.end);
                }
                let prefix =
                    if key.mode == ContentMode::Verbatim || type_node.source_owner.is_some() {
                        TypeBoundaryLayout::Source(gap)
                    } else {
                        TypeBoundaryLayout::Formatted {
                            before: self.prepare_comment_boundary(before, gap.start, colon),
                            after: self.prepare_comment_boundary(after, colon + 1, gap.end),
                        }
                    };
                Ok((type_node, prefix))
            })
            .transpose()?;
        let ending = if annotation.is_none() {
            let gap = Span::new(key.end, content_end - u32::from(node.optional(ast)));
            let source = self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
            if source {
                key.mode = ContentMode::Verbatim;
            }
            if self.has_suppression
                && let Some(computed) = key.computed
            {
                let comments = self
                    .comments
                    .in_range(Span::new(key.node.span(ast).end, computed.end - 1));
                if self.content_mode(comments) == ContentMode::Verbatim {
                    key.mode = ContentMode::Verbatim;
                }
            }
            Some(self.prepare_type_gap(gap, source))
        } else {
            None
        };
        Ok(PropertyMember {
            key,
            optional: node.optional(ast),
            annotation,
            ending,
        })
    }

    fn prepare_signature_member(
        &self,
        node: TsTypeElement,
        owner: MemberSignatureOwner,
    ) -> Result<(MemberContent<'ast>, u32), FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let parts = owner.parts(ast);
        let parameter_span = parts.parameters.span(ast);
        if self.has_suppression
            && parts.receiver.is_none()
            && parts.parameters.items(ast).is_empty()
            && parts.parameters.rest(ast).is_none()
            && self.content_mode(
                self.comments
                    .in_range(Span::new(parameter_span.start + 1, parameter_span.end - 1)),
            ) == ContentMode::Verbatim
        {
            return Ok((MemberContent::Source(span), span.end));
        }
        let head_end = parts
            .generics
            .map_or(parameter_span.start, |node| node.span(ast).start);
        let mut key = match owner {
            MemberSignatureOwner::Method(method) => {
                let keyword = match method.kind(ast) {
                    TsMethodSignatureKind::Method => None,
                    TsMethodSignatureKind::Get => Some("get"),
                    TsMethodSignatureKind::Set => Some("set"),
                };
                Some(self.prepare_member_key(
                    method.key(ast),
                    method.computed(ast),
                    span,
                    head_end,
                    keyword,
                )?)
            }
            _ => None,
        };
        let optional =
            matches!(owner, MemberSignatureOwner::Method(method) if method.optional(ast));
        let construct = matches!(owner, MemberSignatureOwner::Construct(_));
        let prefix_end = key
            .as_ref()
            .map_or(span.start + if construct { 3 } else { 0 }, |key| key.end);
        let head_gap = Span::new(prefix_end, head_end);
        let mut generic_mode = ContentMode::Formatted;
        let mut parameters = self.callable_parameters(parts.parameters, parts.receiver);
        let mut source_return = None;
        let mut incoming = None;
        let head_source = self.has_suppression
            && self.content_mode(self.comments.in_range(head_gap)) == ContentMode::Verbatim;
        if head_source {
            for (_, position) in
                self.suppressed_comment_runs(self.comments.in_range(head_gap), head_end)
            {
                if key.is_some() && matches!(position, SuppressionPosition::EndOfLine) {
                    if let Some(key) = &mut key {
                        key.mode = ContentMode::Verbatim;
                    }
                } else if parts.generics.is_some() {
                    generic_mode = ContentMode::Verbatim;
                } else {
                    match self.signature_incoming(&parameters, parts.result, &mut incoming)? {
                        SignatureIncoming::Parameter(first) => parameters.source_head = Some(first),
                        SignatureIncoming::Return(annotation) => source_return = Some(annotation),
                        SignatureIncoming::Empty => {
                            if let Some(key) = &mut key {
                                key.mode = ContentMode::Verbatim;
                            } else {
                                return Ok((MemberContent::Source(span), span.end));
                            }
                        }
                    }
                }
            }
        }
        let prefix = self.prepare_type_gap(head_gap, head_source);
        let generics = parts
            .generics
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameter_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            match self.signature_incoming(
                                &parameters,
                                parts.result,
                                &mut incoming,
                            )? {
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
                Ok((generic, generic_mode, self.prepare_type_gap(gap, source)))
            })
            .transpose()?;
        let result = parts.result.map(|annotation| {
            self.prepare_signature_return(
                parameter_span.end,
                annotation,
                source_return,
                SignatureReturnDelimiter::Colon,
            )
        });
        self.prepare_callable_parameter_tail(
            &mut parameters,
            result
                .as_ref()
                .map_or(ContentMode::Formatted, |result| result.preceding_mode),
        )?;
        let content_end = parts
            .result
            .map_or(parameter_span.end, |annotation| annotation.span(ast).end);
        Ok((
            MemberContent::Signature(SignatureMember {
                key,
                construct,
                optional,
                prefix,
                generics,
                parameters,
                result,
            }),
            content_end,
        ))
    }

    fn signature_incoming(
        &self,
        parameters: &CallableParameters,
        result: Option<TsTypeAnnotation>,
        selected: &mut Option<SignatureIncoming>,
    ) -> Result<SignatureIncoming, FormatError> {
        if let Some(owner) = *selected {
            return Ok(owner);
        }
        let owner = if let Some(first) = self.first_callable_parameter(parameters)? {
            SignatureIncoming::Parameter(first)
        } else if let Some(annotation) = result {
            SignatureIncoming::Return(annotation)
        } else {
            SignatureIncoming::Empty
        };
        *selected = Some(owner);
        Ok(owner)
    }

    fn prepare_index_member(
        &self,
        node: TsIndexSignature,
    ) -> Result<IndexMember<'ast>, FormatError> {
        let ast = self.ast;
        if node.r#static(ast) {
            return Err(self.unsupported("static index signature", node));
        }
        let span = node.span(ast);
        let annotation = node.type_annotation(ast);
        let Some(first_id) = node.parameters(ast).first() else {
            return Err(self.unsupported("empty index signature", node));
        };
        let first = ast.get_node_in_sub_range(first_id);
        let last = node
            .parameters(ast)
            .last()
            .map(|id| ast.get_node_in_sub_range(id));
        let before = Span::new(span.start, first.span(ast).start);
        let Some(opening) = self.find_token(before, self.comments.in_range(before), "[") else {
            return Err(self.unsupported("index signature opening", node));
        };
        let last_end = last.map_or(first.span(ast).end, |node| node.span(ast).end);
        let after = Span::new(last_end, annotation.span(ast).start);
        let Some(closing) = self.find_token(after, self.comments.in_range(after), "]") else {
            return Err(self.unsupported("index signature closing", node));
        };
        let prefix = node.readonly(ast).then(|| {
            let gap = Span::new(span.start + 8, opening);
            self.prepare_type_gap(
                gap,
                self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim,
            )
        });
        let source_head = if self.has_suppression {
            let start = if prefix.is_some() {
                span.start + 8
            } else {
                opening + 1
            };
            (self.content_mode(
                self.comments
                    .in_range(Span::new(start, first.span(ast).start)),
            ) == ContentMode::Verbatim)
                .then_some(first)
        } else {
            None
        };
        let closing_span = Span::new(last_end, closing);
        let closing_comments = self.comments.in_range(closing_span);
        let mut source_return = None;
        let mut source_tail = None;
        for (_, position) in self.suppressed_comment_runs(closing_comments, closing) {
            if matches!(position, SuppressionPosition::OwnLine) {
                source_return = Some(annotation);
            } else {
                source_tail = last;
            }
        }
        let closing_gap = self.prepare_type_gap(
            closing_span,
            self.content_mode(closing_comments) == ContentMode::Verbatim,
        );
        let result = self.prepare_signature_return(
            closing + 1,
            annotation,
            source_return,
            SignatureReturnDelimiter::Colon,
        );
        if result.preceding_mode == ContentMode::Verbatim {
            source_tail = last;
        }
        Ok(IndexMember {
            node,
            opening,
            closing,
            prefix,
            closing_gap,
            source_head,
            source_tail,
            result,
        })
    }

    fn format_type_member_key(
        &mut self,
        key: MemberKey<'ast>,
        context: PropertyKeyContext,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if let Some((keyword, gap)) = key.prefix {
            writer.text(keyword);
            self.format_type_gap(gap, FlatSeparator::Space, writer)?;
        }
        self.format_property_key_context(key.node, key.computed, key.mode, context, writer)
    }

    fn format_type_member(
        &mut self,
        member: PreparedMember<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match member.content {
            MemberContent::Source(span) => {
                self.format_verbatim(span, writer)?;
                Ok(None)
            }
            MemberContent::Property(property) => {
                let source_prefix = property
                    .annotation
                    .as_ref()
                    .is_some_and(|(_, prefix)| matches!(prefix, TypeBoundaryLayout::Source(_)));
                self.format_type_member_key(property.key, PropertyKeyContext::Ordinary, writer)?;
                if let Some((node, prefix)) = property.annotation {
                    if property.optional && !source_prefix {
                        writer.text("?");
                    }
                    match prefix {
                        TypeBoundaryLayout::Source(span) => self.format_verbatim(span, writer)?,
                        TypeBoundaryLayout::Formatted { before, after } => {
                            self.format_type_comment_gap(&before, FlatSeparator::Empty, writer)?;
                            writer.text(":");
                            self.format_type_comment_gap(&after, FlatSeparator::Space, writer)?;
                        }
                    }
                    self.format_prepared_type(node, TypeContext::Root, writer)
                } else {
                    if let Some(ending) = property.ending {
                        self.format_type_gap(ending, FlatSeparator::Empty, writer)?;
                    }
                    if property.optional {
                        writer.text("?");
                    }
                    Ok(None)
                }
            }
            MemberContent::Signature(signature) => writer.group(false, |writer| {
                if let Some(key) = signature.key {
                    self.format_type_member_key(key, PropertyKeyContext::TypeMethod, writer)?;
                } else if signature.construct {
                    writer.text("new");
                }
                if signature.optional && matches!(signature.prefix, TypeGap::Formatted(_)) {
                    writer.text("?");
                }
                self.format_type_gap(
                    signature.prefix,
                    if signature.construct {
                        FlatSeparator::Space
                    } else {
                        FlatSeparator::Empty
                    },
                    writer,
                )?;
                if let Some((node, mode, gap)) = signature.generics {
                    self.format_type_parameters(
                        node,
                        TypeParameterContext::Callable,
                        mode,
                        writer,
                    )?;
                    self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
                }
                self.format_callable_parameters(signature.parameters, writer)?;
                match signature.result {
                    Some(result) => self.format_signature_return(result, writer),
                    None => Ok(None),
                }
            }),
            MemberContent::Index(index) => self.format_index_member(index, writer),
        }
    }

    fn format_index_member(
        &mut self,
        index: IndexMember<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        if let Some(prefix) = index.prefix {
            writer.text("readonly");
            self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
        }
        writer.group(false, |writer| {
            writer.text("[");
            writer.indent(|writer| {
                let mut members = index
                    .node
                    .parameters(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id))
                    .peekable();
                let mut end = index.opening + 1;
                let mut following_mode = None;
                let mut separator = Separator::Line;
                while let Some(node) = members.next() {
                    let span = node.span(ast);
                    if following_mode.is_some() {
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, span.start),
                            writer,
                        );
                    }
                    let following = members.peek().map_or(
                        FollowingItem::ResolvedTerminal(DelimitedSuppression {
                            preceding: index.source_tail.map(|node| node.span(ast)),
                            following: ContentMode::Formatted,
                        }),
                        |next| FollowingItem::Node(next.span(ast)),
                    );
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, index.closing),
                        false,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if index.source_head == Some(node) || index.source_tail == Some(node) {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let tail = if item.mode == ContentMode::Verbatim {
                        self.format_verbatim(span, writer)?;
                        None
                    } else {
                        let name = node.name(ast);
                        let annotation = self.prepare_annotation_content(
                            name.span(ast).end,
                            node.type_annotation(ast),
                        )?;
                        writer.text(ast.get_utf8(name.name(ast)));
                        self.format_annotation_content(annotation, writer)?
                    };
                    let boundary = if members.peek().is_some() {
                        Some(self.finish_delimited_item(item, tail, writer)?)
                    } else {
                        self.format_expression_tail(tail, writer)?;
                        None
                    };
                    if let Some(boundary) = boundary {
                        end = boundary.end;
                        following_mode = Some(boundary.following_mode);
                        separator = boundary.separator;
                    }
                }
                self.format_type_gap(index.closing_gap, FlatSeparator::Empty, writer)
            })?;
            writer.text("]");
            self.format_signature_return(index.result, writer)
        })
    }
}
