//! Borrowed TypeScript type owners and their declaration punctuation boundaries.

pub(super) mod callable;
mod compound;
mod conditional;
mod context;
pub(super) mod generics;
mod imports;
mod infer;
mod interfaces;
mod mapped;
mod members;
mod postfix;
mod templates;
mod tuple;

use swc_next_ecma_ast::{
    BindingIdentifier, Comment, CommentKind, ExprData, GetSpan, Span, TsLiteralData,
    TsParenthesizedType, TsQualifiedName, TsQualifiedNameLeft, TsQualifiedNameLeftData, TsType,
    TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeData, TsTypeName, TsTypeNameData,
    UnaryOperator, VariableDeclarator,
};

pub(super) use self::context::TypeContext;
pub(in crate::formatter) use self::members::PreparedIndexSignature;
use super::Formatter;
use super::bindings::{DefiniteMarker, InitializerLayout};
use super::comments::{CommentBoundary, CommentRunLayout, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::ExpressionContext;
use super::literals::StringContext;
use super::statements::DeclarationKeyword;
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use super::variables::VariableContext;
use crate::{
    FormatError,
    document::{Fragment, Writer},
};

pub(super) enum TypeGap<'ast> {
    Source(Span),
    Formatted(CommentBoundary<'ast>),
}

#[derive(Clone, Copy)]
enum TypeBoundaryRole {
    Alias,
    Qualified,
    Constraint(LineConstraint),
    Default,
}

enum TypeBoundaryTarget<'a> {
    Alias(&'a mut PreparedType),
    ModuleReference,
    Qualified,
    Constraint {
        name: BindingIdentifier,
        node: &'a mut PreparedType,
    },
    InferConstraint(&'a mut PreparedType),
    Default(&'a mut PreparedType),
}

#[derive(Clone, Copy)]
pub(super) struct PreparedType {
    pub(super) node: TsType,
    pub(super) data: TsTypeData,
    pub(super) source_owner: Option<TsType>,
}

/// Layout relationships are independent of the grammar role of a type.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeLayout {
    Root,
    SignatureParameter,
    AliasRhs,
    ConditionalConsequent,
    ConditionalAlternate,
}

struct PreparedTypeParentheses {
    span: Span,
    child_span: Span,
    child: PreparedType,
    retained: bool,
}

enum TypeBoundaryLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

pub(super) struct TypeBoundary<'ast> {
    role: TypeBoundaryRole,
    pub(super) left_mode: ContentMode,
    pub(super) right_mode: ContentMode,
    layout: TypeBoundaryLayout<'ast>,
}

pub(super) struct PreparedBindingAnnotation<'ast> {
    leading: &'ast [Comment],
    before_leading: LineConstraint,
    leading_source_gap: Option<LineConstraint>,
    colon: u32,
    prefix: BindingAnnotationPrefix<'ast>,
    content: AnnotationContent,
}

enum BindingAnnotationPrefix<'ast> {
    Source(Span),
    Formatted {
        colon: Span,
        before: LineConstraint,
        comments: &'ast [Comment],
        end: u32,
    },
}

struct AnnotationGeometry<'ast> {
    gap: Span,
    colon: Span,
    before: &'ast [Comment],
    after: &'ast [Comment],
    before_mode: ContentMode,
}

pub(super) struct PreparedAnnotation<'ast> {
    content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
}

enum AnnotationContent {
    Source(TsTypeAnnotation),
    Formatted(PreparedType),
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn prepare_type_gap(&self, gap: Span, source: bool) -> TypeGap<'ast> {
        if source {
            return TypeGap::Source(gap);
        }
        let comments = self.comments.in_range(gap);
        TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
    }

    pub(super) fn format_type_gap(
        &mut self,
        gap: TypeGap<'ast>,
        separator: FlatSeparator,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        match gap {
            TypeGap::Source(span) => self.format_verbatim(span, writer),
            TypeGap::Formatted(boundary) => {
                self.format_type_comment_gap(&boundary, separator, writer)
            }
        }
    }

    pub(super) fn format_type_alias(
        &mut self,
        alias: TsTypeAliasDeclaration,
        keyword: Option<DeclarationKeyword>,
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        let ast = self.ast;
        let span = alias.span(ast);
        let identifier = alias.id(ast);
        let identifier_span = identifier.span(ast);
        let annotation = alias.type_annotation(ast);
        let annotation_span = annotation.span(ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let keyword_end =
            keyword.map_or(span.start + "type".len() as u32, |keyword| keyword.span.end);
        let parameters = alias.type_parameters(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(identifier_span.end, node.span(ast).start),
                generics::TypeListKind::Parameters {
                    has_left_owner: true,
                },
            );
            (node, prefix)
        });
        let head_end = parameters
            .as_ref()
            .map_or(identifier_span.end, |(node, _)| node.span(ast).end);
        let mut name_mode = parameters
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
        if keyword.is_some_and(|keyword| keyword.suppresses_name(identifier)) {
            name_mode = ContentMode::Verbatim;
        }
        let boundary = self.prepare_type_boundary(
            Span::new(head_end, annotation_span.start),
            TypeBoundaryTarget::Alias(&mut type_node),
        )?;
        if self.has_suppression {
            let header = self
                .comments
                .in_range(Span::new(keyword_end, identifier_span.start));
            if self.content_mode(header) == ContentMode::Verbatim {
                name_mode = ContentMode::Verbatim;
            }
            let tail = self
                .comments
                .in_range(Span::new(annotation_span.end, span.end));
            if self.content_mode(tail) == ContentMode::Verbatim {
                type_node.source_owner = Some(annotation);
            }
        }
        if parameters.is_none() && boundary.left_mode == ContentMode::Verbatim {
            name_mode = ContentMode::Verbatim;
        }
        writer.text("type");
        self.format_source_token_gap(keyword_end, identifier_span.start, true, writer)?;
        if name_mode == ContentMode::Verbatim {
            self.format_verbatim(identifier_span, writer)?;
        } else {
            writer.text(ast.get_utf8(identifier.name(ast)));
        }
        if let Some((parameters, prefix)) = parameters {
            let mode = if boundary.left_mode == ContentMode::Verbatim {
                ContentMode::Verbatim
            } else {
                prefix.right_mode
            };
            self.format_type_list_prefix(prefix, writer)?;
            self.format_type_parameters(
                parameters,
                generics::TypeParameterContext::TypeDeclaration,
                mode,
                writer,
            )?;
        }
        let tail = self.format_type_alias_rhs(&boundary, type_node, writer)?;
        Ok((annotation_span.end, tail))
    }

    fn format_type_alias_rhs(
        &mut self,
        boundary: &TypeBoundary<'ast>,
        node: PreparedType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let (before, after) = match &boundary.layout {
            TypeBoundaryLayout::Source(gap) => {
                self.format_verbatim(*gap, writer)?;
                return self.format_prepared_type(node, TypeContext::Root, writer);
            }
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        if node.source_owner == Some(node.node) {
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type(node, TypeContext::Root, writer);
        }
        if let TsTypeData::TsParenthesizedType(parentheses) = node.data {
            let parentheses = self.prepare_type_parentheses(parentheses, node.source_owner);
            if !parentheses.retained {
                return self.format_type_alias_rhs(boundary, parentheses.child, writer);
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type_parentheses(
                parentheses,
                TypeContext::Root,
                TypeLayout::Root,
                writer,
            );
        }
        let single_compound = match node.data {
            TsTypeData::TsUnionType(union) => union.types(self.ast).len() == 1,
            TsTypeData::TsIntersectionType(intersection) => intersection.types(self.ast).len() == 1,
            _ => false,
        };
        if single_compound && node.source_owner.is_none() {
            let single = self.prepare_single_compound(node)?;
            if single.movable() {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                let (head, content) = writer.record_with_value(|writer| {
                    self.format_comment_boundary_with_layout(
                        after,
                        CommentRunLayout::Leading(single.start()),
                        writer,
                    )?;
                    self.format_single_compound_head(single, writer)
                })?;
                return self.format_type_alias_head(head, content, writer);
            }
            if matches!(node.data, TsTypeData::TsUnionType(_)) {
                self.format_comment_boundary(before, writer)?;
                before
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
                writer.text("=");
                return writer.group(false, |writer| {
                    writer.indent(|writer| {
                        self.format_comment_boundary(after, writer)?;
                        after
                            .gap()
                            .write(writer, FlatSeparator::Space, Writer::line);
                        self.format_single_type_content(
                            compound::SingleTypeContent::Compound(single),
                            TypeContext::Root,
                            TypeLayout::AliasRhs,
                            writer,
                        )
                    })
                });
            }
            self.format_type_boundary(boundary, writer)?;
            return self.format_single_type_content(
                compound::SingleTypeContent::Compound(single),
                TypeContext::Root,
                TypeLayout::Root,
                writer,
            );
        }
        if node.source_owner.is_none()
            && !after.is_empty()
            && matches!(
                self.comment_boundary_binding(after, node.node.span(self.ast)),
                super::comments::CommentBinding::Unbound
            )
        {
            self.format_comment_boundary(before, writer)?;
            before
                .gap()
                .write(writer, FlatSeparator::Space, Writer::space);
            writer.text("=");
            let (head, content) = writer.record_with_value(|writer| {
                self.format_comment_boundary_with_layout(
                    after,
                    CommentRunLayout::GroupedLeading(node.node.span(self.ast).start),
                    writer,
                )?;
                Ok::<_, FormatError>(compound::SingleTypeContent::Type(node))
            })?;
            return self.format_type_alias_head(head, content, writer);
        }
        if !matches!(node.data, TsTypeData::TsUnionType(_)) {
            self.format_type_boundary(boundary, writer)?;
            return self.format_prepared_type(node, TypeContext::Root, writer);
        }
        self.format_comment_boundary(before, writer)?;
        before
            .gap()
            .write(writer, FlatSeparator::Space, Writer::space);
        writer.text("=");
        writer.group(false, |writer| {
            writer.indent(|writer| {
                self.format_comment_boundary(after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::line);
                self.format_prepared_type_with_layout(
                    node,
                    TypeContext::Root,
                    TypeLayout::AliasRhs,
                    writer,
                )
            })
        })
    }

    fn format_type_alias_head(
        &mut self,
        head: Fragment<'ast>,
        content: compound::SingleTypeContent<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let layout = content.alias_layout();
        let expand = head.preferred_breaks();
        if expand || layout == TypeLayout::AliasRhs {
            writer.group(expand, |writer| {
                writer.indent(|writer| {
                    writer.line();
                    writer.write_fragment(head);
                    self.format_single_type_content(content, TypeContext::Root, layout, writer)
                })
            })
        } else {
            writer.space();
            writer.group(false, |writer| writer.write_fragment(head));
            self.format_single_type_content(content, TypeContext::Root, layout, writer)
        }
    }

    pub(super) fn prepare_module_reference_boundary(
        &self,
        gap: Span,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        self.prepare_type_boundary(gap, TypeBoundaryTarget::ModuleReference)
    }

    pub(super) fn prepare_qualified_name_boundary(
        &self,
        gap: Span,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        self.prepare_type_boundary(gap, TypeBoundaryTarget::Qualified)
    }

    fn prepare_type_boundary(
        &self,
        gap: Span,
        mut target: TypeBoundaryTarget<'_>,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        let (token, mut role) = match &target {
            TypeBoundaryTarget::Alias(_) | TypeBoundaryTarget::ModuleReference => {
                ("=", TypeBoundaryRole::Alias)
            }
            TypeBoundaryTarget::Qualified => (".", TypeBoundaryRole::Qualified),
            TypeBoundaryTarget::Constraint { name, .. } => {
                // A modifier-token name before `extends` needs a line terminator
                // to remain the name. Read the real identifier, including escapes.
                let leading = if matches!(
                    self.ast.get_utf8(name.name(self.ast)),
                    "const" | "in" | "out"
                ) {
                    LineConstraint::NewLine(1)
                } else {
                    LineConstraint::Unconstrained
                };
                ("extends", TypeBoundaryRole::Constraint(leading))
            }
            TypeBoundaryTarget::InferConstraint(_) => (
                "extends",
                TypeBoundaryRole::Constraint(LineConstraint::Unconstrained),
            ),
            TypeBoundaryTarget::Default(_) => ("=", TypeBoundaryRole::Default),
        };
        let comments = self.comments.in_range(gap);
        let Some(start) = self.find_token(gap, comments, token) else {
            return Err(FormatError::unsupported(
                "type punctuation boundary",
                None,
                gap,
            ));
        };
        let operator = Span::new(start, start + token.len() as u32);
        let split = comments.partition_point(|comment| comment.span.end <= operator.start);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(before, operator.start) {
                for &comment in run {
                    if !is_suppression_comment(self.ast, comment) {
                        continue;
                    }
                    let preceding = match role {
                        TypeBoundaryRole::Qualified => {
                            !matches!(position, SuppressionPosition::OwnLine)
                        }
                        TypeBoundaryRole::Constraint(_) | TypeBoundaryRole::Default => {
                            !matches!(position, SuppressionPosition::OwnLine)
                        }
                        TypeBoundaryRole::Alias => {
                            matches!(position, SuppressionPosition::Inline)
                                || matches!(position, SuppressionPosition::EndOfLine)
                                    && comment.kind == CommentKind::Line
                        }
                    };
                    if preceding {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                        if let TypeBoundaryTarget::Alias(node)
                        | TypeBoundaryTarget::Constraint { node, .. }
                        | TypeBoundaryTarget::InferConstraint(node)
                        | TypeBoundaryTarget::Default(node) = &mut target
                        {
                            self.select_type_head(node, position);
                        }
                    }
                }
            }
            for (_, position) in self.suppressed_comment_runs(after, gap.end) {
                if !matches!(role, TypeBoundaryRole::Alias)
                    && matches!(position, SuppressionPosition::EndOfLine)
                {
                    left_mode = ContentMode::Verbatim;
                } else {
                    right_mode = ContentMode::Verbatim;
                    if let TypeBoundaryTarget::Alias(node)
                    | TypeBoundaryTarget::Constraint { node, .. }
                    | TypeBoundaryTarget::InferConstraint(node)
                    | TypeBoundaryTarget::Default(node) = &mut target
                    {
                        self.select_type_head(node, position);
                    }
                }
            }
        }
        let layout = if left_mode == ContentMode::Verbatim || right_mode == ContentMode::Verbatim {
            TypeBoundaryLayout::Source(gap)
        } else {
            if matches!(
                role,
                TypeBoundaryRole::Constraint(LineConstraint::NewLine(1))
            ) && before
                .last()
                .is_some_and(|comment| self.line_breaks(gap.start, comment.span.end) != 0)
            {
                // SourceAfter prints this preceding run's existing line once.
                role = TypeBoundaryRole::Constraint(LineConstraint::Unconstrained);
            }
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, operator.start),
                after: self.prepare_comment_boundary(after, operator.end, gap.end),
            }
        };
        Ok(TypeBoundary {
            role,
            left_mode,
            right_mode,
            layout,
        })
    }

    pub(super) fn format_type_boundary(
        &mut self,
        boundary: &TypeBoundary<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let (token, space) = match boundary.role {
            TypeBoundaryRole::Alias => ("=", FlatSeparator::Space),
            TypeBoundaryRole::Qualified => (".", FlatSeparator::Empty),
            TypeBoundaryRole::Constraint(_) => ("extends", FlatSeparator::Space),
            TypeBoundaryRole::Default => ("=", FlatSeparator::Space),
        };
        let (before, after) = match &boundary.layout {
            TypeBoundaryLayout::Source(gap) => return self.format_verbatim(*gap, writer),
            TypeBoundaryLayout::Formatted { before, after } => (before, after),
        };
        self.format_comment_boundary(before, writer)?;
        if matches!(
            boundary.role,
            TypeBoundaryRole::Constraint(LineConstraint::NewLine(1))
        ) && !before.gap().requires_line()
        {
            writer.hard_line();
        } else {
            before.gap().write(writer, space, |writer| {
                if matches!(space, FlatSeparator::Space) {
                    writer.space();
                }
            });
        }
        writer.text(token);
        self.format_comment_boundary(after, writer)?;
        after.gap().write(writer, space, |writer| {
            if matches!(
                boundary.role,
                TypeBoundaryRole::Constraint(_) | TypeBoundaryRole::Default
            ) {
                writer.line();
            } else if matches!(space, FlatSeparator::Space) {
                writer.space();
            }
        });
        Ok(())
    }

    fn format_type_with_context(
        &mut self,
        annotation: TsType,
        mode: ContentMode,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let node = self.prepare_type(annotation, mode);
        self.format_prepared_type(node, context, writer)
    }

    pub(super) fn prepare_type(&self, node: TsType, mode: ContentMode) -> PreparedType {
        PreparedType {
            node,
            data: self.ast.ts_type_data(node),
            source_owner: (mode == ContentMode::Verbatim).then_some(node),
        }
    }

    fn select_type_head(&self, node: &mut PreparedType, position: SuppressionPosition) {
        if node.source_owner == Some(node.node) {
            return;
        }
        let mut data = node.data;
        let owner = if matches!(position, SuppressionPosition::OwnLine) {
            loop {
                match data {
                    TsTypeData::TsParenthesizedType(parentheses) => {
                        let child = parentheses.type_annotation(self.ast);
                        data = self.ast.ts_type_data(child);
                    }
                    TsTypeData::TsUnionType(union) => {
                        break union
                            .types(self.ast)
                            .first()
                            .map_or(node.node, |id| self.ast.get_node_in_sub_range(id));
                    }
                    _ => break node.node,
                }
            }
        } else {
            node.node
        };
        node.source_owner = Some(owner);
    }

    fn select_type_head_comments(
        &self,
        node: &mut PreparedType,
        comments: &[swc_next_ecma_ast::Comment],
        end: u32,
    ) {
        if !self.has_suppression || self.content_mode(comments) != ContentMode::Verbatim {
            return;
        }
        for (_, position) in self.suppressed_comment_runs(comments, end) {
            self.select_type_head(node, position);
        }
    }

    fn select_delimited_type_head(
        &self,
        node: &mut PreparedType,
        comments: &[Comment],
        end: u32,
        first: bool,
    ) {
        for (run, position) in self.suppressed_comment_runs(comments, end) {
            let preceding = !first
                && !matches!(position, SuppressionPosition::OwnLine)
                && (matches!(position, SuppressionPosition::EndOfLine)
                    || run.last().is_some_and(|last| {
                        self.find_comma(Span::new(last.span.end, end), comments)
                            .is_some()
                    }));
            if !preceding {
                self.select_type_head(node, position);
            }
        }
    }

    pub(super) fn format_prepared_type(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_prepared_type_with_layout(node, context, TypeLayout::Root, writer)
    }

    fn format_prepared_type_with_layout(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsParenthesizedType(parentheses) = node.data
        {
            let parentheses = self.prepare_type_parentheses(parentheses, node.source_owner);
            return self.format_prepared_type_parentheses(parentheses, context, layout, writer);
        }
        if node.source_owner.is_none() {
            let single = match node.data {
                TsTypeData::TsUnionType(union) => {
                    let types = union.types(self.ast);
                    (types.len() == 1).then_some((types, compound::CompoundTypeKind::Union))
                }
                TsTypeData::TsIntersectionType(intersection) => {
                    let types = intersection.types(self.ast);
                    (types.len() == 1).then_some((types, compound::CompoundTypeKind::Intersection))
                }
                _ => None,
            };
            if let Some((types, kind)) = single {
                return self.format_compound_type(node, types, kind, context, layout, writer);
            }
        }
        self.format_type_in_context(
            node.data,
            context,
            layout,
            writer,
            |formatter, layout, writer| formatter.format_type_data(node, layout, writer),
        )
    }

    fn format_type_in_context(
        &mut self,
        data: TsTypeData,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
        print: impl FnOnce(
            &mut Self,
            TypeLayout,
            &mut Writer<'ast>,
        ) -> Result<Option<ExpressionTail<'ast>>, FormatError>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if context.needs_parentheses(data, self.ast) {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = print(self, TypeLayout::Root, writer)?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        print(self, layout, writer)
    }

    fn format_type_data(
        &mut self,
        node: PreparedType,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if node.source_owner == Some(node.node) {
            self.format_verbatim(node.node.span(self.ast), writer)?;
            return Ok(None);
        }
        let keyword = match node.data {
            TsTypeData::TsAnyKeyword(_) => Some("any"),
            TsTypeData::TsUnknownKeyword(_) => Some("unknown"),
            TsTypeData::TsNeverKeyword(_) => Some("never"),
            TsTypeData::TsVoidKeyword(_) => Some("void"),
            TsTypeData::TsNullKeyword(_) => Some("null"),
            TsTypeData::TsUndefinedKeyword(_) => Some("undefined"),
            TsTypeData::TsStringKeyword(_) => Some("string"),
            TsTypeData::TsNumberKeyword(_) => Some("number"),
            TsTypeData::TsBigIntKeyword(_) => Some("bigint"),
            TsTypeData::TsBooleanKeyword(_) => Some("boolean"),
            TsTypeData::TsSymbolKeyword(_) => Some("symbol"),
            TsTypeData::TsObjectKeyword(_) => Some("object"),
            TsTypeData::TsIntrinsicKeyword(_) => Some("intrinsic"),
            TsTypeData::TsThisType(_) => Some("this"),
            _ => None,
        };
        if let Some(keyword) = keyword {
            writer.text(keyword);
            return Ok(None);
        }
        match node.data {
            TsTypeData::TsTypeReference(reference) => self.format_type_reference(reference, writer),
            TsTypeData::TsConditionalType(node) => {
                self.format_conditional_type(node, layout, writer)
            }
            TsTypeData::TsInferType(node) => self.format_infer_type(node, writer),
            TsTypeData::TsMappedType(node) => self.format_mapped_type(node, writer),
            TsTypeData::TsTemplateLiteralType(template) => self
                .format_template_literal_type(template, writer)
                .map(|()| None),
            TsTypeData::TsLiteralType(literal) => {
                match self.ast.ts_literal_data(literal.literal(self.ast)) {
                    TsLiteralData::TemplateLiteral(template) => {
                        self.format_zero_slot_template(template, writer)?
                    }
                    TsLiteralData::StringLiteral(node) => {
                        self.format_string_literal(node, StringContext::Expression, writer)
                    }
                    TsLiteralData::NumericLiteral(node) => {
                        self.format_numeric_literal(node, writer)
                    }
                    TsLiteralData::BigIntLiteral(node) => self.format_bigint_literal(node, writer),
                    TsLiteralData::BooleanLiteral(node) => writer.text(if node.value(self.ast) {
                        "true"
                    } else {
                        "false"
                    }),
                    TsLiteralData::UnaryExpression(node) => {
                        if node.operator(self.ast) != UnaryOperator::Negate {
                            return Err(self.unsupported("type literal operator", node));
                        }
                        let argument = node.argument(self.ast);
                        let span =
                            Span::new(node.span(self.ast).start + 1, argument.span(self.ast).start);
                        let mode = if self.has_suppression {
                            self.content_mode(self.comments.in_range(span))
                        } else {
                            ContentMode::Formatted
                        };
                        writer.text("-");
                        if mode == ContentMode::Verbatim {
                            self.format_verbatim(span, writer)?;
                            self.format_verbatim(argument.span(self.ast), writer)?;
                        } else {
                            self.format_source_token_gap(span.start, span.end, false, writer)?;
                            match self.ast.expr_data(argument) {
                                ExprData::NumericLiteral(node) => {
                                    self.format_numeric_literal(node, writer)
                                }
                                ExprData::BigIntLiteral(node) => {
                                    self.format_bigint_literal(node, writer)
                                }
                                _ => return Err(self.unsupported("type literal operand", argument)),
                            }
                        }
                    }
                }
                Ok(None)
            }
            TsTypeData::TsUnionType(union) => self.format_compound_type(
                node,
                union.types(self.ast),
                compound::CompoundTypeKind::Union,
                TypeContext::Root,
                layout,
                writer,
            ),
            TsTypeData::TsIntersectionType(intersection) => self.format_compound_type(
                node,
                intersection.types(self.ast),
                compound::CompoundTypeKind::Intersection,
                TypeContext::Root,
                layout,
                writer,
            ),
            TsTypeData::TsTypeLiteral(node) => self.format_type_literal(node, layout, writer),
            TsTypeData::TsTupleType(node) => self.format_tuple_type(node, writer),
            TsTypeData::TsFunctionType(node) => {
                self.format_callable_type(callable::CallableType::Function(node), writer)
            }
            TsTypeData::TsConstructorType(node) => {
                self.format_callable_type(callable::CallableType::Constructor(node), writer)
            }
            TsTypeData::TsTypePredicate(node) => self.format_type_predicate(node, writer),
            TsTypeData::TsOptionalType(node) => self.format_optional_type(node, writer),
            TsTypeData::TsRestType(node) => self.format_rest_type(node, writer),
            TsTypeData::TsArrayType(node) => self.format_array_type(node, writer),
            TsTypeData::TsIndexedAccessType(node) => self.format_indexed_type(node, writer),
            TsTypeData::TsTypeOperator(node) => self.format_type_operator(node, writer),
            TsTypeData::TsTypeQuery(node) => self.format_type_query(node, writer),
            TsTypeData::TsImportType(node) => self.format_import_type(node, writer),
            TsTypeData::TsParenthesizedType(node) => {
                let parentheses = self.prepare_type_parentheses(node, None);
                self.format_prepared_type_parentheses(
                    parentheses,
                    TypeContext::Root,
                    layout,
                    writer,
                )
            }
            _ => Err(self.unsupported("TypeScript type", node.node)),
        }
    }

    pub(super) fn format_type_name(
        &mut self,
        name: TsTypeName,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(name.span(self.ast), writer);
        }
        match self.ast.ts_type_name_data(name) {
            TsTypeNameData::IdentifierReference(identifier) => {
                writer.text(self.ast.get_utf8(identifier.name(self.ast)));
                Ok(())
            }
            TsTypeNameData::TsQualifiedName(name) => self.format_qualified_type_name(name, writer),
        }
    }

    pub(super) fn format_qualified_type_name(
        &mut self,
        name: TsQualifiedName,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let left = name.left(ast);
        let right = name.right(ast);
        let boundary = self.prepare_type_boundary(
            Span::new(left.span(ast).end, right.span(ast).start),
            TypeBoundaryTarget::Qualified,
        )?;
        self.format_qualified_type_left(left, boundary.left_mode, writer)?;
        self.format_type_boundary(&boundary, writer)?;
        if boundary.right_mode == ContentMode::Verbatim {
            self.format_verbatim(right.span(ast), writer)
        } else {
            writer.text(ast.get_utf8(right.name(ast)));
            Ok(())
        }
    }

    fn format_qualified_type_left(
        &mut self,
        left: TsQualifiedNameLeft,
        mode: ContentMode,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        if mode == ContentMode::Verbatim {
            return self.format_verbatim(left.span(self.ast), writer);
        }
        match self.ast.ts_qualified_name_left_data(left) {
            TsQualifiedNameLeftData::IdentifierReference(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::IdentifierName(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::BindingIdentifier(node) => {
                writer.text(self.ast.get_utf8(node.name(self.ast)))
            }
            TsQualifiedNameLeftData::ThisExpression(_) => writer.text("this"),
            TsQualifiedNameLeftData::TsQualifiedName(node) => {
                return self.format_qualified_type_name(node, writer);
            }
        }
        Ok(())
    }

    fn prepare_type_parentheses(
        &self,
        node: TsParenthesizedType,
        source_owner: Option<TsType>,
    ) -> PreparedTypeParentheses {
        let (parentheses, _, _) = self.prepare_type_parentheses_with_comments(node, source_owner);
        parentheses
    }

    fn prepare_type_parentheses_with_comments(
        &self,
        node: TsParenthesizedType,
        source_owner: Option<TsType>,
    ) -> (PreparedTypeParentheses, &'ast [Comment], &'ast [Comment]) {
        let ast = self.ast;
        let span = node.span(ast);
        let child = node.type_annotation(ast);
        let child_span = child.span(ast);
        let mut child_type = self.prepare_type(child, ContentMode::Formatted);
        child_type.source_owner = source_owner;
        let before = self
            .comments
            .in_range(Span::new(span.start + 1, child_span.start));
        let after = self
            .comments
            .in_range(Span::new(child_span.end, span.end - 1));
        let retained =
            !before.is_empty() || !after.is_empty() || self.diagnostic_range_is_sensitive(span);
        if retained {
            self.select_type_head_comments(&mut child_type, before, child_span.start);
            if self.content_mode(after) == ContentMode::Verbatim {
                child_type.source_owner = Some(child);
            }
        }
        (
            PreparedTypeParentheses {
                span,
                child_span,
                child: child_type,
                retained,
            },
            before,
            after,
        )
    }

    fn format_prepared_type_parentheses(
        &mut self,
        parentheses: PreparedTypeParentheses,
        context: TypeContext,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let PreparedTypeParentheses {
            span,
            child_span,
            child,
            retained,
        } = parentheses;
        if !retained {
            return self.format_prepared_type_with_layout(child, context, layout, writer);
        }
        writer.group(false, |writer| {
            writer.text("(");
            self.format_verbatim(Span::new(span.start + 1, child_span.start), writer)?;
            let tail = self.format_prepared_type(child, TypeContext::Root, writer)?;
            self.format_expression_tail(tail, writer)?;
            self.format_verbatim(Span::new(child_span.end, span.end - 1), writer)?;
            writer.text(")");
            Ok(None)
        })
    }

    fn prepare_annotation_content(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedAnnotation<'ast>, FormatError> {
        self.prepare_annotation_content_mode(preceding_end, annotation, ContentMode::Formatted)
    }

    pub(super) fn prepare_annotation_content_mode(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
        mode: ContentMode,
    ) -> Result<PreparedAnnotation<'ast>, FormatError> {
        let ast = self.ast;
        if mode == ContentMode::Verbatim {
            return Ok(PreparedAnnotation {
                content: AnnotationContent::Source(annotation),
                prefix: TypeBoundaryLayout::Source(Span::new(
                    preceding_end,
                    annotation.span(ast).start,
                )),
            });
        }
        let node = annotation.type_annotation(ast);
        let gap = Span::new(preceding_end, node.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("type annotation colon", annotation));
        };
        let colon = Span::new(colon_start, colon_start + 1);
        let geometry = self.annotation_geometry(gap, colon, comments);
        let content = if geometry.before_mode == ContentMode::Verbatim {
            AnnotationContent::Source(annotation)
        } else {
            let mut prepared = self.prepare_type(node, ContentMode::Formatted);
            self.select_type_head_comments(&mut prepared, geometry.after, geometry.gap.end);
            AnnotationContent::Formatted(prepared)
        };
        let prefix =
            match self.annotation_source_prefix(&content, geometry.gap.end, geometry.gap.start) {
                Some(span) => TypeBoundaryLayout::Source(span),
                None => TypeBoundaryLayout::Formatted {
                    before: self.prepare_comment_boundary(
                        geometry.before,
                        geometry.gap.start,
                        geometry.colon.start,
                    ),
                    after: self.prepare_comment_boundary(
                        geometry.after,
                        geometry.colon.end,
                        geometry.gap.end,
                    ),
                },
            };
        Ok(PreparedAnnotation { content, prefix })
    }

    fn annotation_geometry(
        &self,
        gap: Span,
        colon: Span,
        comments: &'ast [Comment],
    ) -> AnnotationGeometry<'ast> {
        let split = comments.partition_point(|comment| comment.span.end <= colon.start);
        let (before, after) = comments.split_at(split);
        AnnotationGeometry {
            gap,
            colon,
            before,
            after,
            before_mode: if self.has_suppression {
                self.content_mode(before)
            } else {
                ContentMode::Formatted
            },
        }
    }

    fn annotation_source_prefix(
        &self,
        content: &AnnotationContent,
        gap_end: u32,
        start: u32,
    ) -> Option<Span> {
        match content {
            AnnotationContent::Source(annotation) => {
                Some(Span::new(start, annotation.span(self.ast).start))
            }
            AnnotationContent::Formatted(node) if node.source_owner.is_some() => {
                Some(Span::new(start, gap_end))
            }
            AnnotationContent::Formatted(_) => None,
        }
    }

    pub(super) fn prepare_binding_annotation_content(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedBindingAnnotation<'ast>, FormatError> {
        let node = annotation.type_annotation(self.ast);
        let gap = Span::new(preceding_end, node.span(self.ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("type annotation colon", annotation));
        };
        let colon_span = Span::new(colon_start, colon_start + 1);
        let geometry = self.annotation_geometry(gap, colon_span, comments);
        let leading = geometry.before;
        let colon = geometry.colon.start;
        let before_leading = self.line_constraint(
            preceding_end,
            leading.first().map_or(colon, |comment| comment.span.start),
        );
        let leading_source_gap = if geometry.before_mode == ContentMode::Verbatim {
            Some(
                self.source_line_constraint(
                    leading
                        .last()
                        .map_or(preceding_end, |comment| comment.span.end),
                    colon,
                ),
            )
        } else {
            None
        };
        let content = if geometry.before_mode == ContentMode::Verbatim {
            AnnotationContent::Source(annotation)
        } else {
            let mut prepared = self.prepare_type(node, ContentMode::Formatted);
            self.select_type_head_comments(&mut prepared, geometry.after, geometry.gap.end);
            if prepared.source_owner == Some(prepared.node) {
                let mut selected = prepared.node;
                let mut data = prepared.data;
                while let TsTypeData::TsParenthesizedType(parentheses) = data {
                    selected = parentheses.type_annotation(self.ast);
                    data = self.ast.ts_type_data(selected);
                }
                prepared.source_owner = Some(selected);
            }
            AnnotationContent::Formatted(prepared)
        };
        let prefix = match self.annotation_source_prefix(&content, geometry.gap.end, colon) {
            Some(span) => BindingAnnotationPrefix::Source(span),
            None => BindingAnnotationPrefix::Formatted {
                colon: geometry.colon,
                before: self.line_constraint(
                    geometry.colon.end,
                    geometry
                        .after
                        .first()
                        .map_or(geometry.gap.end, |comment| comment.span.start),
                ),
                comments: geometry.after,
                end: geometry.gap.end,
            },
        };
        Ok(PreparedBindingAnnotation {
            leading,
            before_leading,
            leading_source_gap,
            colon,
            prefix,
            content,
        })
    }

    pub(super) fn format_binding_annotation_content(
        &mut self,
        annotation: PreparedBindingAnnotation<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if !annotation.leading.is_empty() {
            annotation
                .before_leading
                .write(writer, FlatSeparator::Space, Writer::space);
            self.comments.claim(annotation.leading)?;
            let layout = if annotation.leading_source_gap.is_some() {
                CommentRunLayout::Source
            } else {
                CommentRunLayout::FlatLeading(annotation.colon)
            };
            self.format_comment_run(annotation.leading, layout, writer)?;
            if let Some(gap) = annotation.leading_source_gap {
                gap.write(writer, FlatSeparator::Space, Writer::space);
            }
        }
        match annotation.prefix {
            BindingAnnotationPrefix::Source(span) => self.format_verbatim(span, writer)?,
            BindingAnnotationPrefix::Formatted {
                colon,
                before,
                comments,
                end,
            } => {
                writer.text(self.source(colon));
                before.write(writer, FlatSeparator::Space, Writer::space);
                self.comments.claim(comments)?;
                self.format_comment_run(comments, CommentRunLayout::FlatLeading(end), writer)?;
            }
        }
        self.format_annotation_type_content(annotation.content, writer)
    }

    pub(super) fn format_annotation_content(
        &mut self,
        content: PreparedAnnotation<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_annotation_content_with_layout(content, TypeLayout::Root, writer)
    }

    fn format_annotation_content_with_layout(
        &mut self,
        content: PreparedAnnotation<'ast>,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content.prefix {
            TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before.gap().write(writer, FlatSeparator::Empty, |_| {});
                writer.text(":");
                self.format_comment_boundary(&after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
            }
        }
        self.format_annotation_type_content_with_layout(content.content, layout, writer)
    }

    fn format_annotation_type_content(
        &mut self,
        content: AnnotationContent,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_annotation_type_content_with_layout(content, TypeLayout::Root, writer)
    }

    fn format_annotation_type_content_with_layout(
        &mut self,
        content: AnnotationContent,
        layout: TypeLayout,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content {
            AnnotationContent::Source(annotation) => {
                self.format_verbatim(annotation.span(self.ast), writer)?;
                Ok(None)
            }
            AnnotationContent::Formatted(type_node) => {
                self.format_prepared_type_with_layout(type_node, TypeContext::Root, layout, writer)
            }
        }
    }

    pub(super) fn format_typed_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        annotation: Option<TsTypeAnnotation>,
        context: VariableContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let pattern = declarator.id(ast);
        let pattern_span = pattern.span(ast);
        let is_definite = declarator.definite(ast);
        let mut definite = if is_definite && annotation.is_none() {
            Some(self.variable_definite_marker(declarator, annotation))
        } else {
            None
        };
        let marker_end = definite
            .as_ref()
            .and_then(|marker| marker.source)
            .map_or(pattern_span.end, |span| span.end);
        let left_end = annotation.map_or(marker_end, |annotation| annotation.span(ast).end);
        let owner = Span::new(pattern_span.start, left_end);
        let Some(right) = declarator.init(ast) else {
            if is_definite && definite.is_none() {
                definite = Some(self.variable_definite_marker(declarator, annotation));
            }
            let binding = self.prepare_annotated_binding(
                pattern,
                owner,
                definite,
                annotation,
                ContentMode::Formatted,
            )?;
            return self.format_annotated_binding(binding, writer);
        };
        let gap = Span::new(left_end, right.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(operator) = self.find_token(gap, comments, "=") else {
            return Err(self.unsupported("typed variable initializer boundary", declarator));
        };
        let split = comments.partition_point(|comment| comment.span.end <= operator);
        let (before, after) = comments.split_at(split);
        let mut left_mode = ContentMode::Formatted;
        let mut right_mode = ContentMode::Formatted;
        if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
            for (run, position) in self.suppressed_comment_runs(before, operator) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    if matches!(position, SuppressionPosition::Inline)
                        || matches!(position, SuppressionPosition::EndOfLine)
                            && comment.kind == CommentKind::Line
                    {
                        left_mode = ContentMode::Verbatim;
                    } else {
                        right_mode = ContentMode::Verbatim;
                    }
                }
            }
            for (run, position) in self.suppressed_comment_runs(after, gap.end) {
                for &comment in run {
                    if !is_suppression_comment(ast, comment) {
                        continue;
                    }
                    if matches!(position, SuppressionPosition::EndOfLine)
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
            InitializerLayout::Bound { gap, right_mode }
        } else {
            let content = self.expression_content(right);
            let boundary =
                self.assignment_boundary(left_end, &content, Span::new(operator, operator + 1))?;
            let rhs_layout = self.ordinary_rhs_layout(content.data, &boundary);
            InitializerLayout::Formatted {
                boundary,
                rhs_layout,
            }
        };
        if left_mode == ContentMode::Verbatim {
            self.format_verbatim(owner, writer)?;
        } else {
            if is_definite && definite.is_none() {
                definite = Some(self.variable_definite_marker(declarator, annotation));
            }
            let binding = self.prepare_annotated_binding(
                pattern,
                owner,
                definite,
                annotation,
                ContentMode::Formatted,
            )?;
            let tail = self.format_annotated_binding(binding, writer)?;
            self.format_expression_tail(tail, writer)?;
        }
        let context = if context == VariableContext::ForInitializer {
            self.for_initializer_context(right)
        } else {
            ExpressionContext::variable_initializer()
        };
        self.format_initializer_right(left_end, right, layout, context, writer)
    }

    fn variable_definite_marker(
        &self,
        declarator: VariableDeclarator,
        annotation: Option<TsTypeAnnotation>,
    ) -> DefiniteMarker {
        if annotation.is_some() {
            return DefiniteMarker { source: None };
        }
        let ast = self.ast;
        let end = declarator
            .init(ast)
            .map_or(declarator.span(ast).end, |right| right.span(ast).start);
        let gap = Span::new(declarator.id(ast).span(ast).end, end);
        let source = self
            .find_token(gap, self.comments.in_range(gap), "!")
            .map(|marker| Span::new(marker, marker + 1));
        DefiniteMarker { source }
    }
}
