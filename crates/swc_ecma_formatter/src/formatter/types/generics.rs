//! Real generic list owners share the item/comma protocol and retain child bindings.

use swc_next_ecma_ast::{
    BindingIdentifier, CommentKind, GetSpan, Span, TsType, TsTypeParameter,
    TsTypeParameterDeclaration, TsTypeParameterInstantiation, TsTypeReference,
};

use super::{Formatter, PreparedType, TypeBoundary, TypeBoundaryTarget, TypeContext};
use crate::formatter::comments::{CommentBoundary, CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::expressions::arguments::{
    ArgumentLayout, FollowingItem, Separator, TrailingCommaPolicy,
};
use crate::formatter::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use crate::{FormatError, TrailingCommas, document::Writer};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeListKind {
    Arguments,
    Parameters { has_left_owner: bool },
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeParameterContext {
    TypeDeclaration,
    Callable,
    Class,
    Infer,
}

#[derive(Clone, Copy)]
enum TypeList {
    Arguments(TsTypeParameterInstantiation),
    Parameters(TsTypeParameterDeclaration),
}

impl TypeList {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        match self {
            Self::Arguments(node) => node.span(ast),
            Self::Parameters(node) => node.span(ast),
        }
    }
}

#[derive(Clone, Copy)]
enum GenericMember {
    Type(TsType),
    Parameter(TsTypeParameter, TypeParameterContext),
}

impl GenericMember {
    fn span(self, ast: &swc_next_ecma_ast::Ast<'_>) -> Span {
        match self {
            Self::Type(node) => node.span(ast),
            Self::Parameter(node, _) => node.span(ast),
        }
    }
}

enum PreparedGenericMember {
    Type(PreparedType),
    Parameter(TsTypeParameter, TypeParameterContext),
}

enum TypeListPrefixLayout<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

pub(in crate::formatter) struct TypeListPrefix<'ast> {
    pub(in crate::formatter) left_mode: ContentMode,
    pub(in crate::formatter) right_mode: ContentMode,
    layout: TypeListPrefixLayout<'ast>,
}

struct ParameterBound<'ast> {
    node: PreparedType,
    boundary: TypeBoundary<'ast>,
}

#[derive(Clone, Copy)]
struct TypeParameterModifiers {
    r#const: bool,
    r#in: bool,
    out: bool,
}

impl TypeParameterModifiers {
    fn is_empty(self) -> bool {
        !self.r#const && !self.r#in && !self.out
    }
}

enum PreparedParameterPrefix<'ast> {
    Canonical {
        modifiers: TypeParameterModifiers,
        name_leading: Option<CommentBoundary<'ast>>,
    },
    Source(Span),
}

pub(super) struct PreparedParameter<'ast> {
    prefix: PreparedParameterPrefix<'ast>,
    name: BindingIdentifier,
    name_mode: ContentMode,
    constraint: Option<ParameterBound<'ast>>,
    default: Option<ParameterBound<'ast>>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn prepare_type_list_prefix(
        &self,
        gap: Span,
        kind: TypeListKind,
    ) -> TypeListPrefix<'ast> {
        let comments = self.comments.in_range(gap);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(comments, gap.end) {
                for &comment in run {
                    if !is_suppression_comment(self.ast, comment) {
                        continue;
                    }
                    if matches!(
                        kind,
                        TypeListKind::Parameters {
                            has_left_owner: true
                        }
                    ) && matches!(position, SuppressionPosition::EndOfLine)
                        && comment.kind == CommentKind::Line
                    {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                    }
                }
            }
        }
        let layout = if left_mode == ContentMode::Verbatim || right_mode == ContentMode::Verbatim {
            TypeListPrefixLayout::Source(gap)
        } else {
            TypeListPrefixLayout::Formatted(
                self.prepare_comment_boundary(comments, gap.start, gap.end),
            )
        };
        TypeListPrefix {
            left_mode,
            right_mode,
            layout,
        }
    }

    pub(in crate::formatter) fn format_type_list_prefix(
        &mut self,
        prefix: TypeListPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match prefix.layout {
            TypeListPrefixLayout::Source(gap) => self.format_verbatim(gap, writer),
            TypeListPrefixLayout::Formatted(boundary) => {
                self.format_type_comment_gap(&boundary, FlatSeparator::Empty, writer)
            }
        }
    }

    pub(super) fn format_type_reference(
        &mut self,
        node: TsTypeReference,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = node.type_name(ast);
        let arguments = node.type_arguments(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(name.span(ast).end, node.span(ast).start),
                TypeListKind::Arguments,
            );
            (node, prefix)
        });
        let mode = arguments
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
        self.format_type_name(name, mode, writer)?;
        if let Some((arguments, prefix)) = arguments {
            let mode = prefix.right_mode;
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_arguments(arguments, mode, writer)?;
        }
        Ok(None)
    }

    pub(in crate::formatter) fn format_type_arguments(
        &mut self,
        node: TsTypeParameterInstantiation,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        let ast = self.ast;
        let members = node
            .params(ast)
            .iter()
            .map(|id| GenericMember::Type(ast.get_node_in_sub_range(id)));
        self.format_generic_list(
            TypeList::Arguments(node),
            members,
            TrailingCommaPolicy::Forbidden,
            writer,
        )
    }

    pub(in crate::formatter) fn format_type_parameters(
        &mut self,
        node: TsTypeParameterDeclaration,
        context: TypeParameterContext,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail = if self.options.trailing_commas == TrailingCommas::None {
            TrailingCommaPolicy::Forbidden
        } else {
            TrailingCommaPolicy::IfBreak
        };
        self.format_type_parameters_with_tail(node, context, mode, tail, writer)
    }

    pub(in crate::formatter) fn format_type_parameters_with_tail(
        &mut self,
        node: TsTypeParameterDeclaration,
        context: TypeParameterContext,
        mode: ContentMode,
        tail: TrailingCommaPolicy,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(node.span(self.ast), writer);
        }
        let ast = self.ast;
        let members = node
            .params(ast)
            .iter()
            .map(|id| GenericMember::Parameter(ast.get_node_in_sub_range(id), context));
        self.format_generic_list(TypeList::Parameters(node), members, tail, writer)
    }

    fn format_generic_list(
        &mut self,
        node: TypeList,
        members: impl Iterator<Item = GenericMember>,
        tail: TrailingCommaPolicy,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let mut members = members.peekable();
        let mut first_type = None;
        let leading_source = if self.has_suppression {
            members.peek().and_then(|member| {
                let GenericMember::Type(first) = *member else {
                    return None;
                };
                let gap = Span::new(content.start, first.span(ast).start);
                let comments = self.comments.in_range(gap);
                if self.content_mode(comments) != ContentMode::Verbatim {
                    return None;
                }
                let mut node = self.prepare_type(first, ContentMode::Formatted);
                self.select_delimited_type_head(&mut node, comments, gap.end, true);
                let mut moved = node;
                moved.source_owner = None;
                self.select_type_head(&mut moved, SuppressionPosition::OwnLine);
                first_type = Some(node);
                (moved.source_owner != node.source_owner).then_some(gap)
            })
        } else {
            None
        };
        writer.group(false, |writer| {
            writer.text("<");
            let end = writer.indent(|writer| {
                if leading_source.is_none() {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut end = content.start;
                let mut source_end = content.start;
                let mut separator = Separator::Line;
                let mut following_mode = None;
                while let Some(member) = members.next() {
                    let member_span = member.span(ast);
                    let leading = self
                        .comments
                        .in_range(Span::new(source_end, member_span.start));
                    let mode = following_mode.unwrap_or_else(|| self.content_mode(leading));
                    let mut prepared = match member {
                        GenericMember::Type(node) => PreparedGenericMember::Type(
                            first_type.take().unwrap_or_else(|| self.prepare_type(node, ContentMode::Formatted)),
                        ),
                        GenericMember::Parameter(node, context) => PreparedGenericMember::Parameter(node, context),
                    };
                    if mode == ContentMode::Verbatim
                        && let PreparedGenericMember::Type(node) = &mut prepared
                        && node.source_owner.is_none()
                    {
                        self.select_delimited_type_head(node, leading, member_span.start, following_mode.is_none());
                    }
                    if following_mode.is_some() {
                        let next_start = self
                            .comments
                            .between(Span::new(end, member_span.start))?
                            .first()
                            .map_or(member_span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next_start),
                            writer,
                        );
                    }
                    let following = members.peek().map_or(FollowingItem::End, |member| {
                        FollowingItem::Node(member.span(ast))
                    });
                    // The first marker's physical run selects Whole/FirstMember.
                    // Keep that delimiter gap before claiming the comments, so a
                    // broken list cannot change its actual type owner next pass.
                    let item_start = if following_mode.is_none()
                        && let Some(gap) = leading_source
                    {
                        self.format_verbatim(gap, writer)?;
                        member_span.start
                    } else {
                        end
                    };
                    let mut item = self.prepare_delimited_item(
                        member_span,
                        following,
                        Span::new(item_start, content.end),
                        false,
                        writer,
                    )?;
                    item.set_trailing_comma(tail);
                    let selected_child = matches!(&prepared, PreparedGenericMember::Type(node) if node.source_owner.is_some_and(|owner| owner != node.node));
                    item.set_leading_mode(if selected_child { ContentMode::Formatted } else { mode });
                    let tail = match prepared {
                        PreparedGenericMember::Type(mut node) => {
                            if item.mode == ContentMode::Verbatim {
                                node.source_owner = Some(node.node);
                            }
                            self.format_prepared_type(node, TypeContext::Root, writer)?
                        }
                        PreparedGenericMember::Parameter(node, context) => self.format_type_parameter(node, context, item.mode, writer)?,
                    };
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    source_end = member_span.end;
                    separator = boundary.separator;
                    following_mode = Some(boundary.following_mode);
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                let dangling = self.comments.take(dangling.len());
                if dangling
                    .iter()
                    .any(|comment| comment.kind == CommentKind::Line)
                {
                    writer.break_parent();
                }
                if following_mode.is_some()
                    && let Some(first) = dangling.first()
                {
                    self.comment_gap(end, first.span.start, writer);
                }
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>(dangling.last().map_or(end, |comment| comment.span.end))
            })?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            writer.text(">");
            Ok(())
        })
    }

    pub(super) fn prepare_type_parameter(
        &self,
        node: TsTypeParameter,
        context: TypeParameterContext,
    ) -> Result<PreparedParameter<'ast>, FormatError> {
        let ast = self.ast;
        let modifiers = TypeParameterModifiers {
            r#const: node.r#const(ast),
            r#in: node.r#in(ast),
            out: node.out(ast),
        };
        if matches!(context, TypeParameterContext::Infer)
            && (!modifiers.is_empty() || node.default(ast).is_some())
        {
            return Err(self.unsupported("infer type parameter modifiers or default", node));
        }
        let name = node.name(ast);
        let name_end = name.span(ast).end;
        let mut name_mode = ContentMode::Formatted;
        let mut constraint = if let Some(type_node) = node.constraint(ast) {
            let mut type_node = self.prepare_type(type_node, ContentMode::Formatted);
            let boundary = self.prepare_type_boundary(
                Span::new(name_end, type_node.node.span(ast).start),
                if matches!(context, TypeParameterContext::Infer) {
                    TypeBoundaryTarget::InferConstraint(&mut type_node)
                } else {
                    TypeBoundaryTarget::Constraint {
                        name,
                        node: &mut type_node,
                    }
                },
            )?;
            name_mode = boundary.left_mode;
            Some(ParameterBound {
                node: type_node,
                boundary,
            })
        } else {
            None
        };
        let default = if let Some(type_node) = node.default(ast) {
            let mut type_node = self.prepare_type(type_node, ContentMode::Formatted);
            let left_end = constraint
                .as_ref()
                .map_or(name_end, |bound| bound.node.node.span(ast).end);
            let boundary = self.prepare_type_boundary(
                Span::new(left_end, type_node.node.span(ast).start),
                TypeBoundaryTarget::Default(&mut type_node),
            )?;
            if boundary.left_mode == ContentMode::Verbatim {
                if let Some(bound) = &mut constraint {
                    bound.node.source_owner = Some(bound.node.node);
                } else {
                    name_mode = ContentMode::Verbatim;
                }
            }
            Some(ParameterBound {
                node: type_node,
                boundary,
            })
        } else {
            None
        };
        let prefix = self.prepare_parameter_prefix(node, name, modifiers, &mut name_mode);
        Ok(PreparedParameter {
            prefix,
            name,
            name_mode,
            constraint,
            default,
        })
    }

    fn prepare_parameter_prefix(
        &self,
        node: TsTypeParameter,
        name: BindingIdentifier,
        modifiers: TypeParameterModifiers,
        name_mode: &mut ContentMode,
    ) -> PreparedParameterPrefix<'ast> {
        if modifiers.is_empty() || self.ast.comments().is_empty() {
            return PreparedParameterPrefix::Canonical {
                modifiers,
                name_leading: None,
            };
        }
        let name_span = name.span(self.ast);
        let span = Span::new(node.span(self.ast).start, name_span.start);
        let comments = self.comments.in_range(span);
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            *name_mode = ContentMode::Verbatim;
        }
        let binding = self.comment_binding(comments, span, Some(name_span));
        if !binding.allows_relocation(Some(name_span)) {
            return PreparedParameterPrefix::Source(span);
        }
        let name_leading = (!comments.is_empty())
            .then(|| self.prepare_comment_boundary(comments, span.start, span.end));
        PreparedParameterPrefix::Canonical {
            modifiers,
            name_leading,
        }
    }

    fn format_type_parameter(
        &mut self,
        node: TsTypeParameter,
        context: TypeParameterContext,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if mode == ContentMode::Verbatim {
            self.format_verbatim(node.span(self.ast), writer)?;
            return Ok(None);
        }
        let parameter = self.prepare_type_parameter(node, context)?;
        self.format_prepared_type_parameter(parameter, writer)
    }

    pub(super) fn format_prepared_type_parameter(
        &mut self,
        parameter: PreparedParameter<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        writer.group(false, |writer| {
            self.format_parameter_prefix(parameter.prefix, writer)?;
            if parameter.name_mode == ContentMode::Verbatim {
                self.format_verbatim(parameter.name.span(ast), writer)?;
            } else {
                writer.text(ast.get_utf8(parameter.name.name(ast)));
            }
            writer.indent(|writer| {
                if let Some(bound) = parameter.constraint {
                    self.format_type_boundary(&bound.boundary, writer)?;
                    let tail =
                        self.format_prepared_type(bound.node, TypeContext::TypeConstraint, writer)?;
                    if parameter.default.is_none() {
                        return Ok(tail);
                    }
                }
                if let Some(bound) = parameter.default {
                    self.format_type_boundary(&bound.boundary, writer)?;
                    return self.format_prepared_type(bound.node, TypeContext::Root, writer);
                }
                Ok(None)
            })
        })
    }

    fn format_parameter_prefix(
        &mut self,
        prefix: PreparedParameterPrefix<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (modifiers, name_leading) = match prefix {
            PreparedParameterPrefix::Source(span) => return self.format_verbatim(span, writer),
            PreparedParameterPrefix::Canonical {
                modifiers,
                name_leading,
            } => (modifiers, name_leading),
        };
        if modifiers.r#const {
            writer.text("const");
            writer.space();
        }
        if modifiers.r#in {
            writer.text("in");
            writer.space();
        }
        if modifiers.out {
            writer.text("out");
            writer.space();
        }
        if let Some(boundary) = name_leading {
            self.format_type_comment_gap(&boundary, FlatSeparator::Space, writer)?;
        }
        Ok(())
    }
}
