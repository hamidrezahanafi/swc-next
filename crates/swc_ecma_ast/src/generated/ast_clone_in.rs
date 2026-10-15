#![allow(unused, clippy::manual_map)]
use crate::{Ast, CloneIn, GetSpan, NodeKind};
use crate::{ast::*, node_id::*};
impl<'a, 'id> CloneIn<'a, 'id> for Program<'id> {
    type Cloned = Program<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let hashbang = self.hashbang(ast).clone_in(ast);
        ast.program(span, directives, body, hashbang)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Stmt<'id> {
    type Cloned = Stmt<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::BlockStatement => Self::BlockStatement(BlockStatement(self.0).clone_in(ast)),
            NodeKind::BreakStatement => Self::BreakStatement(BreakStatement(self.0).clone_in(ast)),
            NodeKind::ContinueStatement => {
                Self::ContinueStatement(ContinueStatement(self.0).clone_in(ast))
            }
            NodeKind::DebuggerStatement => {
                Self::DebuggerStatement(DebuggerStatement(self.0).clone_in(ast))
            }
            NodeKind::DoWhileStatement => {
                Self::DoWhileStatement(DoWhileStatement(self.0).clone_in(ast))
            }
            NodeKind::EmptyStatement => Self::EmptyStatement(EmptyStatement(self.0).clone_in(ast)),
            NodeKind::ExpressionStatement => {
                Self::ExpressionStatement(ExpressionStatement(self.0).clone_in(ast))
            }
            NodeKind::ForInStatement => Self::ForInStatement(ForInStatement(self.0).clone_in(ast)),
            NodeKind::ForOfStatement => Self::ForOfStatement(ForOfStatement(self.0).clone_in(ast)),
            NodeKind::ForStatement => Self::ForStatement(ForStatement(self.0).clone_in(ast)),
            NodeKind::IfStatement => Self::IfStatement(IfStatement(self.0).clone_in(ast)),
            NodeKind::LabeledStatement => {
                Self::LabeledStatement(LabeledStatement(self.0).clone_in(ast))
            }
            NodeKind::ReturnStatement => {
                Self::ReturnStatement(ReturnStatement(self.0).clone_in(ast))
            }
            NodeKind::SwitchStatement => {
                Self::SwitchStatement(SwitchStatement(self.0).clone_in(ast))
            }
            NodeKind::ThrowStatement => Self::ThrowStatement(ThrowStatement(self.0).clone_in(ast)),
            NodeKind::TryStatement => Self::TryStatement(TryStatement(self.0).clone_in(ast)),
            NodeKind::WhileStatement => Self::WhileStatement(WhileStatement(self.0).clone_in(ast)),
            NodeKind::WithStatement => Self::WithStatement(WithStatement(self.0).clone_in(ast)),
            NodeKind::VariableDeclaration
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::TsFunction
            | NodeKind::TsTypeAliasDeclaration
            | NodeKind::TsInterfaceDeclaration
            | NodeKind::TsEnumDeclaration
            | NodeKind::TsModuleDeclaration
            | NodeKind::TsGlobalDeclaration
            | NodeKind::TsImportEqualsDeclaration => Self::Declaration(Decl(self.0).clone_in(ast)),
            NodeKind::ImportDeclaration => {
                Self::ImportDeclaration(ImportDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportNamedDeclaration => {
                Self::ExportNamedDeclaration(ExportNamedDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportDefaultDeclaration => {
                Self::ExportDefaultDeclaration(ExportDefaultDeclaration(self.0).clone_in(ast))
            }
            NodeKind::ExportAllDeclaration => {
                Self::ExportAllDeclaration(ExportAllDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsExportAssignment => {
                Self::TsExportAssignment(TsExportAssignment(self.0).clone_in(ast))
            }
            NodeKind::TsNamespaceExportDeclaration => Self::TsNamespaceExportDeclaration(
                TsNamespaceExportDeclaration(self.0).clone_in(ast),
            ),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Decl<'id> {
    type Cloned = Decl<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::Function => Self::Function(Function(self.0).clone_in(ast)),
            NodeKind::Class => Self::Class(Class(self.0).clone_in(ast)),
            NodeKind::TsFunction => Self::TsFunction(TsFunction(self.0).clone_in(ast)),
            NodeKind::TsTypeAliasDeclaration => {
                Self::TsTypeAliasDeclaration(TsTypeAliasDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsInterfaceDeclaration => {
                Self::TsInterfaceDeclaration(TsInterfaceDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsEnumDeclaration => {
                Self::TsEnumDeclaration(TsEnumDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsModuleDeclaration => {
                Self::TsModuleDeclaration(TsModuleDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsGlobalDeclaration => {
                Self::TsGlobalDeclaration(TsGlobalDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsImportEqualsDeclaration => {
                Self::TsImportEqualsDeclaration(TsImportEqualsDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Expr<'id> {
    type Cloned = Expr<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::Super => Self::Super(Super(self.0).clone_in(ast)),
            NodeKind::MetaProperty => Self::MetaProperty(MetaProperty(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::NumericLiteral => Self::NumericLiteral(NumericLiteral(self.0).clone_in(ast)),
            NodeKind::BigIntLiteral => Self::BigIntLiteral(BigIntLiteral(self.0).clone_in(ast)),
            NodeKind::BooleanLiteral => Self::BooleanLiteral(BooleanLiteral(self.0).clone_in(ast)),
            NodeKind::NullLiteral => Self::NullLiteral(NullLiteral(self.0).clone_in(ast)),
            NodeKind::RegExpLiteral => Self::RegExpLiteral(RegExpLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            NodeKind::BinaryExpression => {
                Self::BinaryExpression(BinaryExpression(self.0).clone_in(ast))
            }
            NodeKind::LogicalExpression => {
                Self::LogicalExpression(LogicalExpression(self.0).clone_in(ast))
            }
            NodeKind::UnaryExpression => {
                Self::UnaryExpression(UnaryExpression(self.0).clone_in(ast))
            }
            NodeKind::UpdateExpression => {
                Self::UpdateExpression(UpdateExpression(self.0).clone_in(ast))
            }
            NodeKind::AssignmentExpression => {
                Self::AssignmentExpression(AssignmentExpression(self.0).clone_in(ast))
            }
            NodeKind::ConditionalExpression => {
                Self::ConditionalExpression(ConditionalExpression(self.0).clone_in(ast))
            }
            NodeKind::SequenceExpression => {
                Self::SequenceExpression(SequenceExpression(self.0).clone_in(ast))
            }
            NodeKind::MemberExpression => {
                Self::MemberExpression(MemberExpression(self.0).clone_in(ast))
            }
            NodeKind::CallExpression => Self::CallExpression(CallExpression(self.0).clone_in(ast)),
            NodeKind::ChainExpression => {
                Self::ChainExpression(ChainExpression(self.0).clone_in(ast))
            }
            NodeKind::NewExpression => Self::NewExpression(NewExpression(self.0).clone_in(ast)),
            NodeKind::TaggedTemplateExpression => {
                Self::TaggedTemplateExpression(TaggedTemplateExpression(self.0).clone_in(ast))
            }
            NodeKind::ArrowFunctionExpression => {
                Self::ArrowFunctionExpression(ArrowFunctionExpression(self.0).clone_in(ast))
            }
            NodeKind::ArrayExpression => {
                Self::ArrayExpression(ArrayExpression(self.0).clone_in(ast))
            }
            NodeKind::ObjectExpression => {
                Self::ObjectExpression(ObjectExpression(self.0).clone_in(ast))
            }
            NodeKind::ParenthesizedExpression => {
                Self::ParenthesizedExpression(ParenthesizedExpression(self.0).clone_in(ast))
            }
            NodeKind::Function => Self::Function(Function(self.0).clone_in(ast)),
            NodeKind::Class => Self::Class(Class(self.0).clone_in(ast)),
            NodeKind::ImportExpression => {
                Self::ImportExpression(ImportExpression(self.0).clone_in(ast))
            }
            NodeKind::AwaitExpression => {
                Self::AwaitExpression(AwaitExpression(self.0).clone_in(ast))
            }
            NodeKind::YieldExpression => {
                Self::YieldExpression(YieldExpression(self.0).clone_in(ast))
            }
            NodeKind::PrivateIdentifier => {
                Self::PrivateIdentifier(PrivateIdentifier(self.0).clone_in(ast))
            }
            NodeKind::TsAsExpression => Self::TsAsExpression(TsAsExpression(self.0).clone_in(ast)),
            NodeKind::TsSatisfiesExpression => {
                Self::TsSatisfiesExpression(TsSatisfiesExpression(self.0).clone_in(ast))
            }
            NodeKind::TsTypeAssertion => {
                Self::TsTypeAssertion(TsTypeAssertion(self.0).clone_in(ast))
            }
            NodeKind::TsNonNullExpression => {
                Self::TsNonNullExpression(TsNonNullExpression(self.0).clone_in(ast))
            }
            NodeKind::TsInstantiationExpression => {
                Self::TsInstantiationExpression(TsInstantiationExpression(self.0).clone_in(ast))
            }
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BindingPattern<'id> {
    type Cloned = BindingPattern<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression => {
                Self::SimpleAssignmentTarget(SimpleAssignmentTarget(self.0).clone_in(ast))
            }
            NodeKind::AssignmentPattern => {
                Self::AssignmentPattern(AssignmentPattern(self.0).clone_in(ast))
            }
            NodeKind::BindingRestElement => {
                Self::BindingRestElement(BindingRestElement(self.0).clone_in(ast))
            }
            NodeKind::ArrayPattern => Self::ArrayPattern(ArrayPattern(self.0).clone_in(ast)),
            NodeKind::ObjectPattern => Self::ObjectPattern(ObjectPattern(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for AssignmentTarget<'id> {
    type Cloned = AssignmentTarget<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression => {
                Self::SimpleAssignmentTarget(SimpleAssignmentTarget(self.0).clone_in(ast))
            }
            NodeKind::ArrayPattern => {
                Self::ArrayAssignmentTarget(ArrayPattern(self.0).clone_in(ast))
            }
            NodeKind::ObjectPattern => {
                Self::ObjectAssignmentTarget(ObjectPattern(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for SimpleAssignmentTarget<'id> {
    type Cloned = SimpleAssignmentTarget<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::MemberExpression => {
                Self::MemberExpression(MemberExpression(self.0).clone_in(ast))
            }
            NodeKind::TsAsExpression => Self::TsAsExpression(TsAsExpression(self.0).clone_in(ast)),
            NodeKind::TsSatisfiesExpression => {
                Self::TsSatisfiesExpression(TsSatisfiesExpression(self.0).clone_in(ast))
            }
            NodeKind::TsTypeAssertion => {
                Self::TsTypeAssertion(TsTypeAssertion(self.0).clone_in(ast))
            }
            NodeKind::TsNonNullExpression => {
                Self::TsNonNullExpression(TsNonNullExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for PropertyKey<'id> {
    type Cloned = PropertyKey<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ModuleExportName<'id> {
    type Cloned = ModuleExportName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Argument<'id> {
    type Cloned = Argument<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::SpreadElement => Self::SpreadElement(SpreadElement(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ObjectPropertyKind<'id> {
    type Cloned = ObjectPropertyKind<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::ObjectProperty => Self::ObjectProperty(ObjectProperty(self.0).clone_in(ast)),
            NodeKind::SpreadElement => Self::SpreadElement(SpreadElement(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ForStatementInit<'id> {
    type Cloned = ForStatementInit<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ForStatementLeft<'id> {
    type Cloned = ForStatementLeft<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::VariableDeclaration => {
                Self::VariableDeclaration(VariableDeclaration(self.0).clone_in(ast))
            }
            NodeKind::IdentifierReference
            | NodeKind::MemberExpression
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::ArrayPattern
            | NodeKind::ObjectPattern => {
                Self::AssignmentTarget(AssignmentTarget(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ArrowFunctionBody<'id> {
    type Cloned = ArrowFunctionBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::FunctionBody => Self::FunctionBody(FunctionBody(self.0).clone_in(ast)),
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FormalParameterItem<'id> {
    type Cloned = FormalParameterItem<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::FormalParameter => {
                Self::FormalParameter(FormalParameter(self.0).clone_in(ast))
            }
            NodeKind::TsParameterProperty => {
                Self::TsParameterProperty(TsParameterProperty(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FormalParameterPattern<'id> {
    type Cloned = FormalParameterPattern<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::ArrayPattern => Self::ArrayPattern(ArrayPattern(self.0).clone_in(ast)),
            NodeKind::ObjectPattern => Self::ObjectPattern(ObjectPattern(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ClassElement<'id> {
    type Cloned = ClassElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::MethodDefinition => {
                Self::MethodDefinition(MethodDefinition(self.0).clone_in(ast))
            }
            NodeKind::PropertyDefinition => {
                Self::PropertyDefinition(PropertyDefinition(self.0).clone_in(ast))
            }
            NodeKind::StaticBlock => Self::StaticBlock(StaticBlock(self.0).clone_in(ast)),
            NodeKind::TsMethodDefinition => {
                Self::TsMethodDefinition(TsMethodDefinition(self.0).clone_in(ast))
            }
            NodeKind::TsIndexSignature => {
                Self::TsIndexSignature(TsIndexSignature(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportDeclarationSpecifier<'id> {
    type Cloned = ImportDeclarationSpecifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::ImportSpecifier => {
                Self::ImportSpecifier(ImportSpecifier(self.0).clone_in(ast))
            }
            NodeKind::ImportDefaultSpecifier => {
                Self::ImportDefaultSpecifier(ImportDefaultSpecifier(self.0).clone_in(ast))
            }
            NodeKind::ImportNamespaceSpecifier => {
                Self::ImportNamespaceSpecifier(ImportNamespaceSpecifier(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExportDefaultDeclarationKind<'id> {
    type Cloned = ExportDefaultDeclarationKind<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::Function => {
                let concrete = Function(self.0);
                if concrete.function_type(ast) == FunctionType::FunctionDeclaration {
                    Self::Function(concrete.clone_in(ast))
                } else {
                    Self::Expr(Expr(self.0).clone_in(ast))
                }
            }
            NodeKind::Class => {
                let concrete = Class(self.0);
                if concrete.class_type(ast) == ClassType::ClassDeclaration {
                    Self::Class(concrete.clone_in(ast))
                } else {
                    Self::Expr(Expr(self.0).clone_in(ast))
                }
            }
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::TsFunction => Self::TsFunction(TsFunction(self.0).clone_in(ast)),
            NodeKind::TsInterfaceDeclaration => {
                Self::TsInterfaceDeclaration(TsInterfaceDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Super<'id> {
    type Cloned = Super<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.super_(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ThisExpression<'id> {
    type Cloned = ThisExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.this_expression(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for StringLiteral<'id> {
    type Cloned = StringLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.string_literal(span, value, raw)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for NumericLiteral<'id> {
    type Cloned = NumericLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.numeric_literal(span, value, raw)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BigIntLiteral<'id> {
    type Cloned = BigIntLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        ast.big_int_literal(span, raw)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BooleanLiteral<'id> {
    type Cloned = BooleanLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.boolean_literal(span, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for NullLiteral<'id> {
    type Cloned = NullLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.null_literal(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for RegExpLiteral<'id> {
    type Cloned = RegExpLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let pattern = self.pattern(ast).clone_in(ast);
        let flags = self.flags(ast).clone_in(ast);
        ast.reg_exp_literal(span, pattern, flags)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TemplateLiteral<'id> {
    type Cloned = TemplateLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        let expressions = self.expressions(ast).clone_in(ast);
        ast.template_literal(span, quasis, expressions)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TemplateElement<'id> {
    type Cloned = TemplateElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let cooked = self.cooked(ast).clone_in(ast);
        let raw = self.raw(ast).clone_in(ast);
        let tail = self.tail(ast).clone_in(ast);
        let is_cooked_undefined = self.is_cooked_undefined(ast).clone_in(ast);
        ast.template_element(span, cooked, raw, tail, is_cooked_undefined)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for IdentifierReference<'id> {
    type Cloned = IdentifierReference<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.identifier_reference(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for PrivateIdentifier<'id> {
    type Cloned = PrivateIdentifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.private_identifier(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BindingIdentifier<'id> {
    type Cloned = BindingIdentifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.binding_identifier(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for IdentifierName<'id> {
    type Cloned = IdentifierName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.identifier_name(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for LabelIdentifier<'id> {
    type Cloned = LabelIdentifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.label_identifier(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for AssignmentPattern<'id> {
    type Cloned = AssignmentPattern<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.assignment_pattern(span, left, right)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BindingRestElement<'id> {
    type Cloned = BindingRestElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.binding_rest_element(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ArrayPattern<'id> {
    type Cloned = ArrayPattern<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elements = self.elements(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        ast.array_pattern(span, elements, rest)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ObjectPattern<'id> {
    type Cloned = ObjectPattern<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let properties = self.properties(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        ast.object_pattern(span, properties, rest)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BindingProperty<'id> {
    type Cloned = BindingProperty<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let shorthand = self.shorthand(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.binding_property(span, key, value, shorthand, computed)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BinaryExpression<'id> {
    type Cloned = BinaryExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.binary_expression(span, left, right, operator)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for LogicalExpression<'id> {
    type Cloned = LogicalExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.logical_expression(span, left, right, operator)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ConditionalExpression<'id> {
    type Cloned = ConditionalExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        let alternate = self.alternate(ast).clone_in(ast);
        ast.conditional_expression(span, test, consequent, alternate)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for UnaryExpression<'id> {
    type Cloned = UnaryExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.unary_expression(span, argument, operator)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for UpdateExpression<'id> {
    type Cloned = UpdateExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        let prefix = self.prefix(ast).clone_in(ast);
        ast.update_expression(span, argument, operator, prefix)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for AssignmentExpression<'id> {
    type Cloned = AssignmentExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        ast.assignment_expression(span, left, right, operator)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for VariableDeclaration<'id> {
    type Cloned = VariableDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declarators = self.declarators(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.variable_declaration(span, kind, declarators, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for VariableDeclarator<'id> {
    type Cloned = VariableDeclarator<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let definite = self.definite(ast).clone_in(ast);
        ast.variable_declarator(span, id, type_annotation, init, definite)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for IfStatement<'id> {
    type Cloned = IfStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        let alternate = self.alternate(ast).clone_in(ast);
        ast.if_statement(span, test, consequent, alternate)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for SwitchStatement<'id> {
    type Cloned = SwitchStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let discriminant = self.discriminant(ast).clone_in(ast);
        let cases = self.cases(ast).clone_in(ast);
        ast.switch_statement(span, discriminant, cases)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for SwitchCase<'id> {
    type Cloned = SwitchCase<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let consequent = self.consequent(ast).clone_in(ast);
        ast.switch_case(span, test, consequent)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ForStatement<'id> {
    type Cloned = ForStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let init = self.init(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let update = self.update(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_statement(span, init, test, update, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ForInStatement<'id> {
    type Cloned = ForInStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.for_in_statement(span, left, right, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ForOfStatement<'id> {
    type Cloned = ForOfStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let r#await = self.r#await(ast).clone_in(ast);
        ast.for_of_statement(span, left, right, body, r#await)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TryStatement<'id> {
    type Cloned = TryStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let block = self.block(ast).clone_in(ast);
        let handler = self.handler(ast).clone_in(ast);
        let finalizer = self.finalizer(ast).clone_in(ast);
        ast.try_statement(span, block, handler, finalizer)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for CatchClause<'id> {
    type Cloned = CatchClause<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let param = self.param(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.catch_clause(span, param, type_annotation, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BreakStatement<'id> {
    type Cloned = BreakStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.break_statement(span, label)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ContinueStatement<'id> {
    type Cloned = ContinueStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        ast.continue_statement(span, label)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for LabeledStatement<'id> {
    type Cloned = LabeledStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.labeled_statement(span, label, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExpressionStatement<'id> {
    type Cloned = ExpressionStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.expression_statement(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ReturnStatement<'id> {
    type Cloned = ReturnStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.return_statement(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ThrowStatement<'id> {
    type Cloned = ThrowStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.throw_statement(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for DebuggerStatement<'id> {
    type Cloned = DebuggerStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.debugger_statement(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for EmptyStatement<'id> {
    type Cloned = EmptyStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.empty_statement(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Directive<'id> {
    type Cloned = Directive<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.directive(span, expression, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Decorator<'id> {
    type Cloned = Decorator<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.decorator(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Class<'id> {
    type Cloned = Class<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let class_type = self.class_type(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let super_class = self.super_class(ast).clone_in(ast);
        let super_type_arguments = self.super_type_arguments(ast).clone_in(ast);
        let implements = self.implements(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.class(
            span,
            class_type,
            decorators,
            id,
            type_parameters,
            super_class,
            super_type_arguments,
            implements,
            body,
            r#abstract,
            declare,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ClassBody<'id> {
    type Cloned = ClassBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.class_body(span, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for MethodDefinition<'id> {
    type Cloned = MethodDefinition<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.method_definition(
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for PropertyDefinition<'id> {
    type Cloned = PropertyDefinition<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let accessor = self.accessor(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let definite = self.definite(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.property_definition(
            span,
            decorators,
            key,
            type_annotation,
            value,
            computed,
            r#static,
            accessor,
            declare,
            r#override,
            optional,
            definite,
            readonly,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for StaticBlock<'id> {
    type Cloned = StaticBlock<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.static_block(span, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Function<'id> {
    type Cloned = Function<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let function_type = self.function_type(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        ast.function(
            span,
            function_type,
            id,
            type_parameters,
            this_param,
            params,
            return_type,
            body,
            generator,
            r#async,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FunctionBody<'id> {
    type Cloned = FunctionBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.function_body(span, directives, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for BlockStatement<'id> {
    type Cloned = BlockStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.block_statement(span, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for WhileStatement<'id> {
    type Cloned = WhileStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.while_statement(span, test, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for DoWhileStatement<'id> {
    type Cloned = DoWhileStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let test = self.test(ast).clone_in(ast);
        ast.do_while_statement(span, body, test)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for WithStatement<'id> {
    type Cloned = WithStatement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.with_statement(span, object, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FormalParameters<'id> {
    type Cloned = FormalParameters<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let items = self.items(ast).clone_in(ast);
        let rest = self.rest(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        ast.formal_parameters(span, items, rest, kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FormalParameter<'id> {
    type Cloned = FormalParameter<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let pattern = self.pattern(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let initializer = self.initializer(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.formal_parameter(
            span,
            decorators,
            pattern,
            type_annotation,
            initializer,
            optional,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for FormalParameterRest<'id> {
    type Cloned = FormalParameterRest<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.formal_parameter_rest(span, decorators, argument, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ArrayExpression<'id> {
    type Cloned = ArrayExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let elements = self.elements(ast).clone_in(ast);
        ast.array_expression(span, elements)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ObjectExpression<'id> {
    type Cloned = ObjectExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let properties = self.properties(ast).clone_in(ast);
        ast.object_expression(span, properties)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for SpreadElement<'id> {
    type Cloned = SpreadElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.spread_element(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ObjectProperty<'id> {
    type Cloned = ObjectProperty<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let method = self.method(ast).clone_in(ast);
        let shorthand = self.shorthand(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.object_property(span, key, value, kind, method, shorthand, computed)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ParenthesizedExpression<'id> {
    type Cloned = ParenthesizedExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.parenthesized_expression(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ArrowFunctionExpression<'id> {
    type Cloned = ArrowFunctionExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.arrow_function_expression(
            span,
            expression,
            r#async,
            type_parameters,
            params,
            return_type,
            body,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for SequenceExpression<'id> {
    type Cloned = SequenceExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expressions = self.expressions(ast).clone_in(ast);
        ast.sequence_expression(span, expressions)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for MemberExpression<'id> {
    type Cloned = MemberExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.member_expression(span, object, property, computed, optional)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for CallExpression<'id> {
    type Cloned = CallExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let arguments = self.arguments(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.call_expression(span, callee, type_arguments, arguments, optional)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ChainExpression<'id> {
    type Cloned = ChainExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.chain_expression(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TaggedTemplateExpression<'id> {
    type Cloned = TaggedTemplateExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let tag = self.tag(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let quasi = self.quasi(ast).clone_in(ast);
        ast.tagged_template_expression(span, tag, type_arguments, quasi)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for NewExpression<'id> {
    type Cloned = NewExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let callee = self.callee(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let arguments = self.arguments(ast).clone_in(ast);
        ast.new_expression(span, callee, type_arguments, arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for AwaitExpression<'id> {
    type Cloned = AwaitExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.await_expression(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for YieldExpression<'id> {
    type Cloned = YieldExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        let delegate = self.delegate(ast).clone_in(ast);
        ast.yield_expression(span, argument, delegate)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for MetaProperty<'id> {
    type Cloned = MetaProperty<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let meta = self.meta(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        ast.meta_property(span, meta, property)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportExpression<'id> {
    type Cloned = ImportExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let options = self.options(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        let sync = self.sync(ast).clone_in(ast);
        ast.import_expression(span, source, options, phase, sync)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportDeclaration<'id> {
    type Cloned = ImportDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let phase = self.phase(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.import_declaration(span, specifiers, source, attributes, phase, import_kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportSpecifier<'id> {
    type Cloned = ImportSpecifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let imported = self.imported(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.import_specifier(span, imported, local, import_kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportDefaultSpecifier<'id> {
    type Cloned = ImportDefaultSpecifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_default_specifier(span, local)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportNamespaceSpecifier<'id> {
    type Cloned = ImportNamespaceSpecifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        ast.import_namespace_specifier(span, local)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ImportAttribute<'id> {
    type Cloned = ImportAttribute<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.import_attribute(span, key, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExportNamedDeclaration<'id> {
    type Cloned = ExportNamedDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declaration = self.declaration(ast).clone_in(ast);
        let specifiers = self.specifiers(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_named_declaration(
            span,
            declaration,
            specifiers,
            source,
            attributes,
            export_kind,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExportDefaultDeclaration<'id> {
    type Cloned = ExportDefaultDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let declaration = self.declaration(ast).clone_in(ast);
        ast.export_default_declaration(span, declaration)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExportAllDeclaration<'id> {
    type Cloned = ExportAllDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_all_declaration(span, exported, source, attributes, export_kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for ExportSpecifier<'id> {
    type Cloned = ExportSpecifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let local = self.local(ast).clone_in(ast);
        let exported = self.exported(ast).clone_in(ast);
        let export_kind = self.export_kind(ast).clone_in(ast);
        ast.export_specifier(span, local, exported, export_kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for Hashbang<'id> {
    type Cloned = Hashbang<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.hashbang(span, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsType<'id> {
    type Cloned = TsType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::TsAnyKeyword => Self::TsAnyKeyword(TsAnyKeyword(self.0).clone_in(ast)),
            NodeKind::TsUnknownKeyword => {
                Self::TsUnknownKeyword(TsUnknownKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsNeverKeyword => Self::TsNeverKeyword(TsNeverKeyword(self.0).clone_in(ast)),
            NodeKind::TsVoidKeyword => Self::TsVoidKeyword(TsVoidKeyword(self.0).clone_in(ast)),
            NodeKind::TsNullKeyword => Self::TsNullKeyword(TsNullKeyword(self.0).clone_in(ast)),
            NodeKind::TsUndefinedKeyword => {
                Self::TsUndefinedKeyword(TsUndefinedKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsStringKeyword => {
                Self::TsStringKeyword(TsStringKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsNumberKeyword => {
                Self::TsNumberKeyword(TsNumberKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsBigIntKeyword => {
                Self::TsBigIntKeyword(TsBigIntKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsBooleanKeyword => {
                Self::TsBooleanKeyword(TsBooleanKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsSymbolKeyword => {
                Self::TsSymbolKeyword(TsSymbolKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsObjectKeyword => {
                Self::TsObjectKeyword(TsObjectKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsIntrinsicKeyword => {
                Self::TsIntrinsicKeyword(TsIntrinsicKeyword(self.0).clone_in(ast))
            }
            NodeKind::TsThisType => Self::TsThisType(TsThisType(self.0).clone_in(ast)),
            NodeKind::TsTypeReference => {
                Self::TsTypeReference(TsTypeReference(self.0).clone_in(ast))
            }
            NodeKind::TsTypeQuery => Self::TsTypeQuery(TsTypeQuery(self.0).clone_in(ast)),
            NodeKind::TsImportType => Self::TsImportType(TsImportType(self.0).clone_in(ast)),
            NodeKind::TsLiteralType => Self::TsLiteralType(TsLiteralType(self.0).clone_in(ast)),
            NodeKind::TsTemplateLiteralType => {
                Self::TsTemplateLiteralType(TsTemplateLiteralType(self.0).clone_in(ast))
            }
            NodeKind::TsArrayType => Self::TsArrayType(TsArrayType(self.0).clone_in(ast)),
            NodeKind::TsIndexedAccessType => {
                Self::TsIndexedAccessType(TsIndexedAccessType(self.0).clone_in(ast))
            }
            NodeKind::TsTupleType => Self::TsTupleType(TsTupleType(self.0).clone_in(ast)),
            NodeKind::TsOptionalType => Self::TsOptionalType(TsOptionalType(self.0).clone_in(ast)),
            NodeKind::TsRestType => Self::TsRestType(TsRestType(self.0).clone_in(ast)),
            NodeKind::TsJsDocNullableType => {
                Self::TsJsDocNullableType(TsJsDocNullableType(self.0).clone_in(ast))
            }
            NodeKind::TsJsDocNonNullableType => {
                Self::TsJsDocNonNullableType(TsJsDocNonNullableType(self.0).clone_in(ast))
            }
            NodeKind::TsJsDocUnknownType => {
                Self::TsJsDocUnknownType(TsJsDocUnknownType(self.0).clone_in(ast))
            }
            NodeKind::TsUnionType => Self::TsUnionType(TsUnionType(self.0).clone_in(ast)),
            NodeKind::TsIntersectionType => {
                Self::TsIntersectionType(TsIntersectionType(self.0).clone_in(ast))
            }
            NodeKind::TsConditionalType => {
                Self::TsConditionalType(TsConditionalType(self.0).clone_in(ast))
            }
            NodeKind::TsInferType => Self::TsInferType(TsInferType(self.0).clone_in(ast)),
            NodeKind::TsTypeOperator => Self::TsTypeOperator(TsTypeOperator(self.0).clone_in(ast)),
            NodeKind::TsParenthesizedType => {
                Self::TsParenthesizedType(TsParenthesizedType(self.0).clone_in(ast))
            }
            NodeKind::TsFunctionType => Self::TsFunctionType(TsFunctionType(self.0).clone_in(ast)),
            NodeKind::TsConstructorType => {
                Self::TsConstructorType(TsConstructorType(self.0).clone_in(ast))
            }
            NodeKind::TsTypePredicate => {
                Self::TsTypePredicate(TsTypePredicate(self.0).clone_in(ast))
            }
            NodeKind::TsTypeLiteral => Self::TsTypeLiteral(TsTypeLiteral(self.0).clone_in(ast)),
            NodeKind::TsMappedType => Self::TsMappedType(TsMappedType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeName<'id> {
    type Cloned = TsTypeName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsQualifiedNameLeft<'id> {
    type Cloned = TsQualifiedNameLeft<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference => {
                Self::IdentifierReference(IdentifierReference(self.0).clone_in(ast))
            }
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsImportTypeQualifier<'id> {
    type Cloned = TsImportTypeQualifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeQueryExprName<'id> {
    type Cloned = TsTypeQueryExprName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName => {
                Self::TsTypeName(TsTypeName(self.0).clone_in(ast))
            }
            NodeKind::ThisExpression => Self::ThisExpression(ThisExpression(self.0).clone_in(ast)),
            NodeKind::TsImportType => Self::TsImportType(TsImportType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsLiteral<'id> {
    type Cloned = TsLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::NumericLiteral => Self::NumericLiteral(NumericLiteral(self.0).clone_in(ast)),
            NodeKind::BigIntLiteral => Self::BigIntLiteral(BigIntLiteral(self.0).clone_in(ast)),
            NodeKind::BooleanLiteral => Self::BooleanLiteral(BooleanLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            NodeKind::UnaryExpression => {
                Self::UnaryExpression(UnaryExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTupleElement<'id> {
    type Cloned = TsTupleElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::TsAnyKeyword
            | NodeKind::TsUnknownKeyword
            | NodeKind::TsNeverKeyword
            | NodeKind::TsVoidKeyword
            | NodeKind::TsNullKeyword
            | NodeKind::TsUndefinedKeyword
            | NodeKind::TsStringKeyword
            | NodeKind::TsNumberKeyword
            | NodeKind::TsBigIntKeyword
            | NodeKind::TsBooleanKeyword
            | NodeKind::TsSymbolKeyword
            | NodeKind::TsObjectKeyword
            | NodeKind::TsIntrinsicKeyword
            | NodeKind::TsThisType
            | NodeKind::TsTypeReference
            | NodeKind::TsTypeQuery
            | NodeKind::TsImportType
            | NodeKind::TsLiteralType
            | NodeKind::TsTemplateLiteralType
            | NodeKind::TsArrayType
            | NodeKind::TsIndexedAccessType
            | NodeKind::TsTupleType
            | NodeKind::TsOptionalType
            | NodeKind::TsRestType
            | NodeKind::TsJsDocNullableType
            | NodeKind::TsJsDocNonNullableType
            | NodeKind::TsJsDocUnknownType
            | NodeKind::TsUnionType
            | NodeKind::TsIntersectionType
            | NodeKind::TsConditionalType
            | NodeKind::TsInferType
            | NodeKind::TsTypeOperator
            | NodeKind::TsParenthesizedType
            | NodeKind::TsFunctionType
            | NodeKind::TsConstructorType
            | NodeKind::TsTypePredicate
            | NodeKind::TsTypeLiteral
            | NodeKind::TsMappedType => Self::TsType(TsType(self.0).clone_in(ast)),
            NodeKind::TsNamedTupleMember => {
                Self::TsNamedTupleMember(TsNamedTupleMember(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeElement<'id> {
    type Cloned = TsTypeElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::TsPropertySignature => {
                Self::TsPropertySignature(TsPropertySignature(self.0).clone_in(ast))
            }
            NodeKind::TsMethodSignature => {
                Self::TsMethodSignature(TsMethodSignature(self.0).clone_in(ast))
            }
            NodeKind::TsCallSignatureDeclaration => {
                Self::TsCallSignatureDeclaration(TsCallSignatureDeclaration(self.0).clone_in(ast))
            }
            NodeKind::TsConstructSignatureDeclaration => Self::TsConstructSignatureDeclaration(
                TsConstructSignatureDeclaration(self.0).clone_in(ast),
            ),
            NodeKind::TsIndexSignature => {
                Self::TsIndexSignature(TsIndexSignature(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypePredicateName<'id> {
    type Cloned = TsTypePredicateName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::TsThisType => Self::TsThisType(TsThisType(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsEnumMemberName<'id> {
    type Cloned = TsEnumMemberName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierName => Self::IdentifierName(IdentifierName(self.0).clone_in(ast)),
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::TemplateLiteral => {
                Self::TemplateLiteral(TemplateLiteral(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsModuleName<'id> {
    type Cloned = TsModuleName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::BindingIdentifier => {
                Self::BindingIdentifier(BindingIdentifier(self.0).clone_in(ast))
            }
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::TsQualifiedName => {
                Self::TsQualifiedName(TsQualifiedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsModuleBody<'id> {
    type Cloned = TsModuleBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::TsModuleBlock => Self::TsModuleBlock(TsModuleBlock(self.0).clone_in(ast)),
            NodeKind::TsModuleDeclaration => {
                Self::TsModuleDeclaration(TsModuleDeclaration(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsModuleReference<'id> {
    type Cloned = TsModuleReference<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName => {
                Self::TsTypeName(TsTypeName(self.0).clone_in(ast))
            }
            NodeKind::TsExternalModuleReference => {
                Self::TsExternalModuleReference(TsExternalModuleReference(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsFunction<'id> {
    type Cloned = TsFunction<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let generator = self.generator(ast).clone_in(ast);
        let r#async = self.r#async(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_function(
            span,
            id,
            type_parameters,
            this_param,
            params,
            return_type,
            generator,
            r#async,
            declare,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsMethodDefinition<'id> {
    type Cloned = TsMethodDefinition<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.ts_method_definition(
            span,
            decorators,
            key,
            value,
            kind,
            computed,
            r#static,
            r#override,
            optional,
            r#abstract,
            accessibility,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeAnnotation<'id> {
    type Cloned = TsTypeAnnotation<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_type_annotation(span, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsAnyKeyword<'id> {
    type Cloned = TsAnyKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_any_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsUnknownKeyword<'id> {
    type Cloned = TsUnknownKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_unknown_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNeverKeyword<'id> {
    type Cloned = TsNeverKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_never_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsVoidKeyword<'id> {
    type Cloned = TsVoidKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_void_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNullKeyword<'id> {
    type Cloned = TsNullKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_null_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsUndefinedKeyword<'id> {
    type Cloned = TsUndefinedKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_undefined_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsStringKeyword<'id> {
    type Cloned = TsStringKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_string_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNumberKeyword<'id> {
    type Cloned = TsNumberKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_number_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsBigIntKeyword<'id> {
    type Cloned = TsBigIntKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_big_int_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsBooleanKeyword<'id> {
    type Cloned = TsBooleanKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_boolean_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsSymbolKeyword<'id> {
    type Cloned = TsSymbolKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_symbol_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsObjectKeyword<'id> {
    type Cloned = TsObjectKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_object_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsIntrinsicKeyword<'id> {
    type Cloned = TsIntrinsicKeyword<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_intrinsic_keyword(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsThisType<'id> {
    type Cloned = TsThisType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_this_type(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeReference<'id> {
    type Cloned = TsTypeReference<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_name = self.type_name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_type_reference(span, type_name, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsQualifiedName<'id> {
    type Cloned = TsQualifiedName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let left = self.left(ast).clone_in(ast);
        let right = self.right(ast).clone_in(ast);
        ast.ts_qualified_name(span, left, right)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeQuery<'id> {
    type Cloned = TsTypeQuery<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expr_name = self.expr_name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_type_query(span, expr_name, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsImportType<'id> {
    type Cloned = TsImportType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let source = self.source(ast).clone_in(ast);
        let options = self.options(ast).clone_in(ast);
        let qualifier = self.qualifier(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_import_type(span, source, options, qualifier, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeParameter<'id> {
    type Cloned = TsTypeParameter<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let constraint = self.constraint(ast).clone_in(ast);
        let default = self.default(ast).clone_in(ast);
        let r#in = self.r#in(ast).clone_in(ast);
        let out = self.out(ast).clone_in(ast);
        let r#const = self.r#const(ast).clone_in(ast);
        ast.ts_type_parameter(span, name, constraint, default, r#in, out, r#const)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeParameterDeclaration<'id> {
    type Cloned = TsTypeParameterDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_parameter_declaration(span, params)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeParameterInstantiation<'id> {
    type Cloned = TsTypeParameterInstantiation<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        ast.ts_type_parameter_instantiation(span, params)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsLiteralType<'id> {
    type Cloned = TsLiteralType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let literal = self.literal(ast).clone_in(ast);
        ast.ts_literal_type(span, literal)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTemplateLiteralType<'id> {
    type Cloned = TsTemplateLiteralType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let quasis = self.quasis(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_template_literal_type(span, quasis, types)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsArrayType<'id> {
    type Cloned = TsArrayType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let element_type = self.element_type(ast).clone_in(ast);
        ast.ts_array_type(span, element_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsIndexedAccessType<'id> {
    type Cloned = TsIndexedAccessType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object_type = self.object_type(ast).clone_in(ast);
        let index_type = self.index_type(ast).clone_in(ast);
        ast.ts_indexed_access_type(span, object_type, index_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTupleType<'id> {
    type Cloned = TsTupleType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let element_types = self.element_types(ast).clone_in(ast);
        ast.ts_tuple_type(span, element_types)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNamedTupleMember<'id> {
    type Cloned = TsNamedTupleMember<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let label = self.label(ast).clone_in(ast);
        let element_type = self.element_type(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.ts_named_tuple_member(span, label, element_type, optional)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsOptionalType<'id> {
    type Cloned = TsOptionalType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_optional_type(span, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsRestType<'id> {
    type Cloned = TsRestType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_rest_type(span, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsJsDocNullableType<'id> {
    type Cloned = TsJsDocNullableType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let postfix = self.postfix(ast).clone_in(ast);
        ast.ts_js_doc_nullable_type(span, type_annotation, postfix)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsJsDocNonNullableType<'id> {
    type Cloned = TsJsDocNonNullableType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let postfix = self.postfix(ast).clone_in(ast);
        ast.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsJsDocUnknownType<'id> {
    type Cloned = TsJsDocUnknownType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.ts_js_doc_unknown_type(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsUnionType<'id> {
    type Cloned = TsUnionType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_union_type(span, types)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsIntersectionType<'id> {
    type Cloned = TsIntersectionType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let types = self.types(ast).clone_in(ast);
        ast.ts_intersection_type(span, types)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsConditionalType<'id> {
    type Cloned = TsConditionalType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let check_type = self.check_type(ast).clone_in(ast);
        let extends_type = self.extends_type(ast).clone_in(ast);
        let true_type = self.true_type(ast).clone_in(ast);
        let false_type = self.false_type(ast).clone_in(ast);
        ast.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsInferType<'id> {
    type Cloned = TsInferType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameter = self.type_parameter(ast).clone_in(ast);
        ast.ts_infer_type(span, type_parameter)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeOperator<'id> {
    type Cloned = TsTypeOperator<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let operator = self.operator(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_type_operator(span, operator, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsParenthesizedType<'id> {
    type Cloned = TsParenthesizedType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_parenthesized_type(span, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsFunctionType<'id> {
    type Cloned = TsFunctionType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_function_type(span, type_parameters, this_param, params, return_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsConstructorType<'id> {
    type Cloned = TsConstructorType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let r#abstract = self.r#abstract(ast).clone_in(ast);
        ast.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypePredicate<'id> {
    type Cloned = TsTypePredicate<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let parameter_name = self.parameter_name(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let asserts = self.asserts(ast).clone_in(ast);
        ast.ts_type_predicate(span, parameter_name, type_annotation, asserts)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeLiteral<'id> {
    type Cloned = TsTypeLiteral<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_type_literal(span, members)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsMappedType<'id> {
    type Cloned = TsMappedType<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let constraint = self.constraint(ast).clone_in(ast);
        let name_type = self.name_type(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        ast.ts_mapped_type(
            span,
            key,
            constraint,
            name_type,
            type_annotation,
            optional,
            readonly,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsPropertySignature<'id> {
    type Cloned = TsPropertySignature<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        ast.ts_property_signature(span, key, type_annotation, computed, optional, readonly)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsMethodSignature<'id> {
    type Cloned = TsMethodSignature<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let key = self.key(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        let optional = self.optional(ast).clone_in(ast);
        ast.ts_method_signature(
            span,
            key,
            type_parameters,
            this_param,
            params,
            return_type,
            kind,
            computed,
            optional,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsCallSignatureDeclaration<'id> {
    type Cloned = TsCallSignatureDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let this_param = self.this_param(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_call_signature_declaration(span, type_parameters, this_param, params, return_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsConstructSignatureDeclaration<'id> {
    type Cloned = TsConstructSignatureDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let params = self.params(ast).clone_in(ast);
        let return_type = self.return_type(ast).clone_in(ast);
        ast.ts_construct_signature_declaration(span, type_parameters, params, return_type)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsIndexSignature<'id> {
    type Cloned = TsIndexSignature<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let parameters = self.parameters(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let r#static = self.r#static(ast).clone_in(ast);
        ast.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeAliasDeclaration<'id> {
    type Cloned = TsTypeAliasDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsInterfaceDeclaration<'id> {
    type Cloned = TsInterfaceDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let type_parameters = self.type_parameters(ast).clone_in(ast);
        let extends = self.extends(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsInterfaceBody<'id> {
    type Cloned = TsInterfaceBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_interface_body(span, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsInterfaceHeritage<'id> {
    type Cloned = TsInterfaceHeritage<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_interface_heritage(span, expression, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsClassImplements<'id> {
    type Cloned = TsClassImplements<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_class_implements(span, expression, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsEnumDeclaration<'id> {
    type Cloned = TsEnumDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let is_const = self.is_const(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_enum_declaration(span, id, body, is_const, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsEnumBody<'id> {
    type Cloned = TsEnumBody<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let members = self.members(ast).clone_in(ast);
        ast.ts_enum_body(span, members)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsEnumMember<'id> {
    type Cloned = TsEnumMember<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let initializer = self.initializer(ast).clone_in(ast);
        let computed = self.computed(ast).clone_in(ast);
        ast.ts_enum_member(span, id, initializer, computed)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsModuleDeclaration<'id> {
    type Cloned = TsModuleDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let kind = self.kind(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_module_declaration(span, id, body, kind, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsModuleBlock<'id> {
    type Cloned = TsModuleBlock<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let directives = self.directives(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        ast.ts_module_block(span, directives, body)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsGlobalDeclaration<'id> {
    type Cloned = TsGlobalDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let body = self.body(ast).clone_in(ast);
        let declare = self.declare(ast).clone_in(ast);
        ast.ts_global_declaration(span, id, body, declare)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsParameterProperty<'id> {
    type Cloned = TsParameterProperty<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let decorators = self.decorators(ast).clone_in(ast);
        let parameter = self.parameter(ast).clone_in(ast);
        let r#override = self.r#override(ast).clone_in(ast);
        let readonly = self.readonly(ast).clone_in(ast);
        let accessibility = self.accessibility(ast).clone_in(ast);
        ast.ts_parameter_property(
            span,
            decorators,
            parameter,
            r#override,
            readonly,
            accessibility,
        )
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsThisParameter<'id> {
    type Cloned = TsThisParameter<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_this_parameter(span, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsAsExpression<'id> {
    type Cloned = TsAsExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_as_expression(span, expression, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsSatisfiesExpression<'id> {
    type Cloned = TsSatisfiesExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_satisfies_expression(span, expression, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsTypeAssertion<'id> {
    type Cloned = TsTypeAssertion<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_type_assertion(span, type_annotation, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNonNullExpression<'id> {
    type Cloned = TsNonNullExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_non_null_expression(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsInstantiationExpression<'id> {
    type Cloned = TsInstantiationExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        ast.ts_instantiation_expression(span, expression, type_arguments)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsExportAssignment<'id> {
    type Cloned = TsExportAssignment<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_export_assignment(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsNamespaceExportDeclaration<'id> {
    type Cloned = TsNamespaceExportDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        ast.ts_namespace_export_declaration(span, id)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsImportEqualsDeclaration<'id> {
    type Cloned = TsImportEqualsDeclaration<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let id = self.id(ast).clone_in(ast);
        let module_reference = self.module_reference(ast).clone_in(ast);
        let import_kind = self.import_kind(ast).clone_in(ast);
        ast.ts_import_equals_declaration(span, id, module_reference, import_kind)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsExternalModuleReference<'id> {
    type Cloned = TsExternalModuleReference<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.ts_external_module_reference(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for TsIndexSignatureParameter<'id> {
    type Cloned = TsIndexSignatureParameter<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let type_annotation = self.type_annotation(ast).clone_in(ast);
        ast.ts_index_signature_parameter(span, name, type_annotation)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxElementName<'id> {
    type Cloned = JsxElementName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxNamespacedName => {
                Self::JsxNamespacedName(JsxNamespacedName(self.0).clone_in(ast))
            }
            NodeKind::JsxMemberExpression => {
                Self::JsxMemberExpression(JsxMemberExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxMemberExpressionObject<'id> {
    type Cloned = JsxMemberExpressionObject<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxMemberExpression => {
                Self::JsxMemberExpression(JsxMemberExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxAttributeName<'id> {
    type Cloned = JsxAttributeName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::JsxIdentifier => Self::JsxIdentifier(JsxIdentifier(self.0).clone_in(ast)),
            NodeKind::JsxNamespacedName => {
                Self::JsxNamespacedName(JsxNamespacedName(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxAttributeItem<'id> {
    type Cloned = JsxAttributeItem<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::JsxAttribute => Self::JsxAttribute(JsxAttribute(self.0).clone_in(ast)),
            NodeKind::JsxSpreadAttribute => {
                Self::JsxSpreadAttribute(JsxSpreadAttribute(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxAttributeValue<'id> {
    type Cloned = JsxAttributeValue<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::StringLiteral => Self::StringLiteral(StringLiteral(self.0).clone_in(ast)),
            NodeKind::JsxExpressionContainer => {
                Self::JsxExpressionContainer(JsxExpressionContainer(self.0).clone_in(ast))
            }
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxExpression<'id> {
    type Cloned = JsxExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference
            | NodeKind::ThisExpression
            | NodeKind::Super
            | NodeKind::MetaProperty
            | NodeKind::StringLiteral
            | NodeKind::NumericLiteral
            | NodeKind::BigIntLiteral
            | NodeKind::BooleanLiteral
            | NodeKind::NullLiteral
            | NodeKind::RegExpLiteral
            | NodeKind::TemplateLiteral
            | NodeKind::BinaryExpression
            | NodeKind::LogicalExpression
            | NodeKind::UnaryExpression
            | NodeKind::UpdateExpression
            | NodeKind::AssignmentExpression
            | NodeKind::ConditionalExpression
            | NodeKind::SequenceExpression
            | NodeKind::MemberExpression
            | NodeKind::CallExpression
            | NodeKind::ChainExpression
            | NodeKind::NewExpression
            | NodeKind::TaggedTemplateExpression
            | NodeKind::ArrowFunctionExpression
            | NodeKind::ArrayExpression
            | NodeKind::ObjectExpression
            | NodeKind::ParenthesizedExpression
            | NodeKind::Function
            | NodeKind::Class
            | NodeKind::ImportExpression
            | NodeKind::AwaitExpression
            | NodeKind::YieldExpression
            | NodeKind::PrivateIdentifier
            | NodeKind::TsAsExpression
            | NodeKind::TsSatisfiesExpression
            | NodeKind::TsTypeAssertion
            | NodeKind::TsNonNullExpression
            | NodeKind::TsInstantiationExpression
            | NodeKind::JsxElement
            | NodeKind::JsxFragment => Self::Expr(Expr(self.0).clone_in(ast)),
            NodeKind::JsxEmptyExpression => {
                Self::JsxEmptyExpression(JsxEmptyExpression(self.0).clone_in(ast))
            }
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxChild<'id> {
    type Cloned = JsxChild<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::JsxText => Self::JsxText(JsxText(self.0).clone_in(ast)),
            NodeKind::JsxExpressionContainer => {
                Self::JsxExpressionContainer(JsxExpressionContainer(self.0).clone_in(ast))
            }
            NodeKind::JsxSpreadChild => Self::JsxSpreadChild(JsxSpreadChild(self.0).clone_in(ast)),
            NodeKind::JsxElement => Self::JsxElement(JsxElement(self.0).clone_in(ast)),
            NodeKind::JsxFragment => Self::JsxFragment(JsxFragment(self.0).clone_in(ast)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxElement<'id> {
    type Cloned = JsxElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening_element = self.opening_element(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing_element = self.closing_element(ast).clone_in(ast);
        ast.jsx_element(span, opening_element, children, closing_element)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxOpeningElement<'id> {
    type Cloned = JsxOpeningElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let type_arguments = self.type_arguments(ast).clone_in(ast);
        let attributes = self.attributes(ast).clone_in(ast);
        let self_closing = self.self_closing(ast).clone_in(ast);
        ast.jsx_opening_element(span, name, type_arguments, attributes, self_closing)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxClosingElement<'id> {
    type Cloned = JsxClosingElement<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_closing_element(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxFragment<'id> {
    type Cloned = JsxFragment<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let opening_fragment = self.opening_fragment(ast).clone_in(ast);
        let children = self.children(ast).clone_in(ast);
        let closing_fragment = self.closing_fragment(ast).clone_in(ast);
        ast.jsx_fragment(span, opening_fragment, children, closing_fragment)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxOpeningFragment<'id> {
    type Cloned = JsxOpeningFragment<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_opening_fragment(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxClosingFragment<'id> {
    type Cloned = JsxClosingFragment<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_closing_fragment(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxIdentifier<'id> {
    type Cloned = JsxIdentifier<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_identifier(span, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxNamespacedName<'id> {
    type Cloned = JsxNamespacedName<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let namespace = self.namespace(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        ast.jsx_namespaced_name(span, namespace, name)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxMemberExpression<'id> {
    type Cloned = JsxMemberExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let object = self.object(ast).clone_in(ast);
        let property = self.property(ast).clone_in(ast);
        ast.jsx_member_expression(span, object, property)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxAttribute<'id> {
    type Cloned = JsxAttribute<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let name = self.name(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_attribute(span, name, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxSpreadAttribute<'id> {
    type Cloned = JsxSpreadAttribute<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let argument = self.argument(ast).clone_in(ast);
        ast.jsx_spread_attribute(span, argument)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxExpressionContainer<'id> {
    type Cloned = JsxExpressionContainer<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.jsx_expression_container(span, expression)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxEmptyExpression<'id> {
    type Cloned = JsxEmptyExpression<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        ast.jsx_empty_expression(span)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxText<'id> {
    type Cloned = JsxText<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let value = self.value(ast).clone_in(ast);
        ast.jsx_text(span, value)
    }
}
impl<'a, 'id> CloneIn<'a, 'id> for JsxSpreadChild<'id> {
    type Cloned = JsxSpreadChild<'id>;
    #[inline]
    fn clone_in(&self, ast: &mut Ast<'a, 'id>) -> Self::Cloned {
        let span = self.span(ast).clone_in(ast);
        let expression = self.expression(ast).clone_in(ast);
        ast.jsx_spread_child(span, expression)
    }
}
