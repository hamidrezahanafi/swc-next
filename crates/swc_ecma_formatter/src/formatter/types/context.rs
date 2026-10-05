//! Type grammar roles decide the parentheses around actual borrowed children.

use swc_next_ecma_ast::TsTypeData;

#[derive(Clone, Copy)]
pub(super) enum TypeContext {
    Root,
    PostfixOperand,
    OperatorOperand,
    CompoundMember,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypePrecedence {
    Union,
    Intersection,
    Operator,
    Query,
    Postfix,
    Atomic,
}

impl TypeContext {
    pub(super) fn needs_parentheses(self, data: TsTypeData) -> bool {
        let minimum = match self {
            Self::Root => return false,
            Self::PostfixOperand => TypePrecedence::Postfix,
            Self::OperatorOperand => TypePrecedence::Query,
            Self::CompoundMember => TypePrecedence::Operator,
        };
        let precedence = match data {
            TsTypeData::TsUnionType(_) => TypePrecedence::Union,
            TsTypeData::TsIntersectionType(_) => TypePrecedence::Intersection,
            TsTypeData::TsTypeOperator(_) => TypePrecedence::Operator,
            TsTypeData::TsTypeQuery(_) => TypePrecedence::Query,
            TsTypeData::TsArrayType(_) | TsTypeData::TsIndexedAccessType(_) => {
                TypePrecedence::Postfix
            }
            TsTypeData::TsConditionalType(_)
            | TsTypeData::TsFunctionType(_)
            | TsTypeData::TsConstructorType(_) => TypePrecedence::Union,
            _ => TypePrecedence::Atomic,
        };
        precedence < minimum
    }
}
