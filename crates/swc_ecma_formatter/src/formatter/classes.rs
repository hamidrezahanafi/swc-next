//! Class heads and members retain real keys, signatures, and source terminators.

use swc_next_ecma_ast::{
    Accessibility, BindingIdentifier, Class, ClassBody, ClassElement, ClassElementData, Comment,
    CommentKind, Expr, GetSpan, MethodDefinitionKind, PropertyDefinition, PropertyKey, Span,
    StaticBlock, StmtData, TsFunction, TsIndexSignature, TsMethodDefinition,
    TsTypeParameterDeclaration,
};

use super::Formatter;
use super::bindings::{InitializerLayout, InitializerLeft, PreparedInitializer};
use super::comments::{CommentGap, CommentRunLayout, ExpressionTail, NodeHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::objects::PropertyKeyContext;
use super::expressions::{
    AccessKind, ExpressionContext, ExpressionPosition, ExpressionTypeArguments, TailOwner,
};
use super::functions::bodyless::PreparedBodylessSignature;
use super::functions::{MethodOwner, SignaturePreceding, SourceMethodHeader, SourceMethodPrefix};
use super::statements::IncomingStatement;
use super::suppression::{
    ContentMode, DelimitedFollowing, DelimitedSuppression, SuppressionPosition,
    is_suppression_comment,
};
use super::types::PreparedIndexSignature;
use super::types::generics::{TypeListKind, TypeListPrefix, TypeParameterContext};
use crate::{FormatError, Semicolons, document::Writer};

struct SourceClassHead<'ast> {
    name: Option<BindingIdentifier>,
    name_mode: ContentMode,
    parameters: Option<SourceClassParameters<'ast>>,
    heritage: Option<SourceClassHeritage<'ast>>,
    body: ClassBody,
    body_mode: ContentMode,
    incoming: Option<IncomingClassMember>,
    end: u32,
    body_comments: &'ast [Comment],
    body_gap: CommentGap,
}

struct SourceClassParameters<'ast> {
    node: TsTypeParameterDeclaration,
    prefix: TypeListPrefix<'ast>,
    mode: ContentMode,
}

#[derive(Clone, Copy)]
struct IncomingClassMember {
    node: ClassElement,
    mode: ContentMode,
}

struct SourceClassHeritage<'ast> {
    preceding_end: u32,
    keyword: u32,
    expression: Expr,
    mode: ContentMode,
    boundary: Option<Span>,
    arguments: Option<ExpressionTypeArguments<'ast>>,
}

struct PreparedClassMember<'ast> {
    node: ClassElement,
    content: ClassMemberContent<'ast>,
    sibling: Option<DelimitedSuppression>,
}

enum ClassMemberContent<'ast> {
    Verbatim,
    Method(SourceMethodHeader<'ast>),
    Bodyless(SourceBodylessClassMethod<'ast>),
    BodylessValue {
        prefix: SourceMethodPrefix,
        value: TsFunction,
    },
    Field(SourceClassField<'ast>),
    Static(SourceStaticBlock<'ast>),
    Index(SourceClassIndexSignature<'ast>),
}

struct SourceBodylessClassMethod<'ast> {
    prefix: SourceMethodPrefix,
    signature: PreparedBodylessSignature<'ast>,
    terminal: TypedClassMemberTerminal,
}

struct SourceClassIndexSignature<'ast> {
    signature: PreparedIndexSignature<'ast>,
    terminal: TypedClassMemberTerminal,
}

enum TypedClassMemberTerminal {
    Canonical(Span),
    Source(Span),
}

struct SourceClassField<'ast> {
    node: PropertyDefinition,
    key: PropertyKey,
    computed: Option<Span>,
    key_start: u32,
    key_mode: ContentMode,
    static_end: Option<u32>,
    initializer: Option<PreparedInitializer<'ast>>,
    terminal: FieldTerminal,
}

enum FieldTerminal {
    Canonical { span: Span, semicolon: bool },
    Source(Span),
}

struct SourceStaticBlock<'ast> {
    node: StaticBlock,
    braces: Span,
    incoming: Option<IncomingStatement>,
    whole: bool,
    comments: &'ast [Comment],
    gap: CommentGap,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_class(
        &mut self,
        class: Class,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let head = self.source_class_head(class)?;
        let SourceClassHead {
            name,
            name_mode,
            parameters,
            heritage,
            body,
            body_mode,
            incoming,
            end,
            body_comments,
            body_gap,
        } = head;
        let keyword_end = class.span(self.ast).start + 5;
        writer.text("class");
        if let Some(name) = name {
            self.format_source_token_gap(keyword_end, name.span(self.ast).start, true, writer)?;
            if name_mode == ContentMode::Verbatim {
                self.format_verbatim(name.span(self.ast), writer)?;
            } else {
                writer.text(self.ast.get_utf8(name.name(self.ast)));
            }
        }
        if let Some(parameters) = parameters {
            self.format_type_list_prefix(parameters.prefix, writer)?;
            self.format_type_parameters(
                parameters.node,
                TypeParameterContext::Class,
                parameters.mode,
                writer,
            )?;
        }
        if let Some(SourceClassHeritage {
            preceding_end,
            keyword,
            expression,
            mode,
            boundary,
            arguments,
        }) = heritage
        {
            self.format_source_token_gap(preceding_end, keyword, true, writer)?;
            writer.text("extends");
            let span = expression.span(self.ast);
            let context =
                ExpressionContext::at(ExpressionPosition::Superclass).with_content_mode(mode);
            let context = if arguments.is_some() {
                context.with_tail_owner(TailOwner::ParentOperation)
            } else {
                context
            };
            let tail = if let Some(boundary) = boundary {
                let source =
                    self.format_delimited_source_gap(boundary, DelimitedFollowing::Node, writer)?;
                source
                    .following
                    .write(writer, FlatSeparator::Space, Writer::space);
                self.format_expression(expression, context, writer)?
            } else {
                let comments = self.comments.between(Span::new(keyword + 7, span.start))?;
                let first = comments
                    .first()
                    .map_or(span.start, |comment| comment.span.start);
                self.line_constraint(keyword + 7, first).write(
                    writer,
                    FlatSeparator::Space,
                    Writer::space,
                );
                self.comments.claim(comments)?;
                self.format_expression_with_leading_comments(
                    expression,
                    NodeHead {
                        comments,
                        content_start: span.start,
                        layout: CommentRunLayout::FlatLeading(span.start),
                        gap_end: None,
                    },
                    context,
                    writer,
                )?
            };
            if let Some(arguments) = arguments {
                self.format_expression_type_arguments(arguments, tail, writer)?;
            } else {
                self.format_expression_tail(tail, writer)?;
            }
        }
        self.comments.claim(body_comments)?;
        self.format_comment_run(body_comments, CommentRunLayout::SourceAfter(end), writer)?;
        body_gap.write(writer, FlatSeparator::Space, Writer::space);
        self.format_class_body(body, body_mode, incoming, writer)
    }

    fn source_class_head(&self, class: Class) -> Result<SourceClassHead<'ast>, FormatError> {
        let ast = self.ast;
        if !class.decorators(ast).is_empty()
            || !class.implements(ast).is_empty()
            || class.r#abstract(ast)
            || class.declare(ast)
        {
            return Err(self.unsupported("typed or decorated class", class));
        }
        let body = class.body(ast);
        let name = class.id(ast);
        let start = class.span(ast).start + 5;
        let mut name_mode = ContentMode::Formatted;
        if self.has_suppression
            && let Some(name) = name
        {
            name_mode = self.content_mode(
                self.comments
                    .in_range(Span::new(start, name.span(ast).start)),
            );
        }
        let mut end = name.map_or(start, |name| name.span(ast).end);
        let mut parameters = class.type_parameters(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(end, node.span(ast).start),
                TypeListKind::Parameters {
                    has_left_owner: name.is_some(),
                },
            );
            if prefix.left_mode == ContentMode::Verbatim {
                name_mode = ContentMode::Verbatim;
            }
            end = node.span(ast).end;
            SourceClassParameters {
                node,
                mode: prefix.right_mode,
                prefix,
            }
        });
        let heritage = if let Some(expression) = class.super_class(ast) {
            let preceding_end = end;
            let gap = Span::new(preceding_end, expression.span(ast).start);
            let comments = self.comments.in_range(gap);
            let extends = self
                .find_token(gap, comments, "extends")
                .ok_or_else(|| self.unsupported("class extends boundary", class))?;
            let mut mode = ContentMode::Formatted;
            if self.has_suppression {
                let has_preceding = parameters.is_some() || name.is_some();
                let split = comments.partition_point(|comment| comment.span.end <= extends);
                let mut preceding_mode = ContentMode::Formatted;
                if has_preceding && self.content_mode(&comments[..split]) == ContentMode::Verbatim {
                    preceding_mode = ContentMode::Verbatim;
                }
                for position in
                    self.suppression_positions(&comments[split..], expression.span(ast).start)
                {
                    match position {
                        SuppressionPosition::Inline => mode = ContentMode::Verbatim,
                        _ if has_preceding => preceding_mode = ContentMode::Verbatim,
                        _ => mode = ContentMode::Verbatim,
                    }
                }
                if preceding_mode == ContentMode::Verbatim {
                    if let Some(parameters) = &mut parameters {
                        parameters.mode = ContentMode::Verbatim;
                    } else {
                        name_mode = ContentMode::Verbatim;
                    }
                }
            }
            end = expression.span(ast).end;
            let arguments = if let Some(node) = class.super_type_arguments(ast) {
                let arguments =
                    self.prepare_expression_type_arguments(expression, node, AccessKind::Ordinary)?;
                if arguments.callee_mode == ContentMode::Verbatim {
                    mode = ContentMode::Verbatim;
                }
                end = node.span(ast).end;
                Some(arguments)
            } else {
                None
            };
            let boundary = (self.has_suppression
                && self.content_mode(
                    self.comments
                        .in_range(Span::new(extends + 7, expression.span(ast).start)),
                ) == ContentMode::Verbatim)
                .then_some(Span::new(extends + 7, expression.span(ast).start));
            Some(SourceClassHeritage {
                preceding_end,
                keyword: extends,
                expression,
                mode,
                boundary,
                arguments,
            })
        } else {
            None
        };
        let mut body_mode = ContentMode::Formatted;
        let mut incoming = None;
        let body_comments = self.comments.in_range(Span::new(end, body.span(ast).start));
        let last = body_comments.last().map_or(end, |comment| comment.span.end);
        let mut body_constraint = self.line_constraint(last, body.span(ast).start);
        if self.content_mode(body_comments) == ContentMode::Verbatim {
            let comments = body_comments;
            let first = body
                .body(ast)
                .iter()
                .next()
                .map(|id| ast.get_node_in_sub_range(id));
            for position in self.suppression_positions(comments, body.span(ast).start) {
                match position {
                    SuppressionPosition::Inline => body_mode = ContentMode::Verbatim,
                    _ if first.is_some() => {
                        incoming = first.map(|node| IncomingClassMember {
                            node,
                            mode: ContentMode::Verbatim,
                        })
                    }
                    _ => body_mode = ContentMode::Verbatim,
                }
            }
            if body_mode == ContentMode::Verbatim || incoming.is_some() {
                let last = comments.last().map_or(end, |comment| comment.span.end);
                body_constraint = self.source_line_constraint(last, body.span(ast).start);
            }
        }
        Ok(SourceClassHead {
            name,
            name_mode,
            parameters,
            heritage,
            body,
            body_mode,
            incoming,
            end,
            body_comments,
            body_gap: if body_comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                CommentGap::LineComment
            } else {
                CommentGap::Source(body_constraint)
            },
        })
    }

    fn format_class_body(
        &mut self,
        body: ClassBody,
        mode: ContentMode,
        incoming: Option<IncomingClassMember>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = body.span(ast);
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(span, writer);
        }
        let content = Span::new(span.start + 1, span.end - 1);
        let mut members = body
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let Some(first) = members.next() else {
            if self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim {
                return self.format_verbatim(span, writer);
            }
            writer.text("{");
            let comments = self.comments.between(content)?;
            if !comments.is_empty() {
                writer.indent(|writer| {
                    writer.hard_line();
                    self.comments.claim(comments)?;
                    self.format_comment_run(comments, CommentRunLayout::Lines, writer)
                })?;
                writer.hard_line();
            }
            writer.text("}");
            return Ok(());
        };
        let leading = self
            .comments
            .in_range(Span::new(content.start, first.span(ast).start));
        let first_mode = if incoming
            .is_some_and(|target| target.node == first && target.mode == ContentMode::Verbatim)
            || self.content_mode(leading) == ContentMode::Verbatim
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let mut current =
            self.prepare_class_member(first, members.peek().copied(), content.end, first_mode)?;
        writer.text("{");
        let end = writer.indent(|writer| {
            self.opening_line_constraint(content).write(
                writer,
                FlatSeparator::Space,
                Writer::hard_line,
            );
            self.comments.claim(leading)?;
            self.format_comment_run(
                leading,
                CommentRunLayout::Leading(first.span(ast).start),
                writer,
            )?;
            loop {
                let next = members
                    .next()
                    .map(|node| {
                        let mode = current
                            .sibling
                            .as_ref()
                            .map_or(ContentMode::Formatted, |source| source.following);
                        self.prepare_class_member(node, members.peek().copied(), content.end, mode)
                    })
                    .transpose()?;
                let node_span = current.node.span(ast);
                self.format_class_member(current.content, node_span, next.as_ref(), writer)?;
                let next_start = next
                    .as_ref()
                    .map_or(content.end, |next| next.node.span(ast).start);
                let gap = Span::new(node_span.end, next_start);
                let end = if current.sibling.is_some() {
                    let source = self.format_delimited_source_gap(
                        gap,
                        if next.is_some() {
                            DelimitedFollowing::Node
                        } else {
                            DelimitedFollowing::End
                        },
                        writer,
                    )?;
                    if next.is_some() {
                        source
                            .following
                            .write(writer, FlatSeparator::Space, Writer::hard_line);
                    }
                    source.end
                } else {
                    let comments = self.comments.between(gap)?;
                    self.comments.claim(comments)?;
                    self.format_comment_run(
                        comments,
                        CommentRunLayout::SourceAfter(node_span.end),
                        writer,
                    )?;
                    let end = comments
                        .last()
                        .map_or(node_span.end, |comment| comment.span.end);
                    if next.is_some() {
                        let constraint = self.line_constraint(end, next_start);
                        constraint.write(writer, FlatSeparator::Space, Writer::hard_line);
                        if constraint == LineConstraint::Unconstrained
                            && self.next_line_is_empty(node_span.end, next_start)
                        {
                            writer.hard_line();
                        }
                    }
                    end
                };
                let Some(next) = next else {
                    return Ok::<_, FormatError>(end);
                };
                current = next;
            }
        })?;
        self.line_constraint(end, content.end).write(
            writer,
            FlatSeparator::Space,
            Writer::hard_line,
        );
        writer.text("}");
        Ok(())
    }

    fn prepare_class_member(
        &self,
        node: ClassElement,
        following: Option<ClassElement>,
        end: u32,
        mode: ContentMode,
    ) -> Result<PreparedClassMember<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let sibling = self.delimited_suppression(
            span,
            following.map_or(end, |node| node.span(ast).start),
            if following.is_some() {
                DelimitedFollowing::Node
            } else {
                DelimitedFollowing::End
            },
        );
        let mode = if mode == ContentMode::Verbatim
            || sibling
                .as_ref()
                .and_then(|source| source.preceding)
                .is_some()
        {
            ContentMode::Verbatim
        } else {
            ContentMode::Formatted
        };
        let content = if mode == ContentMode::Verbatim {
            ClassMemberContent::Verbatim
        } else {
            match ast.class_element_data(node) {
                ClassElementData::MethodDefinition(method) => ClassMemberContent::Method(
                    self.source_method_header(MethodOwner::Class(method))?,
                ),
                ClassElementData::TsMethodDefinition(method) => {
                    self.source_bodyless_class_method(method, sibling)?
                }
                ClassElementData::PropertyDefinition(field) => {
                    match self.source_class_field(field, sibling)? {
                        Some(field) => ClassMemberContent::Field(field),
                        None => ClassMemberContent::Verbatim,
                    }
                }
                ClassElementData::StaticBlock(block) => {
                    ClassMemberContent::Static(self.source_static_block(block)?)
                }
                ClassElementData::TsIndexSignature(index) => {
                    ClassMemberContent::Index(self.source_class_index_signature(index, sibling)?)
                }
            }
        };
        Ok(PreparedClassMember {
            node,
            content,
            sibling,
        })
    }

    fn format_class_member(
        &mut self,
        content: ClassMemberContent<'ast>,
        span: Span,
        following: Option<&PreparedClassMember<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match content {
            ClassMemberContent::Verbatim => self.format_verbatim(span, writer),
            ClassMemberContent::Method(header) => self.format_method_header(header, writer),
            ClassMemberContent::Bodyless(method) => {
                self.format_bodyless_class_method(method, writer)
            }
            ClassMemberContent::BodylessValue { prefix, value } => {
                let end = prefix.end(self.ast);
                self.format_method_key_prefix(prefix, writer)?;
                self.format_source_token_gap(end, value.span(self.ast).start, false, writer)?;
                self.format_verbatim(value.span(self.ast), writer)
            }
            ClassMemberContent::Field(field) => self.format_class_field(field, following, writer),
            ClassMemberContent::Static(block) => self.format_static_block(block, writer),
            ClassMemberContent::Index(index) => {
                let tail = self.format_index_signature(index.signature, writer)?;
                self.format_typed_class_member_terminal(index.terminal, tail, writer)
            }
        }
    }

    fn source_bodyless_class_method(
        &self,
        node: TsMethodDefinition,
        sibling: Option<DelimitedSuppression>,
    ) -> Result<ClassMemberContent<'ast>, FormatError> {
        let ast = self.ast;
        let function = node.value(ast);
        if node.kind(ast) != MethodDefinitionKind::Method
            || !node.decorators(ast).is_empty()
            || node.r#static(ast)
            || node.r#override(ast)
            || node.optional(ast)
            || node.r#abstract(ast)
            || node.accessibility(ast) != Accessibility::None
            || function.r#async(ast)
            || function.generator(ast)
        {
            return Err(self.unsupported("typed or decorated bodyless class method", node));
        }
        let source = self.source_bodyless_signature(function);
        let mut prefix = self.source_bodyless_method_prefix(node, source.header_start(ast))?;
        if self.bodyless_empty_parameters_are_verbatim(&source) {
            return Ok(ClassMemberContent::BodylessValue {
                prefix,
                value: source.node,
            });
        }
        let signature = self.prepare_bodyless_signature_suffix(
            source,
            prefix.end(ast),
            Some(SignaturePreceding::Key(prefix.key)),
            prefix.key_mode,
        )?;
        prefix.key_mode = signature.preceding_mode;
        let internal = Span::new(signature.content_end, node.span(ast).end);
        let (terminal, _) = self.prepare_typed_class_member_terminal(internal, sibling);
        Ok(ClassMemberContent::Bodyless(SourceBodylessClassMethod {
            prefix,
            signature,
            terminal,
        }))
    }

    fn format_bodyless_class_method(
        &mut self,
        method: SourceBodylessClassMethod<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail = writer.group(false, |writer| {
            self.format_method_key_prefix(method.prefix, writer)?;
            self.format_bodyless_signature_suffix(method.signature, writer)
        })?;
        self.format_typed_class_member_terminal(method.terminal, tail, writer)
    }

    fn source_class_index_signature(
        &self,
        node: TsIndexSignature,
        sibling: Option<DelimitedSuppression>,
    ) -> Result<SourceClassIndexSignature<'ast>, FormatError> {
        let mut signature = self.prepare_index_signature(node)?;
        let internal = Span::new(signature.content_end(), node.span(self.ast).end);
        let (terminal, mode) = self.prepare_typed_class_member_terminal(internal, sibling);
        if mode == ContentMode::Verbatim {
            signature.protect_result_type();
        }
        Ok(SourceClassIndexSignature {
            signature,
            terminal,
        })
    }

    fn prepare_typed_class_member_terminal(
        &self,
        internal: Span,
        sibling: Option<DelimitedSuppression>,
    ) -> (TypedClassMemberTerminal, ContentMode) {
        let comments = self.comments.in_range(internal);
        let mode = if self.has_suppression {
            self.content_mode(comments)
        } else {
            ContentMode::Formatted
        };
        let source_terminal = mode == ContentMode::Verbatim
            || sibling.is_some() && self.find_token(internal, comments, ";").is_some();
        let terminal = if source_terminal {
            TypedClassMemberTerminal::Source(internal)
        } else {
            TypedClassMemberTerminal::Canonical(internal)
        };
        (terminal, mode)
    }

    fn format_typed_class_member_terminal(
        &mut self,
        terminal: TypedClassMemberTerminal,
        tail: Option<ExpressionTail<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match terminal {
            TypedClassMemberTerminal::Canonical(span) => {
                let tail = self.capture_expression_tail(span, tail)?;
                if self.options.semicolons == Semicolons::Always {
                    writer.text(";");
                }
                self.format_expression_tail(tail, writer)
            }
            TypedClassMemberTerminal::Source(span) => {
                // Type grammar can leave an unclaimed tail before this actual
                // terminal. Its internal comments belong to the raw terminal.
                let before = tail.and_then(|tail| {
                    let count = tail
                        .comments
                        .partition_point(|comment| comment.span.end <= span.start);
                    (!tail.comments[..count].is_empty()).then_some(ExpressionTail {
                        source_end: tail.source_end,
                        comments: &tail.comments[..count],
                    })
                });
                self.format_expression_tail(before, writer)?;
                self.format_verbatim(span, writer)
            }
        }
    }

    fn source_class_field(
        &self,
        node: PropertyDefinition,
        source_sibling: Option<DelimitedSuppression>,
    ) -> Result<Option<SourceClassField<'ast>>, FormatError> {
        let ast = self.ast;
        if !node.decorators(ast).is_empty()
            || node.type_annotation(ast).is_some()
            || node.accessor(ast)
            || node.declare(ast)
            || node.r#override(ast)
            || node.optional(ast)
            || node.definite(ast)
            || node.readonly(ast)
            || node.r#abstract(ast)
            || node.accessibility(ast) != Accessibility::None
        {
            return Err(self.unsupported("typed or decorated class field", node));
        }
        let span = node.span(ast);
        let key = node.key(ast);
        let key_span = key.span(ast);
        let static_end = node.r#static(ast).then_some(span.start + 6);
        let prefix_end = static_end.unwrap_or(span.start);
        let value = node.value(ast);
        let end = value.map_or(key_span.end, |value| value.span(ast).end);
        let internal = Span::new(end, span.end);
        let comments = self.comments.in_range(internal);
        let semicolon = self.find_token(internal, comments, ";");
        let tail_end = semicolon.unwrap_or(span.end);
        let mut key_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        let mut tail_source = false;
        let computed = if node.computed(ast) {
            let before = Span::new(prefix_end, key_span.start);
            let opening = self
                .find_token(before, self.comments.in_range(before), "[")
                .ok_or_else(|| self.unsupported("computed field opening boundary", node))?;
            let after = Span::new(
                key_span.end,
                value.map_or(tail_end, |value| value.span(ast).start),
            );
            let closing = self
                .find_token(after, self.comments.in_range(after), "]")
                .ok_or_else(|| self.unsupported("computed field closing boundary", node))?;
            Some(Span::new(opening, closing + 1))
        } else {
            None
        };
        let key_start = computed.map_or(key_span.start, |span| span.start);
        let key_end = computed.map_or(key_span.end, |span| span.end);
        let end = value.map_or(key_end, |value| value.span(ast).end);
        let internal = Span::new(end, span.end);
        if self.has_suppression {
            let tail = self.comments.in_range(Span::new(end, tail_end));
            // There is no following child after the value. Blocks and Lines
            // therefore do not share the forward EOL relation used by siblings.
            for (run, placement) in self.suppressed_comment_runs(tail, tail_end) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    let position = if matches!(placement, SuppressionPosition::OwnLine) {
                        SuppressionPosition::OwnLine
                    } else {
                        self.suppression_position(std::slice::from_ref(&comment), tail_end)
                            .unwrap_or(placement)
                    };
                    if matches!(position, SuppressionPosition::Inline) {
                        return Ok(None);
                    }
                    if value.is_some() {
                        right_mode = ContentMode::Verbatim;
                    } else {
                        key_mode = ContentMode::Verbatim;
                    }
                    tail_source = true;
                }
            }
        }
        if self.has_suppression {
            if self.content_mode(
                self.comments
                    .in_range(Span::new(span.start, key_span.start)),
            ) == ContentMode::Verbatim
            {
                key_mode = ContentMode::Verbatim;
            }
            if let Some(computed) = computed {
                let trailing = self
                    .comments
                    .in_range(Span::new(key_span.end, computed.end - 1));
                for position in self.suppression_positions(trailing, computed.end - 1) {
                    match position {
                        SuppressionPosition::OwnLine if value.is_some() => {
                            right_mode = ContentMode::Verbatim
                        }
                        _ => key_mode = ContentMode::Verbatim,
                    }
                }
            }
        }
        let initializer = if let Some(right) = value {
            let right_start = right.span(ast).start;
            let gap = Span::new(key_end, right_start);
            let comments = self.comments.in_range(gap);
            let operator = self
                .find_token(gap, comments, "=")
                .ok_or_else(|| self.unsupported("class field initializer boundary", node))?;
            let mut source_gap = false;
            if self.has_suppression {
                let split = comments.partition_point(|comment| comment.span.end <= operator);
                for position in self.suppression_positions(&comments[..split], operator) {
                    match position {
                        SuppressionPosition::OwnLine => right_mode = ContentMode::Verbatim,
                        _ => key_mode = ContentMode::Verbatim,
                    }
                    source_gap = true;
                }
                for position in self.suppression_positions(&comments[split..], right_start) {
                    match position {
                        SuppressionPosition::EndOfLine => key_mode = ContentMode::Verbatim,
                        _ => right_mode = ContentMode::Verbatim,
                    }
                    source_gap = true;
                }
            }
            let layout = if source_gap {
                InitializerLayout::Bound { gap, right_mode }
            } else {
                let content = self.expression_content(right);
                let mut boundary =
                    self.assignment_boundary(key_end, &content, Span::new(operator, operator + 1))?;
                if right_mode == ContentMode::Verbatim {
                    boundary.mode = right_mode;
                }
                let break_after = boundary.own_line
                    || !boundary.operator_line.is_empty()
                    || self.assignment_right_breaks_after_operator(content.data);
                InitializerLayout::Formatted {
                    boundary,
                    break_after,
                }
            };
            Some(PreparedInitializer {
                left: InitializerLeft::Property { key, computed },
                right,
                layout,
                left_mode: key_mode,
                context: ExpressionContext::variable_initializer(),
            })
        } else {
            None
        };
        let terminal = if tail_source || source_sibling.is_some() && semicolon.is_some() {
            FieldTerminal::Source(internal)
        } else {
            FieldTerminal::Canonical {
                span: Span::new(end, tail_end),
                semicolon: self.options.semicolons == Semicolons::Always
                    || value.is_none()
                        && computed.is_none()
                        && matches!(
                            self.property_key_identifier(
                                key,
                                PropertyKeyContext::ClassField,
                                key_mode
                            ),
                            Some("static" | "get" | "set")
                        ),
            }
        };
        Ok(Some(SourceClassField {
            node,
            key,
            computed,
            key_start,
            key_mode,
            static_end,
            initializer,
            terminal,
        }))
    }

    fn format_class_field(
        &mut self,
        field: SourceClassField<'ast>,
        following: Option<&PreparedClassMember<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceClassField {
            node,
            key,
            computed,
            key_start,
            key_mode,
            static_end,
            initializer,
            terminal,
        } = field;
        if let Some(static_end) = static_end {
            writer.text("static");
            self.format_source_token_gap(static_end, key_start, true, writer)?;
        } else {
            self.format_source_token_gap(node.span(self.ast).start, key_start, false, writer)?;
        }
        let tail = match initializer {
            Some(initializer) => self.format_initializer(initializer, writer)?,
            None => {
                self.format_field_key(key, computed, key_mode, writer)?;
                None
            }
        };
        match terminal {
            FieldTerminal::Source(span) => self.format_verbatim(span, writer),
            FieldTerminal::Canonical { span, semicolon } => {
                let tail = self.capture_expression_tail(span, tail)?;
                if semicolon
                    || following
                        .is_some_and(|member| self.class_member_needs_field_separator(member))
                {
                    writer.text(";");
                }
                self.format_expression_tail(tail, writer)
            }
        }
    }

    fn class_member_needs_field_separator(&self, member: &PreparedClassMember<'ast>) -> bool {
        match &member.content {
            ClassMemberContent::Method(header) => header.needs_field_separator(self),
            ClassMemberContent::Bodyless(method) => method.prefix.needs_field_separator(self),
            ClassMemberContent::BodylessValue { prefix, .. } => prefix.needs_field_separator(self),
            ClassMemberContent::Field(field) => {
                field.static_end.is_none()
                    && (field.computed.is_some()
                        || matches!(
                            self.property_key_identifier(
                                field.key,
                                PropertyKeyContext::ClassField,
                                field.key_mode
                            ),
                            Some("in" | "instanceof")
                        ))
            }
            ClassMemberContent::Static(_) => false,
            ClassMemberContent::Index(index) => index.signature.starts_with_bracket(),
            ClassMemberContent::Verbatim => {
                let ast = self.ast;
                let (key, computed, prefix, generator) = match ast.class_element_data(member.node) {
                    ClassElementData::PropertyDefinition(field) => (
                        field.key(ast),
                        field.computed(ast),
                        field.r#static(ast),
                        false,
                    ),
                    ClassElementData::MethodDefinition(method) => {
                        let function = method.value(ast);
                        (
                            method.key(ast),
                            method.computed(ast),
                            method.r#static(ast)
                                || function.r#async(ast)
                                || matches!(
                                    method.kind(ast),
                                    MethodDefinitionKind::Get | MethodDefinitionKind::Set
                                ),
                            function.generator(ast),
                        )
                    }
                    ClassElementData::TsMethodDefinition(method) => {
                        let function = method.value(ast);
                        (
                            method.key(ast),
                            method.computed(ast),
                            method.r#static(ast)
                                || function.r#async(ast)
                                || matches!(
                                    method.kind(ast),
                                    MethodDefinitionKind::Get | MethodDefinitionKind::Set
                                ),
                            function.generator(ast),
                        )
                    }
                    ClassElementData::TsIndexSignature(index) => {
                        return self.source(index.span(ast)).starts_with('[');
                    }
                    ClassElementData::StaticBlock(_) => return false,
                };
                !prefix
                    && (computed
                        || generator
                        || matches!(
                            self.property_key_identifier(
                                key,
                                PropertyKeyContext::Ordinary,
                                ContentMode::Verbatim
                            ),
                            Some("in" | "instanceof")
                        ))
            }
        }
    }

    fn source_static_block(
        &self,
        node: StaticBlock,
    ) -> Result<SourceStaticBlock<'ast>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let first = node
            .body(ast)
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .find(|&statement| !matches!(ast.stmt_data(statement), StmtData::EmptyStatement(_)));
        let header = Span::new(
            span.start + 6,
            first.map_or(span.end - 1, |statement| statement.span(ast).start),
        );
        let opening = self
            .find_token(header, self.comments.in_range(header), "{")
            .ok_or_else(|| self.unsupported("static block opening boundary", node))?;
        let braces = Span::new(opening, span.end);
        let mut whole = false;
        let mut incoming = None;
        let comments = self.comments.in_range(Span::new(span.start + 6, opening));
        let last = comments
            .last()
            .map_or(span.start + 6, |comment| comment.span.end);
        let mut constraint = self.line_constraint(last, opening);
        if self.has_suppression {
            if self.content_mode(comments) == ContentMode::Verbatim {
                if let Some(first) = first {
                    incoming = Some(IncomingStatement {
                        span: first.span(ast),
                        mode: ContentMode::Verbatim,
                    });
                } else {
                    whole = true;
                }
                let last = comments
                    .last()
                    .map_or(span.start + 6, |comment| comment.span.end);
                constraint = self.source_line_constraint(last, opening);
            }
            if first.is_none()
                && self.content_mode(self.comments.in_range(Span::new(opening + 1, span.end - 1)))
                    == ContentMode::Verbatim
            {
                whole = true;
            }
        }
        Ok(SourceStaticBlock {
            node,
            braces,
            incoming,
            whole,
            comments,
            gap: if comments
                .last()
                .is_some_and(|comment| comment.kind == CommentKind::Line)
            {
                CommentGap::LineComment
            } else {
                CommentGap::Source(constraint)
            },
        })
    }

    fn format_static_block(
        &mut self,
        block: SourceStaticBlock<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceStaticBlock {
            node,
            braces,
            incoming,
            whole,
            comments,
            gap,
        } = block;
        if whole {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        writer.text("static");
        let end = node.span(self.ast).start + 6;
        self.comments.claim(comments)?;
        self.format_comment_run(comments, CommentRunLayout::SourceAfter(end), writer)?;
        gap.write(writer, FlatSeparator::Space, Writer::space);
        self.format_static_block_body(node, braces, incoming, writer)
    }
}
