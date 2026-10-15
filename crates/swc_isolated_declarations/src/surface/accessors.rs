//! Pair object and class accessors and select their declaration type sources.

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    Accessibility, Ast, ClassElement, ClassElementData, ExprData, FormalParameterItemData,
    FormalParameters, Function, GetSpan, MethodDefinitionKind, NodeId, ObjectExpression,
    ObjectPropertyKindData, PropertyKind, Span, TsFunction, TsTypeAnnotation,
};
use swc_next_ecma_semantic::Semantic;

use super::keys::{PropertyIdentity, property_identity, supported_object_key};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct AccessorKey<'ast> {
    pub(crate) name: PropertyIdentity<'ast>,
    pub(crate) r#static: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum AccessorFunction<'id> {
    Implementation(Function<'id>),
    Signature(TsFunction<'id>),
}

impl<'id> AccessorFunction<'id> {
    pub(crate) fn node_id(self) -> NodeId<'id> {
        match self {
            Self::Implementation(function) => function.node_id(),
            Self::Signature(function) => function.node_id(),
        }
    }
    pub(crate) fn span(self, ast: &Ast<'_, 'id>) -> Span {
        match self {
            Self::Implementation(function) => function.span(ast),
            Self::Signature(function) => function.span(ast),
        }
    }
    fn return_type(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAnnotation<'id>> {
        match self {
            Self::Implementation(function) => function.return_type(ast),
            Self::Signature(function) => function.return_type(ast),
        }
    }
    fn params(self, ast: &Ast<'_, 'id>) -> FormalParameters<'id> {
        match self {
            Self::Implementation(function) => function.params(ast),
            Self::Signature(function) => function.params(ast),
        }
    }

    fn first_parameter_annotation(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let item = ast.get_node_in_sub_range(self.params(ast).items(ast).first()?);
        let parameter = match ast.formal_parameter_item_data(item) {
            FormalParameterItemData::FormalParameter(parameter) => parameter,
            FormalParameterItemData::TsParameterProperty(property) => property.parameter(ast),
        };
        parameter.type_annotation(ast)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AccessorPair<'id> {
    pub(crate) getter: Option<AccessorFunction<'id>>,
    pub(crate) setter: Option<AccessorFunction<'id>>,
    pub(crate) span: Span,
}

#[derive(Clone, Copy)]
pub(crate) enum AccessorType<'id> {
    Annotation(TsTypeAnnotation<'id>),
    Inferred(Function<'id>),
}

impl<'id> AccessorPair<'id> {
    fn new(span: Span) -> Self {
        Self {
            getter: None,
            setter: None,
            span,
        }
    }

    pub(crate) fn getter_annotation(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAnnotation<'id>> {
        self.getter.and_then(|function| function.return_type(ast))
    }

    pub(crate) fn setter_annotation(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAnnotation<'id>> {
        self.setter?.first_parameter_annotation(ast)
    }

    pub(crate) fn read_type(self, ast: &Ast<'_, 'id>) -> Option<AccessorType<'id>> {
        self.getter_annotation(ast)
            .or_else(|| self.setter_annotation(ast))
            .map(AccessorType::Annotation)
            .or_else(|| match self.getter? {
                AccessorFunction::Implementation(function) => {
                    Some(AccessorType::Inferred(function))
                }
                AccessorFunction::Signature(_) => None,
            })
    }

    pub(crate) fn write_type(self, ast: &Ast<'_, 'id>) -> Option<AccessorType<'id>> {
        self.setter_annotation(ast)
            .map(AccessorType::Annotation)
            .or_else(|| self.read_type(ast))
    }
}

pub(crate) fn object_accessors<'id, 'ast>(
    object: ObjectExpression<'id>,
    ast: &Ast<'ast, 'id>,
    semantic: &Semantic<'_, 'id>,
) -> FxHashMap<AccessorKey<'ast>, AccessorPair<'id>> {
    let mut pairs = FxHashMap::<_, AccessorPair<'id>>::default();
    for property in object.properties(ast).iter() {
        let ObjectPropertyKindData::ObjectProperty(property) =
            ast.object_property_kind_data(ast.get_node_in_sub_range(property))
        else {
            continue;
        };
        if !matches!(property.kind(ast), PropertyKind::Get | PropertyKind::Set)
            || (property.computed(ast) && !supported_object_key(property.key(ast), ast))
        {
            continue;
        }
        let ExprData::Function(function) = ast.expr_data(property.value(ast)) else {
            continue;
        };
        let Some(name) = property_identity(property.key(ast), ast, semantic) else {
            continue;
        };
        let pair = pairs
            .entry(AccessorKey {
                name,
                r#static: false,
            })
            .or_insert_with(|| AccessorPair::new(function.span(ast)));
        match property.kind(ast) {
            PropertyKind::Get => pair.getter = Some(AccessorFunction::Implementation(function)),
            PropertyKind::Set => pair.setter = Some(AccessorFunction::Implementation(function)),
            PropertyKind::Init => {}
        }
    }
    pairs
}

pub(crate) fn class_accessors<'id, 'ast>(
    elements: impl Iterator<Item = ClassElement<'id>>,
    ast: &Ast<'ast, 'id>,
    semantic: &Semantic<'_, 'id>,
) -> FxHashMap<AccessorKey<'ast>, AccessorPair<'id>> {
    let mut pairs = FxHashMap::<_, AccessorPair<'id>>::default();
    for element in elements {
        let (function, kind, key, accessibility, is_static) = match ast.class_element_data(element)
        {
            ClassElementData::MethodDefinition(method) => (
                AccessorFunction::Implementation(method.value(ast)),
                method.kind(ast),
                method.key(ast),
                method.accessibility(ast),
                method.r#static(ast),
            ),
            ClassElementData::TsMethodDefinition(method) => (
                AccessorFunction::Signature(method.value(ast)),
                method.kind(ast),
                method.key(ast),
                method.accessibility(ast),
                method.r#static(ast),
            ),
            _ => continue,
        };
        if !matches!(kind, MethodDefinitionKind::Get | MethodDefinitionKind::Set) {
            continue;
        }
        if accessibility == Accessibility::Private {
            // A private setter's annotation still supplies its public getter's
            // type. Its erased signature and body do not contribute dependencies.
            if kind != MethodDefinitionKind::Set
                || function.first_parameter_annotation(ast).is_none()
            {
                continue;
            }
        }
        let Some(name) = property_identity(key, ast, semantic) else {
            continue;
        };
        let pair = pairs
            .entry(AccessorKey {
                name,
                r#static: is_static,
            })
            .or_insert_with(|| AccessorPair::new(function.span(ast)));
        match kind {
            MethodDefinitionKind::Get => pair.getter = Some(function),
            MethodDefinitionKind::Set => pair.setter = Some(function),
            _ => {}
        }
    }
    pairs
}
