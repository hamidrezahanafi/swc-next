use crate::{
    Accessibility, AssignmentOperator, Ast, AstUtf8, AstWtf8, BinaryOperator, ClassType,
    FormalParameterKind, FunctionType, ImportOrExportKind, ImportPhase, LogicalOperator,
    MethodDefinitionKind, PropertyKind, SourceType, Span, SubRange, TsMappedTypeModifier,
    TsMethodSignatureKind, TsModuleDeclarationKind, TsTypeOperatorKind, UnaryOperator,
    UpdateOperator, VariableKind, node_id::ExtraDataValue,
};

pub trait CloneIn<'a, 'id>: Sized {
    type Cloned;

    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned;
}

macro_rules! impl_clone_in_trivial {
    ($i:ident) => {
        impl<'id, 'a> CloneIn<'a, 'id> for $i {
            type Cloned = $i;

            fn clone_in(&self, _ast: &mut Ast<'a, 'id>) -> Self::Cloned {
                self.clone()
            }
        }
    };
}

impl_clone_in_trivial!(Span);
impl_clone_in_trivial!(bool);
impl_clone_in_trivial!(f64);
impl_clone_in_trivial!(UnaryOperator);
impl_clone_in_trivial!(UpdateOperator);
impl_clone_in_trivial!(BinaryOperator);
impl_clone_in_trivial!(LogicalOperator);
impl_clone_in_trivial!(AssignmentOperator);
impl_clone_in_trivial!(ImportPhase);
impl_clone_in_trivial!(SourceType);
impl_clone_in_trivial!(MethodDefinitionKind);
impl_clone_in_trivial!(Accessibility);
impl_clone_in_trivial!(ClassType);
impl_clone_in_trivial!(VariableKind);
impl_clone_in_trivial!(FunctionType);
impl_clone_in_trivial!(FormalParameterKind);
impl_clone_in_trivial!(PropertyKind);
impl_clone_in_trivial!(ImportOrExportKind);
impl_clone_in_trivial!(TsTypeOperatorKind);
impl_clone_in_trivial!(TsMethodSignatureKind);
impl_clone_in_trivial!(TsModuleDeclarationKind);
impl_clone_in_trivial!(TsMappedTypeModifier);
impl<'a, 'id, C, T: CloneIn<'a, 'id, Cloned = C>> CloneIn<'a, 'id> for Vec<T> {
    type Cloned = Vec<C>;

    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let mut cloned = Vec::with_capacity(self.len());
        for item in self {
            cloned.push(item.clone_in(ast));
        }
        cloned
    }
}

impl<'a, 'id, C, T: CloneIn<'a, 'id, Cloned = C>> CloneIn<'a, 'id> for Option<T> {
    type Cloned = Option<C>;

    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        self.as_ref().map(|it| it.clone_in(ast))
    }
}

impl<'id, 'a, C, T> CloneIn<'a, 'id> for SubRange<'id, T>
where
    C: ExtraDataValue<'id>,
    T: CloneIn<'a, 'id, Cloned = C> + ExtraDataValue<'id>,
{
    type Cloned = SubRange<'id, C>;

    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let mut ids = Vec::with_capacity(self.len());
        for id in self.iter() {
            let node = ast.get_node_in_sub_range(id);
            ids.push(node.clone_in(ast));
        }
        ast.add_sub_range(ids)
    }
}

impl<'id, 'a> CloneIn<'a, 'id> for AstUtf8<'id> {
    type Cloned = Self;

    fn clone_in(&self, _ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        *self
    }
}

impl<'id, 'a> CloneIn<'a, 'id> for AstWtf8<'id> {
    type Cloned = Self;

    fn clone_in(&self, _ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        *self
    }
}
