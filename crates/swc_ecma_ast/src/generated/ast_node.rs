#![allow(unused)]
use crate::node_id::sealed::AstNode as SealedAstNode;
use crate::{Ast, GetSpan, SetSpan};
use crate::{ast::*, node_id::*};
/// The concrete type stored at a node ID.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Program,
    Super,
    ThisExpression,
    StringLiteral,
    NumericLiteral,
    BigIntLiteral,
    BooleanLiteral,
    NullLiteral,
    RegExpLiteral,
    TemplateLiteral,
    TemplateElement,
    IdentifierReference,
    PrivateIdentifier,
    BindingIdentifier,
    IdentifierName,
    LabelIdentifier,
    AssignmentPattern,
    BindingRestElement,
    ArrayPattern,
    ObjectPattern,
    BindingProperty,
    BinaryExpression,
    LogicalExpression,
    ConditionalExpression,
    UnaryExpression,
    UpdateExpression,
    AssignmentExpression,
    VariableDeclaration,
    VariableDeclarator,
    IfStatement,
    SwitchStatement,
    SwitchCase,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    TryStatement,
    CatchClause,
    BreakStatement,
    ContinueStatement,
    LabeledStatement,
    ExpressionStatement,
    ReturnStatement,
    ThrowStatement,
    DebuggerStatement,
    EmptyStatement,
    Directive,
    Decorator,
    Class,
    ClassBody,
    MethodDefinition,
    PropertyDefinition,
    StaticBlock,
    Function,
    FunctionBody,
    BlockStatement,
    WhileStatement,
    DoWhileStatement,
    WithStatement,
    FormalParameters,
    FormalParameter,
    FormalParameterRest,
    ArrayExpression,
    ObjectExpression,
    SpreadElement,
    ObjectProperty,
    ParenthesizedExpression,
    ArrowFunctionExpression,
    SequenceExpression,
    MemberExpression,
    CallExpression,
    ChainExpression,
    TaggedTemplateExpression,
    NewExpression,
    AwaitExpression,
    YieldExpression,
    MetaProperty,
    ImportExpression,
    ImportDeclaration,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ImportAttribute,
    ExportNamedDeclaration,
    ExportDefaultDeclaration,
    ExportAllDeclaration,
    ExportSpecifier,
    Hashbang,
    TsFunction,
    TsMethodDefinition,
    TsTypeAnnotation,
    TsAnyKeyword,
    TsUnknownKeyword,
    TsNeverKeyword,
    TsVoidKeyword,
    TsNullKeyword,
    TsUndefinedKeyword,
    TsStringKeyword,
    TsNumberKeyword,
    TsBigIntKeyword,
    TsBooleanKeyword,
    TsSymbolKeyword,
    TsObjectKeyword,
    TsIntrinsicKeyword,
    TsThisType,
    TsTypeReference,
    TsQualifiedName,
    TsTypeQuery,
    TsImportType,
    TsTypeParameter,
    TsTypeParameterDeclaration,
    TsTypeParameterInstantiation,
    TsLiteralType,
    TsTemplateLiteralType,
    TsArrayType,
    TsIndexedAccessType,
    TsTupleType,
    TsNamedTupleMember,
    TsOptionalType,
    TsRestType,
    TsJsDocNullableType,
    TsJsDocNonNullableType,
    TsJsDocUnknownType,
    TsUnionType,
    TsIntersectionType,
    TsConditionalType,
    TsInferType,
    TsTypeOperator,
    TsParenthesizedType,
    TsFunctionType,
    TsConstructorType,
    TsTypePredicate,
    TsTypeLiteral,
    TsMappedType,
    TsPropertySignature,
    TsMethodSignature,
    TsCallSignatureDeclaration,
    TsConstructSignatureDeclaration,
    TsIndexSignature,
    TsTypeAliasDeclaration,
    TsInterfaceDeclaration,
    TsInterfaceBody,
    TsInterfaceHeritage,
    TsClassImplements,
    TsEnumDeclaration,
    TsEnumBody,
    TsEnumMember,
    TsModuleDeclaration,
    TsModuleBlock,
    TsGlobalDeclaration,
    TsParameterProperty,
    TsThisParameter,
    TsAsExpression,
    TsSatisfiesExpression,
    TsTypeAssertion,
    TsNonNullExpression,
    TsInstantiationExpression,
    TsExportAssignment,
    TsNamespaceExportDeclaration,
    TsImportEqualsDeclaration,
    TsExternalModuleReference,
    TsIndexSignatureParameter,
    JsxElement,
    JsxOpeningElement,
    JsxClosingElement,
    JsxFragment,
    JsxOpeningFragment,
    JsxClosingFragment,
    JsxIdentifier,
    JsxNamespacedName,
    JsxMemberExpression,
    JsxAttribute,
    JsxSpreadAttribute,
    JsxExpressionContainer,
    JsxEmptyExpression,
    JsxText,
    JsxSpreadChild,
}
impl NodeKind {
    pub const COUNT: u8 = 176u8;
    #[inline]
    pub const fn is_stmt(self) -> bool {
        matches!(
            self,
            Self::BlockStatement
                | Self::BreakStatement
                | Self::ContinueStatement
                | Self::DebuggerStatement
                | Self::DoWhileStatement
                | Self::EmptyStatement
                | Self::ExpressionStatement
                | Self::ForInStatement
                | Self::ForOfStatement
                | Self::ForStatement
                | Self::IfStatement
                | Self::LabeledStatement
                | Self::ReturnStatement
                | Self::SwitchStatement
                | Self::ThrowStatement
                | Self::TryStatement
                | Self::WhileStatement
                | Self::WithStatement
                | Self::VariableDeclaration
                | Self::Function
                | Self::Class
                | Self::TsFunction
                | Self::TsTypeAliasDeclaration
                | Self::TsInterfaceDeclaration
                | Self::TsEnumDeclaration
                | Self::TsModuleDeclaration
                | Self::TsGlobalDeclaration
                | Self::TsImportEqualsDeclaration
                | Self::ImportDeclaration
                | Self::ExportNamedDeclaration
                | Self::ExportDefaultDeclaration
                | Self::ExportAllDeclaration
                | Self::TsExportAssignment
                | Self::TsNamespaceExportDeclaration
        )
    }
    #[inline]
    pub const fn is_decl(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::Function
                | Self::Class
                | Self::TsFunction
                | Self::TsTypeAliasDeclaration
                | Self::TsInterfaceDeclaration
                | Self::TsEnumDeclaration
                | Self::TsModuleDeclaration
                | Self::TsGlobalDeclaration
                | Self::TsImportEqualsDeclaration
        )
    }
    #[inline]
    pub const fn is_expr(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_binding_pattern(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier
                | Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::AssignmentPattern
                | Self::BindingRestElement
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_assignment_target(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_simple_assignment_target(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
        )
    }
    #[inline]
    pub const fn is_property_key(self) -> bool {
        matches!(
            self,
            Self::IdentifierName
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_module_export_name(self) -> bool {
        matches!(self, Self::IdentifierName | Self::StringLiteral)
    }
    #[inline]
    pub const fn is_argument(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::SpreadElement
        )
    }
    #[inline]
    pub const fn is_object_property_kind(self) -> bool {
        matches!(self, Self::ObjectProperty | Self::SpreadElement)
    }
    #[inline]
    pub const fn is_for_statement_init(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_for_statement_left(self) -> bool {
        matches!(
            self,
            Self::VariableDeclaration
                | Self::IdentifierReference
                | Self::MemberExpression
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::ArrayPattern
                | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_arrow_function_body(self) -> bool {
        matches!(
            self,
            Self::FunctionBody
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_formal_parameter_item(self) -> bool {
        matches!(self, Self::FormalParameter | Self::TsParameterProperty)
    }
    #[inline]
    pub const fn is_formal_parameter_pattern(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier | Self::ArrayPattern | Self::ObjectPattern
        )
    }
    #[inline]
    pub const fn is_class_element(self) -> bool {
        matches!(
            self,
            Self::MethodDefinition
                | Self::PropertyDefinition
                | Self::StaticBlock
                | Self::TsMethodDefinition
                | Self::TsIndexSignature
        )
    }
    #[inline]
    pub const fn is_import_declaration_specifier(self) -> bool {
        matches!(
            self,
            Self::ImportSpecifier | Self::ImportDefaultSpecifier | Self::ImportNamespaceSpecifier
        )
    }
    #[inline]
    pub const fn is_export_default_declaration_kind(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::TsFunction
                | Self::TsInterfaceDeclaration
        )
    }
    #[inline]
    pub const fn is_ts_type(self) -> bool {
        matches!(
            self,
            Self::TsAnyKeyword
                | Self::TsUnknownKeyword
                | Self::TsNeverKeyword
                | Self::TsVoidKeyword
                | Self::TsNullKeyword
                | Self::TsUndefinedKeyword
                | Self::TsStringKeyword
                | Self::TsNumberKeyword
                | Self::TsBigIntKeyword
                | Self::TsBooleanKeyword
                | Self::TsSymbolKeyword
                | Self::TsObjectKeyword
                | Self::TsIntrinsicKeyword
                | Self::TsThisType
                | Self::TsTypeReference
                | Self::TsTypeQuery
                | Self::TsImportType
                | Self::TsLiteralType
                | Self::TsTemplateLiteralType
                | Self::TsArrayType
                | Self::TsIndexedAccessType
                | Self::TsTupleType
                | Self::TsOptionalType
                | Self::TsRestType
                | Self::TsJsDocNullableType
                | Self::TsJsDocNonNullableType
                | Self::TsJsDocUnknownType
                | Self::TsUnionType
                | Self::TsIntersectionType
                | Self::TsConditionalType
                | Self::TsInferType
                | Self::TsTypeOperator
                | Self::TsParenthesizedType
                | Self::TsFunctionType
                | Self::TsConstructorType
                | Self::TsTypePredicate
                | Self::TsTypeLiteral
                | Self::TsMappedType
        )
    }
    #[inline]
    pub const fn is_ts_type_name(self) -> bool {
        matches!(self, Self::IdentifierReference | Self::TsQualifiedName)
    }
    #[inline]
    pub const fn is_ts_qualified_name_left(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::IdentifierName
                | Self::BindingIdentifier
                | Self::ThisExpression
                | Self::TsQualifiedName
        )
    }
    #[inline]
    pub const fn is_ts_import_type_qualifier(self) -> bool {
        matches!(self, Self::IdentifierName | Self::TsQualifiedName)
    }
    #[inline]
    pub const fn is_ts_type_query_expr_name(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::TsQualifiedName
                | Self::ThisExpression
                | Self::TsImportType
        )
    }
    #[inline]
    pub const fn is_ts_literal(self) -> bool {
        matches!(
            self,
            Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::TemplateLiteral
                | Self::UnaryExpression
        )
    }
    #[inline]
    pub const fn is_ts_tuple_element(self) -> bool {
        matches!(
            self,
            Self::TsAnyKeyword
                | Self::TsUnknownKeyword
                | Self::TsNeverKeyword
                | Self::TsVoidKeyword
                | Self::TsNullKeyword
                | Self::TsUndefinedKeyword
                | Self::TsStringKeyword
                | Self::TsNumberKeyword
                | Self::TsBigIntKeyword
                | Self::TsBooleanKeyword
                | Self::TsSymbolKeyword
                | Self::TsObjectKeyword
                | Self::TsIntrinsicKeyword
                | Self::TsThisType
                | Self::TsTypeReference
                | Self::TsTypeQuery
                | Self::TsImportType
                | Self::TsLiteralType
                | Self::TsTemplateLiteralType
                | Self::TsArrayType
                | Self::TsIndexedAccessType
                | Self::TsTupleType
                | Self::TsOptionalType
                | Self::TsRestType
                | Self::TsJsDocNullableType
                | Self::TsJsDocNonNullableType
                | Self::TsJsDocUnknownType
                | Self::TsUnionType
                | Self::TsIntersectionType
                | Self::TsConditionalType
                | Self::TsInferType
                | Self::TsTypeOperator
                | Self::TsParenthesizedType
                | Self::TsFunctionType
                | Self::TsConstructorType
                | Self::TsTypePredicate
                | Self::TsTypeLiteral
                | Self::TsMappedType
                | Self::TsNamedTupleMember
        )
    }
    #[inline]
    pub const fn is_ts_type_element(self) -> bool {
        matches!(
            self,
            Self::TsPropertySignature
                | Self::TsMethodSignature
                | Self::TsCallSignatureDeclaration
                | Self::TsConstructSignatureDeclaration
                | Self::TsIndexSignature
        )
    }
    #[inline]
    pub const fn is_ts_type_predicate_name(self) -> bool {
        matches!(self, Self::IdentifierName | Self::TsThisType)
    }
    #[inline]
    pub const fn is_ts_enum_member_name(self) -> bool {
        matches!(
            self,
            Self::IdentifierName | Self::StringLiteral | Self::TemplateLiteral
        )
    }
    #[inline]
    pub const fn is_ts_module_name(self) -> bool {
        matches!(
            self,
            Self::BindingIdentifier | Self::StringLiteral | Self::TsQualifiedName
        )
    }
    #[inline]
    pub const fn is_ts_module_body(self) -> bool {
        matches!(self, Self::TsModuleBlock | Self::TsModuleDeclaration)
    }
    #[inline]
    pub const fn is_ts_module_reference(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference | Self::TsQualifiedName | Self::TsExternalModuleReference
        )
    }
    #[inline]
    pub const fn is_jsx_element_name(self) -> bool {
        matches!(
            self,
            Self::JsxIdentifier | Self::JsxNamespacedName | Self::JsxMemberExpression
        )
    }
    #[inline]
    pub const fn is_jsx_member_expression_object(self) -> bool {
        matches!(self, Self::JsxIdentifier | Self::JsxMemberExpression)
    }
    #[inline]
    pub const fn is_jsx_attribute_name(self) -> bool {
        matches!(self, Self::JsxIdentifier | Self::JsxNamespacedName)
    }
    #[inline]
    pub const fn is_jsx_attribute_item(self) -> bool {
        matches!(self, Self::JsxAttribute | Self::JsxSpreadAttribute)
    }
    #[inline]
    pub const fn is_jsx_attribute_value(self) -> bool {
        matches!(
            self,
            Self::StringLiteral
                | Self::JsxExpressionContainer
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
    #[inline]
    pub const fn is_jsx_expression(self) -> bool {
        matches!(
            self,
            Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
                | Self::TemplateLiteral
                | Self::BinaryExpression
                | Self::LogicalExpression
                | Self::UnaryExpression
                | Self::UpdateExpression
                | Self::AssignmentExpression
                | Self::ConditionalExpression
                | Self::SequenceExpression
                | Self::MemberExpression
                | Self::CallExpression
                | Self::ChainExpression
                | Self::NewExpression
                | Self::TaggedTemplateExpression
                | Self::ArrowFunctionExpression
                | Self::ArrayExpression
                | Self::ObjectExpression
                | Self::ParenthesizedExpression
                | Self::Function
                | Self::Class
                | Self::ImportExpression
                | Self::AwaitExpression
                | Self::YieldExpression
                | Self::PrivateIdentifier
                | Self::TsAsExpression
                | Self::TsSatisfiesExpression
                | Self::TsTypeAssertion
                | Self::TsNonNullExpression
                | Self::TsInstantiationExpression
                | Self::JsxElement
                | Self::JsxFragment
                | Self::JsxEmptyExpression
        )
    }
    #[inline]
    pub const fn is_jsx_child(self) -> bool {
        matches!(
            self,
            Self::JsxText
                | Self::JsxExpressionContainer
                | Self::JsxSpreadChild
                | Self::JsxElement
                | Self::JsxFragment
        )
    }
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StmtData<'id> {
    BlockStatement(BlockStatement<'id>) = 1u32,
    BreakStatement(BreakStatement<'id>) = 2u32,
    ContinueStatement(ContinueStatement<'id>) = 3u32,
    DebuggerStatement(DebuggerStatement<'id>) = 4u32,
    DoWhileStatement(DoWhileStatement<'id>) = 5u32,
    EmptyStatement(EmptyStatement<'id>) = 6u32,
    ExpressionStatement(ExpressionStatement<'id>) = 7u32,
    ForInStatement(ForInStatement<'id>) = 8u32,
    ForOfStatement(ForOfStatement<'id>) = 9u32,
    ForStatement(ForStatement<'id>) = 10u32,
    IfStatement(IfStatement<'id>) = 11u32,
    LabeledStatement(LabeledStatement<'id>) = 12u32,
    ReturnStatement(ReturnStatement<'id>) = 13u32,
    SwitchStatement(SwitchStatement<'id>) = 14u32,
    ThrowStatement(ThrowStatement<'id>) = 15u32,
    TryStatement(TryStatement<'id>) = 16u32,
    WhileStatement(WhileStatement<'id>) = 17u32,
    WithStatement(WithStatement<'id>) = 18u32,
    Declaration(Decl<'id>) = 19u32,
    ImportDeclaration(ImportDeclaration<'id>) = 20u32,
    ExportNamedDeclaration(ExportNamedDeclaration<'id>) = 21u32,
    ExportDefaultDeclaration(ExportDefaultDeclaration<'id>) = 22u32,
    ExportAllDeclaration(ExportAllDeclaration<'id>) = 23u32,
    TsExportAssignment(TsExportAssignment<'id>) = 24u32,
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration<'id>) = 25u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DeclData<'id> {
    VariableDeclaration(VariableDeclaration<'id>) = NodeKind::VariableDeclaration as u8,
    Function(Function<'id>) = NodeKind::Function as u8,
    Class(Class<'id>) = NodeKind::Class as u8,
    TsFunction(TsFunction<'id>) = NodeKind::TsFunction as u8,
    TsTypeAliasDeclaration(TsTypeAliasDeclaration<'id>) = NodeKind::TsTypeAliasDeclaration as u8,
    TsInterfaceDeclaration(TsInterfaceDeclaration<'id>) = NodeKind::TsInterfaceDeclaration as u8,
    TsEnumDeclaration(TsEnumDeclaration<'id>) = NodeKind::TsEnumDeclaration as u8,
    TsModuleDeclaration(TsModuleDeclaration<'id>) = NodeKind::TsModuleDeclaration as u8,
    TsGlobalDeclaration(TsGlobalDeclaration<'id>) = NodeKind::TsGlobalDeclaration as u8,
    TsImportEqualsDeclaration(TsImportEqualsDeclaration<'id>) =
        NodeKind::TsImportEqualsDeclaration as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprData<'id> {
    IdentifierReference(IdentifierReference<'id>) = NodeKind::IdentifierReference as u8,
    ThisExpression(ThisExpression<'id>) = NodeKind::ThisExpression as u8,
    Super(Super<'id>) = NodeKind::Super as u8,
    MetaProperty(MetaProperty<'id>) = NodeKind::MetaProperty as u8,
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral<'id>) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral<'id>) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral<'id>) = NodeKind::BooleanLiteral as u8,
    NullLiteral(NullLiteral<'id>) = NodeKind::NullLiteral as u8,
    RegExpLiteral(RegExpLiteral<'id>) = NodeKind::RegExpLiteral as u8,
    TemplateLiteral(TemplateLiteral<'id>) = NodeKind::TemplateLiteral as u8,
    BinaryExpression(BinaryExpression<'id>) = NodeKind::BinaryExpression as u8,
    LogicalExpression(LogicalExpression<'id>) = NodeKind::LogicalExpression as u8,
    UnaryExpression(UnaryExpression<'id>) = NodeKind::UnaryExpression as u8,
    UpdateExpression(UpdateExpression<'id>) = NodeKind::UpdateExpression as u8,
    AssignmentExpression(AssignmentExpression<'id>) = NodeKind::AssignmentExpression as u8,
    ConditionalExpression(ConditionalExpression<'id>) = NodeKind::ConditionalExpression as u8,
    SequenceExpression(SequenceExpression<'id>) = NodeKind::SequenceExpression as u8,
    MemberExpression(MemberExpression<'id>) = NodeKind::MemberExpression as u8,
    CallExpression(CallExpression<'id>) = NodeKind::CallExpression as u8,
    ChainExpression(ChainExpression<'id>) = NodeKind::ChainExpression as u8,
    NewExpression(NewExpression<'id>) = NodeKind::NewExpression as u8,
    TaggedTemplateExpression(TaggedTemplateExpression<'id>) =
        NodeKind::TaggedTemplateExpression as u8,
    ArrowFunctionExpression(ArrowFunctionExpression<'id>) = NodeKind::ArrowFunctionExpression as u8,
    ArrayExpression(ArrayExpression<'id>) = NodeKind::ArrayExpression as u8,
    ObjectExpression(ObjectExpression<'id>) = NodeKind::ObjectExpression as u8,
    ParenthesizedExpression(ParenthesizedExpression<'id>) = NodeKind::ParenthesizedExpression as u8,
    Function(Function<'id>) = NodeKind::Function as u8,
    Class(Class<'id>) = NodeKind::Class as u8,
    ImportExpression(ImportExpression<'id>) = NodeKind::ImportExpression as u8,
    AwaitExpression(AwaitExpression<'id>) = NodeKind::AwaitExpression as u8,
    YieldExpression(YieldExpression<'id>) = NodeKind::YieldExpression as u8,
    PrivateIdentifier(PrivateIdentifier<'id>) = NodeKind::PrivateIdentifier as u8,
    TsAsExpression(TsAsExpression<'id>) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression<'id>) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion<'id>) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression<'id>) = NodeKind::TsNonNullExpression as u8,
    TsInstantiationExpression(TsInstantiationExpression<'id>) =
        NodeKind::TsInstantiationExpression as u8,
    JsxElement(JsxElement<'id>) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment<'id>) = NodeKind::JsxFragment as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BindingPatternData<'id> {
    BindingIdentifier(BindingIdentifier<'id>) = 1u32,
    SimpleAssignmentTarget(SimpleAssignmentTarget<'id>) = 2u32,
    AssignmentPattern(AssignmentPattern<'id>) = 3u32,
    BindingRestElement(BindingRestElement<'id>) = 4u32,
    ArrayPattern(ArrayPattern<'id>) = 5u32,
    ObjectPattern(ObjectPattern<'id>) = 6u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AssignmentTargetData<'id> {
    SimpleAssignmentTarget(SimpleAssignmentTarget<'id>) = 1u32,
    ArrayAssignmentTarget(ArrayPattern<'id>) = 2u32,
    ObjectAssignmentTarget(ObjectPattern<'id>) = 3u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SimpleAssignmentTargetData<'id> {
    IdentifierReference(IdentifierReference<'id>) = NodeKind::IdentifierReference as u8,
    MemberExpression(MemberExpression<'id>) = NodeKind::MemberExpression as u8,
    TsAsExpression(TsAsExpression<'id>) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression<'id>) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion<'id>) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression<'id>) = NodeKind::TsNonNullExpression as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKeyData<'id> {
    IdentifierName(IdentifierName<'id>) = 1u32,
    Expr(Expr<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModuleExportNameData<'id> {
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArgumentData<'id> {
    Expr(Expr<'id>) = 1u32,
    SpreadElement(SpreadElement<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ObjectPropertyKindData<'id> {
    ObjectProperty(ObjectProperty<'id>) = NodeKind::ObjectProperty as u8,
    SpreadElement(SpreadElement<'id>) = NodeKind::SpreadElement as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ForStatementInitData<'id> {
    VariableDeclaration(VariableDeclaration<'id>) = 1u32,
    Expr(Expr<'id>) = 2u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ForStatementLeftData<'id> {
    VariableDeclaration(VariableDeclaration<'id>) = 1u32,
    AssignmentTarget(AssignmentTarget<'id>) = 2u32,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArrowFunctionBodyData<'id> {
    FunctionBody(FunctionBody<'id>) = 1u32,
    Expr(Expr<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FormalParameterItemData<'id> {
    FormalParameter(FormalParameter<'id>) = NodeKind::FormalParameter as u8,
    TsParameterProperty(TsParameterProperty<'id>) = NodeKind::TsParameterProperty as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FormalParameterPatternData<'id> {
    BindingIdentifier(BindingIdentifier<'id>) = NodeKind::BindingIdentifier as u8,
    ArrayPattern(ArrayPattern<'id>) = NodeKind::ArrayPattern as u8,
    ObjectPattern(ObjectPattern<'id>) = NodeKind::ObjectPattern as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ClassElementData<'id> {
    MethodDefinition(MethodDefinition<'id>) = NodeKind::MethodDefinition as u8,
    PropertyDefinition(PropertyDefinition<'id>) = NodeKind::PropertyDefinition as u8,
    StaticBlock(StaticBlock<'id>) = NodeKind::StaticBlock as u8,
    TsMethodDefinition(TsMethodDefinition<'id>) = NodeKind::TsMethodDefinition as u8,
    TsIndexSignature(TsIndexSignature<'id>) = NodeKind::TsIndexSignature as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ImportDeclarationSpecifierData<'id> {
    ImportSpecifier(ImportSpecifier<'id>) = NodeKind::ImportSpecifier as u8,
    ImportDefaultSpecifier(ImportDefaultSpecifier<'id>) = NodeKind::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(ImportNamespaceSpecifier<'id>) =
        NodeKind::ImportNamespaceSpecifier as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExportDefaultDeclarationKindData<'id> {
    Expr(Expr<'id>) = 1u32,
    Function(Function<'id>) = 2u32,
    Class(Class<'id>) = 3u32,
    TsFunction(TsFunction<'id>) = 4u32,
    TsInterfaceDeclaration(TsInterfaceDeclaration<'id>) = 5u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeData<'id> {
    TsAnyKeyword(TsAnyKeyword<'id>) = NodeKind::TsAnyKeyword as u8,
    TsUnknownKeyword(TsUnknownKeyword<'id>) = NodeKind::TsUnknownKeyword as u8,
    TsNeverKeyword(TsNeverKeyword<'id>) = NodeKind::TsNeverKeyword as u8,
    TsVoidKeyword(TsVoidKeyword<'id>) = NodeKind::TsVoidKeyword as u8,
    TsNullKeyword(TsNullKeyword<'id>) = NodeKind::TsNullKeyword as u8,
    TsUndefinedKeyword(TsUndefinedKeyword<'id>) = NodeKind::TsUndefinedKeyword as u8,
    TsStringKeyword(TsStringKeyword<'id>) = NodeKind::TsStringKeyword as u8,
    TsNumberKeyword(TsNumberKeyword<'id>) = NodeKind::TsNumberKeyword as u8,
    TsBigIntKeyword(TsBigIntKeyword<'id>) = NodeKind::TsBigIntKeyword as u8,
    TsBooleanKeyword(TsBooleanKeyword<'id>) = NodeKind::TsBooleanKeyword as u8,
    TsSymbolKeyword(TsSymbolKeyword<'id>) = NodeKind::TsSymbolKeyword as u8,
    TsObjectKeyword(TsObjectKeyword<'id>) = NodeKind::TsObjectKeyword as u8,
    TsIntrinsicKeyword(TsIntrinsicKeyword<'id>) = NodeKind::TsIntrinsicKeyword as u8,
    TsThisType(TsThisType<'id>) = NodeKind::TsThisType as u8,
    TsTypeReference(TsTypeReference<'id>) = NodeKind::TsTypeReference as u8,
    TsTypeQuery(TsTypeQuery<'id>) = NodeKind::TsTypeQuery as u8,
    TsImportType(TsImportType<'id>) = NodeKind::TsImportType as u8,
    TsLiteralType(TsLiteralType<'id>) = NodeKind::TsLiteralType as u8,
    TsTemplateLiteralType(TsTemplateLiteralType<'id>) = NodeKind::TsTemplateLiteralType as u8,
    TsArrayType(TsArrayType<'id>) = NodeKind::TsArrayType as u8,
    TsIndexedAccessType(TsIndexedAccessType<'id>) = NodeKind::TsIndexedAccessType as u8,
    TsTupleType(TsTupleType<'id>) = NodeKind::TsTupleType as u8,
    TsOptionalType(TsOptionalType<'id>) = NodeKind::TsOptionalType as u8,
    TsRestType(TsRestType<'id>) = NodeKind::TsRestType as u8,
    TsJsDocNullableType(TsJsDocNullableType<'id>) = NodeKind::TsJsDocNullableType as u8,
    TsJsDocNonNullableType(TsJsDocNonNullableType<'id>) = NodeKind::TsJsDocNonNullableType as u8,
    TsJsDocUnknownType(TsJsDocUnknownType<'id>) = NodeKind::TsJsDocUnknownType as u8,
    TsUnionType(TsUnionType<'id>) = NodeKind::TsUnionType as u8,
    TsIntersectionType(TsIntersectionType<'id>) = NodeKind::TsIntersectionType as u8,
    TsConditionalType(TsConditionalType<'id>) = NodeKind::TsConditionalType as u8,
    TsInferType(TsInferType<'id>) = NodeKind::TsInferType as u8,
    TsTypeOperator(TsTypeOperator<'id>) = NodeKind::TsTypeOperator as u8,
    TsParenthesizedType(TsParenthesizedType<'id>) = NodeKind::TsParenthesizedType as u8,
    TsFunctionType(TsFunctionType<'id>) = NodeKind::TsFunctionType as u8,
    TsConstructorType(TsConstructorType<'id>) = NodeKind::TsConstructorType as u8,
    TsTypePredicate(TsTypePredicate<'id>) = NodeKind::TsTypePredicate as u8,
    TsTypeLiteral(TsTypeLiteral<'id>) = NodeKind::TsTypeLiteral as u8,
    TsMappedType(TsMappedType<'id>) = NodeKind::TsMappedType as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeNameData<'id> {
    IdentifierReference(IdentifierReference<'id>) = NodeKind::IdentifierReference as u8,
    TsQualifiedName(TsQualifiedName<'id>) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsQualifiedNameLeftData<'id> {
    IdentifierReference(IdentifierReference<'id>) = NodeKind::IdentifierReference as u8,
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    BindingIdentifier(BindingIdentifier<'id>) = NodeKind::BindingIdentifier as u8,
    ThisExpression(ThisExpression<'id>) = NodeKind::ThisExpression as u8,
    TsQualifiedName(TsQualifiedName<'id>) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsImportTypeQualifierData<'id> {
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    TsQualifiedName(TsQualifiedName<'id>) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeQueryExprNameData<'id> {
    TsTypeName(TsTypeName<'id>) = 1u32,
    ThisExpression(ThisExpression<'id>) = 2u32,
    TsImportType(TsImportType<'id>) = 3u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsLiteralData<'id> {
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral<'id>) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral<'id>) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral<'id>) = NodeKind::BooleanLiteral as u8,
    TemplateLiteral(TemplateLiteral<'id>) = NodeKind::TemplateLiteral as u8,
    UnaryExpression(UnaryExpression<'id>) = NodeKind::UnaryExpression as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTupleElementData<'id> {
    TsType(TsType<'id>) = 1u32,
    TsNamedTupleMember(TsNamedTupleMember<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeElementData<'id> {
    TsPropertySignature(TsPropertySignature<'id>) = NodeKind::TsPropertySignature as u8,
    TsMethodSignature(TsMethodSignature<'id>) = NodeKind::TsMethodSignature as u8,
    TsCallSignatureDeclaration(TsCallSignatureDeclaration<'id>) =
        NodeKind::TsCallSignatureDeclaration as u8,
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration<'id>) =
        NodeKind::TsConstructSignatureDeclaration as u8,
    TsIndexSignature(TsIndexSignature<'id>) = NodeKind::TsIndexSignature as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsTypePredicateNameData<'id> {
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    TsThisType(TsThisType<'id>) = NodeKind::TsThisType as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsEnumMemberNameData<'id> {
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    TemplateLiteral(TemplateLiteral<'id>) = NodeKind::TemplateLiteral as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleNameData<'id> {
    BindingIdentifier(BindingIdentifier<'id>) = NodeKind::BindingIdentifier as u8,
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    TsQualifiedName(TsQualifiedName<'id>) = NodeKind::TsQualifiedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleBodyData<'id> {
    TsModuleBlock(TsModuleBlock<'id>) = NodeKind::TsModuleBlock as u8,
    TsModuleDeclaration(TsModuleDeclaration<'id>) = NodeKind::TsModuleDeclaration as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TsModuleReferenceData<'id> {
    TsTypeName(TsTypeName<'id>) = 1u32,
    TsExternalModuleReference(TsExternalModuleReference<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxElementNameData<'id> {
    JsxIdentifier(JsxIdentifier<'id>) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName<'id>) = NodeKind::JsxNamespacedName as u8,
    JsxMemberExpression(JsxMemberExpression<'id>) = NodeKind::JsxMemberExpression as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxMemberExpressionObjectData<'id> {
    JsxIdentifier(JsxIdentifier<'id>) = NodeKind::JsxIdentifier as u8,
    JsxMemberExpression(JsxMemberExpression<'id>) = NodeKind::JsxMemberExpression as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeNameData<'id> {
    JsxIdentifier(JsxIdentifier<'id>) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName<'id>) = NodeKind::JsxNamespacedName as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeItemData<'id> {
    JsxAttribute(JsxAttribute<'id>) = NodeKind::JsxAttribute as u8,
    JsxSpreadAttribute(JsxSpreadAttribute<'id>) = NodeKind::JsxSpreadAttribute as u8,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxAttributeValueData<'id> {
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    JsxExpressionContainer(JsxExpressionContainer<'id>) = NodeKind::JsxExpressionContainer as u8,
    JsxElement(JsxElement<'id>) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment<'id>) = NodeKind::JsxFragment as u8,
}
#[repr(C, u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxExpressionData<'id> {
    Expr(Expr<'id>) = 1u32,
    JsxEmptyExpression(JsxEmptyExpression<'id>) = 2u32,
}
#[repr(C, u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsxChildData<'id> {
    JsxText(JsxText<'id>) = NodeKind::JsxText as u8,
    JsxExpressionContainer(JsxExpressionContainer<'id>) = NodeKind::JsxExpressionContainer as u8,
    JsxSpreadChild(JsxSpreadChild<'id>) = NodeKind::JsxSpreadChild as u8,
    JsxElement(JsxElement<'id>) = NodeKind::JsxElement as u8,
    JsxFragment(JsxFragment<'id>) = NodeKind::JsxFragment as u8,
}
#[repr(C)]
struct AstEnumDataRepr<'id> {
    kind: NodeKind,
    node_id: NodeId<'id>,
}
/// The typed wrapper for the concrete node stored at a node ID.
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKindData<'id> {
    Program(Program<'id>) = NodeKind::Program as u8,
    Super(Super<'id>) = NodeKind::Super as u8,
    ThisExpression(ThisExpression<'id>) = NodeKind::ThisExpression as u8,
    StringLiteral(StringLiteral<'id>) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral<'id>) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral<'id>) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral<'id>) = NodeKind::BooleanLiteral as u8,
    NullLiteral(NullLiteral<'id>) = NodeKind::NullLiteral as u8,
    RegExpLiteral(RegExpLiteral<'id>) = NodeKind::RegExpLiteral as u8,
    TemplateLiteral(TemplateLiteral<'id>) = NodeKind::TemplateLiteral as u8,
    TemplateElement(TemplateElement<'id>) = NodeKind::TemplateElement as u8,
    IdentifierReference(IdentifierReference<'id>) = NodeKind::IdentifierReference as u8,
    PrivateIdentifier(PrivateIdentifier<'id>) = NodeKind::PrivateIdentifier as u8,
    BindingIdentifier(BindingIdentifier<'id>) = NodeKind::BindingIdentifier as u8,
    IdentifierName(IdentifierName<'id>) = NodeKind::IdentifierName as u8,
    LabelIdentifier(LabelIdentifier<'id>) = NodeKind::LabelIdentifier as u8,
    AssignmentPattern(AssignmentPattern<'id>) = NodeKind::AssignmentPattern as u8,
    BindingRestElement(BindingRestElement<'id>) = NodeKind::BindingRestElement as u8,
    ArrayPattern(ArrayPattern<'id>) = NodeKind::ArrayPattern as u8,
    ObjectPattern(ObjectPattern<'id>) = NodeKind::ObjectPattern as u8,
    BindingProperty(BindingProperty<'id>) = NodeKind::BindingProperty as u8,
    BinaryExpression(BinaryExpression<'id>) = NodeKind::BinaryExpression as u8,
    LogicalExpression(LogicalExpression<'id>) = NodeKind::LogicalExpression as u8,
    ConditionalExpression(ConditionalExpression<'id>) = NodeKind::ConditionalExpression as u8,
    UnaryExpression(UnaryExpression<'id>) = NodeKind::UnaryExpression as u8,
    UpdateExpression(UpdateExpression<'id>) = NodeKind::UpdateExpression as u8,
    AssignmentExpression(AssignmentExpression<'id>) = NodeKind::AssignmentExpression as u8,
    VariableDeclaration(VariableDeclaration<'id>) = NodeKind::VariableDeclaration as u8,
    VariableDeclarator(VariableDeclarator<'id>) = NodeKind::VariableDeclarator as u8,
    IfStatement(IfStatement<'id>) = NodeKind::IfStatement as u8,
    SwitchStatement(SwitchStatement<'id>) = NodeKind::SwitchStatement as u8,
    SwitchCase(SwitchCase<'id>) = NodeKind::SwitchCase as u8,
    ForStatement(ForStatement<'id>) = NodeKind::ForStatement as u8,
    ForInStatement(ForInStatement<'id>) = NodeKind::ForInStatement as u8,
    ForOfStatement(ForOfStatement<'id>) = NodeKind::ForOfStatement as u8,
    TryStatement(TryStatement<'id>) = NodeKind::TryStatement as u8,
    CatchClause(CatchClause<'id>) = NodeKind::CatchClause as u8,
    BreakStatement(BreakStatement<'id>) = NodeKind::BreakStatement as u8,
    ContinueStatement(ContinueStatement<'id>) = NodeKind::ContinueStatement as u8,
    LabeledStatement(LabeledStatement<'id>) = NodeKind::LabeledStatement as u8,
    ExpressionStatement(ExpressionStatement<'id>) = NodeKind::ExpressionStatement as u8,
    ReturnStatement(ReturnStatement<'id>) = NodeKind::ReturnStatement as u8,
    ThrowStatement(ThrowStatement<'id>) = NodeKind::ThrowStatement as u8,
    DebuggerStatement(DebuggerStatement<'id>) = NodeKind::DebuggerStatement as u8,
    EmptyStatement(EmptyStatement<'id>) = NodeKind::EmptyStatement as u8,
    Directive(Directive<'id>) = NodeKind::Directive as u8,
    Decorator(Decorator<'id>) = NodeKind::Decorator as u8,
    Class(Class<'id>) = NodeKind::Class as u8,
    ClassBody(ClassBody<'id>) = NodeKind::ClassBody as u8,
    MethodDefinition(MethodDefinition<'id>) = NodeKind::MethodDefinition as u8,
    PropertyDefinition(PropertyDefinition<'id>) = NodeKind::PropertyDefinition as u8,
    StaticBlock(StaticBlock<'id>) = NodeKind::StaticBlock as u8,
    Function(Function<'id>) = NodeKind::Function as u8,
    FunctionBody(FunctionBody<'id>) = NodeKind::FunctionBody as u8,
    BlockStatement(BlockStatement<'id>) = NodeKind::BlockStatement as u8,
    WhileStatement(WhileStatement<'id>) = NodeKind::WhileStatement as u8,
    DoWhileStatement(DoWhileStatement<'id>) = NodeKind::DoWhileStatement as u8,
    WithStatement(WithStatement<'id>) = NodeKind::WithStatement as u8,
    FormalParameters(FormalParameters<'id>) = NodeKind::FormalParameters as u8,
    FormalParameter(FormalParameter<'id>) = NodeKind::FormalParameter as u8,
    FormalParameterRest(FormalParameterRest<'id>) = NodeKind::FormalParameterRest as u8,
    ArrayExpression(ArrayExpression<'id>) = NodeKind::ArrayExpression as u8,
    ObjectExpression(ObjectExpression<'id>) = NodeKind::ObjectExpression as u8,
    SpreadElement(SpreadElement<'id>) = NodeKind::SpreadElement as u8,
    ObjectProperty(ObjectProperty<'id>) = NodeKind::ObjectProperty as u8,
    ParenthesizedExpression(ParenthesizedExpression<'id>) = NodeKind::ParenthesizedExpression as u8,
    ArrowFunctionExpression(ArrowFunctionExpression<'id>) = NodeKind::ArrowFunctionExpression as u8,
    SequenceExpression(SequenceExpression<'id>) = NodeKind::SequenceExpression as u8,
    MemberExpression(MemberExpression<'id>) = NodeKind::MemberExpression as u8,
    CallExpression(CallExpression<'id>) = NodeKind::CallExpression as u8,
    ChainExpression(ChainExpression<'id>) = NodeKind::ChainExpression as u8,
    TaggedTemplateExpression(TaggedTemplateExpression<'id>) =
        NodeKind::TaggedTemplateExpression as u8,
    NewExpression(NewExpression<'id>) = NodeKind::NewExpression as u8,
    AwaitExpression(AwaitExpression<'id>) = NodeKind::AwaitExpression as u8,
    YieldExpression(YieldExpression<'id>) = NodeKind::YieldExpression as u8,
    MetaProperty(MetaProperty<'id>) = NodeKind::MetaProperty as u8,
    ImportExpression(ImportExpression<'id>) = NodeKind::ImportExpression as u8,
    ImportDeclaration(ImportDeclaration<'id>) = NodeKind::ImportDeclaration as u8,
    ImportSpecifier(ImportSpecifier<'id>) = NodeKind::ImportSpecifier as u8,
    ImportDefaultSpecifier(ImportDefaultSpecifier<'id>) = NodeKind::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(ImportNamespaceSpecifier<'id>) =
        NodeKind::ImportNamespaceSpecifier as u8,
    ImportAttribute(ImportAttribute<'id>) = NodeKind::ImportAttribute as u8,
    ExportNamedDeclaration(ExportNamedDeclaration<'id>) = NodeKind::ExportNamedDeclaration as u8,
    ExportDefaultDeclaration(ExportDefaultDeclaration<'id>) =
        NodeKind::ExportDefaultDeclaration as u8,
    ExportAllDeclaration(ExportAllDeclaration<'id>) = NodeKind::ExportAllDeclaration as u8,
    ExportSpecifier(ExportSpecifier<'id>) = NodeKind::ExportSpecifier as u8,
    Hashbang(Hashbang<'id>) = NodeKind::Hashbang as u8,
    TsFunction(TsFunction<'id>) = NodeKind::TsFunction as u8,
    TsMethodDefinition(TsMethodDefinition<'id>) = NodeKind::TsMethodDefinition as u8,
    TsTypeAnnotation(TsTypeAnnotation<'id>) = NodeKind::TsTypeAnnotation as u8,
    TsAnyKeyword(TsAnyKeyword<'id>) = NodeKind::TsAnyKeyword as u8,
    TsUnknownKeyword(TsUnknownKeyword<'id>) = NodeKind::TsUnknownKeyword as u8,
    TsNeverKeyword(TsNeverKeyword<'id>) = NodeKind::TsNeverKeyword as u8,
    TsVoidKeyword(TsVoidKeyword<'id>) = NodeKind::TsVoidKeyword as u8,
    TsNullKeyword(TsNullKeyword<'id>) = NodeKind::TsNullKeyword as u8,
    TsUndefinedKeyword(TsUndefinedKeyword<'id>) = NodeKind::TsUndefinedKeyword as u8,
    TsStringKeyword(TsStringKeyword<'id>) = NodeKind::TsStringKeyword as u8,
    TsNumberKeyword(TsNumberKeyword<'id>) = NodeKind::TsNumberKeyword as u8,
    TsBigIntKeyword(TsBigIntKeyword<'id>) = NodeKind::TsBigIntKeyword as u8,
    TsBooleanKeyword(TsBooleanKeyword<'id>) = NodeKind::TsBooleanKeyword as u8,
    TsSymbolKeyword(TsSymbolKeyword<'id>) = NodeKind::TsSymbolKeyword as u8,
    TsObjectKeyword(TsObjectKeyword<'id>) = NodeKind::TsObjectKeyword as u8,
    TsIntrinsicKeyword(TsIntrinsicKeyword<'id>) = NodeKind::TsIntrinsicKeyword as u8,
    TsThisType(TsThisType<'id>) = NodeKind::TsThisType as u8,
    TsTypeReference(TsTypeReference<'id>) = NodeKind::TsTypeReference as u8,
    TsQualifiedName(TsQualifiedName<'id>) = NodeKind::TsQualifiedName as u8,
    TsTypeQuery(TsTypeQuery<'id>) = NodeKind::TsTypeQuery as u8,
    TsImportType(TsImportType<'id>) = NodeKind::TsImportType as u8,
    TsTypeParameter(TsTypeParameter<'id>) = NodeKind::TsTypeParameter as u8,
    TsTypeParameterDeclaration(TsTypeParameterDeclaration<'id>) =
        NodeKind::TsTypeParameterDeclaration as u8,
    TsTypeParameterInstantiation(TsTypeParameterInstantiation<'id>) =
        NodeKind::TsTypeParameterInstantiation as u8,
    TsLiteralType(TsLiteralType<'id>) = NodeKind::TsLiteralType as u8,
    TsTemplateLiteralType(TsTemplateLiteralType<'id>) = NodeKind::TsTemplateLiteralType as u8,
    TsArrayType(TsArrayType<'id>) = NodeKind::TsArrayType as u8,
    TsIndexedAccessType(TsIndexedAccessType<'id>) = NodeKind::TsIndexedAccessType as u8,
    TsTupleType(TsTupleType<'id>) = NodeKind::TsTupleType as u8,
    TsNamedTupleMember(TsNamedTupleMember<'id>) = NodeKind::TsNamedTupleMember as u8,
    TsOptionalType(TsOptionalType<'id>) = NodeKind::TsOptionalType as u8,
    TsRestType(TsRestType<'id>) = NodeKind::TsRestType as u8,
    TsJsDocNullableType(TsJsDocNullableType<'id>) = NodeKind::TsJsDocNullableType as u8,
    TsJsDocNonNullableType(TsJsDocNonNullableType<'id>) = NodeKind::TsJsDocNonNullableType as u8,
    TsJsDocUnknownType(TsJsDocUnknownType<'id>) = NodeKind::TsJsDocUnknownType as u8,
    TsUnionType(TsUnionType<'id>) = NodeKind::TsUnionType as u8,
    TsIntersectionType(TsIntersectionType<'id>) = NodeKind::TsIntersectionType as u8,
    TsConditionalType(TsConditionalType<'id>) = NodeKind::TsConditionalType as u8,
    TsInferType(TsInferType<'id>) = NodeKind::TsInferType as u8,
    TsTypeOperator(TsTypeOperator<'id>) = NodeKind::TsTypeOperator as u8,
    TsParenthesizedType(TsParenthesizedType<'id>) = NodeKind::TsParenthesizedType as u8,
    TsFunctionType(TsFunctionType<'id>) = NodeKind::TsFunctionType as u8,
    TsConstructorType(TsConstructorType<'id>) = NodeKind::TsConstructorType as u8,
    TsTypePredicate(TsTypePredicate<'id>) = NodeKind::TsTypePredicate as u8,
    TsTypeLiteral(TsTypeLiteral<'id>) = NodeKind::TsTypeLiteral as u8,
    TsMappedType(TsMappedType<'id>) = NodeKind::TsMappedType as u8,
    TsPropertySignature(TsPropertySignature<'id>) = NodeKind::TsPropertySignature as u8,
    TsMethodSignature(TsMethodSignature<'id>) = NodeKind::TsMethodSignature as u8,
    TsCallSignatureDeclaration(TsCallSignatureDeclaration<'id>) =
        NodeKind::TsCallSignatureDeclaration as u8,
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration<'id>) =
        NodeKind::TsConstructSignatureDeclaration as u8,
    TsIndexSignature(TsIndexSignature<'id>) = NodeKind::TsIndexSignature as u8,
    TsTypeAliasDeclaration(TsTypeAliasDeclaration<'id>) = NodeKind::TsTypeAliasDeclaration as u8,
    TsInterfaceDeclaration(TsInterfaceDeclaration<'id>) = NodeKind::TsInterfaceDeclaration as u8,
    TsInterfaceBody(TsInterfaceBody<'id>) = NodeKind::TsInterfaceBody as u8,
    TsInterfaceHeritage(TsInterfaceHeritage<'id>) = NodeKind::TsInterfaceHeritage as u8,
    TsClassImplements(TsClassImplements<'id>) = NodeKind::TsClassImplements as u8,
    TsEnumDeclaration(TsEnumDeclaration<'id>) = NodeKind::TsEnumDeclaration as u8,
    TsEnumBody(TsEnumBody<'id>) = NodeKind::TsEnumBody as u8,
    TsEnumMember(TsEnumMember<'id>) = NodeKind::TsEnumMember as u8,
    TsModuleDeclaration(TsModuleDeclaration<'id>) = NodeKind::TsModuleDeclaration as u8,
    TsModuleBlock(TsModuleBlock<'id>) = NodeKind::TsModuleBlock as u8,
    TsGlobalDeclaration(TsGlobalDeclaration<'id>) = NodeKind::TsGlobalDeclaration as u8,
    TsParameterProperty(TsParameterProperty<'id>) = NodeKind::TsParameterProperty as u8,
    TsThisParameter(TsThisParameter<'id>) = NodeKind::TsThisParameter as u8,
    TsAsExpression(TsAsExpression<'id>) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression<'id>) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion<'id>) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression<'id>) = NodeKind::TsNonNullExpression as u8,
    TsInstantiationExpression(TsInstantiationExpression<'id>) =
        NodeKind::TsInstantiationExpression as u8,
    TsExportAssignment(TsExportAssignment<'id>) = NodeKind::TsExportAssignment as u8,
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration<'id>) =
        NodeKind::TsNamespaceExportDeclaration as u8,
    TsImportEqualsDeclaration(TsImportEqualsDeclaration<'id>) =
        NodeKind::TsImportEqualsDeclaration as u8,
    TsExternalModuleReference(TsExternalModuleReference<'id>) =
        NodeKind::TsExternalModuleReference as u8,
    TsIndexSignatureParameter(TsIndexSignatureParameter<'id>) =
        NodeKind::TsIndexSignatureParameter as u8,
    JsxElement(JsxElement<'id>) = NodeKind::JsxElement as u8,
    JsxOpeningElement(JsxOpeningElement<'id>) = NodeKind::JsxOpeningElement as u8,
    JsxClosingElement(JsxClosingElement<'id>) = NodeKind::JsxClosingElement as u8,
    JsxFragment(JsxFragment<'id>) = NodeKind::JsxFragment as u8,
    JsxOpeningFragment(JsxOpeningFragment<'id>) = NodeKind::JsxOpeningFragment as u8,
    JsxClosingFragment(JsxClosingFragment<'id>) = NodeKind::JsxClosingFragment as u8,
    JsxIdentifier(JsxIdentifier<'id>) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName<'id>) = NodeKind::JsxNamespacedName as u8,
    JsxMemberExpression(JsxMemberExpression<'id>) = NodeKind::JsxMemberExpression as u8,
    JsxAttribute(JsxAttribute<'id>) = NodeKind::JsxAttribute as u8,
    JsxSpreadAttribute(JsxSpreadAttribute<'id>) = NodeKind::JsxSpreadAttribute as u8,
    JsxExpressionContainer(JsxExpressionContainer<'id>) = NodeKind::JsxExpressionContainer as u8,
    JsxEmptyExpression(JsxEmptyExpression<'id>) = NodeKind::JsxEmptyExpression as u8,
    JsxText(JsxText<'id>) = NodeKind::JsxText as u8,
    JsxSpreadChild(JsxSpreadChild<'id>) = NodeKind::JsxSpreadChild as u8,
}
impl<'id> NodeKindData<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        unsafe { std::mem::transmute::<Self, AstEnumDataRepr<'id>>(self).node_id }
    }
}
impl<'id> Program<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Program<'id> {}
impl<'id> AstNode<'id> for Program<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Program<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Program<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Stmt<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Stmt<'id> {}
impl<'id> AstNode<'id> for Stmt<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Stmt<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Stmt<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Stmt<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BlockStatement(node: BlockStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BreakStatement(node: BreakStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ContinueStatement(node: ContinueStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn DebuggerStatement(node: DebuggerStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn DoWhileStatement(node: DoWhileStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn EmptyStatement(node: EmptyStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExpressionStatement(node: ExpressionStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForInStatement(node: ForInStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForOfStatement(node: ForOfStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ForStatement(node: ForStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn IfStatement(node: IfStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn LabeledStatement(node: LabeledStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ReturnStatement(node: ReturnStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SwitchStatement(node: SwitchStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThrowStatement(node: ThrowStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TryStatement(node: TryStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn WhileStatement(node: WhileStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn WithStatement(node: WithStatement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Declaration(node: Decl<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportDeclaration(node: ImportDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportNamedDeclaration(node: ExportNamedDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportDefaultDeclaration(node: ExportDefaultDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ExportAllDeclaration(node: ExportAllDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsExportAssignment(node: TsExportAssignment<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNamespaceExportDeclaration(node: TsNamespaceExportDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> Decl<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Decl<'id> {}
impl<'id> AstNode<'id> for Decl<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Decl<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Decl<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Decl<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunction(node: TsFunction<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAliasDeclaration(node: TsTypeAliasDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInterfaceDeclaration(node: TsInterfaceDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsEnumDeclaration(node: TsEnumDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleDeclaration(node: TsModuleDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsGlobalDeclaration(node: TsGlobalDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportEqualsDeclaration(node: TsImportEqualsDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> Expr<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Expr<'id> {}
impl<'id> AstNode<'id> for Expr<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Expr<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Expr<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Expr<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Super(node: Super<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MetaProperty(node: MetaProperty<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NumericLiteral(node: NumericLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BigIntLiteral(node: BigIntLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BooleanLiteral(node: BooleanLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NullLiteral(node: NullLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn RegExpLiteral(node: RegExpLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BinaryExpression(node: BinaryExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn LogicalExpression(node: LogicalExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UnaryExpression(node: UnaryExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UpdateExpression(node: UpdateExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentExpression(node: AssignmentExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ConditionalExpression(node: ConditionalExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SequenceExpression(node: SequenceExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MemberExpression(node: MemberExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn CallExpression(node: CallExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ChainExpression(node: ChainExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NewExpression(node: NewExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TaggedTemplateExpression(node: TaggedTemplateExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrowFunctionExpression(node: ArrowFunctionExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayExpression(node: ArrayExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectExpression(node: ObjectExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ParenthesizedExpression(node: ParenthesizedExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportExpression(node: ImportExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AwaitExpression(node: AwaitExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn YieldExpression(node: YieldExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn PrivateIdentifier(node: PrivateIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAsExpression(node: TsAsExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSatisfiesExpression(node: TsSatisfiesExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAssertion(node: TsTypeAssertion<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNonNullExpression(node: TsNonNullExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInstantiationExpression(node: TsInstantiationExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> BindingPattern<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BindingPattern<'id> {}
impl<'id> AstNode<'id> for BindingPattern<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BindingPattern<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BindingPattern<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BindingPattern<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SimpleAssignmentTarget(node: SimpleAssignmentTarget<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentPattern(node: AssignmentPattern<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingRestElement(node: BindingRestElement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayPattern(node: ArrayPattern<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectPattern(node: ObjectPattern<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> AssignmentTarget<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for AssignmentTarget<'id> {}
impl<'id> AstNode<'id> for AssignmentTarget<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for AssignmentTarget<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for AssignmentTarget<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> AssignmentTarget<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn SimpleAssignmentTarget(node: SimpleAssignmentTarget<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayAssignmentTarget(node: ArrayPattern<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectAssignmentTarget(node: ObjectPattern<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> SimpleAssignmentTarget<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for SimpleAssignmentTarget<'id> {}
impl<'id> AstNode<'id> for SimpleAssignmentTarget<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for SimpleAssignmentTarget<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for SimpleAssignmentTarget<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> SimpleAssignmentTarget<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn MemberExpression(node: MemberExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAsExpression(node: TsAsExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSatisfiesExpression(node: TsSatisfiesExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeAssertion(node: TsTypeAssertion<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNonNullExpression(node: TsNonNullExpression<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> PropertyKey<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for PropertyKey<'id> {}
impl<'id> AstNode<'id> for PropertyKey<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for PropertyKey<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for PropertyKey<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> PropertyKey<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ModuleExportName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ModuleExportName<'id> {}
impl<'id> AstNode<'id> for ModuleExportName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ModuleExportName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ModuleExportName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ModuleExportName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> Argument<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Argument<'id> {}
impl<'id> AstNode<'id> for Argument<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Argument<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Argument<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Argument<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SpreadElement(node: SpreadElement<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ObjectPropertyKind<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ObjectPropertyKind<'id> {}
impl<'id> AstNode<'id> for ObjectPropertyKind<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ObjectPropertyKind<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ObjectPropertyKind<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ObjectPropertyKind<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectProperty(node: ObjectProperty<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn SpreadElement(node: SpreadElement<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ForStatementInit<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ForStatementInit<'id> {}
impl<'id> AstNode<'id> for ForStatementInit<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ForStatementInit<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ForStatementInit<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ForStatementInit<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ForStatementLeft<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ForStatementLeft<'id> {}
impl<'id> AstNode<'id> for ForStatementLeft<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ForStatementLeft<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ForStatementLeft<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ForStatementLeft<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn VariableDeclaration(node: VariableDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn AssignmentTarget(node: AssignmentTarget<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ArrowFunctionBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ArrowFunctionBody<'id> {}
impl<'id> AstNode<'id> for ArrowFunctionBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ArrowFunctionBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ArrowFunctionBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ArrowFunctionBody<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn FunctionBody(node: FunctionBody<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> FormalParameterItem<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FormalParameterItem<'id> {}
impl<'id> AstNode<'id> for FormalParameterItem<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FormalParameterItem<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FormalParameterItem<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FormalParameterItem<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn FormalParameter(node: FormalParameter<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsParameterProperty(node: TsParameterProperty<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> FormalParameterPattern<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FormalParameterPattern<'id> {}
impl<'id> AstNode<'id> for FormalParameterPattern<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FormalParameterPattern<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FormalParameterPattern<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FormalParameterPattern<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ArrayPattern(node: ArrayPattern<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ObjectPattern(node: ObjectPattern<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ClassElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ClassElement<'id> {}
impl<'id> AstNode<'id> for ClassElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ClassElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ClassElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ClassElement<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn MethodDefinition(node: MethodDefinition<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn PropertyDefinition(node: PropertyDefinition<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StaticBlock(node: StaticBlock<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMethodDefinition(node: TsMethodDefinition<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexSignature(node: TsIndexSignature<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ImportDeclarationSpecifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportDeclarationSpecifier<'id> {}
impl<'id> AstNode<'id> for ImportDeclarationSpecifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportDeclarationSpecifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportDeclarationSpecifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportDeclarationSpecifier<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportSpecifier(node: ImportSpecifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportDefaultSpecifier(node: ImportDefaultSpecifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ImportNamespaceSpecifier(node: ImportNamespaceSpecifier<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> ExportDefaultDeclarationKind<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExportDefaultDeclarationKind<'id> {}
impl<'id> AstNode<'id> for ExportDefaultDeclarationKind<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExportDefaultDeclarationKind<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExportDefaultDeclarationKind<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExportDefaultDeclarationKind<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Function(node: Function<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn Class(node: Class<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunction(node: TsFunction<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInterfaceDeclaration(node: TsInterfaceDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> Super<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Super<'id> {}
impl<'id> AstNode<'id> for Super<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Super<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Super<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ThisExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ThisExpression<'id> {}
impl<'id> AstNode<'id> for ThisExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ThisExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ThisExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> StringLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for StringLiteral<'id> {}
impl<'id> AstNode<'id> for StringLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for StringLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for StringLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> NumericLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for NumericLiteral<'id> {}
impl<'id> AstNode<'id> for NumericLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for NumericLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for NumericLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BigIntLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BigIntLiteral<'id> {}
impl<'id> AstNode<'id> for BigIntLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BigIntLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BigIntLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BooleanLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BooleanLiteral<'id> {}
impl<'id> AstNode<'id> for BooleanLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BooleanLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BooleanLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> NullLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for NullLiteral<'id> {}
impl<'id> AstNode<'id> for NullLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for NullLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for NullLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> RegExpLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for RegExpLiteral<'id> {}
impl<'id> AstNode<'id> for RegExpLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for RegExpLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for RegExpLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TemplateLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TemplateLiteral<'id> {}
impl<'id> AstNode<'id> for TemplateLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TemplateLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TemplateLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TemplateElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TemplateElement<'id> {}
impl<'id> AstNode<'id> for TemplateElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TemplateElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TemplateElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> IdentifierReference<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for IdentifierReference<'id> {}
impl<'id> AstNode<'id> for IdentifierReference<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for IdentifierReference<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for IdentifierReference<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> PrivateIdentifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for PrivateIdentifier<'id> {}
impl<'id> AstNode<'id> for PrivateIdentifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for PrivateIdentifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for PrivateIdentifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BindingIdentifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BindingIdentifier<'id> {}
impl<'id> AstNode<'id> for BindingIdentifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BindingIdentifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BindingIdentifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> IdentifierName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for IdentifierName<'id> {}
impl<'id> AstNode<'id> for IdentifierName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for IdentifierName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for IdentifierName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> LabelIdentifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for LabelIdentifier<'id> {}
impl<'id> AstNode<'id> for LabelIdentifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for LabelIdentifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for LabelIdentifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> AssignmentPattern<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for AssignmentPattern<'id> {}
impl<'id> AstNode<'id> for AssignmentPattern<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for AssignmentPattern<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for AssignmentPattern<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BindingRestElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BindingRestElement<'id> {}
impl<'id> AstNode<'id> for BindingRestElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BindingRestElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BindingRestElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ArrayPattern<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ArrayPattern<'id> {}
impl<'id> AstNode<'id> for ArrayPattern<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ArrayPattern<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ArrayPattern<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ObjectPattern<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ObjectPattern<'id> {}
impl<'id> AstNode<'id> for ObjectPattern<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ObjectPattern<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ObjectPattern<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BindingProperty<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BindingProperty<'id> {}
impl<'id> AstNode<'id> for BindingProperty<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BindingProperty<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BindingProperty<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BinaryExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BinaryExpression<'id> {}
impl<'id> AstNode<'id> for BinaryExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BinaryExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BinaryExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> LogicalExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for LogicalExpression<'id> {}
impl<'id> AstNode<'id> for LogicalExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for LogicalExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for LogicalExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ConditionalExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ConditionalExpression<'id> {}
impl<'id> AstNode<'id> for ConditionalExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ConditionalExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ConditionalExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> UnaryExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for UnaryExpression<'id> {}
impl<'id> AstNode<'id> for UnaryExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for UnaryExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for UnaryExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> UpdateExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for UpdateExpression<'id> {}
impl<'id> AstNode<'id> for UpdateExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for UpdateExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for UpdateExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> AssignmentExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for AssignmentExpression<'id> {}
impl<'id> AstNode<'id> for AssignmentExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for AssignmentExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for AssignmentExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> VariableDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for VariableDeclaration<'id> {}
impl<'id> AstNode<'id> for VariableDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for VariableDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for VariableDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> VariableDeclarator<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for VariableDeclarator<'id> {}
impl<'id> AstNode<'id> for VariableDeclarator<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for VariableDeclarator<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for VariableDeclarator<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> IfStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for IfStatement<'id> {}
impl<'id> AstNode<'id> for IfStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for IfStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for IfStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> SwitchStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for SwitchStatement<'id> {}
impl<'id> AstNode<'id> for SwitchStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for SwitchStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for SwitchStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> SwitchCase<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for SwitchCase<'id> {}
impl<'id> AstNode<'id> for SwitchCase<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for SwitchCase<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for SwitchCase<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ForStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ForStatement<'id> {}
impl<'id> AstNode<'id> for ForStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ForStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ForStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ForInStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ForInStatement<'id> {}
impl<'id> AstNode<'id> for ForInStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ForInStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ForInStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ForOfStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ForOfStatement<'id> {}
impl<'id> AstNode<'id> for ForOfStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ForOfStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ForOfStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TryStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TryStatement<'id> {}
impl<'id> AstNode<'id> for TryStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TryStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TryStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> CatchClause<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for CatchClause<'id> {}
impl<'id> AstNode<'id> for CatchClause<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for CatchClause<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for CatchClause<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BreakStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BreakStatement<'id> {}
impl<'id> AstNode<'id> for BreakStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BreakStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BreakStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ContinueStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ContinueStatement<'id> {}
impl<'id> AstNode<'id> for ContinueStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ContinueStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ContinueStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> LabeledStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for LabeledStatement<'id> {}
impl<'id> AstNode<'id> for LabeledStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for LabeledStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for LabeledStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExpressionStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExpressionStatement<'id> {}
impl<'id> AstNode<'id> for ExpressionStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExpressionStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExpressionStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ReturnStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ReturnStatement<'id> {}
impl<'id> AstNode<'id> for ReturnStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ReturnStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ReturnStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ThrowStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ThrowStatement<'id> {}
impl<'id> AstNode<'id> for ThrowStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ThrowStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ThrowStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> DebuggerStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for DebuggerStatement<'id> {}
impl<'id> AstNode<'id> for DebuggerStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for DebuggerStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for DebuggerStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> EmptyStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for EmptyStatement<'id> {}
impl<'id> AstNode<'id> for EmptyStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for EmptyStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for EmptyStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Directive<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Directive<'id> {}
impl<'id> AstNode<'id> for Directive<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Directive<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Directive<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Decorator<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Decorator<'id> {}
impl<'id> AstNode<'id> for Decorator<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Decorator<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Decorator<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Class<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Class<'id> {}
impl<'id> AstNode<'id> for Class<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Class<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Class<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ClassBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ClassBody<'id> {}
impl<'id> AstNode<'id> for ClassBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ClassBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ClassBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> MethodDefinition<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for MethodDefinition<'id> {}
impl<'id> AstNode<'id> for MethodDefinition<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for MethodDefinition<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for MethodDefinition<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> PropertyDefinition<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for PropertyDefinition<'id> {}
impl<'id> AstNode<'id> for PropertyDefinition<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for PropertyDefinition<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for PropertyDefinition<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> StaticBlock<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for StaticBlock<'id> {}
impl<'id> AstNode<'id> for StaticBlock<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for StaticBlock<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for StaticBlock<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Function<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Function<'id> {}
impl<'id> AstNode<'id> for Function<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Function<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Function<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FunctionBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FunctionBody<'id> {}
impl<'id> AstNode<'id> for FunctionBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FunctionBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FunctionBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> BlockStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for BlockStatement<'id> {}
impl<'id> AstNode<'id> for BlockStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for BlockStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for BlockStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> WhileStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for WhileStatement<'id> {}
impl<'id> AstNode<'id> for WhileStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for WhileStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for WhileStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> DoWhileStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for DoWhileStatement<'id> {}
impl<'id> AstNode<'id> for DoWhileStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for DoWhileStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for DoWhileStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> WithStatement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for WithStatement<'id> {}
impl<'id> AstNode<'id> for WithStatement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for WithStatement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for WithStatement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FormalParameters<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FormalParameters<'id> {}
impl<'id> AstNode<'id> for FormalParameters<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FormalParameters<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FormalParameters<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FormalParameter<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FormalParameter<'id> {}
impl<'id> AstNode<'id> for FormalParameter<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FormalParameter<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FormalParameter<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> FormalParameterRest<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for FormalParameterRest<'id> {}
impl<'id> AstNode<'id> for FormalParameterRest<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for FormalParameterRest<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for FormalParameterRest<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ArrayExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ArrayExpression<'id> {}
impl<'id> AstNode<'id> for ArrayExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ArrayExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ArrayExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ObjectExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ObjectExpression<'id> {}
impl<'id> AstNode<'id> for ObjectExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ObjectExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ObjectExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> SpreadElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for SpreadElement<'id> {}
impl<'id> AstNode<'id> for SpreadElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for SpreadElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for SpreadElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ObjectProperty<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ObjectProperty<'id> {}
impl<'id> AstNode<'id> for ObjectProperty<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ObjectProperty<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ObjectProperty<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ParenthesizedExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ParenthesizedExpression<'id> {}
impl<'id> AstNode<'id> for ParenthesizedExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ParenthesizedExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ParenthesizedExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ArrowFunctionExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ArrowFunctionExpression<'id> {}
impl<'id> AstNode<'id> for ArrowFunctionExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ArrowFunctionExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ArrowFunctionExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> SequenceExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for SequenceExpression<'id> {}
impl<'id> AstNode<'id> for SequenceExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for SequenceExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for SequenceExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> MemberExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for MemberExpression<'id> {}
impl<'id> AstNode<'id> for MemberExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for MemberExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for MemberExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> CallExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for CallExpression<'id> {}
impl<'id> AstNode<'id> for CallExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for CallExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for CallExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ChainExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ChainExpression<'id> {}
impl<'id> AstNode<'id> for ChainExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ChainExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ChainExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TaggedTemplateExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TaggedTemplateExpression<'id> {}
impl<'id> AstNode<'id> for TaggedTemplateExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TaggedTemplateExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TaggedTemplateExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> NewExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for NewExpression<'id> {}
impl<'id> AstNode<'id> for NewExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for NewExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for NewExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> AwaitExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for AwaitExpression<'id> {}
impl<'id> AstNode<'id> for AwaitExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for AwaitExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for AwaitExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> YieldExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for YieldExpression<'id> {}
impl<'id> AstNode<'id> for YieldExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for YieldExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for YieldExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> MetaProperty<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for MetaProperty<'id> {}
impl<'id> AstNode<'id> for MetaProperty<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for MetaProperty<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for MetaProperty<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportExpression<'id> {}
impl<'id> AstNode<'id> for ImportExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportDeclaration<'id> {}
impl<'id> AstNode<'id> for ImportDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportSpecifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportSpecifier<'id> {}
impl<'id> AstNode<'id> for ImportSpecifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportSpecifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportSpecifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportDefaultSpecifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportDefaultSpecifier<'id> {}
impl<'id> AstNode<'id> for ImportDefaultSpecifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportDefaultSpecifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportDefaultSpecifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportNamespaceSpecifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportNamespaceSpecifier<'id> {}
impl<'id> AstNode<'id> for ImportNamespaceSpecifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportNamespaceSpecifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportNamespaceSpecifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ImportAttribute<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ImportAttribute<'id> {}
impl<'id> AstNode<'id> for ImportAttribute<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ImportAttribute<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ImportAttribute<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExportNamedDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExportNamedDeclaration<'id> {}
impl<'id> AstNode<'id> for ExportNamedDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExportNamedDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExportNamedDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExportDefaultDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExportDefaultDeclaration<'id> {}
impl<'id> AstNode<'id> for ExportDefaultDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExportDefaultDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExportDefaultDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExportAllDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExportAllDeclaration<'id> {}
impl<'id> AstNode<'id> for ExportAllDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExportAllDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExportAllDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> ExportSpecifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for ExportSpecifier<'id> {}
impl<'id> AstNode<'id> for ExportSpecifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for ExportSpecifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for ExportSpecifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> Hashbang<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for Hashbang<'id> {}
impl<'id> AstNode<'id> for Hashbang<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for Hashbang<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for Hashbang<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsType<'id> {}
impl<'id> AstNode<'id> for TsType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsType<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsAnyKeyword(node: TsAnyKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUnknownKeyword(node: TsUnknownKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNeverKeyword(node: TsNeverKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsVoidKeyword(node: TsVoidKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNullKeyword(node: TsNullKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUndefinedKeyword(node: TsUndefinedKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsStringKeyword(node: TsStringKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNumberKeyword(node: TsNumberKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsBigIntKeyword(node: TsBigIntKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsBooleanKeyword(node: TsBooleanKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsSymbolKeyword(node: TsSymbolKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsObjectKeyword(node: TsObjectKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIntrinsicKeyword(node: TsIntrinsicKeyword<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsThisType(node: TsThisType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeReference(node: TsTypeReference<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeQuery(node: TsTypeQuery<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportType(node: TsImportType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsLiteralType(node: TsLiteralType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTemplateLiteralType(node: TsTemplateLiteralType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsArrayType(node: TsArrayType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexedAccessType(node: TsIndexedAccessType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTupleType(node: TsTupleType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsOptionalType(node: TsOptionalType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsRestType(node: TsRestType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocNullableType(node: TsJsDocNullableType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocNonNullableType(node: TsJsDocNonNullableType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsJsDocUnknownType(node: TsJsDocUnknownType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsUnionType(node: TsUnionType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIntersectionType(node: TsIntersectionType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConditionalType(node: TsConditionalType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsInferType(node: TsInferType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeOperator(node: TsTypeOperator<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsParenthesizedType(node: TsParenthesizedType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsFunctionType(node: TsFunctionType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConstructorType(node: TsConstructorType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypePredicate(node: TsTypePredicate<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeLiteral(node: TsTypeLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMappedType(node: TsMappedType<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsTypeName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeName<'id> {}
impl<'id> AstNode<'id> for TsTypeName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsQualifiedNameLeft<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsQualifiedNameLeft<'id> {}
impl<'id> AstNode<'id> for TsQualifiedNameLeft<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsQualifiedNameLeft<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsQualifiedNameLeft<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsQualifiedNameLeft<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierReference(node: IdentifierReference<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsImportTypeQualifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsImportTypeQualifier<'id> {}
impl<'id> AstNode<'id> for TsImportTypeQualifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsImportTypeQualifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsImportTypeQualifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsImportTypeQualifier<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsTypeQueryExprName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeQueryExprName<'id> {}
impl<'id> AstNode<'id> for TsTypeQueryExprName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeQueryExprName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeQueryExprName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeQueryExprName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeName(node: TsTypeName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn ThisExpression(node: ThisExpression<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsImportType(node: TsImportType<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsLiteral<'id> {}
impl<'id> AstNode<'id> for TsLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsLiteral<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn NumericLiteral(node: NumericLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BigIntLiteral(node: BigIntLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn BooleanLiteral(node: BooleanLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn UnaryExpression(node: UnaryExpression<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsTupleElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTupleElement<'id> {}
impl<'id> AstNode<'id> for TsTupleElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTupleElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTupleElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTupleElement<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsType(node: TsType<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsNamedTupleMember(node: TsNamedTupleMember<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsTypeElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeElement<'id> {}
impl<'id> AstNode<'id> for TsTypeElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeElement<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsPropertySignature(node: TsPropertySignature<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsMethodSignature(node: TsMethodSignature<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsCallSignatureDeclaration(node: TsCallSignatureDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsConstructSignatureDeclaration(node: TsConstructSignatureDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsIndexSignature(node: TsIndexSignature<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsTypePredicateName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypePredicateName<'id> {}
impl<'id> AstNode<'id> for TsTypePredicateName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypePredicateName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypePredicateName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypePredicateName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsThisType(node: TsThisType<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsEnumMemberName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsEnumMemberName<'id> {}
impl<'id> AstNode<'id> for TsEnumMemberName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsEnumMemberName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsEnumMemberName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsEnumMemberName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn IdentifierName(node: IdentifierName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TemplateLiteral(node: TemplateLiteral<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsModuleName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsModuleName<'id> {}
impl<'id> AstNode<'id> for TsModuleName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsModuleName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsModuleName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsModuleName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn BindingIdentifier(node: BindingIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsQualifiedName(node: TsQualifiedName<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsModuleBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsModuleBody<'id> {}
impl<'id> AstNode<'id> for TsModuleBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsModuleBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsModuleBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsModuleBody<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleBlock(node: TsModuleBlock<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsModuleDeclaration(node: TsModuleDeclaration<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsModuleReference<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsModuleReference<'id> {}
impl<'id> AstNode<'id> for TsModuleReference<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsModuleReference<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsModuleReference<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsModuleReference<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsTypeName(node: TsTypeName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn TsExternalModuleReference(node: TsExternalModuleReference<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> TsFunction<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsFunction<'id> {}
impl<'id> AstNode<'id> for TsFunction<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsFunction<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsFunction<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsMethodDefinition<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsMethodDefinition<'id> {}
impl<'id> AstNode<'id> for TsMethodDefinition<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsMethodDefinition<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsMethodDefinition<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeAnnotation<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeAnnotation<'id> {}
impl<'id> AstNode<'id> for TsTypeAnnotation<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeAnnotation<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeAnnotation<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsAnyKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsAnyKeyword<'id> {}
impl<'id> AstNode<'id> for TsAnyKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsAnyKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsAnyKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsUnknownKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsUnknownKeyword<'id> {}
impl<'id> AstNode<'id> for TsUnknownKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsUnknownKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsUnknownKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNeverKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNeverKeyword<'id> {}
impl<'id> AstNode<'id> for TsNeverKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNeverKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNeverKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsVoidKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsVoidKeyword<'id> {}
impl<'id> AstNode<'id> for TsVoidKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsVoidKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsVoidKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNullKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNullKeyword<'id> {}
impl<'id> AstNode<'id> for TsNullKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNullKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNullKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsUndefinedKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsUndefinedKeyword<'id> {}
impl<'id> AstNode<'id> for TsUndefinedKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsUndefinedKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsUndefinedKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsStringKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsStringKeyword<'id> {}
impl<'id> AstNode<'id> for TsStringKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsStringKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsStringKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNumberKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNumberKeyword<'id> {}
impl<'id> AstNode<'id> for TsNumberKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNumberKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNumberKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsBigIntKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsBigIntKeyword<'id> {}
impl<'id> AstNode<'id> for TsBigIntKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsBigIntKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsBigIntKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsBooleanKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsBooleanKeyword<'id> {}
impl<'id> AstNode<'id> for TsBooleanKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsBooleanKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsBooleanKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsSymbolKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsSymbolKeyword<'id> {}
impl<'id> AstNode<'id> for TsSymbolKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsSymbolKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsSymbolKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsObjectKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsObjectKeyword<'id> {}
impl<'id> AstNode<'id> for TsObjectKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsObjectKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsObjectKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsIntrinsicKeyword<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsIntrinsicKeyword<'id> {}
impl<'id> AstNode<'id> for TsIntrinsicKeyword<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsIntrinsicKeyword<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsIntrinsicKeyword<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsThisType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsThisType<'id> {}
impl<'id> AstNode<'id> for TsThisType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsThisType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsThisType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeReference<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeReference<'id> {}
impl<'id> AstNode<'id> for TsTypeReference<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeReference<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeReference<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsQualifiedName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsQualifiedName<'id> {}
impl<'id> AstNode<'id> for TsQualifiedName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsQualifiedName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsQualifiedName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeQuery<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeQuery<'id> {}
impl<'id> AstNode<'id> for TsTypeQuery<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeQuery<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeQuery<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsImportType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsImportType<'id> {}
impl<'id> AstNode<'id> for TsImportType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsImportType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsImportType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeParameter<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeParameter<'id> {}
impl<'id> AstNode<'id> for TsTypeParameter<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeParameter<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeParameter<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeParameterDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeParameterDeclaration<'id> {}
impl<'id> AstNode<'id> for TsTypeParameterDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeParameterDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeParameterDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeParameterInstantiation<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeParameterInstantiation<'id> {}
impl<'id> AstNode<'id> for TsTypeParameterInstantiation<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeParameterInstantiation<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeParameterInstantiation<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsLiteralType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsLiteralType<'id> {}
impl<'id> AstNode<'id> for TsLiteralType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsLiteralType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsLiteralType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTemplateLiteralType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTemplateLiteralType<'id> {}
impl<'id> AstNode<'id> for TsTemplateLiteralType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTemplateLiteralType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTemplateLiteralType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsArrayType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsArrayType<'id> {}
impl<'id> AstNode<'id> for TsArrayType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsArrayType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsArrayType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsIndexedAccessType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsIndexedAccessType<'id> {}
impl<'id> AstNode<'id> for TsIndexedAccessType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsIndexedAccessType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsIndexedAccessType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTupleType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTupleType<'id> {}
impl<'id> AstNode<'id> for TsTupleType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTupleType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTupleType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNamedTupleMember<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNamedTupleMember<'id> {}
impl<'id> AstNode<'id> for TsNamedTupleMember<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNamedTupleMember<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNamedTupleMember<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsOptionalType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsOptionalType<'id> {}
impl<'id> AstNode<'id> for TsOptionalType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsOptionalType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsOptionalType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsRestType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsRestType<'id> {}
impl<'id> AstNode<'id> for TsRestType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsRestType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsRestType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsJsDocNullableType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsJsDocNullableType<'id> {}
impl<'id> AstNode<'id> for TsJsDocNullableType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsJsDocNullableType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsJsDocNullableType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsJsDocNonNullableType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsJsDocNonNullableType<'id> {}
impl<'id> AstNode<'id> for TsJsDocNonNullableType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsJsDocNonNullableType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsJsDocNonNullableType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsJsDocUnknownType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsJsDocUnknownType<'id> {}
impl<'id> AstNode<'id> for TsJsDocUnknownType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsJsDocUnknownType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsJsDocUnknownType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsUnionType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsUnionType<'id> {}
impl<'id> AstNode<'id> for TsUnionType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsUnionType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsUnionType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsIntersectionType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsIntersectionType<'id> {}
impl<'id> AstNode<'id> for TsIntersectionType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsIntersectionType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsIntersectionType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsConditionalType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsConditionalType<'id> {}
impl<'id> AstNode<'id> for TsConditionalType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsConditionalType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsConditionalType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsInferType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsInferType<'id> {}
impl<'id> AstNode<'id> for TsInferType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsInferType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsInferType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeOperator<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeOperator<'id> {}
impl<'id> AstNode<'id> for TsTypeOperator<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeOperator<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeOperator<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsParenthesizedType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsParenthesizedType<'id> {}
impl<'id> AstNode<'id> for TsParenthesizedType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsParenthesizedType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsParenthesizedType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsFunctionType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsFunctionType<'id> {}
impl<'id> AstNode<'id> for TsFunctionType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsFunctionType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsFunctionType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsConstructorType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsConstructorType<'id> {}
impl<'id> AstNode<'id> for TsConstructorType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsConstructorType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsConstructorType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypePredicate<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypePredicate<'id> {}
impl<'id> AstNode<'id> for TsTypePredicate<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypePredicate<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypePredicate<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeLiteral<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeLiteral<'id> {}
impl<'id> AstNode<'id> for TsTypeLiteral<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeLiteral<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeLiteral<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsMappedType<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsMappedType<'id> {}
impl<'id> AstNode<'id> for TsMappedType<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsMappedType<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsMappedType<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsPropertySignature<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsPropertySignature<'id> {}
impl<'id> AstNode<'id> for TsPropertySignature<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsPropertySignature<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsPropertySignature<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsMethodSignature<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsMethodSignature<'id> {}
impl<'id> AstNode<'id> for TsMethodSignature<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsMethodSignature<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsMethodSignature<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsCallSignatureDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsCallSignatureDeclaration<'id> {}
impl<'id> AstNode<'id> for TsCallSignatureDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsCallSignatureDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsCallSignatureDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsConstructSignatureDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsConstructSignatureDeclaration<'id> {}
impl<'id> AstNode<'id> for TsConstructSignatureDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsConstructSignatureDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsConstructSignatureDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsIndexSignature<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsIndexSignature<'id> {}
impl<'id> AstNode<'id> for TsIndexSignature<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsIndexSignature<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsIndexSignature<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeAliasDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeAliasDeclaration<'id> {}
impl<'id> AstNode<'id> for TsTypeAliasDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeAliasDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeAliasDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsInterfaceDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsInterfaceDeclaration<'id> {}
impl<'id> AstNode<'id> for TsInterfaceDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsInterfaceDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsInterfaceDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsInterfaceBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsInterfaceBody<'id> {}
impl<'id> AstNode<'id> for TsInterfaceBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsInterfaceBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsInterfaceBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsInterfaceHeritage<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsInterfaceHeritage<'id> {}
impl<'id> AstNode<'id> for TsInterfaceHeritage<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsInterfaceHeritage<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsInterfaceHeritage<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsClassImplements<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsClassImplements<'id> {}
impl<'id> AstNode<'id> for TsClassImplements<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsClassImplements<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsClassImplements<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsEnumDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsEnumDeclaration<'id> {}
impl<'id> AstNode<'id> for TsEnumDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsEnumDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsEnumDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsEnumBody<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsEnumBody<'id> {}
impl<'id> AstNode<'id> for TsEnumBody<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsEnumBody<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsEnumBody<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsEnumMember<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsEnumMember<'id> {}
impl<'id> AstNode<'id> for TsEnumMember<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsEnumMember<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsEnumMember<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsModuleDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsModuleDeclaration<'id> {}
impl<'id> AstNode<'id> for TsModuleDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsModuleDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsModuleDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsModuleBlock<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsModuleBlock<'id> {}
impl<'id> AstNode<'id> for TsModuleBlock<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsModuleBlock<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsModuleBlock<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsGlobalDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsGlobalDeclaration<'id> {}
impl<'id> AstNode<'id> for TsGlobalDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsGlobalDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsGlobalDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsParameterProperty<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsParameterProperty<'id> {}
impl<'id> AstNode<'id> for TsParameterProperty<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsParameterProperty<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsParameterProperty<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsThisParameter<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsThisParameter<'id> {}
impl<'id> AstNode<'id> for TsThisParameter<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsThisParameter<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsThisParameter<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsAsExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsAsExpression<'id> {}
impl<'id> AstNode<'id> for TsAsExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsAsExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsAsExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsSatisfiesExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsSatisfiesExpression<'id> {}
impl<'id> AstNode<'id> for TsSatisfiesExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsSatisfiesExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsSatisfiesExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsTypeAssertion<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsTypeAssertion<'id> {}
impl<'id> AstNode<'id> for TsTypeAssertion<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsTypeAssertion<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsTypeAssertion<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNonNullExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNonNullExpression<'id> {}
impl<'id> AstNode<'id> for TsNonNullExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNonNullExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNonNullExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsInstantiationExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsInstantiationExpression<'id> {}
impl<'id> AstNode<'id> for TsInstantiationExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsInstantiationExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsInstantiationExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsExportAssignment<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsExportAssignment<'id> {}
impl<'id> AstNode<'id> for TsExportAssignment<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsExportAssignment<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsExportAssignment<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsNamespaceExportDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsNamespaceExportDeclaration<'id> {}
impl<'id> AstNode<'id> for TsNamespaceExportDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsNamespaceExportDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsNamespaceExportDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsImportEqualsDeclaration<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsImportEqualsDeclaration<'id> {}
impl<'id> AstNode<'id> for TsImportEqualsDeclaration<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsImportEqualsDeclaration<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsImportEqualsDeclaration<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsExternalModuleReference<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsExternalModuleReference<'id> {}
impl<'id> AstNode<'id> for TsExternalModuleReference<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsExternalModuleReference<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsExternalModuleReference<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> TsIndexSignatureParameter<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for TsIndexSignatureParameter<'id> {}
impl<'id> AstNode<'id> for TsIndexSignatureParameter<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for TsIndexSignatureParameter<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for TsIndexSignatureParameter<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxElementName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxElementName<'id> {}
impl<'id> AstNode<'id> for JsxElementName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxElementName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxElementName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxElementName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxNamespacedName(node: JsxNamespacedName<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxMemberExpression(node: JsxMemberExpression<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxMemberExpressionObject<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxMemberExpressionObject<'id> {}
impl<'id> AstNode<'id> for JsxMemberExpressionObject<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxMemberExpressionObject<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxMemberExpressionObject<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxMemberExpressionObject<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxMemberExpression(node: JsxMemberExpression<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxAttributeName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxAttributeName<'id> {}
impl<'id> AstNode<'id> for JsxAttributeName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxAttributeName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxAttributeName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxAttributeName<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxIdentifier(node: JsxIdentifier<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxNamespacedName(node: JsxNamespacedName<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxAttributeItem<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxAttributeItem<'id> {}
impl<'id> AstNode<'id> for JsxAttributeItem<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxAttributeItem<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxAttributeItem<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxAttributeItem<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxAttribute(node: JsxAttribute<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxSpreadAttribute(node: JsxSpreadAttribute<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxAttributeValue<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxAttributeValue<'id> {}
impl<'id> AstNode<'id> for JsxAttributeValue<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxAttributeValue<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxAttributeValue<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxAttributeValue<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn StringLiteral(node: StringLiteral<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxExpressionContainer(node: JsxExpressionContainer<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxExpression<'id> {}
impl<'id> AstNode<'id> for JsxExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxExpression<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn Expr(node: Expr<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxEmptyExpression(node: JsxEmptyExpression<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxChild<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxChild<'id> {}
impl<'id> AstNode<'id> for JsxChild<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxChild<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxChild<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxChild<'id> {
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxText(node: JsxText<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxExpressionContainer(node: JsxExpressionContainer<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxSpreadChild(node: JsxSpreadChild<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxElement(node: JsxElement<'id>) -> Self {
        Self(node.node_id())
    }
    #[allow(non_snake_case)]
    #[inline]
    pub fn JsxFragment(node: JsxFragment<'id>) -> Self {
        Self(node.node_id())
    }
}
impl<'id> JsxElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxElement<'id> {}
impl<'id> AstNode<'id> for JsxElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxOpeningElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxOpeningElement<'id> {}
impl<'id> AstNode<'id> for JsxOpeningElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxOpeningElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxOpeningElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxClosingElement<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxClosingElement<'id> {}
impl<'id> AstNode<'id> for JsxClosingElement<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxClosingElement<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxClosingElement<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxFragment<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxFragment<'id> {}
impl<'id> AstNode<'id> for JsxFragment<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxFragment<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxFragment<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxOpeningFragment<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxOpeningFragment<'id> {}
impl<'id> AstNode<'id> for JsxOpeningFragment<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxOpeningFragment<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxOpeningFragment<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxClosingFragment<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxClosingFragment<'id> {}
impl<'id> AstNode<'id> for JsxClosingFragment<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxClosingFragment<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxClosingFragment<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxIdentifier<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxIdentifier<'id> {}
impl<'id> AstNode<'id> for JsxIdentifier<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxIdentifier<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxIdentifier<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxNamespacedName<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxNamespacedName<'id> {}
impl<'id> AstNode<'id> for JsxNamespacedName<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxNamespacedName<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxNamespacedName<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxMemberExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxMemberExpression<'id> {}
impl<'id> AstNode<'id> for JsxMemberExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxMemberExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxMemberExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxAttribute<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxAttribute<'id> {}
impl<'id> AstNode<'id> for JsxAttribute<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxAttribute<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxAttribute<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxSpreadAttribute<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxSpreadAttribute<'id> {}
impl<'id> AstNode<'id> for JsxSpreadAttribute<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxSpreadAttribute<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxSpreadAttribute<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxExpressionContainer<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxExpressionContainer<'id> {}
impl<'id> AstNode<'id> for JsxExpressionContainer<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxExpressionContainer<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxExpressionContainer<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxEmptyExpression<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxEmptyExpression<'id> {}
impl<'id> AstNode<'id> for JsxEmptyExpression<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxEmptyExpression<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxEmptyExpression<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxText<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxText<'id> {}
impl<'id> AstNode<'id> for JsxText<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxText<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxText<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'id> JsxSpreadChild<'id> {
    #[inline]
    pub fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> SealedAstNode for JsxSpreadChild<'id> {}
impl<'id> AstNode<'id> for JsxSpreadChild<'id> {
    #[inline]
    unsafe fn from_node_id_unchecked(node_id: NodeId<'id>) -> Self {
        Self(node_id)
    }
    #[inline]
    fn node_id(self) -> NodeId<'id> {
        self.0
    }
}
impl<'id> GetSpan<'id> for JsxSpreadChild<'id> {
    #[inline]
    fn span(self, ast: &Ast<'_, 'id>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0.raw()) }
    }
}
impl<'id> SetSpan<'id> for JsxSpreadChild<'id> {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_, 'id>, span: crate::Span) {
        unsafe {
            *ast.nodes.span_mut_unchecked(self.0.raw()) = span;
        }
    }
}
impl<'a, 'id> Ast<'a, 'id> {
    /// Returns the typed wrapper for the concrete node stored at `node_id`.
    #[inline]
    pub fn kind_data(&self, node_id: NodeId<'id>) -> NodeKindData<'id> {
        let kind = self.node_kind(node_id);
        unsafe { std::mem::transmute(AstEnumDataRepr { kind, node_id }) }
    }
    #[inline]
    pub fn stmt_data(&self, node: Stmt<'id>) -> StmtData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::BlockStatement => StmtData::BlockStatement(BlockStatement(node.0)),
            NodeKind::BreakStatement => StmtData::BreakStatement(BreakStatement(node.0)),
            NodeKind::ContinueStatement => StmtData::ContinueStatement(ContinueStatement(node.0)),
            NodeKind::DebuggerStatement => StmtData::DebuggerStatement(DebuggerStatement(node.0)),
            NodeKind::DoWhileStatement => StmtData::DoWhileStatement(DoWhileStatement(node.0)),
            NodeKind::EmptyStatement => StmtData::EmptyStatement(EmptyStatement(node.0)),
            NodeKind::ExpressionStatement => {
                StmtData::ExpressionStatement(ExpressionStatement(node.0))
            }
            NodeKind::ForInStatement => StmtData::ForInStatement(ForInStatement(node.0)),
            NodeKind::ForOfStatement => StmtData::ForOfStatement(ForOfStatement(node.0)),
            NodeKind::ForStatement => StmtData::ForStatement(ForStatement(node.0)),
            NodeKind::IfStatement => StmtData::IfStatement(IfStatement(node.0)),
            NodeKind::LabeledStatement => StmtData::LabeledStatement(LabeledStatement(node.0)),
            NodeKind::ReturnStatement => StmtData::ReturnStatement(ReturnStatement(node.0)),
            NodeKind::SwitchStatement => StmtData::SwitchStatement(SwitchStatement(node.0)),
            NodeKind::ThrowStatement => StmtData::ThrowStatement(ThrowStatement(node.0)),
            NodeKind::TryStatement => StmtData::TryStatement(TryStatement(node.0)),
            NodeKind::WhileStatement => StmtData::WhileStatement(WhileStatement(node.0)),
            NodeKind::WithStatement => StmtData::WithStatement(WithStatement(node.0)),
            NodeKind::VariableDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::Function => StmtData::Declaration(Decl(node.0)),
            NodeKind::Class => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsFunction => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsTypeAliasDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsInterfaceDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsEnumDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsModuleDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsGlobalDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::TsImportEqualsDeclaration => StmtData::Declaration(Decl(node.0)),
            NodeKind::ImportDeclaration => StmtData::ImportDeclaration(ImportDeclaration(node.0)),
            NodeKind::ExportNamedDeclaration => {
                StmtData::ExportNamedDeclaration(ExportNamedDeclaration(node.0))
            }
            NodeKind::ExportDefaultDeclaration => {
                StmtData::ExportDefaultDeclaration(ExportDefaultDeclaration(node.0))
            }
            NodeKind::ExportAllDeclaration => {
                StmtData::ExportAllDeclaration(ExportAllDeclaration(node.0))
            }
            NodeKind::TsExportAssignment => {
                StmtData::TsExportAssignment(TsExportAssignment(node.0))
            }
            NodeKind::TsNamespaceExportDeclaration => {
                StmtData::TsNamespaceExportDeclaration(TsNamespaceExportDeclaration(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn decl_data(&self, node: Decl<'id>) -> DeclData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_decl());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn expr_data(&self, node: Expr<'id>) -> ExprData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_expr());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn binding_pattern_data(&self, node: BindingPattern<'id>) -> BindingPatternData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::BindingIdentifier => {
                BindingPatternData::BindingIdentifier(BindingIdentifier(node.0))
            }
            NodeKind::IdentifierReference => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                BindingPatternData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::AssignmentPattern => {
                BindingPatternData::AssignmentPattern(AssignmentPattern(node.0))
            }
            NodeKind::BindingRestElement => {
                BindingPatternData::BindingRestElement(BindingRestElement(node.0))
            }
            NodeKind::ArrayPattern => BindingPatternData::ArrayPattern(ArrayPattern(node.0)),
            NodeKind::ObjectPattern => BindingPatternData::ObjectPattern(ObjectPattern(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn assignment_target_data(&self, node: AssignmentTarget<'id>) -> AssignmentTargetData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(SimpleAssignmentTarget(node.0))
            }
            NodeKind::ArrayPattern => {
                AssignmentTargetData::ArrayAssignmentTarget(ArrayPattern(node.0))
            }
            NodeKind::ObjectPattern => {
                AssignmentTargetData::ObjectAssignmentTarget(ObjectPattern(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn simple_assignment_target_data(
        &self,
        node: SimpleAssignmentTarget<'id>,
    ) -> SimpleAssignmentTargetData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_simple_assignment_target());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn property_key_data(&self, node: PropertyKey<'id>) -> PropertyKeyData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierName => PropertyKeyData::IdentifierName(IdentifierName(node.0)),
            NodeKind::IdentifierReference => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Super => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::CallExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::NewExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Function => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::Class => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::JsxElement => PropertyKeyData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => PropertyKeyData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_export_name_data(
        &self,
        node: ModuleExportName<'id>,
    ) -> ModuleExportNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_module_export_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn argument_data(&self, node: Argument<'id>) -> ArgumentData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Super => ArgumentData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ArgumentData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ArgumentData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Function => ArgumentData::Expr(Expr(node.0)),
            NodeKind::Class => ArgumentData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ArgumentData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ArgumentData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ArgumentData::Expr(Expr(node.0)),
            NodeKind::SpreadElement => ArgumentData::SpreadElement(SpreadElement(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn object_property_kind_data(
        &self,
        node: ObjectPropertyKind<'id>,
    ) -> ObjectPropertyKindData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_object_property_kind());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn for_statement_init_data(
        &self,
        node: ForStatementInit<'id>,
    ) -> ForStatementInitData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::VariableDeclaration => {
                ForStatementInitData::VariableDeclaration(VariableDeclaration(node.0))
            }
            NodeKind::IdentifierReference => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Super => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Function => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::Class => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ForStatementInitData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ForStatementInitData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn for_statement_left_data(
        &self,
        node: ForStatementLeft<'id>,
    ) -> ForStatementLeftData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::VariableDeclaration => {
                ForStatementLeftData::VariableDeclaration(VariableDeclaration(node.0))
            }
            NodeKind::IdentifierReference => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::MemberExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsAsExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsSatisfiesExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsTypeAssertion => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::TsNonNullExpression => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::ArrayPattern => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            NodeKind::ObjectPattern => {
                ForStatementLeftData::AssignmentTarget(AssignmentTarget(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn arrow_function_body_data(
        &self,
        node: ArrowFunctionBody<'id>,
    ) -> ArrowFunctionBodyData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::FunctionBody => ArrowFunctionBodyData::FunctionBody(FunctionBody(node.0)),
            NodeKind::IdentifierReference => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Super => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Function => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::Class => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::JsxElement => ArrowFunctionBodyData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ArrowFunctionBodyData::Expr(Expr(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn formal_parameter_item_data(
        &self,
        node: FormalParameterItem<'id>,
    ) -> FormalParameterItemData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_formal_parameter_item());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn formal_parameter_pattern_data(
        &self,
        node: FormalParameterPattern<'id>,
    ) -> FormalParameterPatternData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_formal_parameter_pattern());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn class_element_data(&self, node: ClassElement<'id>) -> ClassElementData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_class_element());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn import_declaration_specifier_data(
        &self,
        node: ImportDeclarationSpecifier<'id>,
    ) -> ImportDeclarationSpecifierData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_import_declaration_specifier());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn export_default_declaration_kind_data(
        &self,
        node: ExportDefaultDeclarationKind<'id>,
    ) -> ExportDefaultDeclarationKindData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::Super => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::CallExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::NewExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::ArrowFunctionExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::ArrayExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::Function => {
                let function = Function(node.0);
                if function.function_type(self) == FunctionType::FunctionDeclaration {
                    ExportDefaultDeclarationKindData::Function(function)
                } else {
                    ExportDefaultDeclarationKindData::Expr(Expr(node.0))
                }
            }
            NodeKind::Class => {
                let class = Class(node.0);
                if class.class_type(self) == ClassType::ClassDeclaration {
                    ExportDefaultDeclarationKindData::Class(class)
                } else {
                    ExportDefaultDeclarationKindData::Expr(Expr(node.0))
                }
            }
            NodeKind::ImportExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => {
                ExportDefaultDeclarationKindData::Expr(Expr(node.0))
            }
            NodeKind::JsxElement => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => ExportDefaultDeclarationKindData::Expr(Expr(node.0)),
            NodeKind::TsFunction => {
                ExportDefaultDeclarationKindData::TsFunction(TsFunction(node.0))
            }
            NodeKind::TsInterfaceDeclaration => {
                ExportDefaultDeclarationKindData::TsInterfaceDeclaration(TsInterfaceDeclaration(
                    node.0,
                ))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_data(&self, node: TsType<'id>) -> TsTypeData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_type());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_name_data(&self, node: TsTypeName<'id>) -> TsTypeNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_type_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_qualified_name_left_data(
        &self,
        node: TsQualifiedNameLeft<'id>,
    ) -> TsQualifiedNameLeftData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_qualified_name_left());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_import_type_qualifier_data(
        &self,
        node: TsImportTypeQualifier<'id>,
    ) -> TsImportTypeQualifierData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_import_type_qualifier());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_query_expr_name_data(
        &self,
        node: TsTypeQueryExprName<'id>,
    ) -> TsTypeQueryExprNameData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => {
                TsTypeQueryExprNameData::TsTypeName(TsTypeName(node.0))
            }
            NodeKind::TsQualifiedName => TsTypeQueryExprNameData::TsTypeName(TsTypeName(node.0)),
            NodeKind::ThisExpression => {
                TsTypeQueryExprNameData::ThisExpression(ThisExpression(node.0))
            }
            NodeKind::TsImportType => TsTypeQueryExprNameData::TsImportType(TsImportType(node.0)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_literal_data(&self, node: TsLiteral<'id>) -> TsLiteralData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_literal());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_tuple_element_data(&self, node: TsTupleElement<'id>) -> TsTupleElementData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::TsAnyKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUnknownKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNeverKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsVoidKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNullKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUndefinedKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsStringKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNumberKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsBigIntKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsBooleanKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsSymbolKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsObjectKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIntrinsicKeyword => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsThisType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeReference => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeQuery => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsImportType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsLiteralType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTemplateLiteralType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsArrayType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIndexedAccessType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTupleType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsOptionalType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsRestType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocNullableType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocNonNullableType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsJsDocUnknownType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsUnionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsIntersectionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsConditionalType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsInferType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeOperator => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsParenthesizedType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsFunctionType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsConstructorType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypePredicate => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsTypeLiteral => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsMappedType => TsTupleElementData::TsType(TsType(node.0)),
            NodeKind::TsNamedTupleMember => {
                TsTupleElementData::TsNamedTupleMember(TsNamedTupleMember(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_element_data(&self, node: TsTypeElement<'id>) -> TsTypeElementData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_type_element());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_type_predicate_name_data(
        &self,
        node: TsTypePredicateName<'id>,
    ) -> TsTypePredicateNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_type_predicate_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_enum_member_name_data(
        &self,
        node: TsEnumMemberName<'id>,
    ) -> TsEnumMemberNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_enum_member_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_name_data(&self, node: TsModuleName<'id>) -> TsModuleNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_module_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_body_data(&self, node: TsModuleBody<'id>) -> TsModuleBodyData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_ts_module_body());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn ts_module_reference_data(
        &self,
        node: TsModuleReference<'id>,
    ) -> TsModuleReferenceData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => TsModuleReferenceData::TsTypeName(TsTypeName(node.0)),
            NodeKind::TsQualifiedName => TsModuleReferenceData::TsTypeName(TsTypeName(node.0)),
            NodeKind::TsExternalModuleReference => {
                TsModuleReferenceData::TsExternalModuleReference(TsExternalModuleReference(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_element_name_data(&self, node: JsxElementName<'id>) -> JsxElementNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_element_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_member_expression_object_data(
        &self,
        node: JsxMemberExpressionObject<'id>,
    ) -> JsxMemberExpressionObjectData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_member_expression_object());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_name_data(
        &self,
        node: JsxAttributeName<'id>,
    ) -> JsxAttributeNameData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_attribute_name());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_item_data(
        &self,
        node: JsxAttributeItem<'id>,
    ) -> JsxAttributeItemData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_attribute_item());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_attribute_value_data(
        &self,
        node: JsxAttributeValue<'id>,
    ) -> JsxAttributeValueData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_attribute_value());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
    #[inline]
    pub fn jsx_expression_data(&self, node: JsxExpression<'id>) -> JsxExpressionData<'id> {
        match unsafe { *self.nodes.kind_unchecked(node.0.raw()) } {
            NodeKind::IdentifierReference => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ThisExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Super => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::MetaProperty => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::StringLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NumericLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BigIntLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BooleanLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NullLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::RegExpLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TemplateLiteral => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::BinaryExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::LogicalExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::UnaryExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::UpdateExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::AssignmentExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ConditionalExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::SequenceExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::MemberExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::CallExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ChainExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::NewExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TaggedTemplateExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ArrowFunctionExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ArrayExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ObjectExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ParenthesizedExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Function => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::Class => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::ImportExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::AwaitExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::YieldExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::PrivateIdentifier => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsAsExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsSatisfiesExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsTypeAssertion => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsNonNullExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::TsInstantiationExpression => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxElement => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxFragment => JsxExpressionData::Expr(Expr(node.0)),
            NodeKind::JsxEmptyExpression => {
                JsxExpressionData::JsxEmptyExpression(JsxEmptyExpression(node.0))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_child_data(&self, node: JsxChild<'id>) -> JsxChildData<'id> {
        let kind = unsafe { *self.nodes.kind_unchecked(node.0.raw()) };
        debug_assert!(kind.is_jsx_child());
        unsafe {
            std::mem::transmute(AstEnumDataRepr {
                kind,
                node_id: node.0,
            })
        }
    }
}
