//! Type grammar roles decide the parentheses around actual borrowed children.

use swc_next_ecma_ast::{Ast, TsType, TsTypeData};

#[derive(Clone, Copy)]
pub(in crate::formatter) enum TypeContext {
    Root,
    ArrowReturn,
    PostfixOperand,
    ModifierOperand,
    CompoundMember,
    ConditionalCheck,
    ConditionalExtends,
    TypeConstraint,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypePrecedence {
    Conditional,
    Union,
    Intersection,
    Operator,
    Query,
    Postfix,
    Atomic,
}

impl TypeContext {
    pub(super) fn needs_parentheses(self, data: TsTypeData, ast: &Ast<'_>) -> bool {
        let minimum = match self {
            Self::Root => return false,
            Self::ArrowReturn => return matches!(data, TsTypeData::TsFunctionType(_)),
            Self::ConditionalCheck => {
                return matches!(
                    data,
                    TsTypeData::TsConditionalType(_)
                        | TsTypeData::TsFunctionType(_)
                        | TsTypeData::TsConstructorType(_)
                );
            }
            Self::ConditionalExtends => {
                return match data {
                    TsTypeData::TsConditionalType(_) => true,
                    TsTypeData::TsFunctionType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                    }
                    TsTypeData::TsConstructorType(node) => {
                        return_is_constrained_infer(node.return_type(ast).type_annotation(ast), ast)
                    }
                    _ => false,
                };
            }
            Self::TypeConstraint => return matches!(data, TsTypeData::TsConditionalType(_)),
            Self::PostfixOperand => TypePrecedence::Postfix,
            Self::ModifierOperand => TypePrecedence::Query,
            Self::CompoundMember => {
                if let TsTypeData::TsInferType(node) = data {
                    return node.type_parameter(ast).constraint(ast).is_some();
                }
                TypePrecedence::Operator
            }
        };
        let precedence = match data {
            TsTypeData::TsConditionalType(_) => TypePrecedence::Conditional,
            TsTypeData::TsUnionType(_)
            | TsTypeData::TsFunctionType(_)
            | TsTypeData::TsConstructorType(_) => TypePrecedence::Union,
            TsTypeData::TsIntersectionType(_) => TypePrecedence::Intersection,
            TsTypeData::TsTypeOperator(_) | TsTypeData::TsInferType(_) => TypePrecedence::Operator,
            TsTypeData::TsTypeQuery(_) => TypePrecedence::Query,
            TsTypeData::TsArrayType(_) | TsTypeData::TsIndexedAccessType(_) => {
                TypePrecedence::Postfix
            }
            _ => TypePrecedence::Atomic,
        };
        precedence < minimum
    }
}

fn return_is_constrained_infer(mut node: TsType, ast: &Ast<'_>) -> bool {
    loop {
        match ast.ts_type_data(node) {
            TsTypeData::TsParenthesizedType(parentheses) => {
                node = parentheses.type_annotation(ast);
            }
            TsTypeData::TsTypePredicate(predicate) => {
                let Some(annotation) = predicate.type_annotation(ast) else {
                    return false;
                };
                node = annotation.type_annotation(ast);
            }
            TsTypeData::TsInferType(infer) => {
                return infer.type_parameter(ast).constraint(ast).is_some();
            }
            _ => return false,
        }
    }
}
