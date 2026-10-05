//! Interface heads keep their declaration, heritage, body and incoming member owners.

use swc_next_ecma_ast::{
    BindingIdentifier, GetSpan, Span, SubRange, TsInterfaceBody, TsInterfaceDeclaration,
    TsInterfaceHeritage, TsTypeElement, TsTypeParameterDeclaration,
};

use super::generics::TypeParameterContext;
use super::members::{IncomingTypeMember, TypeMemberBodyKind};
use super::{Formatter, TypeGap};
use crate::formatter::comments::CommentBoundary;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, ItemBoundary};
use crate::formatter::expressions::{ExpressionContext, ExpressionPosition};
use crate::formatter::statements::DeclarationKeyword;
use crate::formatter::suppression::{ContentMode, SuppressionPosition};
use crate::{FormatError, document::Writer};

struct InterfaceParameters<'ast> {
    node: TsTypeParameterDeclaration,
    mode: ContentMode,
    prefix: TypeGap<'ast>,
}

struct InterfaceHead<'ast> {
    name: BindingIdentifier,
    name_mode: ContentMode,
    prefix: TypeGap<'ast>,
    parameters: Option<InterfaceParameters<'ast>>,
    heritage: Option<InterfaceHeritageList<'ast>>,
    body: TsInterfaceBody,
    body_mode: ContentMode,
    incoming: Option<IncomingTypeMember>,
    body_prefix: TypeGap<'ast>,
}

struct InterfaceHeritageList<'ast> {
    first: TsInterfaceHeritage,
    remaining: SubRange<TsInterfaceHeritage>,
    first_mode: ContentMode,
    end: u32,
    prefix: HeritagePrefix<'ast>,
}

#[derive(Clone, Copy)]
enum InterfaceBodyOwner {
    Member(TsTypeElement),
    Body,
}

enum HeritagePrefix<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn format_interface(
        &mut self,
        node: TsInterfaceDeclaration,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let head = self.prepare_interface(node, keyword)?;
        writer.group(false, |writer| {
            writer.text("interface");
            self.format_type_gap(head.prefix, FlatSeparator::Space, writer)?;
            if head.name_mode == ContentMode::Verbatim {
                self.format_verbatim(head.name.span(self.ast), writer)?;
            } else {
                writer.text(self.ast.get_utf8(head.name.name(self.ast)));
            }
            if let Some(parameters) = head.parameters {
                self.format_type_gap(parameters.prefix, FlatSeparator::Empty, writer)?;
                self.format_type_parameters(
                    parameters.node,
                    TypeParameterContext::TypeDeclaration,
                    parameters.mode,
                    writer,
                )?;
            }
            if let Some(heritage) = head.heritage {
                self.format_interface_heritage(heritage, writer)?;
            }
            self.format_type_gap(head.body_prefix, FlatSeparator::Space, writer)?;
            let body_span = head.body.span(self.ast);
            if head.body_mode == ContentMode::Verbatim {
                self.format_verbatim(body_span, writer)
            } else {
                self.format_type_member_body(
                    body_span,
                    head.body.body(self.ast),
                    TypeMemberBodyKind::Interface,
                    head.incoming,
                    writer,
                )
            }
        })
    }

    fn prepare_interface(
        &self,
        node: TsInterfaceDeclaration,
        keyword: Option<DeclarationKeyword>,
    ) -> Result<InterfaceHead<'ast>, FormatError> {
        let ast = self.ast;
        let name = node.id(ast);
        let name_span = name.span(ast);
        let keyword_end = keyword
            .map_or(node.span(ast).start + "interface".len() as u32, |keyword| {
                keyword.span.end
            });
        let head_gap = Span::new(keyword_end, name_span.start);
        let comments = self.comments.in_range(head_gap);
        let mut name_mode = self.content_mode(comments);
        if keyword.is_some_and(|keyword| keyword.suppresses_name(name)) {
            name_mode = ContentMode::Verbatim;
        }
        let prefix = if name_mode == ContentMode::Verbatim {
            TypeGap::Source(head_gap)
        } else {
            TypeGap::Formatted(self.prepare_comment_boundary(
                comments,
                head_gap.start,
                head_gap.end,
            ))
        };
        let mut parameters = node.type_parameters(ast).map(|node| {
            let gap = Span::new(name_span.end, node.span(ast).start);
            let comments = self.comments.in_range(gap);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            let mut mode = ContentMode::Formatted;
            if source {
                for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                    if matches!(position, SuppressionPosition::EndOfLine) {
                        name_mode = ContentMode::Verbatim;
                    } else {
                        mode = ContentMode::Verbatim;
                    }
                }
            }
            InterfaceParameters {
                node,
                mode,
                prefix: if source {
                    TypeGap::Source(gap)
                } else {
                    TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
                },
            }
        });
        let head_end = parameters
            .as_ref()
            .map_or(name_span.end, |parameters| parameters.node.span(ast).end);
        let body = node.body(ast);
        let mut heritage_nodes = node.extends(ast);
        let mut end = head_end;
        let heritage = if let Some(first) = heritage_nodes.first() {
            let first = ast.get_node_in_sub_range(first);
            let remaining = heritage_nodes.split_off(1);
            end = remaining.last().map_or(first.span(ast).end, |last| {
                ast.get_node_in_sub_range(last).span(ast).end
            });
            let gap = Span::new(head_end, first.span(ast).start);
            let comments = self.comments.in_range(gap);
            let keyword = self
                .find_token(gap, comments, "extends")
                .ok_or_else(|| FormatError::unsupported("interface extends boundary", None, gap))?;
            let keyword = Span::new(keyword, keyword + "extends".len() as u32);
            let split = comments.partition_point(|comment| comment.span.end <= keyword.start);
            let (before, after) = comments.split_at(split);
            let source =
                self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
            let mut first_mode = ContentMode::Formatted;
            let mut preceding_mode = ContentMode::Formatted;
            if source {
                if self.content_mode(before) == ContentMode::Verbatim {
                    preceding_mode = ContentMode::Verbatim;
                }
                for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                    if matches!(position, SuppressionPosition::Inline) {
                        first_mode = ContentMode::Verbatim;
                    } else {
                        preceding_mode = ContentMode::Verbatim;
                    }
                }
            }
            if preceding_mode == ContentMode::Verbatim {
                if let Some(parameters) = &mut parameters {
                    parameters.mode = ContentMode::Verbatim;
                } else {
                    name_mode = ContentMode::Verbatim;
                }
            }
            Some(InterfaceHeritageList {
                first,
                remaining,
                first_mode,
                end,
                prefix: if source {
                    HeritagePrefix::Source(gap)
                } else {
                    HeritagePrefix::Formatted {
                        before: self.prepare_comment_boundary(before, gap.start, keyword.start),
                        after: self.prepare_comment_boundary(after, keyword.end, gap.end),
                    }
                },
            })
        } else {
            None
        };
        let body_gap = Span::new(end, body.span(ast).start);
        let comments = self.comments.in_range(body_gap);
        let source = self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
        let mut body_mode = ContentMode::Formatted;
        let mut incoming = None;
        if source {
            let mut selected = None;
            for (_, position) in self.suppressed_comment_runs(comments, body_gap.end) {
                if matches!(position, SuppressionPosition::Inline) {
                    body_mode = ContentMode::Verbatim;
                } else {
                    let owner = *selected.get_or_insert_with(|| {
                        body.body(ast)
                            .first()
                            .map_or(InterfaceBodyOwner::Body, |first| {
                                InterfaceBodyOwner::Member(ast.get_node_in_sub_range(first))
                            })
                    });
                    match owner {
                        InterfaceBodyOwner::Member(node) => {
                            incoming = Some(IncomingTypeMember {
                                node,
                                mode: ContentMode::Verbatim,
                            });
                        }
                        InterfaceBodyOwner::Body => body_mode = ContentMode::Verbatim,
                    }
                }
            }
        }
        Ok(InterfaceHead {
            name,
            name_mode,
            prefix,
            parameters,
            heritage,
            body,
            body_mode,
            incoming,
            body_prefix: if source {
                TypeGap::Source(body_gap)
            } else {
                TypeGap::Formatted(self.prepare_comment_boundary(
                    comments,
                    body_gap.start,
                    body_gap.end,
                ))
            },
        })
    }

    fn format_interface_heritage(
        &mut self,
        list: InterfaceHeritageList<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match list.prefix {
            HeritagePrefix::Source(span) => self.format_verbatim(span, writer)?,
            HeritagePrefix::Formatted { before, after } => {
                self.format_type_comment_gap(&before, FlatSeparator::Space, writer)?;
                writer.text("extends");
                self.format_type_comment_gap(&after, FlatSeparator::Space, writer)?;
            }
        }
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
                        previous.map_or(list.first_mode, |previous| previous.following_mode),
                    );
                    let tail = if item.mode == ContentMode::Verbatim {
                        self.format_verbatim(span, writer)?;
                        None
                    } else {
                        let expression = node.expression(ast);
                        if let Some(arguments) = node.type_arguments(ast) {
                            let gap =
                                Span::new(expression.span(ast).end, arguments.span(ast).start);
                            let comments = self.comments.in_range(gap);
                            let source = self.has_suppression
                                && self.content_mode(comments) == ContentMode::Verbatim;
                            let mut expression_mode = ContentMode::Formatted;
                            let mut arguments_mode = ContentMode::Formatted;
                            if source {
                                for (_, position) in self.suppressed_comment_runs(comments, gap.end)
                                {
                                    if matches!(position, SuppressionPosition::EndOfLine) {
                                        expression_mode = ContentMode::Verbatim;
                                    } else {
                                        arguments_mode = ContentMode::Verbatim;
                                    }
                                }
                            }
                            let prefix = if source {
                                TypeGap::Source(gap)
                            } else {
                                TypeGap::Formatted(
                                    self.prepare_comment_boundary(comments, gap.start, gap.end),
                                )
                            };
                            let context = ExpressionContext::at(ExpressionPosition::Nested)
                                .with_content_mode(expression_mode);
                            let tail = self.format_expression(expression, context, writer)?;
                            self.format_expression_tail(tail, writer)?;
                            self.format_type_gap(prefix, FlatSeparator::Empty, writer)?;
                            self.format_type_arguments(arguments, arguments_mode, writer)?;
                            None
                        } else {
                            self.format_expression(
                                expression,
                                ExpressionContext::at(ExpressionPosition::Nested),
                                writer,
                            )?
                        }
                    };
                    previous = Some(self.finish_delimited_item(item, tail, writer)?);
                }
                Ok(())
            })
        })
    }
}
