#![allow(unused)]
use crate::{Ast, GetSpan, SetSpan};
use crate::{ast::*, node_id::*};
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
                | Self::PrivateIdentifier
                | Self::StringLiteral
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
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
                | Self::TsThisParameter
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
            Self::IdentifierName
                | Self::StringLiteral
                | Self::TemplateLiteral
                | Self::IdentifierReference
                | Self::ThisExpression
                | Self::Super
                | Self::MetaProperty
                | Self::NumericLiteral
                | Self::BigIntLiteral
                | Self::BooleanLiteral
                | Self::NullLiteral
                | Self::RegExpLiteral
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
    #[inline]
    pub const fn uses_extra_data(self) -> bool {
        match self {
            Self::Program => true,
            Self::Super => false,
            Self::ThisExpression => false,
            Self::StringLiteral => false,
            Self::NumericLiteral => false,
            Self::BigIntLiteral => false,
            Self::BooleanLiteral => false,
            Self::NullLiteral => false,
            Self::RegExpLiteral => false,
            Self::TemplateLiteral => false,
            Self::TemplateElement => true,
            Self::IdentifierReference => false,
            Self::PrivateIdentifier => false,
            Self::BindingIdentifier => true,
            Self::IdentifierName => false,
            Self::LabelIdentifier => false,
            Self::AssignmentPattern => true,
            Self::BindingRestElement => true,
            Self::ArrayPattern => true,
            Self::ObjectPattern => true,
            Self::BindingProperty => false,
            Self::BinaryExpression => false,
            Self::LogicalExpression => false,
            Self::ConditionalExpression => false,
            Self::UnaryExpression => false,
            Self::UpdateExpression => false,
            Self::AssignmentExpression => false,
            Self::VariableDeclaration => false,
            Self::VariableDeclarator => false,
            Self::IfStatement => false,
            Self::SwitchStatement => false,
            Self::SwitchCase => false,
            Self::ForStatement => false,
            Self::ForInStatement => false,
            Self::ForOfStatement => false,
            Self::TryStatement => false,
            Self::CatchClause => false,
            Self::BreakStatement => false,
            Self::ContinueStatement => false,
            Self::LabeledStatement => false,
            Self::ExpressionStatement => false,
            Self::ReturnStatement => false,
            Self::ThrowStatement => false,
            Self::DebuggerStatement => false,
            Self::EmptyStatement => false,
            Self::Directive => false,
            Self::Decorator => false,
            Self::Class => false,
            Self::MethodDefinition => true,
            Self::PropertyDefinition => true,
            Self::StaticBlock => false,
            Self::Function => true,
            Self::FunctionBody => false,
            Self::BlockStatement => false,
            Self::WhileStatement => false,
            Self::DoWhileStatement => false,
            Self::WithStatement => false,
            Self::FormalParameters => false,
            Self::FormalParameter => false,
            Self::ArrayExpression => false,
            Self::ObjectExpression => false,
            Self::SpreadElement => false,
            Self::ObjectProperty => false,
            Self::ParenthesizedExpression => false,
            Self::ArrowFunctionExpression => true,
            Self::SequenceExpression => false,
            Self::MemberExpression => false,
            Self::CallExpression => true,
            Self::ChainExpression => false,
            Self::TaggedTemplateExpression => false,
            Self::NewExpression => false,
            Self::AwaitExpression => false,
            Self::YieldExpression => false,
            Self::MetaProperty => false,
            Self::ImportExpression => true,
            Self::ImportDeclaration => true,
            Self::ImportSpecifier => false,
            Self::ImportDefaultSpecifier => false,
            Self::ImportNamespaceSpecifier => false,
            Self::ImportAttribute => false,
            Self::ExportNamedDeclaration => true,
            Self::ExportDefaultDeclaration => false,
            Self::ExportAllDeclaration => true,
            Self::ExportSpecifier => false,
            Self::Hashbang => false,
            Self::TsFunction => true,
            Self::TsMethodDefinition => true,
            Self::TsTypeAnnotation => false,
            Self::TsAnyKeyword => false,
            Self::TsUnknownKeyword => false,
            Self::TsNeverKeyword => false,
            Self::TsVoidKeyword => false,
            Self::TsNullKeyword => false,
            Self::TsUndefinedKeyword => false,
            Self::TsStringKeyword => false,
            Self::TsNumberKeyword => false,
            Self::TsBigIntKeyword => false,
            Self::TsBooleanKeyword => false,
            Self::TsSymbolKeyword => false,
            Self::TsObjectKeyword => false,
            Self::TsIntrinsicKeyword => false,
            Self::TsThisType => false,
            Self::TsTypeReference => false,
            Self::TsQualifiedName => false,
            Self::TsTypeQuery => false,
            Self::TsImportType => false,
            Self::TsTypeParameter => false,
            Self::TsTypeParameterDeclaration => false,
            Self::TsTypeParameterInstantiation => false,
            Self::TsLiteralType => false,
            Self::TsTemplateLiteralType => false,
            Self::TsArrayType => false,
            Self::TsIndexedAccessType => false,
            Self::TsTupleType => false,
            Self::TsNamedTupleMember => false,
            Self::TsOptionalType => false,
            Self::TsRestType => false,
            Self::TsJsDocNullableType => false,
            Self::TsJsDocNonNullableType => false,
            Self::TsJsDocUnknownType => false,
            Self::TsUnionType => false,
            Self::TsIntersectionType => false,
            Self::TsConditionalType => false,
            Self::TsInferType => false,
            Self::TsTypeOperator => false,
            Self::TsParenthesizedType => false,
            Self::TsFunctionType => false,
            Self::TsConstructorType => false,
            Self::TsTypePredicate => false,
            Self::TsTypeLiteral => false,
            Self::TsMappedType => true,
            Self::TsPropertySignature => false,
            Self::TsMethodSignature => true,
            Self::TsCallSignatureDeclaration => false,
            Self::TsConstructSignatureDeclaration => false,
            Self::TsIndexSignature => false,
            Self::TsTypeAliasDeclaration => false,
            Self::TsInterfaceDeclaration => true,
            Self::TsInterfaceBody => false,
            Self::TsInterfaceHeritage => false,
            Self::TsClassImplements => false,
            Self::TsEnumDeclaration => false,
            Self::TsEnumBody => false,
            Self::TsEnumMember => false,
            Self::TsModuleDeclaration => false,
            Self::TsModuleBlock => false,
            Self::TsGlobalDeclaration => false,
            Self::TsParameterProperty => false,
            Self::TsThisParameter => false,
            Self::TsAsExpression => false,
            Self::TsSatisfiesExpression => false,
            Self::TsTypeAssertion => false,
            Self::TsNonNullExpression => false,
            Self::TsInstantiationExpression => false,
            Self::TsExportAssignment => false,
            Self::TsNamespaceExportDeclaration => false,
            Self::TsImportEqualsDeclaration => false,
            Self::TsExternalModuleReference => false,
            Self::JsxElement => false,
            Self::JsxOpeningElement => true,
            Self::JsxClosingElement => false,
            Self::JsxFragment => false,
            Self::JsxOpeningFragment => false,
            Self::JsxClosingFragment => false,
            Self::JsxIdentifier => false,
            Self::JsxNamespacedName => false,
            Self::JsxMemberExpression => false,
            Self::JsxAttribute => false,
            Self::JsxSpreadAttribute => false,
            Self::JsxExpressionContainer => false,
            Self::JsxEmptyExpression => false,
            Self::JsxText => false,
            Self::JsxSpreadChild => false,
        }
    }
    pub(crate) fn map_ast_ids(
        self,
        payload: &mut crate::NodePayload,
        extra_data: &mut [crate::ExtraData],
        mut map: impl FnMut(AstId) -> AstId,
    ) {
        match self {
            Self::Program => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::Super => {}
            Self::ThisExpression => {}
            Self::StringLiteral => {}
            Self::NumericLiteral => {}
            Self::BigIntLiteral => {}
            Self::BooleanLiteral => {}
            Self::NullLiteral => {}
            Self::RegExpLiteral => {}
            Self::TemplateLiteral => {}
            Self::TemplateElement => {}
            Self::IdentifierReference => {}
            Self::PrivateIdentifier => {}
            Self::BindingIdentifier => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
            }
            Self::IdentifierName => {}
            Self::LabelIdentifier => {}
            Self::AssignmentPattern => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::BindingRestElement => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ArrayPattern => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ObjectPattern => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::BindingProperty => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::BinaryExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::LogicalExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ConditionalExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::UnaryExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::UpdateExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::AssignmentExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::VariableDeclaration => {}
            Self::VariableDeclarator => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::IfStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::SwitchStatement => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::SwitchCase => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::ForStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::ForInStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::ForOfStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TryStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::CatchClause => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::BreakStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ContinueStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::LabeledStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ExpressionStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ReturnStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ThrowStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::DebuggerStatement => {}
            Self::EmptyStatement => {}
            Self::Directive => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::Decorator => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::Class => {}
            Self::MethodDefinition => {
                let raw = (payload.0 >> 56usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !309485009749287474686853120u128) | ((mapped as u128) << 56usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::PropertyDefinition => {
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 2usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 2usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 3usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 3usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::StaticBlock => {}
            Self::Function => {
                let raw = (payload.0 >> 24usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !72057594021150720u128) | ((mapped as u128) << 24usize);
                let raw = (payload.0 >> 56usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !309485009749287474686853120u128) | ((mapped as u128) << 56usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 2usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 2usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::FunctionBody => {}
            Self::BlockStatement => {}
            Self::WhileStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::DoWhileStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::WithStatement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::FormalParameters => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::FormalParameter => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ArrayExpression => {}
            Self::ObjectExpression => {}
            Self::SpreadElement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ObjectProperty => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ParenthesizedExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ArrowFunctionExpression => {
                let raw = (payload.0 >> 16usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !281474976645120u128) | ((mapped as u128) << 16usize);
                let raw = (payload.0 >> 48usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !1208925819333154197995520u128) | ((mapped as u128) << 48usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::SequenceExpression => {}
            Self::MemberExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::CallExpression => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ChainExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TaggedTemplateExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::NewExpression => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::AwaitExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::YieldExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::MetaProperty => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ImportExpression => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ImportDeclaration => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
            }
            Self::ImportSpecifier => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ImportDefaultSpecifier => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ImportNamespaceSpecifier => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ImportAttribute => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::ExportNamedDeclaration => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ExportDefaultDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::ExportAllDeclaration => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::ExportSpecifier => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::Hashbang => {}
            Self::TsFunction => {
                let raw = (payload.0 >> 24usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !72057594021150720u128) | ((mapped as u128) << 24usize);
                let raw = (payload.0 >> 56usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !309485009749287474686853120u128) | ((mapped as u128) << 56usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::TsMethodDefinition => {
                let raw = (payload.0 >> 56usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !309485009749287474686853120u128) | ((mapped as u128) << 56usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::TsTypeAnnotation => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsAnyKeyword => {}
            Self::TsUnknownKeyword => {}
            Self::TsNeverKeyword => {}
            Self::TsVoidKeyword => {}
            Self::TsNullKeyword => {}
            Self::TsUndefinedKeyword => {}
            Self::TsStringKeyword => {}
            Self::TsNumberKeyword => {}
            Self::TsBigIntKeyword => {}
            Self::TsBooleanKeyword => {}
            Self::TsSymbolKeyword => {}
            Self::TsObjectKeyword => {}
            Self::TsIntrinsicKeyword => {}
            Self::TsThisType => {}
            Self::TsTypeReference => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsQualifiedName => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsTypeQuery => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsImportType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::TsTypeParameter => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsTypeParameterDeclaration => {}
            Self::TsTypeParameterInstantiation => {}
            Self::TsLiteralType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsTemplateLiteralType => {}
            Self::TsArrayType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsIndexedAccessType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsTupleType => {}
            Self::TsNamedTupleMember => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsOptionalType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsRestType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsJsDocNullableType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsJsDocNonNullableType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsJsDocUnknownType => {}
            Self::TsUnionType => {}
            Self::TsIntersectionType => {}
            Self::TsConditionalType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::TsInferType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsTypeOperator => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsParenthesizedType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsFunctionType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsConstructorType => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsTypePredicate => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsTypeLiteral => {}
            Self::TsMappedType => {
                let raw = (payload.0 >> 16usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !281474976645120u128) | ((mapped as u128) << 16usize);
                let raw = (payload.0 >> 48usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !1208925819333154197995520u128) | ((mapped as u128) << 48usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::TsPropertySignature => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsMethodSignature => {
                let raw = (payload.0 >> 24usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !72057594021150720u128) | ((mapped as u128) << 24usize);
                let raw = (payload.0 >> 56usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !309485009749287474686853120u128) | ((mapped as u128) << 56usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::TsCallSignatureDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsConstructSignatureDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsIndexSignature => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsTypeAliasDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsInterfaceDeclaration => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
                let base = (payload.0 >> 96) as u32 as usize;
                let word = unsafe { extra_data[base + 1usize].other };
                let mapped = map(AstId::from_raw(word as u32)).raw();
                extra_data[base + 1usize] = crate::ExtraData {
                    other: (word & !u32::MAX as u64) | mapped as u64,
                };
            }
            Self::TsInterfaceBody => {}
            Self::TsInterfaceHeritage => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsClassImplements => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsEnumDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsEnumBody => {}
            Self::TsEnumMember => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsModuleDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsModuleBlock => {}
            Self::TsGlobalDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsParameterProperty => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
            }
            Self::TsThisParameter => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsAsExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsSatisfiesExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsTypeAssertion => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsNonNullExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsInstantiationExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsExportAssignment => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsNamespaceExportDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::TsImportEqualsDeclaration => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::TsExternalModuleReference => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::JsxElement => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::JsxOpeningElement => {
                let raw = (payload.0 >> 8usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !1099511627520u128) | ((mapped as u128) << 8usize);
                let raw = (payload.0 >> 40usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 =
                    (payload.0 & !4722366481770133585920u128) | ((mapped as u128) << 40usize);
            }
            Self::JsxClosingElement => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::JsxFragment => {
                let raw = (payload.0 >> 64usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !79228162495817593519834398720u128)
                    | ((mapped as u128) << 64usize);
                let raw = (payload.0 >> 96usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !340282366841710300949110269838224261120u128)
                    | ((mapped as u128) << 96usize);
            }
            Self::JsxOpeningFragment => {}
            Self::JsxClosingFragment => {}
            Self::JsxIdentifier => {}
            Self::JsxNamespacedName => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::JsxMemberExpression => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::JsxAttribute => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
                let raw = (payload.0 >> 32usize) as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !18446744069414584320u128) | ((mapped as u128) << 32usize);
            }
            Self::JsxSpreadAttribute => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::JsxExpressionContainer => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
            Self::JsxEmptyExpression => {}
            Self::JsxText => {}
            Self::JsxSpreadChild => {
                let raw = payload.0 as u32;
                let mapped = map(AstId::from_raw(raw)).raw();
                payload.0 = (payload.0 & !4294967295u128) | mapped as u128;
            }
        }
    }
}
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKindData {
    Program(Program) = NodeKind::Program as u8,
    Super(Super) = NodeKind::Super as u8,
    ThisExpression(ThisExpression) = NodeKind::ThisExpression as u8,
    StringLiteral(StringLiteral) = NodeKind::StringLiteral as u8,
    NumericLiteral(NumericLiteral) = NodeKind::NumericLiteral as u8,
    BigIntLiteral(BigIntLiteral) = NodeKind::BigIntLiteral as u8,
    BooleanLiteral(BooleanLiteral) = NodeKind::BooleanLiteral as u8,
    NullLiteral(NullLiteral) = NodeKind::NullLiteral as u8,
    RegExpLiteral(RegExpLiteral) = NodeKind::RegExpLiteral as u8,
    TemplateLiteral(TemplateLiteral) = NodeKind::TemplateLiteral as u8,
    TemplateElement(TemplateElement) = NodeKind::TemplateElement as u8,
    IdentifierReference(IdentifierReference) = NodeKind::IdentifierReference as u8,
    PrivateIdentifier(PrivateIdentifier) = NodeKind::PrivateIdentifier as u8,
    BindingIdentifier(BindingIdentifier) = NodeKind::BindingIdentifier as u8,
    IdentifierName(IdentifierName) = NodeKind::IdentifierName as u8,
    LabelIdentifier(LabelIdentifier) = NodeKind::LabelIdentifier as u8,
    AssignmentPattern(AssignmentPattern) = NodeKind::AssignmentPattern as u8,
    BindingRestElement(BindingRestElement) = NodeKind::BindingRestElement as u8,
    ArrayPattern(ArrayPattern) = NodeKind::ArrayPattern as u8,
    ObjectPattern(ObjectPattern) = NodeKind::ObjectPattern as u8,
    BindingProperty(BindingProperty) = NodeKind::BindingProperty as u8,
    BinaryExpression(BinaryExpression) = NodeKind::BinaryExpression as u8,
    LogicalExpression(LogicalExpression) = NodeKind::LogicalExpression as u8,
    ConditionalExpression(ConditionalExpression) = NodeKind::ConditionalExpression as u8,
    UnaryExpression(UnaryExpression) = NodeKind::UnaryExpression as u8,
    UpdateExpression(UpdateExpression) = NodeKind::UpdateExpression as u8,
    AssignmentExpression(AssignmentExpression) = NodeKind::AssignmentExpression as u8,
    VariableDeclaration(VariableDeclaration) = NodeKind::VariableDeclaration as u8,
    VariableDeclarator(VariableDeclarator) = NodeKind::VariableDeclarator as u8,
    IfStatement(IfStatement) = NodeKind::IfStatement as u8,
    SwitchStatement(SwitchStatement) = NodeKind::SwitchStatement as u8,
    SwitchCase(SwitchCase) = NodeKind::SwitchCase as u8,
    ForStatement(ForStatement) = NodeKind::ForStatement as u8,
    ForInStatement(ForInStatement) = NodeKind::ForInStatement as u8,
    ForOfStatement(ForOfStatement) = NodeKind::ForOfStatement as u8,
    TryStatement(TryStatement) = NodeKind::TryStatement as u8,
    CatchClause(CatchClause) = NodeKind::CatchClause as u8,
    BreakStatement(BreakStatement) = NodeKind::BreakStatement as u8,
    ContinueStatement(ContinueStatement) = NodeKind::ContinueStatement as u8,
    LabeledStatement(LabeledStatement) = NodeKind::LabeledStatement as u8,
    ExpressionStatement(ExpressionStatement) = NodeKind::ExpressionStatement as u8,
    ReturnStatement(ReturnStatement) = NodeKind::ReturnStatement as u8,
    ThrowStatement(ThrowStatement) = NodeKind::ThrowStatement as u8,
    DebuggerStatement(DebuggerStatement) = NodeKind::DebuggerStatement as u8,
    EmptyStatement(EmptyStatement) = NodeKind::EmptyStatement as u8,
    Directive(Directive) = NodeKind::Directive as u8,
    Decorator(Decorator) = NodeKind::Decorator as u8,
    Class(ClassId) = NodeKind::Class as u8,
    MethodDefinition(MethodDefinition) = NodeKind::MethodDefinition as u8,
    PropertyDefinition(PropertyDefinition) = NodeKind::PropertyDefinition as u8,
    StaticBlock(StaticBlock) = NodeKind::StaticBlock as u8,
    Function(Function) = NodeKind::Function as u8,
    FunctionBody(FunctionBody) = NodeKind::FunctionBody as u8,
    BlockStatement(BlockStatement) = NodeKind::BlockStatement as u8,
    WhileStatement(WhileStatement) = NodeKind::WhileStatement as u8,
    DoWhileStatement(DoWhileStatement) = NodeKind::DoWhileStatement as u8,
    WithStatement(WithStatement) = NodeKind::WithStatement as u8,
    FormalParameters(FormalParameters) = NodeKind::FormalParameters as u8,
    FormalParameter(FormalParameter) = NodeKind::FormalParameter as u8,
    ArrayExpression(ArrayExpression) = NodeKind::ArrayExpression as u8,
    ObjectExpression(ObjectExpression) = NodeKind::ObjectExpression as u8,
    SpreadElement(SpreadElement) = NodeKind::SpreadElement as u8,
    ObjectProperty(ObjectProperty) = NodeKind::ObjectProperty as u8,
    ParenthesizedExpression(ParenthesizedExpression) = NodeKind::ParenthesizedExpression as u8,
    ArrowFunctionExpression(ArrowFunctionExpression) = NodeKind::ArrowFunctionExpression as u8,
    SequenceExpression(SequenceExpression) = NodeKind::SequenceExpression as u8,
    MemberExpression(MemberExpression) = NodeKind::MemberExpression as u8,
    CallExpression(CallExpression) = NodeKind::CallExpression as u8,
    ChainExpression(ChainExpression) = NodeKind::ChainExpression as u8,
    TaggedTemplateExpression(TaggedTemplateExpression) = NodeKind::TaggedTemplateExpression as u8,
    NewExpression(NewExpression) = NodeKind::NewExpression as u8,
    AwaitExpression(AwaitExpression) = NodeKind::AwaitExpression as u8,
    YieldExpression(YieldExpression) = NodeKind::YieldExpression as u8,
    MetaProperty(MetaProperty) = NodeKind::MetaProperty as u8,
    ImportExpression(ImportExpression) = NodeKind::ImportExpression as u8,
    ImportDeclaration(ImportDeclaration) = NodeKind::ImportDeclaration as u8,
    ImportSpecifier(ImportSpecifier) = NodeKind::ImportSpecifier as u8,
    ImportDefaultSpecifier(ImportDefaultSpecifier) = NodeKind::ImportDefaultSpecifier as u8,
    ImportNamespaceSpecifier(ImportNamespaceSpecifier) = NodeKind::ImportNamespaceSpecifier as u8,
    ImportAttribute(ImportAttribute) = NodeKind::ImportAttribute as u8,
    ExportNamedDeclaration(ExportNamedDeclaration) = NodeKind::ExportNamedDeclaration as u8,
    ExportDefaultDeclaration(ExportDefaultDeclaration) = NodeKind::ExportDefaultDeclaration as u8,
    ExportAllDeclaration(ExportAllDeclaration) = NodeKind::ExportAllDeclaration as u8,
    ExportSpecifier(ExportSpecifier) = NodeKind::ExportSpecifier as u8,
    Hashbang(Hashbang) = NodeKind::Hashbang as u8,
    TsFunction(TsFunction) = NodeKind::TsFunction as u8,
    TsMethodDefinition(TsMethodDefinition) = NodeKind::TsMethodDefinition as u8,
    TsTypeAnnotation(TsTypeAnnotation) = NodeKind::TsTypeAnnotation as u8,
    TsAnyKeyword(TsAnyKeyword) = NodeKind::TsAnyKeyword as u8,
    TsUnknownKeyword(TsUnknownKeyword) = NodeKind::TsUnknownKeyword as u8,
    TsNeverKeyword(TsNeverKeyword) = NodeKind::TsNeverKeyword as u8,
    TsVoidKeyword(TsVoidKeyword) = NodeKind::TsVoidKeyword as u8,
    TsNullKeyword(TsNullKeyword) = NodeKind::TsNullKeyword as u8,
    TsUndefinedKeyword(TsUndefinedKeyword) = NodeKind::TsUndefinedKeyword as u8,
    TsStringKeyword(TsStringKeyword) = NodeKind::TsStringKeyword as u8,
    TsNumberKeyword(TsNumberKeyword) = NodeKind::TsNumberKeyword as u8,
    TsBigIntKeyword(TsBigIntKeyword) = NodeKind::TsBigIntKeyword as u8,
    TsBooleanKeyword(TsBooleanKeyword) = NodeKind::TsBooleanKeyword as u8,
    TsSymbolKeyword(TsSymbolKeyword) = NodeKind::TsSymbolKeyword as u8,
    TsObjectKeyword(TsObjectKeyword) = NodeKind::TsObjectKeyword as u8,
    TsIntrinsicKeyword(TsIntrinsicKeyword) = NodeKind::TsIntrinsicKeyword as u8,
    TsThisType(TsThisType) = NodeKind::TsThisType as u8,
    TsTypeReference(TsTypeReference) = NodeKind::TsTypeReference as u8,
    TsQualifiedName(TsQualifiedName) = NodeKind::TsQualifiedName as u8,
    TsTypeQuery(TsTypeQuery) = NodeKind::TsTypeQuery as u8,
    TsImportType(TsImportType) = NodeKind::TsImportType as u8,
    TsTypeParameter(TsTypeParameter) = NodeKind::TsTypeParameter as u8,
    TsTypeParameterDeclaration(TsTypeParameterDeclaration) =
        NodeKind::TsTypeParameterDeclaration as u8,
    TsTypeParameterInstantiation(TsTypeParameterInstantiation) =
        NodeKind::TsTypeParameterInstantiation as u8,
    TsLiteralType(TsLiteralType) = NodeKind::TsLiteralType as u8,
    TsTemplateLiteralType(TsTemplateLiteralType) = NodeKind::TsTemplateLiteralType as u8,
    TsArrayType(TsArrayType) = NodeKind::TsArrayType as u8,
    TsIndexedAccessType(TsIndexedAccessType) = NodeKind::TsIndexedAccessType as u8,
    TsTupleType(TsTupleType) = NodeKind::TsTupleType as u8,
    TsNamedTupleMember(TsNamedTupleMember) = NodeKind::TsNamedTupleMember as u8,
    TsOptionalType(TsOptionalType) = NodeKind::TsOptionalType as u8,
    TsRestType(TsRestType) = NodeKind::TsRestType as u8,
    TsJsDocNullableType(TsJsDocNullableType) = NodeKind::TsJsDocNullableType as u8,
    TsJsDocNonNullableType(TsJsDocNonNullableType) = NodeKind::TsJsDocNonNullableType as u8,
    TsJsDocUnknownType(TsJsDocUnknownType) = NodeKind::TsJsDocUnknownType as u8,
    TsUnionType(TsUnionType) = NodeKind::TsUnionType as u8,
    TsIntersectionType(TsIntersectionType) = NodeKind::TsIntersectionType as u8,
    TsConditionalType(TsConditionalType) = NodeKind::TsConditionalType as u8,
    TsInferType(TsInferType) = NodeKind::TsInferType as u8,
    TsTypeOperator(TsTypeOperator) = NodeKind::TsTypeOperator as u8,
    TsParenthesizedType(TsParenthesizedType) = NodeKind::TsParenthesizedType as u8,
    TsFunctionType(TsFunctionType) = NodeKind::TsFunctionType as u8,
    TsConstructorType(TsConstructorType) = NodeKind::TsConstructorType as u8,
    TsTypePredicate(TsTypePredicate) = NodeKind::TsTypePredicate as u8,
    TsTypeLiteral(TsTypeLiteral) = NodeKind::TsTypeLiteral as u8,
    TsMappedType(TsMappedType) = NodeKind::TsMappedType as u8,
    TsPropertySignature(TsPropertySignature) = NodeKind::TsPropertySignature as u8,
    TsMethodSignature(TsMethodSignature) = NodeKind::TsMethodSignature as u8,
    TsCallSignatureDeclaration(TsCallSignatureDeclaration) =
        NodeKind::TsCallSignatureDeclaration as u8,
    TsConstructSignatureDeclaration(TsConstructSignatureDeclaration) =
        NodeKind::TsConstructSignatureDeclaration as u8,
    TsIndexSignature(TsIndexSignature) = NodeKind::TsIndexSignature as u8,
    TsTypeAliasDeclaration(TsTypeAliasDeclaration) = NodeKind::TsTypeAliasDeclaration as u8,
    TsInterfaceDeclaration(TsInterfaceDeclaration) = NodeKind::TsInterfaceDeclaration as u8,
    TsInterfaceBody(TsInterfaceBody) = NodeKind::TsInterfaceBody as u8,
    TsInterfaceHeritage(TsInterfaceHeritage) = NodeKind::TsInterfaceHeritage as u8,
    TsClassImplements(TsClassImplements) = NodeKind::TsClassImplements as u8,
    TsEnumDeclaration(TsEnumDeclaration) = NodeKind::TsEnumDeclaration as u8,
    TsEnumBody(TsEnumBody) = NodeKind::TsEnumBody as u8,
    TsEnumMember(TsEnumMember) = NodeKind::TsEnumMember as u8,
    TsModuleDeclaration(TsModuleDeclaration) = NodeKind::TsModuleDeclaration as u8,
    TsModuleBlock(TsModuleBlock) = NodeKind::TsModuleBlock as u8,
    TsGlobalDeclaration(TsGlobalDeclaration) = NodeKind::TsGlobalDeclaration as u8,
    TsParameterProperty(TsParameterProperty) = NodeKind::TsParameterProperty as u8,
    TsThisParameter(TsThisParameter) = NodeKind::TsThisParameter as u8,
    TsAsExpression(TsAsExpression) = NodeKind::TsAsExpression as u8,
    TsSatisfiesExpression(TsSatisfiesExpression) = NodeKind::TsSatisfiesExpression as u8,
    TsTypeAssertion(TsTypeAssertion) = NodeKind::TsTypeAssertion as u8,
    TsNonNullExpression(TsNonNullExpression) = NodeKind::TsNonNullExpression as u8,
    TsInstantiationExpression(TsInstantiationExpression) =
        NodeKind::TsInstantiationExpression as u8,
    TsExportAssignment(TsExportAssignment) = NodeKind::TsExportAssignment as u8,
    TsNamespaceExportDeclaration(TsNamespaceExportDeclaration) =
        NodeKind::TsNamespaceExportDeclaration as u8,
    TsImportEqualsDeclaration(TsImportEqualsDeclaration) =
        NodeKind::TsImportEqualsDeclaration as u8,
    TsExternalModuleReference(TsExternalModuleReference) =
        NodeKind::TsExternalModuleReference as u8,
    JsxElement(JsxElement) = NodeKind::JsxElement as u8,
    JsxOpeningElement(JsxOpeningElement) = NodeKind::JsxOpeningElement as u8,
    JsxClosingElement(JsxClosingElement) = NodeKind::JsxClosingElement as u8,
    JsxFragment(JsxFragment) = NodeKind::JsxFragment as u8,
    JsxOpeningFragment(JsxOpeningFragment) = NodeKind::JsxOpeningFragment as u8,
    JsxClosingFragment(JsxClosingFragment) = NodeKind::JsxClosingFragment as u8,
    JsxIdentifier(JsxIdentifier) = NodeKind::JsxIdentifier as u8,
    JsxNamespacedName(JsxNamespacedName) = NodeKind::JsxNamespacedName as u8,
    JsxMemberExpression(JsxMemberExpression) = NodeKind::JsxMemberExpression as u8,
    JsxAttribute(JsxAttribute) = NodeKind::JsxAttribute as u8,
    JsxSpreadAttribute(JsxSpreadAttribute) = NodeKind::JsxSpreadAttribute as u8,
    JsxExpressionContainer(JsxExpressionContainer) = NodeKind::JsxExpressionContainer as u8,
    JsxEmptyExpression(JsxEmptyExpression) = NodeKind::JsxEmptyExpression as u8,
    JsxText(JsxText) = NodeKind::JsxText as u8,
    JsxSpreadChild(JsxSpreadChild) = NodeKind::JsxSpreadChild as u8,
}
impl Program {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Program {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for Program {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Program {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Program {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl Stmt {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Stmt {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for Stmt {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Stmt {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for Stmt {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl Decl {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Decl {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for Decl {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Decl {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for Decl {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl Expr {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Expr {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for Expr {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Expr {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for Expr {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl BindingPattern {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BindingPattern {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for BindingPattern {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BindingPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for BindingPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl AssignmentTarget {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for AssignmentTarget {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for AssignmentTarget {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for AssignmentTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for AssignmentTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl SimpleAssignmentTarget {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for SimpleAssignmentTarget {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for SimpleAssignmentTarget {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for SimpleAssignmentTarget {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for SimpleAssignmentTarget {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl PropertyKey {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for PropertyKey {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for PropertyKey {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for PropertyKey {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for PropertyKey {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ModuleExportName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ModuleExportName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ModuleExportName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ModuleExportName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ModuleExportName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl Argument {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Argument {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for Argument {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Argument {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for Argument {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ObjectPropertyKind {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ObjectPropertyKind {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ObjectPropertyKind {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ObjectPropertyKind {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ObjectPropertyKind {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ForStatementInit {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ForStatementInit {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ForStatementInit {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ForStatementInit {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ForStatementInit {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ForStatementLeft {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ForStatementLeft {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ForStatementLeft {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ForStatementLeft {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ForStatementLeft {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ArrowFunctionBody {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ArrowFunctionBody {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ArrowFunctionBody {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ArrowFunctionBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ArrowFunctionBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl FormalParameterItem {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for FormalParameterItem {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for FormalParameterItem {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for FormalParameterItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for FormalParameterItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl FormalParameterPattern {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for FormalParameterPattern {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for FormalParameterPattern {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for FormalParameterPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for FormalParameterPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ClassElement {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ClassElement {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ClassElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ClassElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ClassElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ImportDeclarationSpecifier {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportDeclarationSpecifier {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ImportDeclarationSpecifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportDeclarationSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ImportDeclarationSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl ExportDefaultDeclarationKind {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExportDefaultDeclarationKind {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for ExportDefaultDeclarationKind {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExportDefaultDeclarationKind {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ExportDefaultDeclarationKind {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl Super {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Super {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for Super {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Super {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Super {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ThisExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ThisExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ThisExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ThisExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThisExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl StringLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for StringLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for StringLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for StringLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for StringLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl NumericLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for NumericLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for NumericLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for NumericLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NumericLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BigIntLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BigIntLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BigIntLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BigIntLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BigIntLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BooleanLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BooleanLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BooleanLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BooleanLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BooleanLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl NullLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for NullLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for NullLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for NullLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NullLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl RegExpLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for RegExpLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for RegExpLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for RegExpLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for RegExpLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TemplateLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TemplateLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TemplateLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TemplateLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TemplateLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TemplateElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TemplateElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TemplateElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TemplateElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TemplateElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl IdentifierReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for IdentifierReference {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for IdentifierReference {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for IdentifierReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IdentifierReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl PrivateIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for PrivateIdentifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for PrivateIdentifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for PrivateIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PrivateIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BindingIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BindingIdentifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BindingIdentifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BindingIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl IdentifierName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for IdentifierName {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for IdentifierName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for IdentifierName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IdentifierName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl LabelIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for LabelIdentifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for LabelIdentifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for LabelIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LabelIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl AssignmentPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for AssignmentPattern {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for AssignmentPattern {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for AssignmentPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignmentPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BindingRestElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BindingRestElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BindingRestElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BindingRestElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingRestElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ArrayPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ArrayPattern {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ArrayPattern {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ArrayPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ObjectPattern {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ObjectPattern {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ObjectPattern {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ObjectPattern {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectPattern {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BindingProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BindingProperty {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BindingProperty {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BindingProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BindingProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BinaryExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BinaryExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BinaryExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BinaryExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BinaryExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl LogicalExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for LogicalExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for LogicalExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for LogicalExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LogicalExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ConditionalExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ConditionalExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ConditionalExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ConditionalExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ConditionalExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl UnaryExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for UnaryExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for UnaryExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for UnaryExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UnaryExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl UpdateExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for UpdateExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for UpdateExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for UpdateExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for UpdateExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl AssignmentExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for AssignmentExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for AssignmentExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for AssignmentExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AssignmentExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl VariableDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for VariableDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for VariableDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for VariableDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VariableDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl VariableDeclarator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for VariableDeclarator {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for VariableDeclarator {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for VariableDeclarator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for VariableDeclarator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl IfStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for IfStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for IfStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for IfStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for IfStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl SwitchStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for SwitchStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for SwitchStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for SwitchStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl SwitchCase {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for SwitchCase {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for SwitchCase {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for SwitchCase {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SwitchCase {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ForStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ForStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ForStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ForStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ForInStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ForInStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ForInStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ForInStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForInStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ForOfStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ForOfStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ForOfStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ForOfStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ForOfStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TryStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TryStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TryStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TryStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TryStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl CatchClause {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for CatchClause {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for CatchClause {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for CatchClause {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CatchClause {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BreakStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BreakStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BreakStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BreakStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BreakStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ContinueStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ContinueStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ContinueStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ContinueStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ContinueStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl LabeledStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for LabeledStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for LabeledStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for LabeledStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for LabeledStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ExpressionStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExpressionStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ExpressionStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExpressionStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExpressionStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ReturnStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ReturnStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ReturnStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ReturnStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ReturnStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ThrowStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ThrowStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ThrowStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ThrowStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ThrowStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl DebuggerStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for DebuggerStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for DebuggerStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for DebuggerStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DebuggerStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl EmptyStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for EmptyStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for EmptyStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for EmptyStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for EmptyStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl Directive {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Directive {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for Directive {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Directive {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Directive {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl Decorator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Decorator {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for Decorator {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Decorator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Decorator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl AstNode for ClassId {
    type Id = ClassId;
    #[inline]
    fn id(self) -> ClassId {
        self
    }
}
impl CommonAstNode for ClassId {
    #[inline]
    fn ast_id(self) -> AstId {
        self.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <ClassId as TypedAstId>::from_ast_id_unchecked(ast_id) }
    }
}
impl GetSpan for ClassId {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for ClassId {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl MethodDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for MethodDefinition {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for MethodDefinition {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for MethodDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MethodDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl PropertyDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for PropertyDefinition {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for PropertyDefinition {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for PropertyDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for PropertyDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl StaticBlock {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for StaticBlock {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for StaticBlock {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for StaticBlock {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for StaticBlock {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl Function {
    #[inline]
    pub fn node_id(self) -> FunctionId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Function {
    type Id = FunctionId;
    #[inline]
    fn id(self) -> FunctionId {
        self.0
    }
}
impl CommonAstNode for Function {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<FunctionId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Function {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for Function {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl FunctionBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for FunctionBody {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for FunctionBody {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for FunctionBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FunctionBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl BlockStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for BlockStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for BlockStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for BlockStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for BlockStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl WhileStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for WhileStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for WhileStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for WhileStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WhileStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl DoWhileStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for DoWhileStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for DoWhileStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for DoWhileStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for DoWhileStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl WithStatement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for WithStatement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for WithStatement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for WithStatement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for WithStatement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl FormalParameters {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for FormalParameters {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for FormalParameters {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for FormalParameters {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameters {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl FormalParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for FormalParameter {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for FormalParameter {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for FormalParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for FormalParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ArrayExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ArrayExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ArrayExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ArrayExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrayExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ObjectExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ObjectExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ObjectExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ObjectExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl SpreadElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for SpreadElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for SpreadElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for SpreadElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SpreadElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ObjectProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ObjectProperty {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ObjectProperty {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ObjectProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ObjectProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ParenthesizedExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ParenthesizedExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ParenthesizedExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ParenthesizedExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ParenthesizedExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ArrowFunctionExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ArrowFunctionExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ArrowFunctionExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ArrowFunctionExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ArrowFunctionExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl SequenceExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for SequenceExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for SequenceExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for SequenceExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for SequenceExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl MemberExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for MemberExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for MemberExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for MemberExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MemberExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl CallExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for CallExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for CallExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for CallExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for CallExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ChainExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ChainExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ChainExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ChainExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ChainExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TaggedTemplateExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TaggedTemplateExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TaggedTemplateExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TaggedTemplateExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TaggedTemplateExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl NewExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for NewExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for NewExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for NewExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for NewExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl AwaitExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for AwaitExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for AwaitExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for AwaitExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for AwaitExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl YieldExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for YieldExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for YieldExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for YieldExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for YieldExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl MetaProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for MetaProperty {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for MetaProperty {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for MetaProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for MetaProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportSpecifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportSpecifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportDefaultSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportDefaultSpecifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportDefaultSpecifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportDefaultSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportDefaultSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportNamespaceSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportNamespaceSpecifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportNamespaceSpecifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportNamespaceSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportNamespaceSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ImportAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ImportAttribute {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ImportAttribute {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ImportAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ImportAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ExportNamedDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExportNamedDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ExportNamedDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExportNamedDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportNamedDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ExportDefaultDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExportDefaultDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ExportDefaultDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExportDefaultDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportDefaultDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ExportAllDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExportAllDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ExportAllDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExportAllDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportAllDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl ExportSpecifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for ExportSpecifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for ExportSpecifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for ExportSpecifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for ExportSpecifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl Hashbang {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for Hashbang {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for Hashbang {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for Hashbang {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for Hashbang {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsType {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsType {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsTypeName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsTypeName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsTypeName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsQualifiedNameLeft {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsQualifiedNameLeft {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsQualifiedNameLeft {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsQualifiedNameLeft {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsQualifiedNameLeft {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsImportTypeQualifier {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsImportTypeQualifier {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsImportTypeQualifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsImportTypeQualifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsImportTypeQualifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsTypeQueryExprName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeQueryExprName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsTypeQueryExprName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeQueryExprName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsTypeQueryExprName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsLiteral {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsLiteral {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsTupleElement {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTupleElement {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsTupleElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTupleElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsTupleElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsTypeElement {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeElement {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsTypeElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsTypeElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsTypePredicateName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypePredicateName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsTypePredicateName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypePredicateName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsTypePredicateName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsEnumMemberName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsEnumMemberName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsEnumMemberName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsEnumMemberName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsEnumMemberName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsModuleName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsModuleName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsModuleName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsModuleName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsModuleName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsModuleBody {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsModuleBody {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsModuleBody {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsModuleBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsModuleBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsModuleReference {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsModuleReference {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for TsModuleReference {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsModuleReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for TsModuleReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl TsFunction {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsFunction {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsFunction {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsFunction {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsFunction {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsMethodDefinition {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsMethodDefinition {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsMethodDefinition {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsMethodDefinition {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMethodDefinition {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeAnnotation {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeAnnotation {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeAnnotation {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeAnnotation {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAnnotation {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsAnyKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsAnyKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsAnyKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsAnyKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsAnyKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsUnknownKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsUnknownKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsUnknownKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsUnknownKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUnknownKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNeverKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNeverKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNeverKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNeverKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNeverKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsVoidKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsVoidKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsVoidKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsVoidKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsVoidKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNullKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNullKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNullKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNullKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNullKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsUndefinedKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsUndefinedKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsUndefinedKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsUndefinedKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUndefinedKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsStringKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsStringKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsStringKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsStringKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsStringKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNumberKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNumberKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNumberKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNumberKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNumberKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsBigIntKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsBigIntKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsBigIntKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsBigIntKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsBigIntKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsBooleanKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsBooleanKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsBooleanKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsBooleanKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsBooleanKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsSymbolKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsSymbolKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsSymbolKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsSymbolKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsSymbolKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsObjectKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsObjectKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsObjectKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsObjectKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsObjectKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsIntrinsicKeyword {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsIntrinsicKeyword {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsIntrinsicKeyword {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsIntrinsicKeyword {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIntrinsicKeyword {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsThisType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsThisType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsThisType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsThisType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsThisType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeReference {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeReference {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsQualifiedName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsQualifiedName {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsQualifiedName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsQualifiedName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsQualifiedName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeQuery {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeQuery {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeQuery {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeQuery {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeQuery {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsImportType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsImportType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsImportType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsImportType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsImportType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeParameter {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeParameter {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeParameterDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeParameterDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeParameterDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeParameterDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameterDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeParameterInstantiation {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeParameterInstantiation {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeParameterInstantiation {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeParameterInstantiation {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeParameterInstantiation {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsLiteralType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsLiteralType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsLiteralType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsLiteralType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsLiteralType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTemplateLiteralType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTemplateLiteralType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTemplateLiteralType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTemplateLiteralType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTemplateLiteralType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsArrayType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsArrayType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsArrayType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsArrayType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsArrayType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsIndexedAccessType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsIndexedAccessType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsIndexedAccessType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsIndexedAccessType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIndexedAccessType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTupleType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTupleType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTupleType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTupleType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTupleType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNamedTupleMember {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNamedTupleMember {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNamedTupleMember {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNamedTupleMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNamedTupleMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsOptionalType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsOptionalType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsOptionalType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsOptionalType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsOptionalType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsRestType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsRestType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsRestType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsRestType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsRestType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsJsDocNullableType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsJsDocNullableType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsJsDocNullableType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsJsDocNullableType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocNullableType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsJsDocNonNullableType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsJsDocNonNullableType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsJsDocNonNullableType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsJsDocNonNullableType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocNonNullableType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsJsDocUnknownType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsJsDocUnknownType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsJsDocUnknownType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsJsDocUnknownType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsJsDocUnknownType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsUnionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsUnionType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsUnionType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsUnionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsUnionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsIntersectionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsIntersectionType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsIntersectionType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsIntersectionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIntersectionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsConditionalType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsConditionalType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsConditionalType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsConditionalType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConditionalType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsInferType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsInferType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsInferType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsInferType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInferType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeOperator {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeOperator {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeOperator {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeOperator {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeOperator {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsParenthesizedType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsParenthesizedType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsParenthesizedType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsParenthesizedType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsParenthesizedType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsFunctionType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsFunctionType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsFunctionType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsFunctionType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsFunctionType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsConstructorType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsConstructorType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsConstructorType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsConstructorType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConstructorType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypePredicate {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypePredicate {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypePredicate {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypePredicate {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypePredicate {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeLiteral {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeLiteral {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeLiteral {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeLiteral {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeLiteral {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsMappedType {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsMappedType {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsMappedType {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsMappedType {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMappedType {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsPropertySignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsPropertySignature {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsPropertySignature {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsPropertySignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsPropertySignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsMethodSignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsMethodSignature {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsMethodSignature {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsMethodSignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsMethodSignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsCallSignatureDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsCallSignatureDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsCallSignatureDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsCallSignatureDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsCallSignatureDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsConstructSignatureDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsConstructSignatureDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsConstructSignatureDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsConstructSignatureDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsConstructSignatureDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsIndexSignature {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsIndexSignature {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsIndexSignature {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsIndexSignature {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsIndexSignature {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeAliasDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeAliasDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeAliasDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeAliasDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAliasDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsInterfaceDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsInterfaceDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsInterfaceDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsInterfaceDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsInterfaceBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsInterfaceBody {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsInterfaceBody {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsInterfaceBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsInterfaceHeritage {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsInterfaceHeritage {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsInterfaceHeritage {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsInterfaceHeritage {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInterfaceHeritage {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsClassImplements {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsClassImplements {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsClassImplements {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsClassImplements {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsClassImplements {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsEnumDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsEnumDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsEnumDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsEnumDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsEnumBody {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsEnumBody {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsEnumBody {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsEnumBody {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumBody {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsEnumMember {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsEnumMember {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsEnumMember {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsEnumMember {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsEnumMember {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsModuleDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsModuleDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsModuleDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsModuleDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsModuleBlock {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsModuleBlock {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsModuleBlock {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsModuleBlock {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsModuleBlock {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsGlobalDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsGlobalDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsGlobalDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsGlobalDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsGlobalDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsParameterProperty {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsParameterProperty {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsParameterProperty {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsParameterProperty {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsParameterProperty {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsThisParameter {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsThisParameter {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsThisParameter {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsThisParameter {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsThisParameter {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsAsExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsAsExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsAsExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsAsExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsAsExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsSatisfiesExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsSatisfiesExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsSatisfiesExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsSatisfiesExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsSatisfiesExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsTypeAssertion {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsTypeAssertion {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsTypeAssertion {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsTypeAssertion {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsTypeAssertion {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNonNullExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNonNullExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNonNullExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNonNullExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNonNullExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsInstantiationExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsInstantiationExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsInstantiationExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsInstantiationExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsInstantiationExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsExportAssignment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsExportAssignment {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsExportAssignment {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsExportAssignment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsExportAssignment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsNamespaceExportDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsNamespaceExportDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsNamespaceExportDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsNamespaceExportDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsNamespaceExportDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsImportEqualsDeclaration {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsImportEqualsDeclaration {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsImportEqualsDeclaration {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsImportEqualsDeclaration {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsImportEqualsDeclaration {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl TsExternalModuleReference {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for TsExternalModuleReference {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for TsExternalModuleReference {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for TsExternalModuleReference {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for TsExternalModuleReference {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxElementName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxElementName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxElementName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxElementName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxElementName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxMemberExpressionObject {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxMemberExpressionObject {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxMemberExpressionObject {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxMemberExpressionObject {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxMemberExpressionObject {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxAttributeName {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxAttributeName {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxAttributeName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxAttributeName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxAttributeName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxAttributeItem {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxAttributeItem {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxAttributeItem {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxAttributeItem {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxAttributeItem {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxAttributeValue {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxAttributeValue {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxAttributeValue {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxAttributeValue {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxAttributeValue {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxExpression {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxExpression {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxChild {
    #[inline]
    pub fn node_id(self) -> AstId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxChild {
    type Id = AstId;
    #[inline]
    fn id(self) -> AstId {
        self.0
    }
}
impl CommonAstNode for JsxChild {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<AstId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        ast.span(self.ast_id())
    }
}
impl SetSpan for JsxChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        ast.set_span(self.ast_id(), span)
    }
}
impl JsxElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxOpeningElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxOpeningElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxOpeningElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxOpeningElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxOpeningElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxClosingElement {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxClosingElement {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxClosingElement {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxClosingElement {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxClosingElement {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxFragment {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxFragment {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxOpeningFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxOpeningFragment {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxOpeningFragment {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxOpeningFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxOpeningFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxClosingFragment {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxClosingFragment {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxClosingFragment {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxClosingFragment {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxClosingFragment {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxIdentifier {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxIdentifier {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxIdentifier {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxIdentifier {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxIdentifier {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxNamespacedName {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxNamespacedName {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxNamespacedName {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxNamespacedName {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxNamespacedName {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxMemberExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxMemberExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxMemberExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxMemberExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxMemberExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxAttribute {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxAttribute {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxSpreadAttribute {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxSpreadAttribute {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxSpreadAttribute {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxSpreadAttribute {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxSpreadAttribute {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxExpressionContainer {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxExpressionContainer {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxExpressionContainer {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxExpressionContainer {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxExpressionContainer {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxEmptyExpression {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxEmptyExpression {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxEmptyExpression {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxEmptyExpression {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxEmptyExpression {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxText {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxText {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxText {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxText {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxText {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl JsxSpreadChild {
    #[inline]
    pub fn node_id(self) -> NodeId {
        self.0
    }
    #[inline]
    pub fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    /// # Safety
    ///
    /// ast_id must identify this concrete common node type.
    #[inline]
    pub unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe { <Self as CommonAstNode>::from_ast_id_unchecked(ast_id) }
    }
}
impl AstNode for JsxSpreadChild {
    type Id = NodeId;
    #[inline]
    fn id(self) -> NodeId {
        self.0
    }
}
impl CommonAstNode for JsxSpreadChild {
    #[inline]
    fn ast_id(self) -> AstId {
        self.0.ast_id()
    }
    #[inline]
    unsafe fn from_ast_id_unchecked(ast_id: AstId) -> Self {
        unsafe {
            <Self as AstNode>::from_id_unchecked(<NodeId as TypedAstId>::from_ast_id_unchecked(
                ast_id,
            ))
        }
    }
}
impl GetSpan for JsxSpreadChild {
    #[inline]
    fn span(self, ast: &Ast<'_>) -> crate::Span {
        unsafe { *ast.nodes.span_unchecked(self.0) }
    }
}
impl SetSpan for JsxSpreadChild {
    #[inline]
    fn set_span(self, ast: &mut Ast<'_>, span: crate::Span) {
        unsafe { *ast.nodes.span_mut_unchecked(self.0) = span }
    }
}
impl<'a> Ast<'a> {
    #[inline]
    pub fn kind_data<I: Into<AstId>>(&self, node_id: I) -> NodeKindData {
        let ast_id = node_id.into();
        match self.node_kind(ast_id) {
            NodeKind::Program => NodeKindData::Program(unsafe {
                <Program as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Super => NodeKindData::Super(unsafe {
                <Super as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ThisExpression => NodeKindData::ThisExpression(unsafe {
                <ThisExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::StringLiteral => NodeKindData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::NumericLiteral => NodeKindData::NumericLiteral(unsafe {
                <NumericLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BigIntLiteral => NodeKindData::BigIntLiteral(unsafe {
                <BigIntLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BooleanLiteral => NodeKindData::BooleanLiteral(unsafe {
                <BooleanLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::NullLiteral => NodeKindData::NullLiteral(unsafe {
                <NullLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::RegExpLiteral => NodeKindData::RegExpLiteral(unsafe {
                <RegExpLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TemplateLiteral => NodeKindData::TemplateLiteral(unsafe {
                <TemplateLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TemplateElement => NodeKindData::TemplateElement(unsafe {
                <TemplateElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::IdentifierReference => NodeKindData::IdentifierReference(unsafe {
                <IdentifierReference as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::PrivateIdentifier => NodeKindData::PrivateIdentifier(unsafe {
                <PrivateIdentifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BindingIdentifier => NodeKindData::BindingIdentifier(unsafe {
                <BindingIdentifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::IdentifierName => NodeKindData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::LabelIdentifier => NodeKindData::LabelIdentifier(unsafe {
                <LabelIdentifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::AssignmentPattern => NodeKindData::AssignmentPattern(unsafe {
                <AssignmentPattern as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BindingRestElement => NodeKindData::BindingRestElement(unsafe {
                <BindingRestElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ArrayPattern => NodeKindData::ArrayPattern(unsafe {
                <ArrayPattern as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ObjectPattern => NodeKindData::ObjectPattern(unsafe {
                <ObjectPattern as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BindingProperty => NodeKindData::BindingProperty(unsafe {
                <BindingProperty as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BinaryExpression => NodeKindData::BinaryExpression(unsafe {
                <BinaryExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::LogicalExpression => NodeKindData::LogicalExpression(unsafe {
                <LogicalExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ConditionalExpression => NodeKindData::ConditionalExpression(unsafe {
                <ConditionalExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::UnaryExpression => NodeKindData::UnaryExpression(unsafe {
                <UnaryExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::UpdateExpression => NodeKindData::UpdateExpression(unsafe {
                <UpdateExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::AssignmentExpression => NodeKindData::AssignmentExpression(unsafe {
                <AssignmentExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::VariableDeclaration => NodeKindData::VariableDeclaration(unsafe {
                <VariableDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::VariableDeclarator => NodeKindData::VariableDeclarator(unsafe {
                <VariableDeclarator as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::IfStatement => NodeKindData::IfStatement(unsafe {
                <IfStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::SwitchStatement => NodeKindData::SwitchStatement(unsafe {
                <SwitchStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::SwitchCase => NodeKindData::SwitchCase(unsafe {
                <SwitchCase as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ForStatement => NodeKindData::ForStatement(unsafe {
                <ForStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ForInStatement => NodeKindData::ForInStatement(unsafe {
                <ForInStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ForOfStatement => NodeKindData::ForOfStatement(unsafe {
                <ForOfStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TryStatement => NodeKindData::TryStatement(unsafe {
                <TryStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::CatchClause => NodeKindData::CatchClause(unsafe {
                <CatchClause as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BreakStatement => NodeKindData::BreakStatement(unsafe {
                <BreakStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ContinueStatement => NodeKindData::ContinueStatement(unsafe {
                <ContinueStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::LabeledStatement => NodeKindData::LabeledStatement(unsafe {
                <LabeledStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ExpressionStatement => NodeKindData::ExpressionStatement(unsafe {
                <ExpressionStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ReturnStatement => NodeKindData::ReturnStatement(unsafe {
                <ReturnStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ThrowStatement => NodeKindData::ThrowStatement(unsafe {
                <ThrowStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::DebuggerStatement => NodeKindData::DebuggerStatement(unsafe {
                <DebuggerStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::EmptyStatement => NodeKindData::EmptyStatement(unsafe {
                <EmptyStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Directive => NodeKindData::Directive(unsafe {
                <Directive as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Decorator => NodeKindData::Decorator(unsafe {
                <Decorator as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Class => NodeKindData::Class(unsafe {
                <ClassId as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::MethodDefinition => NodeKindData::MethodDefinition(unsafe {
                <MethodDefinition as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::PropertyDefinition => NodeKindData::PropertyDefinition(unsafe {
                <PropertyDefinition as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::StaticBlock => NodeKindData::StaticBlock(unsafe {
                <StaticBlock as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Function => NodeKindData::Function(unsafe {
                <Function as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::FunctionBody => NodeKindData::FunctionBody(unsafe {
                <FunctionBody as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::BlockStatement => NodeKindData::BlockStatement(unsafe {
                <BlockStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::WhileStatement => NodeKindData::WhileStatement(unsafe {
                <WhileStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::DoWhileStatement => NodeKindData::DoWhileStatement(unsafe {
                <DoWhileStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::WithStatement => NodeKindData::WithStatement(unsafe {
                <WithStatement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::FormalParameters => NodeKindData::FormalParameters(unsafe {
                <FormalParameters as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::FormalParameter => NodeKindData::FormalParameter(unsafe {
                <FormalParameter as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ArrayExpression => NodeKindData::ArrayExpression(unsafe {
                <ArrayExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ObjectExpression => NodeKindData::ObjectExpression(unsafe {
                <ObjectExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::SpreadElement => NodeKindData::SpreadElement(unsafe {
                <SpreadElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ObjectProperty => NodeKindData::ObjectProperty(unsafe {
                <ObjectProperty as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ParenthesizedExpression => NodeKindData::ParenthesizedExpression(unsafe {
                <ParenthesizedExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ArrowFunctionExpression => NodeKindData::ArrowFunctionExpression(unsafe {
                <ArrowFunctionExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::SequenceExpression => NodeKindData::SequenceExpression(unsafe {
                <SequenceExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::MemberExpression => NodeKindData::MemberExpression(unsafe {
                <MemberExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::CallExpression => NodeKindData::CallExpression(unsafe {
                <CallExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ChainExpression => NodeKindData::ChainExpression(unsafe {
                <ChainExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TaggedTemplateExpression => NodeKindData::TaggedTemplateExpression(unsafe {
                <TaggedTemplateExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::NewExpression => NodeKindData::NewExpression(unsafe {
                <NewExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::AwaitExpression => NodeKindData::AwaitExpression(unsafe {
                <AwaitExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::YieldExpression => NodeKindData::YieldExpression(unsafe {
                <YieldExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::MetaProperty => NodeKindData::MetaProperty(unsafe {
                <MetaProperty as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportExpression => NodeKindData::ImportExpression(unsafe {
                <ImportExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportDeclaration => NodeKindData::ImportDeclaration(unsafe {
                <ImportDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportSpecifier => NodeKindData::ImportSpecifier(unsafe {
                <ImportSpecifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportDefaultSpecifier => NodeKindData::ImportDefaultSpecifier(unsafe {
                <ImportDefaultSpecifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportNamespaceSpecifier => NodeKindData::ImportNamespaceSpecifier(unsafe {
                <ImportNamespaceSpecifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ImportAttribute => NodeKindData::ImportAttribute(unsafe {
                <ImportAttribute as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ExportNamedDeclaration => NodeKindData::ExportNamedDeclaration(unsafe {
                <ExportNamedDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ExportDefaultDeclaration => NodeKindData::ExportDefaultDeclaration(unsafe {
                <ExportDefaultDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ExportAllDeclaration => NodeKindData::ExportAllDeclaration(unsafe {
                <ExportAllDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::ExportSpecifier => NodeKindData::ExportSpecifier(unsafe {
                <ExportSpecifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::Hashbang => NodeKindData::Hashbang(unsafe {
                <Hashbang as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsFunction => NodeKindData::TsFunction(unsafe {
                <TsFunction as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsMethodDefinition => NodeKindData::TsMethodDefinition(unsafe {
                <TsMethodDefinition as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeAnnotation => NodeKindData::TsTypeAnnotation(unsafe {
                <TsTypeAnnotation as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsAnyKeyword => NodeKindData::TsAnyKeyword(unsafe {
                <TsAnyKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsUnknownKeyword => NodeKindData::TsUnknownKeyword(unsafe {
                <TsUnknownKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNeverKeyword => NodeKindData::TsNeverKeyword(unsafe {
                <TsNeverKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsVoidKeyword => NodeKindData::TsVoidKeyword(unsafe {
                <TsVoidKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNullKeyword => NodeKindData::TsNullKeyword(unsafe {
                <TsNullKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsUndefinedKeyword => NodeKindData::TsUndefinedKeyword(unsafe {
                <TsUndefinedKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsStringKeyword => NodeKindData::TsStringKeyword(unsafe {
                <TsStringKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNumberKeyword => NodeKindData::TsNumberKeyword(unsafe {
                <TsNumberKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsBigIntKeyword => NodeKindData::TsBigIntKeyword(unsafe {
                <TsBigIntKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsBooleanKeyword => NodeKindData::TsBooleanKeyword(unsafe {
                <TsBooleanKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsSymbolKeyword => NodeKindData::TsSymbolKeyword(unsafe {
                <TsSymbolKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsObjectKeyword => NodeKindData::TsObjectKeyword(unsafe {
                <TsObjectKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsIntrinsicKeyword => NodeKindData::TsIntrinsicKeyword(unsafe {
                <TsIntrinsicKeyword as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsThisType => NodeKindData::TsThisType(unsafe {
                <TsThisType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeReference => NodeKindData::TsTypeReference(unsafe {
                <TsTypeReference as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsQualifiedName => NodeKindData::TsQualifiedName(unsafe {
                <TsQualifiedName as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeQuery => NodeKindData::TsTypeQuery(unsafe {
                <TsTypeQuery as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsImportType => NodeKindData::TsImportType(unsafe {
                <TsImportType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeParameter => NodeKindData::TsTypeParameter(unsafe {
                <TsTypeParameter as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeParameterDeclaration => {
                NodeKindData::TsTypeParameterDeclaration(unsafe {
                    <TsTypeParameterDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsTypeParameterInstantiation => {
                NodeKindData::TsTypeParameterInstantiation(unsafe {
                    <TsTypeParameterInstantiation as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsLiteralType => NodeKindData::TsLiteralType(unsafe {
                <TsLiteralType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTemplateLiteralType => NodeKindData::TsTemplateLiteralType(unsafe {
                <TsTemplateLiteralType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsArrayType => NodeKindData::TsArrayType(unsafe {
                <TsArrayType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsIndexedAccessType => NodeKindData::TsIndexedAccessType(unsafe {
                <TsIndexedAccessType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTupleType => NodeKindData::TsTupleType(unsafe {
                <TsTupleType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNamedTupleMember => NodeKindData::TsNamedTupleMember(unsafe {
                <TsNamedTupleMember as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsOptionalType => NodeKindData::TsOptionalType(unsafe {
                <TsOptionalType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsRestType => NodeKindData::TsRestType(unsafe {
                <TsRestType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsJsDocNullableType => NodeKindData::TsJsDocNullableType(unsafe {
                <TsJsDocNullableType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsJsDocNonNullableType => NodeKindData::TsJsDocNonNullableType(unsafe {
                <TsJsDocNonNullableType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsJsDocUnknownType => NodeKindData::TsJsDocUnknownType(unsafe {
                <TsJsDocUnknownType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsUnionType => NodeKindData::TsUnionType(unsafe {
                <TsUnionType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsIntersectionType => NodeKindData::TsIntersectionType(unsafe {
                <TsIntersectionType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsConditionalType => NodeKindData::TsConditionalType(unsafe {
                <TsConditionalType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsInferType => NodeKindData::TsInferType(unsafe {
                <TsInferType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeOperator => NodeKindData::TsTypeOperator(unsafe {
                <TsTypeOperator as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsParenthesizedType => NodeKindData::TsParenthesizedType(unsafe {
                <TsParenthesizedType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsFunctionType => NodeKindData::TsFunctionType(unsafe {
                <TsFunctionType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsConstructorType => NodeKindData::TsConstructorType(unsafe {
                <TsConstructorType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypePredicate => NodeKindData::TsTypePredicate(unsafe {
                <TsTypePredicate as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeLiteral => NodeKindData::TsTypeLiteral(unsafe {
                <TsTypeLiteral as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsMappedType => NodeKindData::TsMappedType(unsafe {
                <TsMappedType as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsPropertySignature => NodeKindData::TsPropertySignature(unsafe {
                <TsPropertySignature as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsMethodSignature => NodeKindData::TsMethodSignature(unsafe {
                <TsMethodSignature as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsCallSignatureDeclaration => {
                NodeKindData::TsCallSignatureDeclaration(unsafe {
                    <TsCallSignatureDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsConstructSignatureDeclaration => {
                NodeKindData::TsConstructSignatureDeclaration(unsafe {
                    <TsConstructSignatureDeclaration as CommonAstNode>::from_ast_id_unchecked(
                        ast_id,
                    )
                })
            }
            NodeKind::TsIndexSignature => NodeKindData::TsIndexSignature(unsafe {
                <TsIndexSignature as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeAliasDeclaration => NodeKindData::TsTypeAliasDeclaration(unsafe {
                <TsTypeAliasDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsInterfaceDeclaration => NodeKindData::TsInterfaceDeclaration(unsafe {
                <TsInterfaceDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsInterfaceBody => NodeKindData::TsInterfaceBody(unsafe {
                <TsInterfaceBody as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsInterfaceHeritage => NodeKindData::TsInterfaceHeritage(unsafe {
                <TsInterfaceHeritage as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsClassImplements => NodeKindData::TsClassImplements(unsafe {
                <TsClassImplements as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsEnumDeclaration => NodeKindData::TsEnumDeclaration(unsafe {
                <TsEnumDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsEnumBody => NodeKindData::TsEnumBody(unsafe {
                <TsEnumBody as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsEnumMember => NodeKindData::TsEnumMember(unsafe {
                <TsEnumMember as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsModuleDeclaration => NodeKindData::TsModuleDeclaration(unsafe {
                <TsModuleDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsModuleBlock => NodeKindData::TsModuleBlock(unsafe {
                <TsModuleBlock as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsGlobalDeclaration => NodeKindData::TsGlobalDeclaration(unsafe {
                <TsGlobalDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsParameterProperty => NodeKindData::TsParameterProperty(unsafe {
                <TsParameterProperty as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsThisParameter => NodeKindData::TsThisParameter(unsafe {
                <TsThisParameter as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsAsExpression => NodeKindData::TsAsExpression(unsafe {
                <TsAsExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsSatisfiesExpression => NodeKindData::TsSatisfiesExpression(unsafe {
                <TsSatisfiesExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsTypeAssertion => NodeKindData::TsTypeAssertion(unsafe {
                <TsTypeAssertion as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNonNullExpression => NodeKindData::TsNonNullExpression(unsafe {
                <TsNonNullExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsInstantiationExpression => {
                NodeKindData::TsInstantiationExpression(unsafe {
                    <TsInstantiationExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsExportAssignment => NodeKindData::TsExportAssignment(unsafe {
                <TsExportAssignment as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::TsNamespaceExportDeclaration => {
                NodeKindData::TsNamespaceExportDeclaration(unsafe {
                    <TsNamespaceExportDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsImportEqualsDeclaration => {
                NodeKindData::TsImportEqualsDeclaration(unsafe {
                    <TsImportEqualsDeclaration as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::TsExternalModuleReference => {
                NodeKindData::TsExternalModuleReference(unsafe {
                    <TsExternalModuleReference as CommonAstNode>::from_ast_id_unchecked(ast_id)
                })
            }
            NodeKind::JsxElement => NodeKindData::JsxElement(unsafe {
                <JsxElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxOpeningElement => NodeKindData::JsxOpeningElement(unsafe {
                <JsxOpeningElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxClosingElement => NodeKindData::JsxClosingElement(unsafe {
                <JsxClosingElement as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxFragment => NodeKindData::JsxFragment(unsafe {
                <JsxFragment as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxOpeningFragment => NodeKindData::JsxOpeningFragment(unsafe {
                <JsxOpeningFragment as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxClosingFragment => NodeKindData::JsxClosingFragment(unsafe {
                <JsxClosingFragment as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxIdentifier => NodeKindData::JsxIdentifier(unsafe {
                <JsxIdentifier as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxNamespacedName => NodeKindData::JsxNamespacedName(unsafe {
                <JsxNamespacedName as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxMemberExpression => NodeKindData::JsxMemberExpression(unsafe {
                <JsxMemberExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxAttribute => NodeKindData::JsxAttribute(unsafe {
                <JsxAttribute as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxSpreadAttribute => NodeKindData::JsxSpreadAttribute(unsafe {
                <JsxSpreadAttribute as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxExpressionContainer => NodeKindData::JsxExpressionContainer(unsafe {
                <JsxExpressionContainer as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxEmptyExpression => NodeKindData::JsxEmptyExpression(unsafe {
                <JsxEmptyExpression as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxText => NodeKindData::JsxText(unsafe {
                <JsxText as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
            NodeKind::JsxSpreadChild => NodeKindData::JsxSpreadChild(unsafe {
                <JsxSpreadChild as CommonAstNode>::from_ast_id_unchecked(ast_id)
            }),
        }
    }
    #[inline]
    pub fn stmt_data(&self, node: Stmt) -> StmtData {
        match self.node_kind(node.0) {
            NodeKind::BlockStatement => StmtData::BlockStatement(unsafe {
                <BlockStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BreakStatement => StmtData::BreakStatement(unsafe {
                <BreakStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ContinueStatement => StmtData::ContinueStatement(unsafe {
                <ContinueStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::DebuggerStatement => StmtData::DebuggerStatement(unsafe {
                <DebuggerStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::DoWhileStatement => StmtData::DoWhileStatement(unsafe {
                <DoWhileStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::EmptyStatement => StmtData::EmptyStatement(unsafe {
                <EmptyStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ExpressionStatement => StmtData::ExpressionStatement(unsafe {
                <ExpressionStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ForInStatement => StmtData::ForInStatement(unsafe {
                <ForInStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ForOfStatement => StmtData::ForOfStatement(unsafe {
                <ForOfStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ForStatement => StmtData::ForStatement(unsafe {
                <ForStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IfStatement => StmtData::IfStatement(unsafe {
                <IfStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LabeledStatement => StmtData::LabeledStatement(unsafe {
                <LabeledStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ReturnStatement => StmtData::ReturnStatement(unsafe {
                <ReturnStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SwitchStatement => StmtData::SwitchStatement(unsafe {
                <SwitchStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThrowStatement => StmtData::ThrowStatement(unsafe {
                <ThrowStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TryStatement => StmtData::TryStatement(unsafe {
                <TryStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::WhileStatement => StmtData::WhileStatement(unsafe {
                <WhileStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::WithStatement => StmtData::WithStatement(unsafe {
                <WithStatement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::VariableDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsFunction => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAliasDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInterfaceDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsEnumDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsModuleDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsGlobalDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsImportEqualsDeclaration => StmtData::Declaration(unsafe {
                <Decl as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportDeclaration => StmtData::ImportDeclaration(unsafe {
                <ImportDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ExportNamedDeclaration => StmtData::ExportNamedDeclaration(unsafe {
                <ExportNamedDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ExportDefaultDeclaration => StmtData::ExportDefaultDeclaration(unsafe {
                <ExportDefaultDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ExportAllDeclaration => StmtData::ExportAllDeclaration(unsafe {
                <ExportAllDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsExportAssignment => StmtData::TsExportAssignment(unsafe {
                <TsExportAssignment as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNamespaceExportDeclaration => {
                StmtData::TsNamespaceExportDeclaration(unsafe {
                    <TsNamespaceExportDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn decl_data(&self, node: Decl) -> DeclData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_decl());
        match kind {
            NodeKind::VariableDeclaration => DeclData::VariableDeclaration(unsafe {
                <VariableDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => DeclData::Function(unsafe {
                <Function as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => DeclData::Class(unsafe {
                <ClassId as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsFunction => DeclData::TsFunction(unsafe {
                <TsFunction as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAliasDeclaration => DeclData::TsTypeAliasDeclaration(unsafe {
                <TsTypeAliasDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInterfaceDeclaration => DeclData::TsInterfaceDeclaration(unsafe {
                <TsInterfaceDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsEnumDeclaration => DeclData::TsEnumDeclaration(unsafe {
                <TsEnumDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsModuleDeclaration => DeclData::TsModuleDeclaration(unsafe {
                <TsModuleDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsGlobalDeclaration => DeclData::TsGlobalDeclaration(unsafe {
                <TsGlobalDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsImportEqualsDeclaration => DeclData::TsImportEqualsDeclaration(unsafe {
                <TsImportEqualsDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn expr_data(&self, node: Expr) -> ExprData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_expr());
        match kind {
            NodeKind::IdentifierReference => ExprData::IdentifierReference(unsafe {
                <IdentifierReference as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => ExprData::ThisExpression(unsafe {
                <ThisExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => {
                ExprData::Super(unsafe { <Super as CommonAstNode>::from_ast_id_unchecked(node.0) })
            }
            NodeKind::MetaProperty => ExprData::MetaProperty(unsafe {
                <MetaProperty as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ExprData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => ExprData::NumericLiteral(unsafe {
                <NumericLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => ExprData::BigIntLiteral(unsafe {
                <BigIntLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => ExprData::BooleanLiteral(unsafe {
                <BooleanLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => ExprData::NullLiteral(unsafe {
                <NullLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => ExprData::RegExpLiteral(unsafe {
                <RegExpLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => ExprData::TemplateLiteral(unsafe {
                <TemplateLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => ExprData::BinaryExpression(unsafe {
                <BinaryExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => ExprData::LogicalExpression(unsafe {
                <LogicalExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => ExprData::UnaryExpression(unsafe {
                <UnaryExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => ExprData::UpdateExpression(unsafe {
                <UpdateExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => ExprData::AssignmentExpression(unsafe {
                <AssignmentExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => ExprData::ConditionalExpression(unsafe {
                <ConditionalExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => ExprData::SequenceExpression(unsafe {
                <SequenceExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ExprData::MemberExpression(unsafe {
                <MemberExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => ExprData::CallExpression(unsafe {
                <CallExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => ExprData::ChainExpression(unsafe {
                <ChainExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => ExprData::NewExpression(unsafe {
                <NewExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => ExprData::TaggedTemplateExpression(unsafe {
                <TaggedTemplateExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => ExprData::ArrowFunctionExpression(unsafe {
                <ArrowFunctionExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => ExprData::ArrayExpression(unsafe {
                <ArrayExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => ExprData::ObjectExpression(unsafe {
                <ObjectExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => ExprData::ParenthesizedExpression(unsafe {
                <ParenthesizedExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => ExprData::Function(unsafe {
                <Function as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => ExprData::Class(unsafe {
                <ClassId as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => ExprData::ImportExpression(unsafe {
                <ImportExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => ExprData::AwaitExpression(unsafe {
                <AwaitExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => ExprData::YieldExpression(unsafe {
                <YieldExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => ExprData::PrivateIdentifier(unsafe {
                <PrivateIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ExprData::TsAsExpression(unsafe {
                <TsAsExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ExprData::TsSatisfiesExpression(unsafe {
                <TsSatisfiesExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ExprData::TsTypeAssertion(unsafe {
                <TsTypeAssertion as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ExprData::TsNonNullExpression(unsafe {
                <TsNonNullExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => ExprData::TsInstantiationExpression(unsafe {
                <TsInstantiationExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => ExprData::JsxElement(unsafe {
                <JsxElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => ExprData::JsxFragment(unsafe {
                <JsxFragment as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn binding_pattern_data(&self, node: BindingPattern) -> BindingPatternData {
        match self.node_kind(node.0) {
            NodeKind::BindingIdentifier => BindingPatternData::BindingIdentifier(unsafe {
                <BindingIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => BindingPatternData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentPattern => BindingPatternData::AssignmentPattern(unsafe {
                <AssignmentPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BindingRestElement => BindingPatternData::BindingRestElement(unsafe {
                <BindingRestElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayPattern => BindingPatternData::ArrayPattern(unsafe {
                <ArrayPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectPattern => BindingPatternData::ObjectPattern(unsafe {
                <ObjectPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn assignment_target_data(&self, node: AssignmentTarget) -> AssignmentTargetData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => {
                AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                    <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::TsTypeAssertion => AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => AssignmentTargetData::SimpleAssignmentTarget(unsafe {
                <SimpleAssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayPattern => AssignmentTargetData::ArrayAssignmentTarget(unsafe {
                <ArrayPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectPattern => AssignmentTargetData::ObjectAssignmentTarget(unsafe {
                <ObjectPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn simple_assignment_target_data(
        &self,
        node: SimpleAssignmentTarget,
    ) -> SimpleAssignmentTargetData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_simple_assignment_target());
        match kind {
            NodeKind::IdentifierReference => {
                SimpleAssignmentTargetData::IdentifierReference(unsafe {
                    <IdentifierReference as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::MemberExpression => SimpleAssignmentTargetData::MemberExpression(unsafe {
                <MemberExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => SimpleAssignmentTargetData::TsAsExpression(unsafe {
                <TsAsExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => {
                SimpleAssignmentTargetData::TsSatisfiesExpression(unsafe {
                    <TsSatisfiesExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::TsTypeAssertion => SimpleAssignmentTargetData::TsTypeAssertion(unsafe {
                <TsTypeAssertion as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => {
                SimpleAssignmentTargetData::TsNonNullExpression(unsafe {
                    <TsNonNullExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn property_key_data(&self, node: PropertyKey) -> PropertyKeyData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierName => PropertyKeyData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => PropertyKeyData::PrivateIdentifier(unsafe {
                <PrivateIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => PropertyKeyData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => PropertyKeyData::NumericLiteral(unsafe {
                <NumericLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => PropertyKeyData::BigIntLiteral(unsafe {
                <BigIntLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => PropertyKeyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn module_export_name_data(&self, node: ModuleExportName) -> ModuleExportNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_module_export_name());
        match kind {
            NodeKind::IdentifierName => ModuleExportNameData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ModuleExportNameData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn argument_data(&self, node: Argument) -> ArgumentData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => ArgumentData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SpreadElement => ArgumentData::SpreadElement(unsafe {
                <SpreadElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn object_property_kind_data(&self, node: ObjectPropertyKind) -> ObjectPropertyKindData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_object_property_kind());
        match kind {
            NodeKind::ObjectProperty => ObjectPropertyKindData::ObjectProperty(unsafe {
                <ObjectProperty as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SpreadElement => ObjectPropertyKindData::SpreadElement(unsafe {
                <SpreadElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn for_statement_init_data(&self, node: ForStatementInit) -> ForStatementInitData {
        match self.node_kind(node.0) {
            NodeKind::VariableDeclaration => ForStatementInitData::VariableDeclaration(unsafe {
                <VariableDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => ForStatementInitData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn for_statement_left_data(&self, node: ForStatementLeft) -> ForStatementLeftData {
        match self.node_kind(node.0) {
            NodeKind::VariableDeclaration => ForStatementLeftData::VariableDeclaration(unsafe {
                <VariableDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayPattern => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectPattern => ForStatementLeftData::AssignmentTarget(unsafe {
                <AssignmentTarget as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn arrow_function_body_data(&self, node: ArrowFunctionBody) -> ArrowFunctionBodyData {
        match self.node_kind(node.0) {
            NodeKind::FunctionBody => ArrowFunctionBodyData::FunctionBody(unsafe {
                <FunctionBody as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => ArrowFunctionBodyData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn formal_parameter_item_data(&self, node: FormalParameterItem) -> FormalParameterItemData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_formal_parameter_item());
        match kind {
            NodeKind::FormalParameter => FormalParameterItemData::FormalParameter(unsafe {
                <FormalParameter as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsParameterProperty => FormalParameterItemData::TsParameterProperty(unsafe {
                <TsParameterProperty as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn formal_parameter_pattern_data(
        &self,
        node: FormalParameterPattern,
    ) -> FormalParameterPatternData {
        match self.node_kind(node.0) {
            NodeKind::BindingIdentifier => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentPattern => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BindingRestElement => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayPattern => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectPattern => FormalParameterPatternData::BindingPattern(unsafe {
                <BindingPattern as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsThisParameter => FormalParameterPatternData::TsThisParameter(unsafe {
                <TsThisParameter as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn class_element_data(&self, node: ClassElement) -> ClassElementData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_class_element());
        match kind {
            NodeKind::MethodDefinition => ClassElementData::MethodDefinition(unsafe {
                <MethodDefinition as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PropertyDefinition => ClassElementData::PropertyDefinition(unsafe {
                <PropertyDefinition as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StaticBlock => ClassElementData::StaticBlock(unsafe {
                <StaticBlock as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsMethodDefinition => ClassElementData::TsMethodDefinition(unsafe {
                <TsMethodDefinition as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIndexSignature => ClassElementData::TsIndexSignature(unsafe {
                <TsIndexSignature as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn import_declaration_specifier_data(
        &self,
        node: ImportDeclarationSpecifier,
    ) -> ImportDeclarationSpecifierData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_import_declaration_specifier());
        match kind {
            NodeKind::ImportSpecifier => ImportDeclarationSpecifierData::ImportSpecifier(unsafe {
                <ImportSpecifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportDefaultSpecifier => {
                ImportDeclarationSpecifierData::ImportDefaultSpecifier(unsafe {
                    <ImportDefaultSpecifier as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::ImportNamespaceSpecifier => {
                ImportDeclarationSpecifierData::ImportNamespaceSpecifier(unsafe {
                    <ImportNamespaceSpecifier as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn export_default_declaration_kind_data(
        &self,
        node: ExportDefaultDeclarationKind,
    ) -> ExportDefaultDeclarationKindData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => {
                let function =
                    unsafe { <Function as CommonAstNode>::from_ast_id_unchecked(node.0) };
                if function.function_type(self) == FunctionType::FunctionDeclaration {
                    ExportDefaultDeclarationKindData::Function(function)
                } else {
                    ExportDefaultDeclarationKindData::Expr(unsafe {
                        <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
                    })
                }
            }
            NodeKind::Class => {
                let class = unsafe { <ClassId as CommonAstNode>::from_ast_id_unchecked(node.0) };
                if class.class_type(self) == ClassType::ClassDeclaration {
                    ExportDefaultDeclarationKindData::Class(class)
                } else {
                    ExportDefaultDeclarationKindData::Expr(unsafe {
                        <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
                    })
                }
            }
            NodeKind::ImportExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => ExportDefaultDeclarationKindData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsFunction => ExportDefaultDeclarationKindData::TsFunction(unsafe {
                <TsFunction as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInterfaceDeclaration => {
                ExportDefaultDeclarationKindData::TsInterfaceDeclaration(unsafe {
                    <TsInterfaceDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_data(&self, node: TsType) -> TsTypeData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_type());
        match kind {
            NodeKind::TsAnyKeyword => TsTypeData::TsAnyKeyword(unsafe {
                <TsAnyKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUnknownKeyword => TsTypeData::TsUnknownKeyword(unsafe {
                <TsUnknownKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNeverKeyword => TsTypeData::TsNeverKeyword(unsafe {
                <TsNeverKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsVoidKeyword => TsTypeData::TsVoidKeyword(unsafe {
                <TsVoidKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNullKeyword => TsTypeData::TsNullKeyword(unsafe {
                <TsNullKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUndefinedKeyword => TsTypeData::TsUndefinedKeyword(unsafe {
                <TsUndefinedKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsStringKeyword => TsTypeData::TsStringKeyword(unsafe {
                <TsStringKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNumberKeyword => TsTypeData::TsNumberKeyword(unsafe {
                <TsNumberKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsBigIntKeyword => TsTypeData::TsBigIntKeyword(unsafe {
                <TsBigIntKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsBooleanKeyword => TsTypeData::TsBooleanKeyword(unsafe {
                <TsBooleanKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSymbolKeyword => TsTypeData::TsSymbolKeyword(unsafe {
                <TsSymbolKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsObjectKeyword => TsTypeData::TsObjectKeyword(unsafe {
                <TsObjectKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIntrinsicKeyword => TsTypeData::TsIntrinsicKeyword(unsafe {
                <TsIntrinsicKeyword as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsThisType => TsTypeData::TsThisType(unsafe {
                <TsThisType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeReference => TsTypeData::TsTypeReference(unsafe {
                <TsTypeReference as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeQuery => TsTypeData::TsTypeQuery(unsafe {
                <TsTypeQuery as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsImportType => TsTypeData::TsImportType(unsafe {
                <TsImportType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsLiteralType => TsTypeData::TsLiteralType(unsafe {
                <TsLiteralType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTemplateLiteralType => TsTypeData::TsTemplateLiteralType(unsafe {
                <TsTemplateLiteralType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsArrayType => TsTypeData::TsArrayType(unsafe {
                <TsArrayType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIndexedAccessType => TsTypeData::TsIndexedAccessType(unsafe {
                <TsIndexedAccessType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTupleType => TsTypeData::TsTupleType(unsafe {
                <TsTupleType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsOptionalType => TsTypeData::TsOptionalType(unsafe {
                <TsOptionalType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsRestType => TsTypeData::TsRestType(unsafe {
                <TsRestType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocNullableType => TsTypeData::TsJsDocNullableType(unsafe {
                <TsJsDocNullableType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocNonNullableType => TsTypeData::TsJsDocNonNullableType(unsafe {
                <TsJsDocNonNullableType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocUnknownType => TsTypeData::TsJsDocUnknownType(unsafe {
                <TsJsDocUnknownType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUnionType => TsTypeData::TsUnionType(unsafe {
                <TsUnionType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIntersectionType => TsTypeData::TsIntersectionType(unsafe {
                <TsIntersectionType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsConditionalType => TsTypeData::TsConditionalType(unsafe {
                <TsConditionalType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInferType => TsTypeData::TsInferType(unsafe {
                <TsInferType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeOperator => TsTypeData::TsTypeOperator(unsafe {
                <TsTypeOperator as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsParenthesizedType => TsTypeData::TsParenthesizedType(unsafe {
                <TsParenthesizedType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsFunctionType => TsTypeData::TsFunctionType(unsafe {
                <TsFunctionType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsConstructorType => TsTypeData::TsConstructorType(unsafe {
                <TsConstructorType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypePredicate => TsTypeData::TsTypePredicate(unsafe {
                <TsTypePredicate as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeLiteral => TsTypeData::TsTypeLiteral(unsafe {
                <TsTypeLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsMappedType => TsTypeData::TsMappedType(unsafe {
                <TsMappedType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_name_data(&self, node: TsTypeName) -> TsTypeNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_type_name());
        match kind {
            NodeKind::IdentifierReference => TsTypeNameData::IdentifierReference(unsafe {
                <IdentifierReference as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsTypeNameData::TsQualifiedName(unsafe {
                <TsQualifiedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_qualified_name_left_data(
        &self,
        node: TsQualifiedNameLeft,
    ) -> TsQualifiedNameLeftData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_qualified_name_left());
        match kind {
            NodeKind::IdentifierReference => TsQualifiedNameLeftData::IdentifierReference(unsafe {
                <IdentifierReference as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierName => TsQualifiedNameLeftData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BindingIdentifier => TsQualifiedNameLeftData::BindingIdentifier(unsafe {
                <BindingIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => TsQualifiedNameLeftData::ThisExpression(unsafe {
                <ThisExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsQualifiedNameLeftData::TsQualifiedName(unsafe {
                <TsQualifiedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_import_type_qualifier_data(
        &self,
        node: TsImportTypeQualifier,
    ) -> TsImportTypeQualifierData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_import_type_qualifier());
        match kind {
            NodeKind::IdentifierName => TsImportTypeQualifierData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsImportTypeQualifierData::TsQualifiedName(unsafe {
                <TsQualifiedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_query_expr_name_data(
        &self,
        node: TsTypeQueryExprName,
    ) -> TsTypeQueryExprNameData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => TsTypeQueryExprNameData::TsTypeName(unsafe {
                <TsTypeName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsTypeQueryExprNameData::TsTypeName(unsafe {
                <TsTypeName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => TsTypeQueryExprNameData::ThisExpression(unsafe {
                <ThisExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsImportType => TsTypeQueryExprNameData::TsImportType(unsafe {
                <TsImportType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_literal_data(&self, node: TsLiteral) -> TsLiteralData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_literal());
        match kind {
            NodeKind::StringLiteral => TsLiteralData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => TsLiteralData::NumericLiteral(unsafe {
                <NumericLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => TsLiteralData::BigIntLiteral(unsafe {
                <BigIntLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => TsLiteralData::BooleanLiteral(unsafe {
                <BooleanLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => TsLiteralData::TemplateLiteral(unsafe {
                <TemplateLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => TsLiteralData::UnaryExpression(unsafe {
                <UnaryExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_tuple_element_data(&self, node: TsTupleElement) -> TsTupleElementData {
        match self.node_kind(node.0) {
            NodeKind::TsAnyKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUnknownKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNeverKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsVoidKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNullKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUndefinedKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsStringKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNumberKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsBigIntKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsBooleanKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSymbolKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsObjectKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIntrinsicKeyword => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsThisType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeReference => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeQuery => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsImportType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsLiteralType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTemplateLiteralType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsArrayType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIndexedAccessType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTupleType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsOptionalType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsRestType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocNullableType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocNonNullableType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsJsDocUnknownType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsUnionType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsIntersectionType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsConditionalType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInferType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeOperator => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsParenthesizedType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsFunctionType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsConstructorType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypePredicate => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeLiteral => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsMappedType => TsTupleElementData::TsType(unsafe {
                <TsType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNamedTupleMember => TsTupleElementData::TsNamedTupleMember(unsafe {
                <TsNamedTupleMember as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_element_data(&self, node: TsTypeElement) -> TsTypeElementData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_type_element());
        match kind {
            NodeKind::TsPropertySignature => TsTypeElementData::TsPropertySignature(unsafe {
                <TsPropertySignature as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsMethodSignature => TsTypeElementData::TsMethodSignature(unsafe {
                <TsMethodSignature as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsCallSignatureDeclaration => {
                TsTypeElementData::TsCallSignatureDeclaration(unsafe {
                    <TsCallSignatureDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::TsConstructSignatureDeclaration => {
                TsTypeElementData::TsConstructSignatureDeclaration(unsafe {
                    <TsConstructSignatureDeclaration as CommonAstNode>::from_ast_id_unchecked(
                        node.0,
                    )
                })
            }
            NodeKind::TsIndexSignature => TsTypeElementData::TsIndexSignature(unsafe {
                <TsIndexSignature as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_type_predicate_name_data(
        &self,
        node: TsTypePredicateName,
    ) -> TsTypePredicateNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_type_predicate_name());
        match kind {
            NodeKind::IdentifierName => TsTypePredicateNameData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsThisType => TsTypePredicateNameData::TsThisType(unsafe {
                <TsThisType as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_enum_member_name_data(&self, node: TsEnumMemberName) -> TsEnumMemberNameData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierName => TsEnumMemberNameData::IdentifierName(unsafe {
                <IdentifierName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => TsEnumMemberNameData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => TsEnumMemberNameData::TemplateLiteral(unsafe {
                <TemplateLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::IdentifierReference => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => TsEnumMemberNameData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_module_name_data(&self, node: TsModuleName) -> TsModuleNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_module_name());
        match kind {
            NodeKind::BindingIdentifier => TsModuleNameData::BindingIdentifier(unsafe {
                <BindingIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => TsModuleNameData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsModuleNameData::TsQualifiedName(unsafe {
                <TsQualifiedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_module_body_data(&self, node: TsModuleBody) -> TsModuleBodyData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_ts_module_body());
        match kind {
            NodeKind::TsModuleBlock => TsModuleBodyData::TsModuleBlock(unsafe {
                <TsModuleBlock as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsModuleDeclaration => TsModuleBodyData::TsModuleDeclaration(unsafe {
                <TsModuleDeclaration as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn ts_module_reference_data(&self, node: TsModuleReference) -> TsModuleReferenceData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => TsModuleReferenceData::TsTypeName(unsafe {
                <TsTypeName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsQualifiedName => TsModuleReferenceData::TsTypeName(unsafe {
                <TsTypeName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsExternalModuleReference => {
                TsModuleReferenceData::TsExternalModuleReference(unsafe {
                    <TsExternalModuleReference as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_element_name_data(&self, node: JsxElementName) -> JsxElementNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_element_name());
        match kind {
            NodeKind::JsxIdentifier => JsxElementNameData::JsxIdentifier(unsafe {
                <JsxIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxNamespacedName => JsxElementNameData::JsxNamespacedName(unsafe {
                <JsxNamespacedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxMemberExpression => JsxElementNameData::JsxMemberExpression(unsafe {
                <JsxMemberExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_member_expression_object_data(
        &self,
        node: JsxMemberExpressionObject,
    ) -> JsxMemberExpressionObjectData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_member_expression_object());
        match kind {
            NodeKind::JsxIdentifier => JsxMemberExpressionObjectData::JsxIdentifier(unsafe {
                <JsxIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxMemberExpression => {
                JsxMemberExpressionObjectData::JsxMemberExpression(unsafe {
                    <JsxMemberExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_name_data(&self, node: JsxAttributeName) -> JsxAttributeNameData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_attribute_name());
        match kind {
            NodeKind::JsxIdentifier => JsxAttributeNameData::JsxIdentifier(unsafe {
                <JsxIdentifier as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxNamespacedName => JsxAttributeNameData::JsxNamespacedName(unsafe {
                <JsxNamespacedName as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_item_data(&self, node: JsxAttributeItem) -> JsxAttributeItemData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_attribute_item());
        match kind {
            NodeKind::JsxAttribute => JsxAttributeItemData::JsxAttribute(unsafe {
                <JsxAttribute as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxSpreadAttribute => JsxAttributeItemData::JsxSpreadAttribute(unsafe {
                <JsxSpreadAttribute as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_attribute_value_data(&self, node: JsxAttributeValue) -> JsxAttributeValueData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_attribute_value());
        match kind {
            NodeKind::StringLiteral => JsxAttributeValueData::StringLiteral(unsafe {
                <StringLiteral as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxExpressionContainer => {
                JsxAttributeValueData::JsxExpressionContainer(unsafe {
                    <JsxExpressionContainer as CommonAstNode>::from_ast_id_unchecked(node.0)
                })
            }
            NodeKind::JsxElement => JsxAttributeValueData::JsxElement(unsafe {
                <JsxElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => JsxAttributeValueData::JsxFragment(unsafe {
                <JsxFragment as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_expression_data(&self, node: JsxExpression) -> JsxExpressionData {
        match self.node_kind(node.0) {
            NodeKind::IdentifierReference => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ThisExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Super => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MetaProperty => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::StringLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NumericLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BigIntLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BooleanLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NullLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::RegExpLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TemplateLiteral => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::BinaryExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::LogicalExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UnaryExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::UpdateExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AssignmentExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ConditionalExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::SequenceExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::MemberExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::CallExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ChainExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::NewExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TaggedTemplateExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrowFunctionExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ArrayExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ObjectExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ParenthesizedExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Function => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::Class => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::ImportExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::AwaitExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::YieldExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::PrivateIdentifier => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsAsExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsSatisfiesExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsTypeAssertion => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsNonNullExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::TsInstantiationExpression => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => JsxExpressionData::Expr(unsafe {
                <Expr as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxEmptyExpression => JsxExpressionData::JsxEmptyExpression(unsafe {
                <JsxEmptyExpression as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
    #[inline]
    pub fn jsx_child_data(&self, node: JsxChild) -> JsxChildData {
        let kind = self.node_kind(node.0);
        debug_assert!(kind.is_jsx_child());
        match kind {
            NodeKind::JsxText => JsxChildData::JsxText(unsafe {
                <JsxText as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxExpressionContainer => JsxChildData::JsxExpressionContainer(unsafe {
                <JsxExpressionContainer as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxSpreadChild => JsxChildData::JsxSpreadChild(unsafe {
                <JsxSpreadChild as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxElement => JsxChildData::JsxElement(unsafe {
                <JsxElement as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            NodeKind::JsxFragment => JsxChildData::JsxFragment(unsafe {
                <JsxFragment as CommonAstNode>::from_ast_id_unchecked(node.0)
            }),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
