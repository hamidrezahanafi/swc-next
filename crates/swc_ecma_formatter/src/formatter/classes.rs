//! Class heads and members retain real keys, signatures, and source terminators.

use swc_next_ecma_ast::{
    BindingIdentifier, Class, ClassBody, ClassElement, ClassElementData, Comment, CommentKind,
    Expr, GetSpan, MethodDefinitionKind, PropertyDefinition, PropertyKey, Span, StaticBlock,
    StmtData, SubRange, TsClassImplements, TsFunction, TsIndexSignature, TsMethodDefinition,
    TsTypeAnnotation, TsTypeParameterDeclaration,
};

pub(in crate::formatter) mod member_prefix;

use super::Formatter;
use super::bindings::InitializerLayout;
use super::comments::{CommentBoundary, CommentGap, CommentRunLayout, ExpressionTail, NodeHead};
use super::decorators::{DecoratorLayout, DecoratorPrefix, RemovedDecoratorHead};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
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
use super::types::generics::{TypeListKind, TypeListPrefix, TypeParameterContext};
use super::types::{PreparedAnnotation, PreparedIndexSignature, TypeGap};
use crate::{FormatError, Semicolons, document::Writer};
use member_prefix::{ClassMemberPrefix, ClassMemberPrefixOwner};

#[expect(
    clippy::large_enum_variant,
    reason = "Keep the prepared borrowed header inline instead of allocating per class."
)]
enum PreparedClassHead<'ast> {
    Verbatim,
    Source(SourceClassHead<'ast>),
}

pub(super) struct PreparedClass<'ast> {
    head: PreparedClassHead<'ast>,
    prefix: Option<DecoratorPrefix>,
    removed: Option<RemovedDecoratorHead<'ast>>,
    core_start: u32,
    span: Span,
}

impl PreparedClass<'_> {
    pub(super) fn is_verbatim(&self) -> bool {
        matches!(self.head, PreparedClassHead::Verbatim)
    }
    pub(super) fn core_start(&self) -> u32 {
        self.core_start
    }
}

struct SourceClassHead<'ast> {
    prefix: Option<SourceClassPrefix<'ast>>,
    keyword_end: u32,
    name: Option<BindingIdentifier>,
    name_mode: ContentMode,
    parameters: Option<SourceClassParameters<'ast>>,
    heritage: Option<SourceClassHeritage<'ast>>,
    implements: Option<SourceClassImplements<'ast>>,
    body: ClassBody,
    body_mode: ContentMode,
    incoming: Option<IncomingClassMember>,
    end: u32,
    body_comments: &'ast [Comment],
    body_gap: CommentGap,
}

enum SourceClassPrefix<'ast> {
    Formatted {
        declare: bool,
        abstract_class: bool,
        name_gap: Option<CommentBoundary<'ast>>,
    },
    Source(Span),
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

struct PreparedClassImplementsBoundary<'ast> {
    first: TsClassImplements,
    gap: Span,
    comments: &'ast [Comment],
    source: bool,
}

struct SourceClassImplements<'ast> {
    first: TsClassImplements,
    remaining: SubRange<TsClassImplements>,
    end: u32,
    before_keyword: TypeGap<'ast>,
    after_keyword: TypeGap<'ast>,
}

struct PreparedClassMember<'ast> {
    node: ClassElement,
    span: Span,
    decorators: Option<DecoratorPrefix>,
    removed: Option<RemovedDecoratorHead<'ast>>,
    core_start: u32,
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
    key: PropertyKey,
    computed: Option<Span>,
    key_start: u32,
    key_mode: ContentMode,
    prefix: ClassMemberPrefix,
    accessor: Option<ClassFieldAccessor>,
    key_context: PropertyKeyContext,
    suffix: Option<FieldSuffix<'ast>>,
    left_end: u32,
    initializer: Option<FieldInitializer<'ast>>,
    terminal: FieldTerminal,
}

#[derive(Clone, Copy)]
enum ClassFieldAccessor {
    Token(Span),
    SourcePrefix,
}

struct FieldInitializer<'ast> {
    right: Expr,
    layout: InitializerLayout<'ast>,
}

struct SourceFieldSuffix {
    key_end: u32,
    marker: Option<(Span, &'static str)>,
    annotation: Option<TsTypeAnnotation>,
    end: u32,
}

struct FieldSuffix<'ast> {
    key_end: u32,
    marker: Option<(Span, &'static str)>,
    annotation: Option<PreparedAnnotation<'ast>>,
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
        let prefix =
            DecoratorPrefix::new(self.ast, class.decorators(self.ast), class.span(self.ast));
        let prepared = self.prepare_class(class, prefix)?;
        self.format_prepared_class(prepared, true, writer)
    }

    pub(super) fn prepare_class(
        &self,
        class: Class,
        prefix: Option<DecoratorPrefix>,
    ) -> Result<PreparedClass<'ast>, FormatError> {
        let native = class.span(self.ast);
        let span = prefix.map_or(native, |prefix| {
            Span::new(native.start.min(prefix.physical.start), native.end)
        });
        let implements_boundary = self.prepare_whole_class_boundary(class);
        if implements_boundary
            .as_ref()
            .is_some_and(|boundary| boundary.source)
        {
            return Ok(PreparedClass {
                head: PreparedClassHead::Verbatim,
                prefix,
                removed: None,
                core_start: native.start,
                span,
            });
        }
        let displaced = prefix.is_none()
            && (self.source_starts_decorator(native)
                || !class.r#abstract(self.ast)
                    && !class.declare(self.ast)
                    && !self.source(native).starts_with("class"));
        let core_start = if prefix.is_some() || displaced {
            let ast = self.ast;
            let limit = class.id(ast).map_or_else(
                || {
                    class.type_parameters(ast).map_or_else(
                        || {
                            class
                                .super_class(ast)
                                .map_or(class.body(ast).span(ast).start, |node| {
                                    node.span(ast).start
                                })
                        },
                        |node| node.span(ast).start,
                    )
                },
                |node| node.span(ast).start,
            );
            let start = match prefix {
                Some(prefix) => self.decorator_core_start(prefix, native, limit)?,
                None => self.removed_class_core_start(class, limit)?,
            };
            if !class.r#abstract(ast)
                && !class.declare(ast)
                && !self.source(Span::new(start, limit)).starts_with("class")
            {
                self.removed_class_core_start(class, limit)?
            } else {
                start
            }
        } else {
            native.start
        };
        let removed = if displaced {
            self.removed_decorator_head(native, core_start)?
        } else {
            None
        };
        Ok(PreparedClass {
            head: self.source_class_head(class, core_start, implements_boundary)?,
            prefix,
            removed,
            core_start,
            span,
        })
    }

    pub(super) fn format_prepared_class(
        &mut self,
        prepared: PreparedClass<'ast>,
        print_prefix: bool,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let head = match prepared.head {
            PreparedClassHead::Verbatim => {
                return self.format_verbatim(
                    if print_prefix {
                        prepared.span
                    } else {
                        Span::new(prepared.core_start, prepared.span.end)
                    },
                    writer,
                );
            }
            PreparedClassHead::Source(head) => head,
        };
        if let Some(head) = prepared.removed {
            self.format_removed_decorator_head(head, writer)?;
        }
        if print_prefix && let Some(prefix) = prepared.prefix {
            self.format_decorators(prefix, DecoratorLayout::Class, prepared.core_start, writer)?;
        }
        let SourceClassHead {
            prefix,
            keyword_end,
            name,
            name_mode,
            parameters,
            heritage,
            implements,
            body,
            body_mode,
            incoming,
            end,
            body_comments,
            body_gap,
        } = head;
        let name_gap_formatted = match prefix {
            Some(SourceClassPrefix::Formatted {
                declare,
                abstract_class,
                name_gap,
            }) => {
                if declare {
                    writer.text("declare");
                    writer.space();
                }
                if abstract_class {
                    writer.text("abstract");
                    writer.space();
                }
                writer.text("class");
                let has_name = name_gap.is_some();
                if let Some(boundary) = name_gap {
                    self.format_type_gap(
                        TypeGap::Formatted(boundary),
                        FlatSeparator::Space,
                        writer,
                    )?;
                }
                has_name
            }
            Some(SourceClassPrefix::Source(span)) => {
                self.format_verbatim(span, writer)?;
                false
            }
            None => {
                writer.text("class");
                false
            }
        };
        if let Some(name) = name {
            if !name_gap_formatted {
                self.format_source_token_gap(keyword_end, name.span(self.ast).start, true, writer)?;
            }
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
        if let Some(implements) = implements {
            self.format_class_implements(implements, writer)?;
        }
        self.comments.claim(body_comments)?;
        self.format_comment_run(body_comments, CommentRunLayout::SourceAfter(end), writer)?;
        body_gap.write(writer, FlatSeparator::Space, Writer::space);
        self.format_class_body(body, body_mode, incoming, writer)
    }

    fn prepare_whole_class_boundary(
        &self,
        class: Class,
    ) -> Option<PreparedClassImplementsBoundary<'ast>> {
        if !self.has_suppression {
            return None;
        }
        let ast = self.ast;
        let first = ast.get_node_in_sub_range(class.implements(ast).first()?);
        let end = if let Some(arguments) = class.super_type_arguments(ast) {
            arguments.span(ast).end
        } else if class.id(ast).is_none()
            && class.type_parameters(ast).is_none()
            && class.super_class(ast).is_none()
        {
            // This is source-owner geometry; no decorator list/core is prepared.
            self.removed_class_core_start(class, first.span(ast).start)
                .ok()?
                + 5
        } else {
            return None;
        };
        let gap = Span::new(end, first.span(ast).start);
        let comments = self.comments.in_range(gap);
        Some(PreparedClassImplementsBoundary {
            first,
            gap,
            comments,
            source: self.content_mode(comments) == ContentMode::Verbatim,
        })
    }

    fn source_class_head(
        &self,
        class: Class,
        core_start: u32,
        implements_boundary: Option<PreparedClassImplementsBoundary<'ast>>,
    ) -> Result<PreparedClassHead<'ast>, FormatError> {
        let ast = self.ast;
        let body = class.body(ast);
        let name = class.id(ast);
        let (start, prefix) = if class.r#abstract(ast) || class.declare(ast) {
            let (end, prefix) = self.prepare_class_prefix(class, core_start)?;
            (end, Some(prefix))
        } else {
            (core_start + 5, None)
        };
        let mut name_mode = ContentMode::Formatted;
        if self.has_suppression {
            let prefix_start = if prefix.is_some() { core_start } else { start };
            let prefix_end = name.map_or(start, |name| name.span(ast).start);
            let comments = self.comments.in_range(Span::new(prefix_start, prefix_end));
            name_mode = self.content_mode(comments);
            if name.is_none() && name_mode == ContentMode::Verbatim {
                return Ok(PreparedClassHead::Verbatim);
            }
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
        let mut heritage = if let Some(expression) = class.super_class(ast) {
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
        let mut implements_nodes = class.implements(ast);
        let implements = if let Some(first) = implements_nodes.first() {
            let first = ast.get_node_in_sub_range(first);
            let remaining = implements_nodes.split_off(1);
            let gap = Span::new(end, first.span(ast).start);
            let (comments, source) = match implements_boundary {
                Some(boundary) if boundary.first == first && boundary.gap == gap => {
                    (boundary.comments, boundary.source)
                }
                _ => {
                    let comments = self.comments.in_range(gap);
                    (
                        comments,
                        self.content_mode(comments) == ContentMode::Verbatim,
                    )
                }
            };
            let keyword = self
                .find_token(gap, comments, "implements")
                .ok_or_else(|| self.unsupported("class implements boundary", class))?;
            let keyword = Span::new(keyword, keyword + "implements".len() as u32);
            let split = comments.partition_point(|comment| comment.span.end <= keyword.start);
            let (before, after) = comments.split_at(split);
            if source {
                // The class handler attaches first-entry comments to a real
                // preceding header node, or to the complete class when that
                // node is a superclass argument list or is absent.
                if let Some(heritage) = &mut heritage {
                    if heritage.arguments.is_some() {
                        return Ok(PreparedClassHead::Verbatim);
                    }
                    heritage.mode = ContentMode::Verbatim;
                } else if let Some(parameters) = &mut parameters {
                    parameters.mode = ContentMode::Verbatim;
                } else if name.is_some() {
                    name_mode = ContentMode::Verbatim;
                } else {
                    return Ok(PreparedClassHead::Verbatim);
                }
            }
            end = remaining.last().map_or(first.span(ast).end, |last| {
                ast.get_node_in_sub_range(last).span(ast).end
            });
            Some(SourceClassImplements {
                first,
                remaining,
                end,
                before_keyword: if source {
                    TypeGap::Source(Span::new(gap.start, keyword.start))
                } else {
                    TypeGap::Formatted(self.prepare_comment_boundary(
                        before,
                        gap.start,
                        keyword.start,
                    ))
                },
                after_keyword: if source {
                    TypeGap::Source(Span::new(keyword.end, gap.end))
                } else {
                    TypeGap::Formatted(self.prepare_comment_boundary(after, keyword.end, gap.end))
                },
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
        Ok(PreparedClassHead::Source(SourceClassHead {
            prefix,
            keyword_end: start,
            name,
            name_mode,
            parameters,
            heritage,
            implements,
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
        }))
    }

    fn prepare_class_prefix(
        &self,
        class: Class,
        core_start: u32,
    ) -> Result<(u32, SourceClassPrefix<'ast>), FormatError> {
        let ast = self.ast;
        let declare = class.declare(ast);
        let abstract_class = class.r#abstract(ast);
        let name = class.id(ast);
        let mut limit = class.body(ast).span(ast).start;
        if let Some(first) = class.implements(ast).first() {
            limit = ast.get_node_in_sub_range(first).span(ast).start;
        }
        if let Some(superclass) = class.super_class(ast) {
            limit = superclass.span(ast).start;
        }
        if let Some(parameters) = class.type_parameters(ast) {
            limit = parameters.span(ast).start;
        }
        if let Some(name) = name {
            limit = name.span(ast).start;
        }
        let span = Span::new(core_start, limit);
        let comments = self.comments.in_range(span);
        let keyword = self
            .find_token(span, comments, "class")
            .ok_or_else(|| self.unsupported("class modifier boundary", class))?;
        let keyword_end = keyword + "class".len() as u32;
        let target = name.map(|node| node.span(ast));
        let binding = self.comment_binding(comments, span, target);
        let prefix = if binding.allows_relocation(target) && (name.is_some() || comments.is_empty())
        {
            SourceClassPrefix::Formatted {
                declare,
                abstract_class,
                name_gap: name.map(|node| {
                    self.prepare_comment_boundary(comments, span.start, node.span(ast).start)
                }),
            }
        } else {
            SourceClassPrefix::Source(Span::new(span.start, keyword_end))
        };
        Ok((keyword_end, prefix))
    }

    fn format_class_implements(
        &mut self,
        list: SourceClassImplements<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        self.format_type_gap(list.before_keyword, FlatSeparator::Space, writer)?;
        writer.text("implements");
        self.format_type_gap(list.after_keyword, FlatSeparator::Space, writer)?;
        let ast = self.ast;
        let mut nodes = std::iter::once(list.first)
            .chain(
                list.remaining
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            )
            .peekable();
        writer.group(false, |writer| {
            writer.indent(|writer| {
                let mut previous: Option<ItemBoundary> = None;
                while let Some(node) = nodes.next() {
                    let span = node.span(ast);
                    let end = previous.map_or(span.start, |previous| previous.end);
                    if let Some(previous) = previous {
                        let next = self
                            .comments
                            .between(Span::new(end, span.start))?
                            .first()
                            .map_or(span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            previous.separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = nodes.peek().map_or(FollowingItem::End, |next| {
                        FollowingItem::Node(next.span(ast))
                    });
                    let mut item = self.prepare_delimited_item(
                        span,
                        following,
                        Span::new(end, list.end),
                        false,
                        writer,
                    )?;
                    item.set_leading_mode(
                        previous.map_or(ContentMode::Formatted, |previous| previous.following_mode),
                    );
                    let tail = if item.mode == ContentMode::Verbatim {
                        self.format_verbatim(span, writer)?;
                        None
                    } else {
                        let expression = node.expression(ast);
                        let context = ExpressionContext::at(ExpressionPosition::Nested);
                        if let Some(arguments) = node.type_arguments(ast) {
                            let arguments = self.prepare_expression_type_arguments(
                                expression,
                                arguments,
                                AccessKind::Ordinary,
                            )?;
                            let context = context
                                .with_content_mode(arguments.callee_mode)
                                .with_tail_owner(TailOwner::ParentOperation);
                            let tail = self.format_expression(expression, context, writer)?;
                            self.format_expression_type_arguments(arguments, tail, writer)?;
                            None
                        } else {
                            self.format_expression(expression, context, writer)?
                        }
                    };
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                Ok(())
            })
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
                let node_span = current.span;
                if !matches!(current.content, ClassMemberContent::Verbatim) {
                    if let Some(head) = current.removed {
                        self.format_removed_decorator_head(head, writer)?;
                    }
                    if let Some(prefix) = current.decorators {
                        self.format_decorators(
                            prefix,
                            DecoratorLayout::Member,
                            current.core_start,
                            writer,
                        )?;
                    }
                }
                self.format_class_member(current.content, node_span, next.as_ref(), writer)?;
                let next_start = next.as_ref().map_or(content.end, |next| next.span.start);
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
        let native = node.span(ast);
        let decorators = self.member_decorators(node);
        let span = decorators.map_or(native, |prefix| prefix.envelope);
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
        if mode == ContentMode::Verbatim {
            return Ok(PreparedClassMember {
                node,
                span,
                decorators,
                removed: None,
                core_start: native.start,
                content: ClassMemberContent::Verbatim,
                sibling,
            });
        }
        let displaced = decorators.is_none() && self.source_starts_decorator(native);
        let core_start = if decorators.is_some() || displaced {
            let limit = match ast.class_element_data(node) {
                ClassElementData::MethodDefinition(node) => node.key(ast).span(ast).start,
                ClassElementData::TsMethodDefinition(node) => node.key(ast).span(ast).start,
                ClassElementData::PropertyDefinition(node) => node.key(ast).span(ast).start,
                _ => native.end,
            };
            match decorators {
                Some(prefix) => self.decorator_core_start(prefix, native, limit)?,
                None => self.removed_member_core_start(native, limit),
            }
        } else {
            native.start
        };
        let core_start = if decorators.is_some() || displaced {
            let owner = match ast.class_element_data(node) {
                ClassElementData::MethodDefinition(node) => {
                    Some(ClassMemberPrefixOwner::Method(node))
                }
                ClassElementData::TsMethodDefinition(node) => {
                    Some(ClassMemberPrefixOwner::Bodyless(node))
                }
                ClassElementData::PropertyDefinition(node) => {
                    Some(ClassMemberPrefixOwner::Field(node))
                }
                _ => None,
            };
            owner
                .and_then(|owner| owner.plain_key_start(ast))
                .unwrap_or(core_start)
        } else {
            core_start
        };
        let removed = if displaced {
            self.removed_decorator_head(native, core_start)?
        } else {
            None
        };
        let content = match ast.class_element_data(node) {
            ClassElementData::MethodDefinition(method) => ClassMemberContent::Method(
                self.source_method_header_at(MethodOwner::Class(method), core_start)?,
            ),
            ClassElementData::TsMethodDefinition(method) => {
                self.source_bodyless_class_method(method, sibling, core_start)?
            }
            ClassElementData::PropertyDefinition(field) => {
                match self.source_class_field(field, sibling, core_start)? {
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
        };
        Ok(PreparedClassMember {
            node,
            span,
            decorators,
            removed,
            core_start,
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
        core_start: u32,
    ) -> Result<ClassMemberContent<'ast>, FormatError> {
        let ast = self.ast;
        let function = node.value(ast);
        if node.optional(ast) && node.kind(ast) != MethodDefinitionKind::Method
            || function.r#async(ast)
            || function.generator(ast)
        {
            return Err(self.unsupported("typed or decorated bodyless class method", node));
        }
        let kind = node.kind(ast);
        if kind != MethodDefinitionKind::Method {
            if function.type_parameters(ast).is_some()
                || (kind != MethodDefinitionKind::Get && function.return_type(ast).is_some())
            {
                return Err(self.unsupported("typed accessor or constructor header", function));
            }
            if function.this_param(ast).is_some() {
                return Err(self.unsupported("typed function", function));
            }
        }
        let mut source = self.source_bodyless_signature(function);
        let mut prefix = self.source_bodyless_method_prefix(
            node,
            source.header_start(ast),
            source.generic.is_some(),
            core_start,
        )?;
        source.set_leading_mode(prefix.following_mode);
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
        prefix.key_mode = self.method_key_mode(&prefix, signature.preceding_mode);
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
        core_start: u32,
    ) -> Result<Option<SourceClassField<'ast>>, FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let key = node.key(ast);
        let key_span = key.span(ast);
        let parts =
            self.source_class_member_prefix_at(ClassMemberPrefixOwner::Field(node), core_start)?;
        let opening = parts.as_ref().and_then(|parts| parts.opening);
        let accessor = parts.as_ref().and_then(|parts| {
            parts.keyword.map(|(_, span)| {
                if parts.member.is_source() && parts.member.end() > span.start {
                    ClassFieldAccessor::SourcePrefix
                } else {
                    ClassFieldAccessor::Token(span)
                }
            })
        });
        let key_context = if accessor.is_some() {
            PropertyKeyContext::Ordinary
        } else {
            PropertyKeyContext::ClassField
        };
        let prefix = parts.map_or_else(
            || ClassMemberPrefix::ordinary(core_start, node.r#static(ast)),
            |parts| parts.member,
        );
        let prefix_end = match accessor {
            Some(ClassFieldAccessor::Token(span)) => span.end,
            _ => prefix.end(),
        };
        let value = node.value(ast);
        let annotation = node.type_annotation(ast);
        let end = value.map_or_else(
            || annotation.map_or(key_span.end, |annotation| annotation.span(ast).end),
            |value| value.span(ast).end,
        );
        let internal = Span::new(end, span.end);
        let comments = self.comments.in_range(internal);
        let semicolon = self.find_token(internal, comments, ";");
        let tail_end = semicolon.unwrap_or(span.end);
        let mut key_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        let mut annotation_mode = ContentMode::Formatted;
        let mut tail_source = false;
        let computed = if node.computed(ast) {
            let before = Span::new(prefix_end, key_span.start);
            let opening = match opening {
                Some(opening) => opening,
                None => self
                    .find_token(before, self.comments.in_range(before), "[")
                    .ok_or_else(|| self.unsupported("computed field opening boundary", node))?,
            };
            let after = Span::new(
                key_span.end,
                annotation.map_or_else(
                    || value.map_or(tail_end, |value| value.span(ast).start),
                    |annotation| annotation.span(ast).start,
                ),
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
        let suffix = self.source_field_suffix(node, key_end, tail_end)?;
        if self.has_suppression
            && let Some((marker, _)) = suffix.as_ref().and_then(|suffix| suffix.marker)
        {
            let before = self.comments.in_range(Span::new(key_end, marker.start));
            for position in self.suppression_positions(before, marker.start) {
                match position {
                    SuppressionPosition::OwnLine if annotation.is_some() => {
                        annotation_mode = ContentMode::Verbatim;
                    }
                    SuppressionPosition::OwnLine if value.is_some() => {
                        right_mode = ContentMode::Verbatim;
                    }
                    _ => key_mode = ContentMode::Verbatim,
                }
            }
        }
        if self.has_suppression
            && suffix
                .as_ref()
                .is_some_and(|suffix| suffix.marker.is_none())
            && let Some(annotation) = annotation
        {
            let colon = annotation.span(ast).start;
            let before = self.comments.in_range(Span::new(key_end, colon));
            if self
                .suppression_positions(before, colon)
                .any(|position| matches!(position, SuppressionPosition::EndOfLine))
            {
                key_mode = ContentMode::Verbatim;
            }
        }
        let left_end = suffix.as_ref().map_or(key_end, |suffix| suffix.end);
        let end = value.map_or(left_end, |value| value.span(ast).end);
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
                        SuppressionPosition::OwnLine if annotation.is_some() => {
                            annotation_mode = ContentMode::Verbatim
                        }
                        SuppressionPosition::OwnLine if value.is_some() => {
                            right_mode = ContentMode::Verbatim
                        }
                        _ => key_mode = ContentMode::Verbatim,
                    }
                }
            }
        }
        let has_suffix = suffix.is_some();
        let initializer = if let Some(right) = value {
            let right_start = right.span(ast).start;
            let gap = Span::new(left_end, right_start);
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
                        _ if annotation.is_some() => annotation_mode = ContentMode::Verbatim,
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
                let mut boundary = self.assignment_boundary(
                    left_end,
                    &content,
                    Span::new(operator, operator + 1),
                )?;
                if right_mode == ContentMode::Verbatim {
                    boundary.mode = right_mode;
                }
                let rhs_layout = self.ordinary_rhs_layout(content.data, &boundary);
                InitializerLayout::Formatted {
                    boundary,
                    rhs_layout,
                }
            };
            Some(FieldInitializer { right, layout })
        } else {
            None
        };
        let suffix = suffix
            .map(|suffix| self.prepare_field_suffix(suffix, annotation_mode))
            .transpose()?;
        if accessor.is_some() {
            key_mode = self.unquoted_key_mode(key, computed, key_context, key_mode);
        }
        let terminal = if tail_source || source_sibling.is_some() && semicolon.is_some() {
            FieldTerminal::Source(internal)
        } else {
            FieldTerminal::Canonical {
                span: Span::new(end, tail_end),
                semicolon: self.options.semicolons == Semicolons::Always
                    || initializer.is_none()
                        && !has_suffix
                        && computed.is_none()
                        && matches!(
                            self.property_key_identifier(key, key_context, key_mode),
                            Some("static" | "get" | "set")
                        ),
            }
        };
        Ok(Some(SourceClassField {
            key,
            computed,
            key_start,
            key_mode,
            prefix,
            accessor,
            key_context,
            suffix,
            left_end,
            initializer,
            terminal,
        }))
    }

    fn source_field_suffix(
        &self,
        node: PropertyDefinition,
        key_end: u32,
        terminal_start: u32,
    ) -> Result<Option<SourceFieldSuffix>, FormatError> {
        let ast = self.ast;
        let annotation = node.type_annotation(ast);
        if !node.optional(ast) && !node.definite(ast) && annotation.is_none() {
            return Ok(None);
        }
        let marker = if node.optional(ast) || node.definite(ast) {
            let gap_end = annotation.map_or_else(
                || {
                    node.value(ast)
                        .map_or(terminal_start, |value| value.span(ast).start)
                },
                |annotation| annotation.span(ast).start,
            );
            let gap = Span::new(key_end, gap_end);
            let token = if node.optional(ast) { "?" } else { "!" };
            let start = self
                .find_token(gap, self.comments.in_range(gap), token)
                .ok_or_else(|| self.unsupported("class field suffix marker", node))?;
            Some((Span::new(start, start + 1), token))
        } else {
            None
        };
        let preceding_end = marker.map_or(key_end, |(span, _)| span.end);
        let end = annotation.map_or(preceding_end, |annotation| annotation.span(ast).end);
        Ok(Some(SourceFieldSuffix {
            key_end,
            marker,
            annotation,
            end,
        }))
    }

    fn prepare_field_suffix(
        &self,
        suffix: SourceFieldSuffix,
        annotation_mode: ContentMode,
    ) -> Result<FieldSuffix<'ast>, FormatError> {
        let preceding_end = suffix.marker.map_or(suffix.key_end, |(span, _)| span.end);
        let annotation = suffix
            .annotation
            .map(|annotation| {
                self.prepare_annotation_content_mode(preceding_end, annotation, annotation_mode)
            })
            .transpose()?;
        Ok(FieldSuffix {
            key_end: suffix.key_end,
            marker: suffix.marker,
            annotation,
        })
    }

    fn format_field_suffix(
        &mut self,
        suffix: FieldSuffix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if let Some((marker, token)) = suffix.marker {
            self.format_source_token_gap(suffix.key_end, marker.start, false, writer)?;
            writer.text(token);
        }
        if let Some(annotation) = suffix.annotation {
            self.format_annotation_content(annotation, writer)
        } else {
            Ok(None)
        }
    }

    fn format_class_field(
        &mut self,
        field: SourceClassField<'ast>,
        following: Option<&PreparedClassMember<'ast>>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let SourceClassField {
            key,
            computed,
            key_start,
            key_mode,
            prefix,
            accessor,
            key_context,
            suffix,
            left_end,
            initializer,
            terminal,
        } = field;
        self.format_class_member_prefix(&prefix, writer)?;
        let prefix_end = if let Some(ClassFieldAccessor::Token(accessor)) = accessor {
            if !prefix.is_formatted() && (prefix.has_modifiers() || prefix.is_source()) {
                self.format_source_token_gap(
                    prefix.end(),
                    accessor.start,
                    prefix.has_modifiers() || prefix.source_span().is_some(),
                    writer,
                )?;
            }
            writer.text("accessor");
            accessor.end
        } else {
            prefix.end()
        };
        self.format_class_member_key_gap(
            &prefix,
            prefix_end,
            key_start,
            accessor.is_some() || prefix.has_modifiers() && !prefix.is_formatted(),
            writer,
        )?;
        self.format_property_key_context(key, computed, key_mode, key_context, writer)?;
        let tail = if let Some(suffix) = suffix {
            self.format_field_suffix(suffix, writer)?
        } else {
            None
        };
        let tail = if let Some(FieldInitializer { right, layout }) = initializer {
            self.format_expression_tail(tail, writer)?;
            self.format_initializer_right(
                left_end,
                right,
                layout,
                ExpressionContext::variable_initializer(),
                writer,
            )?
        } else {
            tail
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
        if member.decorators.is_some() {
            return matches!(member.content, ClassMemberContent::Verbatim)
                && self.source(member.span).starts_with('[');
        }
        match &member.content {
            ClassMemberContent::Method(header) => header.needs_field_separator(self),
            ClassMemberContent::Bodyless(method) => method.prefix.needs_field_separator(self),
            ClassMemberContent::BodylessValue { prefix, .. } => prefix.needs_field_separator(self),
            ClassMemberContent::Field(field) => {
                if let Some(span) = field.prefix.source_span() {
                    return self.source_class_member_needs_field_separator(span, field.key);
                }
                !field.prefix.has_modifiers()
                    && (field.accessor.is_none() && field.computed.is_some()
                        || matches!(
                            self.property_key_identifier(
                                field.key,
                                field.key_context,
                                field.key_mode
                            ),
                            Some("in" | "instanceof")
                        ))
            }
            ClassMemberContent::Static(_) => false,
            ClassMemberContent::Index(index) => index.signature.starts_with_bracket(),
            ClassMemberContent::Verbatim => {
                let ast = self.ast;
                let key = match ast.class_element_data(member.node) {
                    ClassElementData::PropertyDefinition(field) => field.key(ast),
                    ClassElementData::MethodDefinition(method) => method.key(ast),
                    ClassElementData::TsMethodDefinition(method) => method.key(ast),
                    ClassElementData::TsIndexSignature(index) => {
                        return self.source(index.span(ast)).starts_with('[');
                    }
                    ClassElementData::StaticBlock(_) => return false,
                };
                self.source_class_member_needs_field_separator(member.node.span(ast), key)
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
