//! Shared parameter items retain real bindings and annotation wrappers.

use swc_next_ecma_ast::{
    Ast, BindingPattern, Expr, FormalParameter, FormalParameterItemData, FormalParameterRest,
    FormalParameters, GetSpan, Span, TsConstructorType, TsFunctionType, TsThisParameter,
    TsTypeAnnotation, TsTypeParameterDeclaration, TsTypePredicate, TsTypePredicateNameData,
};

use super::generics::TypeParameterContext;
use super::{
    Formatter, PreparedAnnotation, PreparedType, TypeBoundaryLayout, TypeContext, TypeGap,
};
use crate::formatter::bindings::{DefaultOwner, InitializerLayout};
use crate::formatter::comments::{CommentRunLayout, ExpressionTail};
use crate::formatter::diagnostics::{FlatSeparator, LineConstraint};
use crate::formatter::expressions::ExpressionContext;
use crate::formatter::expressions::arguments::{ArgumentLayout, FollowingItem, Separator};
use crate::formatter::suppression::{ContentMode, DelimitedSuppression, SuppressionPosition};
use crate::{FormatError, TrailingCommas, document::Writer};

#[derive(Clone, Copy)]
pub(super) enum CallableType {
    Function(TsFunctionType),
    Constructor(TsConstructorType),
}

struct CallableParts {
    parameters: FormalParameters,
    this_parameter: Option<TsThisParameter>,
    type_parameters: Option<TsTypeParameterDeclaration>,
    return_type: TsTypeAnnotation,
}

impl CallableType {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Function(node) => node.span(ast),
            Self::Constructor(node) => node.span(ast),
        }
    }

    fn parts(self, ast: &Ast<'_>) -> CallableParts {
        match self {
            Self::Function(node) => CallableParts {
                parameters: node.params(ast),
                this_parameter: node.this_param(ast),
                type_parameters: node.type_parameters(ast),
                return_type: node.return_type(ast),
            },
            Self::Constructor(node) => CallableParts {
                parameters: node.params(ast),
                this_parameter: None,
                type_parameters: node.type_parameters(ast),
                return_type: node.return_type(ast),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum SignatureIncoming {
    Parameter(CallableParameter),
    Return(TsTypeAnnotation),
    Empty,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::formatter) enum CallableParameter {
    This(TsThisParameter),
    Formal(FormalParameter),
    Rest(FormalParameterRest),
}

impl CallableParameter {
    pub(in crate::formatter) fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::This(node) => node.span(ast),
            Self::Formal(node) => node.span(ast),
            Self::Rest(node) => node.span(ast),
        }
    }
}

pub(super) enum AnnotationContent {
    Wrapper(TsTypeAnnotation),
    Type(PreparedType),
}

struct PredicateBound<'ast> {
    content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
}

pub(in crate::formatter) struct SignatureReturn<'ast> {
    pub(super) content: AnnotationContent,
    prefix: TypeBoundaryLayout<'ast>,
    pub(in crate::formatter) preceding_mode: ContentMode,
    delimiter: SignatureReturnDelimiter,
}

#[derive(Clone, Copy)]
pub(in crate::formatter) enum SignatureReturnDelimiter {
    Arrow,
    Colon,
}

pub(in crate::formatter) enum SignatureReturnEolTarget<'mode> {
    Parameter,
    Header(&'mode mut ContentMode),
    Annotation,
}

impl SignatureReturnDelimiter {
    fn token(self) -> &'static str {
        match self {
            Self::Arrow => "=>",
            Self::Colon => ":",
        }
    }

    fn preceding_space(self) -> FlatSeparator {
        match self {
            Self::Arrow => FlatSeparator::Space,
            Self::Colon => FlatSeparator::Empty,
        }
    }
}

enum CallableHead<'ast> {
    Function,
    Constructor { abstract_gap: Option<TypeGap<'ast>> },
}

#[derive(Clone, Copy)]
enum CallableIncomingOwner {
    Parameter(CallableParameter),
    Return(TsTypeAnnotation),
}

struct CallableTerminal {
    parameter: CallableParameter,
    closing: LineConstraint,
}

pub(in crate::formatter) struct CallableParameters {
    node: FormalParameters,
    this_parameter: Option<TsThisParameter>,
    has_parameters: bool,
    pub(in crate::formatter) source_head: Option<CallableParameter>,
    suppressed_tail: Option<CallableParameter>,
    terminal: Option<CallableTerminal>,
}

impl CallableParameters {
    pub(in crate::formatter) fn has_parameters(&self) -> bool {
        self.has_parameters
    }
}

enum ParameterBinding<'ast> {
    Pattern {
        node: BindingPattern,
        mode: ContentMode,
        spread: Option<TypeGap<'ast>>,
    },
    This(TsThisParameter),
}

struct PreparedParameterDefault<'ast> {
    left: Span,
    right: Expr,
    left_mode: ContentMode,
    layout: InitializerLayout<'ast>,
}

struct ParameterLeft<'ast> {
    binding: ParameterBinding<'ast>,
    optional: bool,
    annotation: Option<PreparedAnnotation<'ast>>,
    end: u32,
}

enum PreparedParameterLeft<'ast> {
    Source(Span),
    Formatted(ParameterLeft<'ast>),
}

impl PreparedParameterLeft<'_> {
    fn end(&self) -> u32 {
        match self {
            Self::Source(span) => span.end,
            Self::Formatted(left) => left.end,
        }
    }
}

#[derive(Clone, Copy)]
enum ParameterItemSource {
    Callable(CallableParameter),
    Property(swc_next_ecma_ast::TsParameterProperty),
}

impl ParameterItemSource {
    fn span(self, ast: &Ast<'_>) -> Span {
        match self {
            Self::Callable(node) => node.span(ast),
            Self::Property(node) => node.span(ast),
        }
    }
}

enum ParameterItemState<'ast> {
    Source(ParameterItemSource),
    Formatted(ParameterLeft<'ast>),
    Default {
        left: PreparedParameterLeft<'ast>,
        right: Expr,
        layout: InitializerLayout<'ast>,
    },
}

pub(in crate::formatter) struct PreparedParameterItem<'ast>(ParameterItemState<'ast>);

impl PreparedParameterItem<'_> {
    pub(in crate::formatter) fn is_source(&self) -> bool {
        matches!(self.0, ParameterItemState::Source(_))
    }
}

#[derive(Clone, Copy)]
enum ParameterConsumer {
    CallableSignature,
    Runtime,
    ConstructorProperty(swc_next_ecma_ast::TsParameterProperty),
}

impl ParameterConsumer {
    fn source(self, node: CallableParameter) -> ParameterItemSource {
        match self {
            Self::ConstructorProperty(property) => ParameterItemSource::Property(property),
            Self::CallableSignature | Self::Runtime => ParameterItemSource::Callable(node),
        }
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_callable_type(
        &mut self,
        node: CallableType,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let parts = node.parts(ast);
        let parameters_span = parts.parameters.span(ast);
        let mut parameters = self.callable_parameters(parts.parameters, parts.this_parameter);
        if self.has_suppression
            && !parameters.has_parameters()
            && self.content_mode(self.comments.in_range(parameters_span)) == ContentMode::Verbatim
        {
            self.format_verbatim(node.span(ast), writer)?;
            return Ok(None);
        }
        let head_end = parts
            .type_parameters
            .map_or(parameters_span.start, |node| node.span(ast).start);
        let mut generic_mode = ContentMode::Formatted;
        let mut incoming = None;
        let (head, prefix_end) = match node {
            CallableType::Function(_) => (CallableHead::Function, node.span(ast).start),
            CallableType::Constructor(constructor) => {
                let start = constructor.span(ast).start;
                let gap = Span::new(start, head_end);
                let comments = self.comments.in_range(gap);
                let Some(keyword) = self.find_token(gap, comments, "new") else {
                    return Err(self.unsupported("constructor type keyword", constructor));
                };
                let abstract_gap = if constructor.r#abstract(ast) {
                    let gap = Span::new(start + 8, keyword);
                    let source = self.has_suppression
                        && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim;
                    if source {
                        if parts.type_parameters.is_some() {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            self.select_callable_incoming(
                                &parameters,
                                parts.return_type,
                                &mut incoming,
                            )?;
                        }
                    }
                    Some(self.prepare_type_gap(gap, source))
                } else {
                    None
                };
                (CallableHead::Constructor { abstract_gap }, keyword + 3)
            }
        };
        let head_gap = Span::new(prefix_end, head_end);
        let head_source = self.has_suppression
            && self.content_mode(self.comments.in_range(head_gap)) == ContentMode::Verbatim;
        if head_source {
            if parts.type_parameters.is_some() {
                generic_mode = ContentMode::Verbatim;
            } else {
                self.select_callable_incoming(&parameters, parts.return_type, &mut incoming)?;
            }
        }
        let prefix = self.prepare_type_gap(head_gap, head_source);
        let generic_tail = parts
            .type_parameters
            .map(|generic| {
                let gap = Span::new(generic.span(ast).end, parameters_span.start);
                let comments = self.comments.in_range(gap);
                let source =
                    self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim;
                if source {
                    for (_, position) in self.suppressed_comment_runs(comments, gap.end) {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            generic_mode = ContentMode::Verbatim;
                        } else {
                            self.select_callable_incoming(
                                &parameters,
                                parts.return_type,
                                &mut incoming,
                            )?;
                        }
                    }
                }
                Ok((generic, self.prepare_type_gap(gap, source)))
            })
            .transpose()?;
        let source_return = match incoming {
            Some(CallableIncomingOwner::Parameter(parameter)) => {
                parameters.source_head = Some(parameter);
                None
            }
            Some(CallableIncomingOwner::Return(annotation)) => Some(annotation),
            None => None,
        };
        let eol_target = if parameters.has_parameters() {
            SignatureReturnEolTarget::Parameter
        } else {
            SignatureReturnEolTarget::Annotation
        };
        let return_type = self.prepare_signature_return(
            parameters_span.end,
            eol_target,
            parts.return_type,
            source_return,
            SignatureReturnDelimiter::Arrow,
        );
        self.prepare_callable_parameter_tail(&mut parameters, return_type.preceding_mode)?;
        writer.group(false, |writer| {
            if let CallableHead::Constructor { abstract_gap } = head {
                if let Some(gap) = abstract_gap {
                    writer.text("abstract");
                    self.format_type_gap(gap, FlatSeparator::Space, writer)?;
                }
                writer.text("new");
                self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
            } else {
                self.format_type_gap(prefix, FlatSeparator::Empty, writer)?;
            }
            if let Some((generic, gap)) = generic_tail {
                self.format_type_parameters(
                    generic,
                    TypeParameterContext::Callable,
                    generic_mode,
                    writer,
                )?;
                self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
            }
            self.format_callable_parameters(parameters, writer)?;
            self.format_signature_return(return_type, writer)
        })
    }

    pub(in crate::formatter) fn signature_incoming(
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

    pub(in crate::formatter) fn callable_parameters(
        &self,
        node: FormalParameters,
        this_parameter: Option<TsThisParameter>,
    ) -> CallableParameters {
        let has_parameters = this_parameter.is_some()
            || !node.items(self.ast).is_empty()
            || node.rest(self.ast).is_some();
        CallableParameters {
            node,
            this_parameter,
            has_parameters,
            source_head: None,
            suppressed_tail: None,
            terminal: None,
        }
    }

    pub(in crate::formatter) fn first_callable_parameter(
        &self,
        parameters: &CallableParameters,
    ) -> Result<Option<CallableParameter>, FormatError> {
        let ast = self.ast;
        let first = if let Some(this) = parameters.this_parameter {
            Some(CallableParameter::This(this))
        } else if let Some(id) = parameters.node.items(ast).iter().next() {
            let item = ast.get_node_in_sub_range(id);
            match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(parameter) => {
                    Some(CallableParameter::Formal(parameter))
                }
                FormalParameterItemData::TsParameterProperty(_) => {
                    return Err(self.unsupported("callable type parameter property", item));
                }
            }
        } else {
            parameters.node.rest(ast).map(CallableParameter::Rest)
        };
        Ok(first)
    }

    pub(in crate::formatter) fn prepare_callable_parameter_tail(
        &self,
        parameters: &mut CallableParameters,
        preceding_mode: ContentMode,
    ) -> Result<(), FormatError> {
        if !self.has_suppression {
            return Ok(());
        }
        let ast = self.ast;
        let parameters_span = parameters.node.span(ast);
        let content = Span::new(parameters_span.start + 1, parameters_span.end - 1);
        let candidate = self.content_mode(self.comments.in_range(content)) == ContentMode::Verbatim;
        if candidate || preceding_mode == ContentMode::Verbatim {
            let last = self.last_callable_parameter(parameters)?;
            if preceding_mode == ContentMode::Verbatim {
                parameters.suppressed_tail = last;
            }
            if let Some(last) = last {
                let end = last.span(ast).end;
                let comments = self.comments.in_range(Span::new(end, content.end));
                if self.content_mode(comments) == ContentMode::Verbatim {
                    parameters.suppressed_tail = Some(last);
                    let last_end = comments.last().map_or(end, |comment| comment.span.end);
                    parameters.terminal = Some(CallableTerminal {
                        parameter: last,
                        closing: self.source_line_constraint(last_end, content.end),
                    });
                }
            }
        }
        Ok(())
    }

    fn select_callable_incoming(
        &self,
        parameters: &CallableParameters,
        annotation: TsTypeAnnotation,
        owner: &mut Option<CallableIncomingOwner>,
    ) -> Result<(), FormatError> {
        if owner.is_some() {
            return Ok(());
        }
        let first = self.first_callable_parameter(parameters)?;
        *owner = Some(first.map_or(
            CallableIncomingOwner::Return(annotation),
            CallableIncomingOwner::Parameter,
        ));
        Ok(())
    }

    pub(in crate::formatter) fn prepare_signature_return(
        &self,
        preceding_end: u32,
        mut eol_target: SignatureReturnEolTarget<'_>,
        annotation: TsTypeAnnotation,
        source_return: Option<TsTypeAnnotation>,
        delimiter: SignatureReturnDelimiter,
    ) -> SignatureReturn<'ast> {
        let ast = self.ast;
        let delimiter_start = annotation.span(ast).start;
        let before = self
            .comments
            .in_range(Span::new(preceding_end, delimiter_start));
        let mut preceding_mode = ContentMode::Formatted;
        let mut source_before = false;
        let mut wrapper_mode =
            source_return.map_or(ContentMode::Formatted, |_| ContentMode::Verbatim);
        if self.has_suppression && self.content_mode(before) == ContentMode::Verbatim {
            for (_, position) in self.suppressed_comment_runs(before, delimiter_start) {
                if matches!(position, SuppressionPosition::EndOfLine) {
                    match &mut eol_target {
                        SignatureReturnEolTarget::Parameter => {
                            preceding_mode = ContentMode::Verbatim;
                            source_before = true;
                        }
                        SignatureReturnEolTarget::Header(mode) => {
                            **mode = ContentMode::Verbatim;
                            source_before = true;
                        }
                        SignatureReturnEolTarget::Annotation => {
                            wrapper_mode = ContentMode::Verbatim;
                        }
                    }
                } else {
                    wrapper_mode = ContentMode::Verbatim;
                }
            }
        }
        if wrapper_mode == ContentMode::Verbatim {
            return SignatureReturn {
                content: AnnotationContent::Wrapper(source_return.unwrap_or(annotation)),
                prefix: TypeBoundaryLayout::Source(Span::new(preceding_end, delimiter_start)),
                preceding_mode,
                delimiter,
            };
        }
        let mut node = self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted);
        let type_start = node.node.span(ast).start;
        let after = self.comments.in_range(Span::new(
            delimiter_start + delimiter.token().len() as u32,
            type_start,
        ));
        self.select_type_head_comments(&mut node, after, type_start);
        let prefix = if source_before || node.source_owner.is_some() {
            TypeBoundaryLayout::Source(Span::new(preceding_end, type_start))
        } else {
            TypeBoundaryLayout::Formatted {
                before: self.prepare_comment_boundary(before, preceding_end, delimiter_start),
                after: self.prepare_comment_boundary(
                    after,
                    delimiter_start + delimiter.token().len() as u32,
                    type_start,
                ),
            }
        };
        SignatureReturn {
            content: AnnotationContent::Type(node),
            prefix,
            preceding_mode,
            delimiter,
        }
    }

    pub(in crate::formatter) fn format_signature_return(
        &mut self,
        result: SignatureReturn<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        self.format_signature_return_with_context(result, TypeContext::Root, writer)
    }

    pub(in crate::formatter) fn format_signature_return_with_context(
        &mut self,
        result: SignatureReturn<'ast>,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match result.prefix {
            TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
            TypeBoundaryLayout::Formatted { before, after } => {
                self.format_comment_boundary(&before, writer)?;
                before
                    .gap()
                    .write(writer, result.delimiter.preceding_space(), |writer| {
                        if matches!(result.delimiter, SignatureReturnDelimiter::Arrow) {
                            writer.space();
                        }
                    });
                writer.text(result.delimiter.token());
                self.format_comment_boundary(&after, writer)?;
                after
                    .gap()
                    .write(writer, FlatSeparator::Space, Writer::space);
            }
        }
        self.format_annotation_owner(result.content, context, writer)
    }

    fn format_annotation_owner(
        &mut self,
        content: AnnotationContent,
        context: TypeContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match content {
            AnnotationContent::Wrapper(annotation) => {
                self.format_verbatim(annotation.span(self.ast), writer)?;
                Ok(None)
            }
            AnnotationContent::Type(node) => self.format_prepared_type(node, context, writer),
        }
    }

    fn last_callable_parameter(
        &self,
        parameters: &CallableParameters,
    ) -> Result<Option<CallableParameter>, FormatError> {
        let ast = self.ast;
        if let Some(rest) = parameters.node.rest(ast) {
            return Ok(Some(CallableParameter::Rest(rest)));
        }
        if let Some(id) = parameters.node.items(ast).iter().next_back() {
            let item = ast.get_node_in_sub_range(id);
            return match ast.formal_parameter_item_data(item) {
                FormalParameterItemData::FormalParameter(node) => {
                    Ok(Some(CallableParameter::Formal(node)))
                }
                FormalParameterItemData::TsParameterProperty(_) => {
                    Err(self.unsupported("callable type parameter property", item))
                }
            };
        }
        Ok(parameters.this_parameter.map(CallableParameter::This))
    }

    pub(in crate::formatter) fn format_callable_parameters(
        &mut self,
        parameters: CallableParameters,
        writer: &mut Writer<'ast>,
    ) -> Result<(), FormatError> {
        let ast = self.ast;
        let span = parameters.node.span(ast);
        let content = Span::new(span.start + 1, span.end - 1);
        let has_parameters = parameters.has_parameters();
        let members = parameters
            .this_parameter
            .into_iter()
            .map(|node| Ok(CallableParameter::This(node)))
            .chain(parameters.node.items(ast).iter().map(|id| {
                let item = ast.get_node_in_sub_range(id);
                match ast.formal_parameter_item_data(item) {
                    FormalParameterItemData::FormalParameter(node) => {
                        Ok(CallableParameter::Formal(node))
                    }
                    FormalParameterItemData::TsParameterProperty(_) => {
                        Err(FormatError::unsupported(
                            "callable type parameter property",
                            None,
                            item.span(ast),
                        ))
                    }
                }
            }))
            .chain(
                parameters
                    .node
                    .rest(ast)
                    .into_iter()
                    .map(|node| Ok(CallableParameter::Rest(node))),
            );
        writer.group(false, |writer| {
            writer.text("(");
            let (end, closing) = writer.indent(|writer| {
                if has_parameters || !self.comments.in_range(content).is_empty() {
                    self.opening_line_constraint(content).write(
                        writer,
                        FlatSeparator::Empty,
                        Writer::soft_line,
                    );
                }
                let mut members = members.peekable();
                let mut end = content.start;
                let mut following_mode = None;
                let mut separator = Separator::Line;
                let mut has_previous = false;
                while let Some(member) = members.next() {
                    let member = member?;
                    let member_span = member.span(ast);
                    if has_previous {
                        let next = self
                            .comments
                            .between(Span::new(end, member_span.start))?
                            .first()
                            .map_or(member_span.start, |comment| comment.span.start);
                        ArgumentLayout::Lines.separator(
                            separator,
                            self.line_constraint(end, next),
                            writer,
                        );
                    }
                    let following = match members.peek() {
                        Some(Ok(node)) => FollowingItem::Node(node.span(ast)),
                        Some(Err(_)) | None => FollowingItem::End,
                    };
                    let following = parameters
                        .terminal
                        .as_ref()
                        .filter(|terminal| terminal.parameter == member)
                        .map_or(following, |_| {
                            FollowingItem::ResolvedTerminal(DelimitedSuppression {
                                preceding: Some(member_span),
                                following: ContentMode::Formatted,
                            })
                        });
                    let mut item = self.prepare_delimited_item(
                        member_span,
                        following,
                        Span::new(end, content.end),
                        !matches!(member, CallableParameter::Rest(_))
                            && self.options.trailing_commas == TrailingCommas::All,
                        writer,
                    )?;
                    if let Some(mode) = following_mode {
                        item.set_leading_mode(mode);
                    }
                    if parameters.source_head == Some(member)
                        || parameters.suppressed_tail == Some(member)
                    {
                        item.set_leading_mode(ContentMode::Verbatim);
                    }
                    let prepared = self.prepare_callable_parameter(member, item.mode)?;
                    let tail = self.format_prepared_parameter_item(prepared, writer)?;
                    let boundary = self.finish_delimited_item(item, tail, writer)?;
                    end = boundary.end;
                    following_mode = Some(boundary.following_mode);
                    separator = boundary.separator;
                    has_previous = true;
                }
                let dangling = self.comments.between(Span::new(end, content.end))?;
                let boundary = self.prepare_comment_boundary(dangling, end, content.end);
                self.comments.claim(dangling)?;
                self.format_comment_run(dangling, CommentRunLayout::Lines, writer)?;
                Ok::<_, FormatError>((
                    dangling.last().map_or(end, |comment| comment.span.end),
                    boundary.gap(),
                ))
            })?;
            if has_parameters || end != content.start {
                if let Some(terminal) = &parameters.terminal {
                    terminal
                        .closing
                        .write(writer, FlatSeparator::Empty, Writer::soft_line);
                } else {
                    closing.write(writer, FlatSeparator::Empty, Writer::soft_line);
                }
            }
            writer.text(")");
            Ok(())
        })
    }

    fn prepare_callable_parameter(
        &self,
        node: CallableParameter,
        mode: ContentMode,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        if mode == ContentMode::Verbatim {
            return Ok(PreparedParameterItem(ParameterItemState::Source(
                ParameterItemSource::Callable(node),
            )));
        }
        let ast = self.ast;
        let (annotation, optional) = match node {
            CallableParameter::This(this) => (this.type_annotation(ast), false),
            CallableParameter::Formal(parameter) => {
                (parameter.type_annotation(ast), parameter.optional(ast))
            }
            CallableParameter::Rest(rest) => (rest.type_annotation(ast), false),
        };
        self.prepare_parameter_item(
            node,
            annotation,
            optional,
            ParameterConsumer::CallableSignature,
        )
    }

    pub(in crate::formatter) fn prepare_runtime_this_parameter(
        &self,
        receiver: TsThisParameter,
        mode: ContentMode,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let node = CallableParameter::This(receiver);
        if mode == ContentMode::Verbatim {
            return Ok(PreparedParameterItem(ParameterItemState::Source(
                ParameterItemSource::Callable(node),
            )));
        }
        let annotation = receiver.type_annotation(self.ast);
        self.prepare_parameter_item(node, annotation, false, ParameterConsumer::Runtime)
    }

    pub(in crate::formatter) fn prepare_typed_formal_parameter(
        &self,
        parameter: FormalParameter,
        mode: ContentMode,
        annotation: Option<TsTypeAnnotation>,
        optional: bool,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let node = CallableParameter::Formal(parameter);
        if mode == ContentMode::Verbatim {
            return Ok(PreparedParameterItem(ParameterItemState::Source(
                ParameterItemSource::Callable(node),
            )));
        }
        self.prepare_parameter_item(node, annotation, optional, ParameterConsumer::Runtime)
    }

    pub(in crate::formatter) fn prepare_typed_rest_parameter(
        &self,
        rest: FormalParameterRest,
        mode: ContentMode,
        annotation: TsTypeAnnotation,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let node = CallableParameter::Rest(rest);
        if mode == ContentMode::Verbatim {
            return Ok(PreparedParameterItem(ParameterItemState::Source(
                ParameterItemSource::Callable(node),
            )));
        }
        self.prepare_parameter_item(node, Some(annotation), false, ParameterConsumer::Runtime)
    }

    pub(in crate::formatter) fn prepare_decorated_rest_parameter(
        &self,
        rest: FormalParameterRest,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        self.prepare_parameter_item(
            CallableParameter::Rest(rest),
            rest.type_annotation(self.ast),
            false,
            ParameterConsumer::Runtime,
        )
    }

    pub(in crate::formatter) fn prepare_constructor_parameter_item(
        &self,
        property: swc_next_ecma_ast::TsParameterProperty,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let parameter = property.parameter(self.ast);
        self.prepare_parameter_item(
            CallableParameter::Formal(parameter),
            parameter.type_annotation(self.ast),
            parameter.optional(self.ast),
            ParameterConsumer::ConstructorProperty(property),
        )
    }

    fn prepare_parameter_item(
        &self,
        node: CallableParameter,
        annotation: Option<TsTypeAnnotation>,
        optional: bool,
        consumer: ParameterConsumer,
    ) -> Result<PreparedParameterItem<'ast>, FormatError> {
        let ast = self.ast;
        let (binding, annotation, end, initializer) = match node {
            CallableParameter::This(this) => (
                ParameterBinding::This(this),
                annotation,
                this.span(ast).start + 4,
                None,
            ),
            CallableParameter::Formal(parameter) => {
                let pattern = self.parameter_binding(parameter.pattern(ast));
                let initializer = if let Some(right) = parameter.initializer(ast) {
                    let left_end = if let Some(annotation) = annotation {
                        annotation.span(ast).end
                    } else if optional {
                        let gap = Span::new(pattern.span(ast).end, right.span(ast).start);
                        self.find_token(gap, self.comments.in_range(gap), "?")
                            .ok_or_else(|| {
                                self.unsupported("optional callable default", parameter)
                            })?
                            + 1
                    } else {
                        pattern.span(ast).end
                    };
                    let left = Span::new(pattern.span(ast).start, left_end);
                    let owner = self.prepare_default_boundary(left, right)?;
                    let (left_mode, layout) = match owner {
                        DefaultOwner::Whole => {
                            return Ok(PreparedParameterItem(ParameterItemState::Source(
                                consumer.source(node),
                            )));
                        }
                        DefaultOwner::Bound {
                            gap,
                            left_mode,
                            right_mode,
                        } => (left_mode, InitializerLayout::Bound { gap, right_mode }),
                        DefaultOwner::Formatted {
                            boundary,
                            break_after,
                        } => (
                            ContentMode::Formatted,
                            InitializerLayout::Formatted {
                                boundary,
                                break_after,
                            },
                        ),
                    };
                    Some(PreparedParameterDefault {
                        left,
                        right,
                        left_mode,
                        layout,
                    })
                } else {
                    None
                };
                if !parameter.decorators(ast).is_empty() {
                    match consumer {
                        ParameterConsumer::CallableSignature => {
                            return Err(self.unsupported("callable parameter decorator", parameter));
                        }
                        ParameterConsumer::ConstructorProperty(_) => {
                            return Err(self
                                .unsupported("nested constructor parameter decorator", parameter));
                        }
                        ParameterConsumer::Runtime => {}
                    }
                }
                (
                    ParameterBinding::Pattern {
                        node: pattern,
                        mode: ContentMode::Formatted,
                        spread: None,
                    },
                    annotation,
                    pattern.span(ast).end,
                    initializer,
                )
            }
            CallableParameter::Rest(rest) => {
                if !rest.decorators(ast).is_empty()
                    && matches!(consumer, ParameterConsumer::CallableSignature)
                {
                    return Err(self.unsupported("callable type rest decorator", rest));
                }
                let pattern = self.parameter_binding(rest.argument(ast));
                let gap = Span::new(rest.span(ast).start + 3, pattern.span(ast).start);
                let mode = if self.has_suppression {
                    self.content_mode(self.comments.in_range(gap))
                } else {
                    ContentMode::Formatted
                };
                let spread = self.prepare_type_gap(gap, mode == ContentMode::Verbatim);
                (
                    ParameterBinding::Pattern {
                        node: pattern,
                        mode,
                        spread: Some(spread),
                    },
                    annotation,
                    pattern.span(ast).end,
                    None,
                )
            }
        };
        if initializer.is_none()
            && annotation.is_none()
            && self.has_suppression
            && self.content_mode(self.comments.in_range(Span::new(end, node.span(ast).end)))
                == ContentMode::Verbatim
        {
            return Ok(PreparedParameterItem(ParameterItemState::Source(
                consumer.source(node),
            )));
        }
        let annotation = if initializer
            .as_ref()
            .is_some_and(|initializer| initializer.left_mode == ContentMode::Verbatim)
        {
            None
        } else {
            annotation
                .map(|annotation| self.prepare_annotation_content(end, annotation))
                .transpose()?
        };
        let left = ParameterLeft {
            binding,
            optional,
            annotation,
            end: initializer
                .as_ref()
                .map_or(node.span(ast).end, |initializer| initializer.left.end),
        };
        let parameter = match initializer {
            Some(initializer) => ParameterItemState::Default {
                left: if initializer.left_mode == ContentMode::Verbatim {
                    PreparedParameterLeft::Source(initializer.left)
                } else {
                    PreparedParameterLeft::Formatted(left)
                },
                right: initializer.right,
                layout: initializer.layout,
            },
            None => ParameterItemState::Formatted(left),
        };
        Ok(PreparedParameterItem(parameter))
    }

    pub(in crate::formatter) fn format_prepared_parameter_item(
        &mut self,
        parameter: PreparedParameterItem<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        match parameter.0 {
            ParameterItemState::Source(node) => {
                self.format_verbatim(node.span(self.ast), writer)?;
                Ok(None)
            }
            ParameterItemState::Formatted(left) => {
                self.format_parameter_left(PreparedParameterLeft::Formatted(left), writer)
            }
            ParameterItemState::Default {
                left,
                right,
                layout,
            } => {
                let end = left.end();
                let tail = self.format_parameter_left(left, writer)?;
                self.format_expression_tail(tail, writer)?;
                self.format_initializer_right(
                    end,
                    right,
                    layout,
                    ExpressionContext::binding_initializer(),
                    writer,
                )
            }
        }
    }

    fn format_parameter_left(
        &mut self,
        left: PreparedParameterLeft<'ast>,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let left = match left {
            PreparedParameterLeft::Source(span) => {
                self.format_verbatim(span, writer)?;
                return Ok(None);
            }
            PreparedParameterLeft::Formatted(left) => left,
        };
        let end = match left.binding {
            ParameterBinding::This(this) => {
                writer.text("this");
                this.span(self.ast).start + 4
            }
            ParameterBinding::Pattern { node, mode, spread } => {
                if let Some(gap) = spread {
                    writer.text("...");
                    self.format_type_gap(gap, FlatSeparator::Empty, writer)?;
                }
                let tail = self.format_declaration_binding(node, mode, writer)?;
                self.format_expression_tail(tail, writer)?;
                node.span(self.ast).end
            }
        };
        let optional = left.optional;
        if let Some(annotation) = left.annotation {
            if optional && matches!(&annotation.prefix, TypeBoundaryLayout::Formatted { .. }) {
                writer.text("?");
            }
            self.format_annotation_content(annotation, writer)
        } else {
            if optional {
                writer.text("?");
            }
            self.format_source_token_gap(end, left.end, false, writer)?;
            Ok(None)
        }
    }

    pub(super) fn format_type_predicate(
        &mut self,
        node: TsTypePredicate,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let name = node.parameter_name(ast);
        let name_span = name.span(ast);
        let mut name_mode = ContentMode::Formatted;
        let asserts_prefix = node.asserts(ast).then(|| {
            let gap = Span::new(node.span(ast).start + 7, name_span.start);
            if self.has_suppression
                && self.content_mode(self.comments.in_range(gap)) == ContentMode::Verbatim
            {
                name_mode = ContentMode::Verbatim;
            }
            self.prepare_type_gap(gap, name_mode == ContentMode::Verbatim)
        });
        let bound = node
            .type_annotation(ast)
            .map(|annotation| {
                let gap = Span::new(name_span.end, annotation.span(ast).start);
                let comments = self.comments.in_range(gap);
                let Some(keyword) = self.find_token(gap, comments, "is") else {
                    return Err(self.unsupported("type predicate keyword", node));
                };
                let split = comments.partition_point(|comment| comment.span.end <= keyword);
                let (before, after) = comments.split_at(split);
                let mut mode = ContentMode::Formatted;
                if self.has_suppression && self.content_mode(comments) == ContentMode::Verbatim {
                    for (_, position) in self.suppressed_comment_runs(before, keyword) {
                        if matches!(position, SuppressionPosition::OwnLine) {
                            mode = ContentMode::Verbatim;
                        } else {
                            name_mode = ContentMode::Verbatim;
                        }
                    }
                    for (_, position) in
                        self.suppressed_comment_runs(after, annotation.span(ast).start)
                    {
                        if matches!(position, SuppressionPosition::EndOfLine) {
                            name_mode = ContentMode::Verbatim;
                        } else {
                            mode = ContentMode::Verbatim;
                        }
                    }
                }
                let prefix = if name_mode == ContentMode::Verbatim || mode == ContentMode::Verbatim
                {
                    TypeBoundaryLayout::Source(gap)
                } else {
                    TypeBoundaryLayout::Formatted {
                        before: self.prepare_comment_boundary(before, gap.start, keyword),
                        after: self.prepare_comment_boundary(after, keyword + 2, gap.end),
                    }
                };
                let content = if mode == ContentMode::Verbatim {
                    AnnotationContent::Wrapper(annotation)
                } else {
                    AnnotationContent::Type(
                        self.prepare_type(annotation.type_annotation(ast), ContentMode::Formatted),
                    )
                };
                Ok(PredicateBound { content, prefix })
            })
            .transpose()?;
        if let Some(prefix) = asserts_prefix {
            writer.text("asserts");
            self.format_type_gap(prefix, FlatSeparator::Space, writer)?;
        }
        if name_mode == ContentMode::Verbatim {
            self.format_verbatim(name_span, writer)?;
        } else {
            match ast.ts_type_predicate_name_data(name) {
                TsTypePredicateNameData::IdentifierName(name) => {
                    writer.text(ast.get_utf8(name.name(ast)))
                }
                TsTypePredicateNameData::TsThisType(_) => writer.text("this"),
            }
        }
        if let Some(bound) = bound {
            match bound.prefix {
                TypeBoundaryLayout::Source(gap) => self.format_verbatim(gap, writer)?,
                TypeBoundaryLayout::Formatted { before, after } => {
                    self.format_comment_boundary(&before, writer)?;
                    before
                        .gap()
                        .write(writer, FlatSeparator::Space, Writer::space);
                    writer.text("is");
                    self.format_comment_boundary(&after, writer)?;
                    after
                        .gap()
                        .write(writer, FlatSeparator::Space, Writer::space);
                }
            }
            return self.format_annotation_owner(bound.content, TypeContext::Root, writer);
        }
        Ok(None)
    }
}
