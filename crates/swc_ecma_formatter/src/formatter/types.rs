//! Borrowed TypeScript type owners and their declaration punctuation boundaries.

mod callable;
mod compound;
mod context;
mod generics;
mod interfaces;
mod members;
mod postfix;
mod tuple;

use swc_next_ecma_ast::{
    BindingIdentifier, BindingPattern, Comment, CommentKind, ExprData, GetSpan, Span,
    TsLiteralData, TsParenthesizedType, TsQualifiedName, TsQualifiedNameLeft,
    TsQualifiedNameLeftData, TsType, TsTypeAliasDeclaration, TsTypeAnnotation, TsTypeData,
    TsTypeName, TsTypeNameData, UnaryOperator, VariableDeclarator,
};

use self::context::TypeContext;
use super::Formatter;
use super::bindings::InitializerLayout;
use super::comments::{CommentBoundary, ExpressionTail};
use super::diagnostics::{FlatSeparator, LineConstraint};
use super::expressions::ExpressionContext;
use super::literals::StringContext;
use super::suppression::{ContentMode, SuppressionPosition, is_suppression_comment};
use super::variables::VariableContext;
use crate::{FormatError, document::Writer};

enum TypeGap<'ast> {
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
    Qualified,
    Constraint {
        name: BindingIdentifier,
        node: &'a mut PreparedType,
    },
    Default(&'a mut PreparedType),
}

#[derive(Clone, Copy)]
struct PreparedType {
    node: TsType,
    data: TsTypeData,
    source_owner: Option<TsType>,
}

enum TypeBoundaryLayout<'ast> {
    Source(Span),
    Formatted {
        before: CommentBoundary<'ast>,
        after: CommentBoundary<'ast>,
    },
}

struct TypeBoundary<'ast> {
    role: TypeBoundaryRole,
    left_mode: ContentMode,
    right_mode: ContentMode,
    layout: TypeBoundaryLayout<'ast>,
}

struct PreparedTypeAnnotation<'ast> {
    pattern: BindingPattern,
    content: PreparedAnnotation<'ast>,
}

struct PreparedAnnotation<'ast> {
    annotation: TsTypeAnnotation,
    wrapper_mode: ContentMode,
    type_node: PreparedType,
    prefix: TypeBoundaryLayout<'ast>,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    fn prepare_type_gap(&self, gap: Span, source: bool) -> TypeGap<'ast> {
        if source {
            return TypeGap::Source(gap);
        }
        let comments = self.comments.in_range(gap);
        TypeGap::Formatted(self.prepare_comment_boundary(comments, gap.start, gap.end))
    }

    fn format_type_gap(
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
        writer: &mut Writer<'ast>,
    ) -> Result<(u32, Option<ExpressionTail<'ast>>), FormatError> {
        let ast = self.ast;
        if alias.declare(ast) {
            return Err(self.unsupported("ambient type alias", alias));
        }
        let span = alias.span(ast);
        let identifier = alias.id(ast);
        let identifier_span = identifier.span(ast);
        let annotation = alias.type_annotation(ast);
        let annotation_span = annotation.span(ast);
        let mut type_node = self.prepare_type(annotation, ContentMode::Formatted);
        let keyword_end = span.start + "type".len() as u32;
        let parameters = alias.type_parameters(ast).map(|node| {
            let prefix = self.prepare_type_list_prefix(
                Span::new(identifier_span.end, node.span(ast).start),
                generics::TypeListKind::Parameters,
            );
            (node, prefix)
        });
        let head_end = parameters
            .as_ref()
            .map_or(identifier_span.end, |(node, _)| node.span(ast).end);
        let mut name_mode = parameters
            .as_ref()
            .map_or(ContentMode::Formatted, |(_, prefix)| prefix.left_mode);
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
        self.format_type_boundary(&boundary, writer)?;
        let tail = self.format_prepared_type(type_node, TypeContext::Root, writer)?;
        Ok((annotation_span.end, tail))
    }

    fn prepare_type_boundary(
        &self,
        gap: Span,
        mut target: TypeBoundaryTarget<'_>,
    ) -> Result<TypeBoundary<'ast>, FormatError> {
        let (token, mut role) = match &target {
            TypeBoundaryTarget::Alias(_) => ("=", TypeBoundaryRole::Alias),
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

    fn format_type_boundary(
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

    fn prepare_type(&self, node: TsType, mode: ContentMode) -> PreparedType {
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

    fn format_prepared_type(
        &mut self,
        node: PreparedType,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        if node.source_owner != Some(node.node)
            && let TsTypeData::TsParenthesizedType(parentheses) = node.data
        {
            return self.format_parenthesized_type(parentheses, node.source_owner, context, writer);
        }
        if context.needs_parentheses(node.data) {
            return writer.group(false, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.soft_line();
                    let tail = self.format_type_data(node, writer)?;
                    self.format_expression_tail(tail, writer)
                })?;
                writer.soft_line();
                writer.text(")");
                Ok(None)
            });
        }
        self.format_type_data(node, writer)
    }

    fn format_type_data(
        &mut self,
        node: PreparedType,
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
            TsTypeData::TsLiteralType(literal) => {
                match self.ast.ts_literal_data(literal.literal(self.ast)) {
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
                    _ => return Err(self.unsupported("type literal", literal)),
                }
                Ok(None)
            }
            TsTypeData::TsUnionType(union) => self.format_compound_type(
                node,
                union.types(self.ast),
                compound::CompoundTypeKind::Union,
                writer,
            ),
            TsTypeData::TsIntersectionType(intersection) => self.format_compound_type(
                node,
                intersection.types(self.ast),
                compound::CompoundTypeKind::Intersection,
                writer,
            ),
            TsTypeData::TsTypeLiteral(node) => self.format_type_literal(node, writer),
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
            TsTypeData::TsParenthesizedType(node) => {
                self.format_parenthesized_type(node, None, TypeContext::Root, writer)
            }
            _ => Err(self.unsupported("TypeScript type", node.node)),
        }
    }

    fn format_type_name(
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

    fn format_qualified_type_name(
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

    fn format_parenthesized_type(
        &mut self,
        node: TsParenthesizedType,
        source_owner: Option<TsType>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
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
        if before.is_empty() && after.is_empty() && !self.diagnostic_range_is_sensitive(span) {
            return self.format_prepared_type(child_type, context, writer);
        }
        self.select_type_head_comments(&mut child_type, before, child_span.start);
        if self.content_mode(after) == ContentMode::Verbatim {
            child_type.source_owner = Some(child);
        }
        writer.group(false, |writer| {
            writer.text("(");
            self.format_verbatim(Span::new(span.start + 1, child_span.start), writer)?;
            let tail = self.format_prepared_type(child_type, TypeContext::Root, writer)?;
            self.format_expression_tail(tail, writer)?;
            self.format_verbatim(Span::new(child_span.end, span.end - 1), writer)?;
            writer.text(")");
            Ok(None)
        })
    }

    fn prepare_type_annotation(
        &self,
        pattern: BindingPattern,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedTypeAnnotation<'ast>, FormatError> {
        let content = self.prepare_annotation_content(pattern.span(self.ast).end, annotation)?;
        Ok(PreparedTypeAnnotation { pattern, content })
    }

    fn prepare_annotation_content(
        &self,
        preceding_end: u32,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedAnnotation<'ast>, FormatError> {
        let ast = self.ast;
        let type_node = annotation.type_annotation(ast);
        let gap = Span::new(preceding_end, type_node.span(ast).start);
        let comments = self.comments.in_range(gap);
        let Some(colon_start) = self.find_token(gap, comments, ":") else {
            return Err(self.unsupported("type annotation colon", annotation));
        };
        let split = comments.partition_point(|comment| comment.span.end <= colon_start);
        let (before, after) = comments.split_at(split);
        let wrapper_mode = self.content_mode(before);
        let mut type_node = self.prepare_type(type_node, ContentMode::Formatted);
        if wrapper_mode == ContentMode::Formatted {
            self.select_type_head_comments(&mut type_node, after, gap.end);
        }
        let prefix = if wrapper_mode == ContentMode::Verbatim {
            TypeBoundaryLayout::Source(Span::new(gap.start, annotation.span(ast).start))
        } else if type_node.source_owner.is_some() {
            TypeBoundaryLayout::Source(gap)
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, gap.start, colon_start),
                after: self.prepare_comment_boundary(after, colon_start + 1, gap.end),
            }
        };
        Ok(PreparedAnnotation {
            annotation,
            wrapper_mode,
            type_node,
            prefix,
        })
    }

    fn format_annotation_content(
        &mut self,
        content: PreparedAnnotation<'ast>,
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
        if content.wrapper_mode == ContentMode::Verbatim {
            self.format_verbatim(content.annotation.span(self.ast), writer)?;
            Ok(None)
        } else {
            self.format_prepared_type(content.type_node, TypeContext::Root, writer)
        }
    }

    fn format_annotated_binding(
        &mut self,
        binding: PreparedTypeAnnotation<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let tail =
            self.format_declaration_binding(binding.pattern, ContentMode::Formatted, writer)?;
        self.format_expression_tail(tail, writer)?;
        let tail = self.format_annotation_content(binding.content, writer)?;
        self.format_expression_tail(tail, writer)
    }

    pub(super) fn format_typed_variable_declarator(
        &mut self,
        declarator: VariableDeclarator,
        annotation: TsTypeAnnotation,
        context: VariableContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let Some(right) = declarator.init(ast) else {
            let binding = self.prepare_type_annotation(declarator.id(ast), annotation)?;
            self.format_annotated_binding(binding, writer)?;
            return Ok(None);
        };
        let left_end = annotation.span(ast).end;
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
            let break_after = boundary.own_line
                || !boundary.operator_line.is_empty()
                || self.assignment_right_breaks_after_operator(content.data);
            InitializerLayout::Formatted {
                boundary,
                break_after,
            }
        };
        if left_mode == ContentMode::Verbatim {
            self.format_verbatim(
                Span::new(declarator.id(ast).span(ast).start, left_end),
                writer,
            )?;
        } else {
            let binding = self.prepare_type_annotation(declarator.id(ast), annotation)?;
            self.format_annotated_binding(binding, writer)?;
        }
        let context = if context == VariableContext::ForInitializer {
            self.for_initializer_context(right)
        } else {
            ExpressionContext::variable_initializer()
        };
        self.format_initializer_right(left_end, right, layout, context, writer)
    }
}
