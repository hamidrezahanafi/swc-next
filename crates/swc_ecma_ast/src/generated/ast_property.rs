#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::erasing_op,
    clippy::let_and_return
)]
use crate::*;
use oxc_index::Idx;
impl<'id> Program<'id> {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Directive<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).program.directives })
    }
    #[inline]
    pub fn hashbang<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Hashbang<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).program.hashbang };
        (raw != u32::MAX).then(|| Hashbang(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .program
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        directives: SubRange<'id, Directive<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .program
                .directives = directives.raw();
        }
    }
    #[inline]
    pub fn set_hashbang<'a>(self, ast: &mut crate::Ast<'a, 'id>, hashbang: Option<Hashbang<'id>>) {
        let field_val: u32 = hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).program.hashbang = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: SubRange<'id, Stmt<'id>>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .program
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl<'id> Stmt<'id> {
    #[inline]
    pub fn is_block_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BlockStatement
    }
    #[inline]
    pub fn is_break_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BreakStatement
    }
    #[inline]
    pub fn is_continue_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ContinueStatement
    }
    #[inline]
    pub fn is_debugger_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::DebuggerStatement
    }
    #[inline]
    pub fn is_do_while_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::DoWhileStatement
    }
    #[inline]
    pub fn is_empty_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::EmptyStatement
    }
    #[inline]
    pub fn is_expression_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExpressionStatement
    }
    #[inline]
    pub fn is_for_in_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForInStatement
    }
    #[inline]
    pub fn is_for_of_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForOfStatement
    }
    #[inline]
    pub fn is_for_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ForStatement
    }
    #[inline]
    pub fn is_if_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IfStatement
    }
    #[inline]
    pub fn is_labeled_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::LabeledStatement
    }
    #[inline]
    pub fn is_return_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ReturnStatement
    }
    #[inline]
    pub fn is_switch_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::SwitchStatement
    }
    #[inline]
    pub fn is_throw_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThrowStatement
    }
    #[inline]
    pub fn is_try_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TryStatement
    }
    #[inline]
    pub fn is_while_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::WhileStatement
    }
    #[inline]
    pub fn is_with_statement(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::WithStatement
    }
    #[inline]
    pub fn is_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::VariableDeclaration
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::TsFunction
                | NodeKind::TsTypeAliasDeclaration
                | NodeKind::TsInterfaceDeclaration
                | NodeKind::TsEnumDeclaration
                | NodeKind::TsModuleDeclaration
                | NodeKind::TsGlobalDeclaration
                | NodeKind::TsImportEqualsDeclaration
        )
    }
    #[inline]
    pub fn is_import_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportDeclaration
    }
    #[inline]
    pub fn is_export_named_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportNamedDeclaration
    }
    #[inline]
    pub fn is_export_default_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportDefaultDeclaration
    }
    #[inline]
    pub fn is_export_all_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ExportAllDeclaration
    }
    #[inline]
    pub fn is_ts_export_assignment(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsExportAssignment
    }
    #[inline]
    pub fn is_ts_namespace_export_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNamespaceExportDeclaration
    }
    #[inline]
    pub fn as_block_statement(self, ast: &Ast<'_, 'id>) -> Option<BlockStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BlockStatement) {
            Some(BlockStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_break_statement(self, ast: &Ast<'_, 'id>) -> Option<BreakStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BreakStatement) {
            Some(BreakStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_continue_statement(self, ast: &Ast<'_, 'id>) -> Option<ContinueStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ContinueStatement) {
            Some(ContinueStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_debugger_statement(self, ast: &Ast<'_, 'id>) -> Option<DebuggerStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::DebuggerStatement) {
            Some(DebuggerStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_do_while_statement(self, ast: &Ast<'_, 'id>) -> Option<DoWhileStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::DoWhileStatement) {
            Some(DoWhileStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_empty_statement(self, ast: &Ast<'_, 'id>) -> Option<EmptyStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::EmptyStatement) {
            Some(EmptyStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expression_statement(self, ast: &Ast<'_, 'id>) -> Option<ExpressionStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ExpressionStatement) {
            Some(ExpressionStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_in_statement(self, ast: &Ast<'_, 'id>) -> Option<ForInStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ForInStatement) {
            Some(ForInStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_of_statement(self, ast: &Ast<'_, 'id>) -> Option<ForOfStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ForOfStatement) {
            Some(ForOfStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_for_statement(self, ast: &Ast<'_, 'id>) -> Option<ForStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ForStatement) {
            Some(ForStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_if_statement(self, ast: &Ast<'_, 'id>) -> Option<IfStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IfStatement) {
            Some(IfStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_labeled_statement(self, ast: &Ast<'_, 'id>) -> Option<LabeledStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::LabeledStatement) {
            Some(LabeledStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_return_statement(self, ast: &Ast<'_, 'id>) -> Option<ReturnStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ReturnStatement) {
            Some(ReturnStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_switch_statement(self, ast: &Ast<'_, 'id>) -> Option<SwitchStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::SwitchStatement) {
            Some(SwitchStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_throw_statement(self, ast: &Ast<'_, 'id>) -> Option<ThrowStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ThrowStatement) {
            Some(ThrowStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_try_statement(self, ast: &Ast<'_, 'id>) -> Option<TryStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TryStatement) {
            Some(TryStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_while_statement(self, ast: &Ast<'_, 'id>) -> Option<WhileStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::WhileStatement) {
            Some(WhileStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_with_statement(self, ast: &Ast<'_, 'id>) -> Option<WithStatement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::WithStatement) {
            Some(WithStatement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_declaration(self, ast: &Ast<'_, 'id>) -> Option<Decl<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::VariableDeclaration
                | NodeKind::Function
                | NodeKind::Class
                | NodeKind::TsFunction
                | NodeKind::TsTypeAliasDeclaration
                | NodeKind::TsInterfaceDeclaration
                | NodeKind::TsEnumDeclaration
                | NodeKind::TsModuleDeclaration
                | NodeKind::TsGlobalDeclaration
                | NodeKind::TsImportEqualsDeclaration
        ) {
            Some(Decl(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_declaration(self, ast: &Ast<'_, 'id>) -> Option<ImportDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ImportDeclaration) {
            Some(ImportDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_named_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ExportNamedDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ExportNamedDeclaration) {
            Some(ExportNamedDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_default_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ExportDefaultDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ExportDefaultDeclaration) {
            Some(ExportDefaultDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_export_all_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ExportAllDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ExportAllDeclaration) {
            Some(ExportAllDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_export_assignment(self, ast: &Ast<'_, 'id>) -> Option<TsExportAssignment<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsExportAssignment) {
            Some(TsExportAssignment(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_namespace_export_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsNamespaceExportDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNamespaceExportDeclaration) {
            Some(TsNamespaceExportDeclaration(self.0))
        } else {
            None
        }
    }
}
impl<'id> Decl<'id> {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::Function
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::Class
    }
    #[inline]
    pub fn is_ts_function(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsFunction
    }
    #[inline]
    pub fn is_ts_type_alias_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAliasDeclaration
    }
    #[inline]
    pub fn is_ts_interface_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInterfaceDeclaration
    }
    #[inline]
    pub fn is_ts_enum_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsEnumDeclaration
    }
    #[inline]
    pub fn is_ts_module_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleDeclaration
    }
    #[inline]
    pub fn is_ts_global_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsGlobalDeclaration
    }
    #[inline]
    pub fn is_ts_import_equals_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportEqualsDeclaration
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_, 'id>) -> Option<VariableDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_, 'id>) -> Option<Function<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::Function) {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_, 'id>) -> Option<Class<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::Class) {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_, 'id>) -> Option<TsFunction<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsFunction) {
            Some(TsFunction(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_alias_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsTypeAliasDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeAliasDeclaration) {
            Some(TsTypeAliasDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsInterfaceDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsInterfaceDeclaration) {
            Some(TsInterfaceDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_enum_declaration(self, ast: &Ast<'_, 'id>) -> Option<TsEnumDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsEnumDeclaration) {
            Some(TsEnumDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_, 'id>) -> Option<TsModuleDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsModuleDeclaration) {
            Some(TsModuleDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_global_declaration(self, ast: &Ast<'_, 'id>) -> Option<TsGlobalDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsGlobalDeclaration) {
            Some(TsGlobalDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_equals_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsImportEqualsDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsImportEqualsDeclaration) {
            Some(TsImportEqualsDeclaration(self.0))
        } else {
            None
        }
    }
}
impl<'id> Expr<'id> {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_super(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::Super
    }
    #[inline]
    pub fn is_meta_property(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::MetaProperty
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_numeric_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::NumericLiteral
    }
    #[inline]
    pub fn is_big_int_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BigIntLiteral
    }
    #[inline]
    pub fn is_boolean_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BooleanLiteral
    }
    #[inline]
    pub fn is_null_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::NullLiteral
    }
    #[inline]
    pub fn is_reg_exp_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::RegExpLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn is_binary_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BinaryExpression
    }
    #[inline]
    pub fn is_logical_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::LogicalExpression
    }
    #[inline]
    pub fn is_unary_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::UnaryExpression
    }
    #[inline]
    pub fn is_update_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::UpdateExpression
    }
    #[inline]
    pub fn is_assignment_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::AssignmentExpression
    }
    #[inline]
    pub fn is_conditional_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ConditionalExpression
    }
    #[inline]
    pub fn is_sequence_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::SequenceExpression
    }
    #[inline]
    pub fn is_member_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::MemberExpression
    }
    #[inline]
    pub fn is_call_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::CallExpression
    }
    #[inline]
    pub fn is_chain_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ChainExpression
    }
    #[inline]
    pub fn is_new_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::NewExpression
    }
    #[inline]
    pub fn is_tagged_template_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TaggedTemplateExpression
    }
    #[inline]
    pub fn is_arrow_function_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrowFunctionExpression
    }
    #[inline]
    pub fn is_array_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayExpression
    }
    #[inline]
    pub fn is_object_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectExpression
    }
    #[inline]
    pub fn is_parenthesized_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ParenthesizedExpression
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::Function
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::Class
    }
    #[inline]
    pub fn is_import_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportExpression
    }
    #[inline]
    pub fn is_await_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::AwaitExpression
    }
    #[inline]
    pub fn is_yield_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::YieldExpression
    }
    #[inline]
    pub fn is_private_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::PrivateIdentifier
    }
    #[inline]
    pub fn is_ts_as_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAsExpression
    }
    #[inline]
    pub fn is_ts_satisfies_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSatisfiesExpression
    }
    #[inline]
    pub fn is_ts_type_assertion(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAssertion
    }
    #[inline]
    pub fn is_ts_non_null_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNonNullExpression
    }
    #[inline]
    pub fn is_ts_instantiation_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInstantiationExpression
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_, 'id>) -> Option<IdentifierReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_, 'id>) -> Option<ThisExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_super(self, ast: &Ast<'_, 'id>) -> Option<Super<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::Super) {
            Some(Super(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_meta_property(self, ast: &Ast<'_, 'id>) -> Option<MetaProperty<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::MetaProperty) {
            Some(MetaProperty(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_, 'id>) -> Option<NumericLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::NumericLiteral) {
            Some(NumericLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_, 'id>) -> Option<BigIntLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BigIntLiteral) {
            Some(BigIntLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_, 'id>) -> Option<BooleanLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BooleanLiteral) {
            Some(BooleanLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_null_literal(self, ast: &Ast<'_, 'id>) -> Option<NullLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::NullLiteral) {
            Some(NullLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_reg_exp_literal(self, ast: &Ast<'_, 'id>) -> Option<RegExpLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::RegExpLiteral) {
            Some(RegExpLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_, 'id>) -> Option<TemplateLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binary_expression(self, ast: &Ast<'_, 'id>) -> Option<BinaryExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BinaryExpression) {
            Some(BinaryExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_logical_expression(self, ast: &Ast<'_, 'id>) -> Option<LogicalExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::LogicalExpression) {
            Some(LogicalExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_, 'id>) -> Option<UnaryExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::UnaryExpression) {
            Some(UnaryExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_update_expression(self, ast: &Ast<'_, 'id>) -> Option<UpdateExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::UpdateExpression) {
            Some(UpdateExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_expression(self, ast: &Ast<'_, 'id>) -> Option<AssignmentExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::AssignmentExpression) {
            Some(AssignmentExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_conditional_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ConditionalExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ConditionalExpression) {
            Some(ConditionalExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_sequence_expression(self, ast: &Ast<'_, 'id>) -> Option<SequenceExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::SequenceExpression) {
            Some(SequenceExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_, 'id>) -> Option<MemberExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::MemberExpression) {
            Some(MemberExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_call_expression(self, ast: &Ast<'_, 'id>) -> Option<CallExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::CallExpression) {
            Some(CallExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_chain_expression(self, ast: &Ast<'_, 'id>) -> Option<ChainExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ChainExpression) {
            Some(ChainExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_new_expression(self, ast: &Ast<'_, 'id>) -> Option<NewExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::NewExpression) {
            Some(NewExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_tagged_template_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TaggedTemplateExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TaggedTemplateExpression) {
            Some(TaggedTemplateExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_arrow_function_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ArrowFunctionExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ArrowFunctionExpression) {
            Some(ArrowFunctionExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_expression(self, ast: &Ast<'_, 'id>) -> Option<ArrayExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ArrayExpression) {
            Some(ArrayExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_expression(self, ast: &Ast<'_, 'id>) -> Option<ObjectExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ObjectExpression) {
            Some(ObjectExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_parenthesized_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ParenthesizedExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ParenthesizedExpression) {
            Some(ParenthesizedExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_, 'id>) -> Option<Function<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::Function) {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_, 'id>) -> Option<Class<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::Class) {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_expression(self, ast: &Ast<'_, 'id>) -> Option<ImportExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ImportExpression) {
            Some(ImportExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_await_expression(self, ast: &Ast<'_, 'id>) -> Option<AwaitExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::AwaitExpression) {
            Some(AwaitExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_yield_expression(self, ast: &Ast<'_, 'id>) -> Option<YieldExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::YieldExpression) {
            Some(YieldExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_private_identifier(self, ast: &Ast<'_, 'id>) -> Option<PrivateIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::PrivateIdentifier) {
            Some(PrivateIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_, 'id>) -> Option<TsAsExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsAsExpression) {
            Some(TsAsExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsSatisfiesExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsSatisfiesExpression) {
            Some(TsSatisfiesExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAssertion<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeAssertion) {
            Some(TsTypeAssertion(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_, 'id>) -> Option<TsNonNullExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNonNullExpression) {
            Some(TsNonNullExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_instantiation_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsInstantiationExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsInstantiationExpression) {
            Some(TsInstantiationExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_, 'id>) -> Option<JsxElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_, 'id>) -> Option<JsxFragment<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl<'id> BindingPattern<'id> {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        )
    }
    #[inline]
    pub fn is_assignment_pattern(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::AssignmentPattern
    }
    #[inline]
    pub fn is_binding_rest_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingRestElement
    }
    #[inline]
    pub fn is_array_pattern(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_pattern(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_, 'id>) -> Option<BindingIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_simple_assignment_target(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<SimpleAssignmentTarget<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        ) {
            Some(SimpleAssignmentTarget(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_pattern(self, ast: &Ast<'_, 'id>) -> Option<AssignmentPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::AssignmentPattern) {
            Some(AssignmentPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binding_rest_element(self, ast: &Ast<'_, 'id>) -> Option<BindingRestElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BindingRestElement) {
            Some(BindingRestElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_pattern(self, ast: &Ast<'_, 'id>) -> Option<ArrayPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_pattern(self, ast: &Ast<'_, 'id>) -> Option<ObjectPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl<'id> AssignmentTarget<'id> {
    #[inline]
    pub fn is_simple_assignment_target(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        )
    }
    #[inline]
    pub fn is_array_assignment_target(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_assignment_target(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_simple_assignment_target(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<SimpleAssignmentTarget<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
        ) {
            Some(SimpleAssignmentTarget(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_assignment_target(self, ast: &Ast<'_, 'id>) -> Option<ArrayPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_assignment_target(self, ast: &Ast<'_, 'id>) -> Option<ObjectPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl<'id> SimpleAssignmentTarget<'id> {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_member_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::MemberExpression
    }
    #[inline]
    pub fn is_ts_as_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAsExpression
    }
    #[inline]
    pub fn is_ts_satisfies_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSatisfiesExpression
    }
    #[inline]
    pub fn is_ts_type_assertion(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeAssertion
    }
    #[inline]
    pub fn is_ts_non_null_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNonNullExpression
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_, 'id>) -> Option<IdentifierReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_member_expression(self, ast: &Ast<'_, 'id>) -> Option<MemberExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::MemberExpression) {
            Some(MemberExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_as_expression(self, ast: &Ast<'_, 'id>) -> Option<TsAsExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsAsExpression) {
            Some(TsAsExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_satisfies_expression(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsSatisfiesExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsSatisfiesExpression) {
            Some(TsSatisfiesExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_assertion(self, ast: &Ast<'_, 'id>) -> Option<TsTypeAssertion<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeAssertion) {
            Some(TsTypeAssertion(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_non_null_expression(self, ast: &Ast<'_, 'id>) -> Option<TsNonNullExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNonNullExpression) {
            Some(TsNonNullExpression(self.0))
        } else {
            None
        }
    }
}
impl<'id> PropertyKey<'id> {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl<'id> ModuleExportName<'id> {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
}
impl<'id> Argument<'id> {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn is_spread_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::SpreadElement
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_, 'id>) -> Option<SpreadElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::SpreadElement) {
            Some(SpreadElement(self.0))
        } else {
            None
        }
    }
}
impl<'id> ObjectPropertyKind<'id> {
    #[inline]
    pub fn is_object_property(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectProperty
    }
    #[inline]
    pub fn is_spread_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::SpreadElement
    }
    #[inline]
    pub fn as_object_property(self, ast: &Ast<'_, 'id>) -> Option<ObjectProperty<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ObjectProperty) {
            Some(ObjectProperty(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_spread_element(self, ast: &Ast<'_, 'id>) -> Option<SpreadElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::SpreadElement) {
            Some(SpreadElement(self.0))
        } else {
            None
        }
    }
}
impl<'id> ForStatementInit<'id> {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_, 'id>) -> Option<VariableDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl<'id> ForStatementLeft<'id> {
    #[inline]
    pub fn is_variable_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::VariableDeclaration
    }
    #[inline]
    pub fn is_assignment_target(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::ArrayPattern
                | NodeKind::ObjectPattern
        )
    }
    #[inline]
    pub fn as_variable_declaration(self, ast: &Ast<'_, 'id>) -> Option<VariableDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::VariableDeclaration) {
            Some(VariableDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_assignment_target(self, ast: &Ast<'_, 'id>) -> Option<AssignmentTarget<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::IdentifierReference
                | NodeKind::MemberExpression
                | NodeKind::TsAsExpression
                | NodeKind::TsSatisfiesExpression
                | NodeKind::TsTypeAssertion
                | NodeKind::TsNonNullExpression
                | NodeKind::ArrayPattern
                | NodeKind::ObjectPattern
        ) {
            Some(AssignmentTarget(self.0))
        } else {
            None
        }
    }
}
impl<'id> ArrowFunctionBody<'id> {
    #[inline]
    pub fn is_function_body(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::FunctionBody
    }
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn as_function_body(self, ast: &Ast<'_, 'id>) -> Option<FunctionBody<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::FunctionBody) {
            Some(FunctionBody(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
}
impl<'id> FormalParameterItem<'id> {
    #[inline]
    pub fn is_formal_parameter(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::FormalParameter
    }
    #[inline]
    pub fn is_ts_parameter_property(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsParameterProperty
    }
    #[inline]
    pub fn as_formal_parameter(self, ast: &Ast<'_, 'id>) -> Option<FormalParameter<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::FormalParameter) {
            Some(FormalParameter(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_parameter_property(self, ast: &Ast<'_, 'id>) -> Option<TsParameterProperty<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsParameterProperty) {
            Some(TsParameterProperty(self.0))
        } else {
            None
        }
    }
}
impl<'id> FormalParameterPattern<'id> {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_array_pattern(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ArrayPattern
    }
    #[inline]
    pub fn is_object_pattern(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ObjectPattern
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_, 'id>) -> Option<BindingIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_array_pattern(self, ast: &Ast<'_, 'id>) -> Option<ArrayPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ArrayPattern) {
            Some(ArrayPattern(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_object_pattern(self, ast: &Ast<'_, 'id>) -> Option<ObjectPattern<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ObjectPattern) {
            Some(ObjectPattern(self.0))
        } else {
            None
        }
    }
}
impl<'id> ClassElement<'id> {
    #[inline]
    pub fn is_method_definition(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::MethodDefinition
    }
    #[inline]
    pub fn is_property_definition(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::PropertyDefinition
    }
    #[inline]
    pub fn is_static_block(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StaticBlock
    }
    #[inline]
    pub fn is_ts_method_definition(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMethodDefinition
    }
    #[inline]
    pub fn is_ts_index_signature(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexSignature
    }
    #[inline]
    pub fn as_method_definition(self, ast: &Ast<'_, 'id>) -> Option<MethodDefinition<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::MethodDefinition) {
            Some(MethodDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_property_definition(self, ast: &Ast<'_, 'id>) -> Option<PropertyDefinition<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::PropertyDefinition) {
            Some(PropertyDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_static_block(self, ast: &Ast<'_, 'id>) -> Option<StaticBlock<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StaticBlock) {
            Some(StaticBlock(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_method_definition(self, ast: &Ast<'_, 'id>) -> Option<TsMethodDefinition<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsMethodDefinition) {
            Some(TsMethodDefinition(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_, 'id>) -> Option<TsIndexSignature<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsIndexSignature) {
            Some(TsIndexSignature(self.0))
        } else {
            None
        }
    }
}
impl<'id> ImportDeclarationSpecifier<'id> {
    #[inline]
    pub fn is_import_specifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportSpecifier
    }
    #[inline]
    pub fn is_import_default_specifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportDefaultSpecifier
    }
    #[inline]
    pub fn is_import_namespace_specifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ImportNamespaceSpecifier
    }
    #[inline]
    pub fn as_import_specifier(self, ast: &Ast<'_, 'id>) -> Option<ImportSpecifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ImportSpecifier) {
            Some(ImportSpecifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_default_specifier(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ImportDefaultSpecifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ImportDefaultSpecifier) {
            Some(ImportDefaultSpecifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_import_namespace_specifier(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<ImportNamespaceSpecifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ImportNamespaceSpecifier) {
            Some(ImportNamespaceSpecifier(self.0))
        } else {
            None
        }
    }
}
impl<'id> ExportDefaultDeclarationKind<'id> {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        match kind {
            NodeKind::IdentifierReference => true,
            NodeKind::ThisExpression => true,
            NodeKind::Super => true,
            NodeKind::MetaProperty => true,
            NodeKind::StringLiteral => true,
            NodeKind::NumericLiteral => true,
            NodeKind::BigIntLiteral => true,
            NodeKind::BooleanLiteral => true,
            NodeKind::NullLiteral => true,
            NodeKind::RegExpLiteral => true,
            NodeKind::TemplateLiteral => true,
            NodeKind::BinaryExpression => true,
            NodeKind::LogicalExpression => true,
            NodeKind::UnaryExpression => true,
            NodeKind::UpdateExpression => true,
            NodeKind::AssignmentExpression => true,
            NodeKind::ConditionalExpression => true,
            NodeKind::SequenceExpression => true,
            NodeKind::MemberExpression => true,
            NodeKind::CallExpression => true,
            NodeKind::ChainExpression => true,
            NodeKind::NewExpression => true,
            NodeKind::TaggedTemplateExpression => true,
            NodeKind::ArrowFunctionExpression => true,
            NodeKind::ArrayExpression => true,
            NodeKind::ObjectExpression => true,
            NodeKind::ParenthesizedExpression => true,
            NodeKind::ImportExpression => true,
            NodeKind::AwaitExpression => true,
            NodeKind::YieldExpression => true,
            NodeKind::PrivateIdentifier => true,
            NodeKind::TsAsExpression => true,
            NodeKind::TsSatisfiesExpression => true,
            NodeKind::TsTypeAssertion => true,
            NodeKind::TsNonNullExpression => true,
            NodeKind::TsInstantiationExpression => true,
            NodeKind::JsxElement => true,
            NodeKind::JsxFragment => true,
            NodeKind::Function => {
                let function = Function(self.0);
                function.function_type(ast) != FunctionType::FunctionDeclaration
            }
            NodeKind::Class => {
                let class = Class(self.0);
                class.class_type(ast) != ClassType::ClassDeclaration
            }
            _ => false,
        }
    }
    #[inline]
    pub fn is_function(self, ast: &Ast<'_, 'id>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if kind == NodeKind::Function {
            let function = Function(self.0);
            function.function_type(ast) == FunctionType::FunctionDeclaration
        } else {
            false
        }
    }
    #[inline]
    pub fn is_class(self, ast: &Ast<'_, 'id>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if kind == NodeKind::Class {
            let class = Class(self.0);
            class.class_type(ast) == ClassType::ClassDeclaration
        } else {
            false
        }
    }
    #[inline]
    pub fn is_ts_function(self, ast: &Ast<'_, 'id>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        matches!(kind, NodeKind::TsFunction)
    }
    #[inline]
    pub fn is_ts_interface_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        matches!(kind, NodeKind::TsInterfaceDeclaration)
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if match kind {
            NodeKind::IdentifierReference => true,
            NodeKind::ThisExpression => true,
            NodeKind::Super => true,
            NodeKind::MetaProperty => true,
            NodeKind::StringLiteral => true,
            NodeKind::NumericLiteral => true,
            NodeKind::BigIntLiteral => true,
            NodeKind::BooleanLiteral => true,
            NodeKind::NullLiteral => true,
            NodeKind::RegExpLiteral => true,
            NodeKind::TemplateLiteral => true,
            NodeKind::BinaryExpression => true,
            NodeKind::LogicalExpression => true,
            NodeKind::UnaryExpression => true,
            NodeKind::UpdateExpression => true,
            NodeKind::AssignmentExpression => true,
            NodeKind::ConditionalExpression => true,
            NodeKind::SequenceExpression => true,
            NodeKind::MemberExpression => true,
            NodeKind::CallExpression => true,
            NodeKind::ChainExpression => true,
            NodeKind::NewExpression => true,
            NodeKind::TaggedTemplateExpression => true,
            NodeKind::ArrowFunctionExpression => true,
            NodeKind::ArrayExpression => true,
            NodeKind::ObjectExpression => true,
            NodeKind::ParenthesizedExpression => true,
            NodeKind::ImportExpression => true,
            NodeKind::AwaitExpression => true,
            NodeKind::YieldExpression => true,
            NodeKind::PrivateIdentifier => true,
            NodeKind::TsAsExpression => true,
            NodeKind::TsSatisfiesExpression => true,
            NodeKind::TsTypeAssertion => true,
            NodeKind::TsNonNullExpression => true,
            NodeKind::TsInstantiationExpression => true,
            NodeKind::JsxElement => true,
            NodeKind::JsxFragment => true,
            NodeKind::Function => {
                let function = Function(self.0);
                function.function_type(ast) != FunctionType::FunctionDeclaration
            }
            NodeKind::Class => {
                let class = Class(self.0);
                class.class_type(ast) != ClassType::ClassDeclaration
            }
            _ => false,
        } {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_function(self, ast: &Ast<'_, 'id>) -> Option<Function<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if if kind == NodeKind::Function {
            let function = Function(self.0);
            function.function_type(ast) == FunctionType::FunctionDeclaration
        } else {
            false
        } {
            Some(Function(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_class(self, ast: &Ast<'_, 'id>) -> Option<Class<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if if kind == NodeKind::Class {
            let class = Class(self.0);
            class.class_type(ast) == ClassType::ClassDeclaration
        } else {
            false
        } {
            Some(Class(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function(self, ast: &Ast<'_, 'id>) -> Option<TsFunction<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsFunction) {
            Some(TsFunction(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_interface_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsInterfaceDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsInterfaceDeclaration) {
            Some(TsInterfaceDeclaration(self.0))
        } else {
            None
        }
    }
}
impl<'id> Super<'id> {}
impl<'id> ThisExpression<'id> {}
impl<'id> StringLiteral<'id> {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstWtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).string_literal.value };
        AstWtf8::from_raw(raw)
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).string_literal.raw };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: AstWtf8<'id>) {
        let field_val: u64 = value.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .string_literal
                .value = field_val;
        }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a, 'id>, raw: AstUtf8<'id>) {
        let field_val: u64 = raw.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .string_literal
                .raw = field_val;
        }
    }
}
impl<'id> NumericLiteral<'id> {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> f64 {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).numeric_literal.value }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<AstUtf8<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).numeric_literal.raw };
        (raw != u64::from(u32::MAX)).then(|| AstUtf8::from_raw(raw))
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: f64) {
        if unsafe { ast.nodes.data_unchecked(self.0.raw()).numeric_literal.value }.to_bits()
            == value.to_bits()
        {
            return;
        }
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .numeric_literal
                .value = value;
        }
        self.set_raw(ast, None);
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a, 'id>, raw: Option<AstUtf8<'id>>) {
        let field_val: u64 = raw.map_or(u64::from(u32::MAX), AstUtf8::raw);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .numeric_literal
                .raw = field_val;
        }
    }
}
impl<'id> BigIntLiteral<'id> {
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).big_int_literal.raw };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a, 'id>, raw: AstUtf8<'id>) {
        let field_val: u64 = raw.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .big_int_literal
                .raw = field_val;
        }
    }
}
impl<'id> BooleanLiteral<'id> {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).boolean_literal.value }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .boolean_literal
                .value = value;
        }
    }
}
impl<'id> NullLiteral<'id> {}
impl<'id> RegExpLiteral<'id> {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .reg_exp_literal
                .pattern
        };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn flags<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).reg_exp_literal.flags };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_pattern<'a>(self, ast: &mut crate::Ast<'a, 'id>, pattern: AstUtf8<'id>) {
        let field_val: u64 = pattern.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .reg_exp_literal
                .pattern = field_val;
        }
    }
    #[inline]
    pub fn set_flags<'a>(self, ast: &mut crate::Ast<'a, 'id>, flags: AstUtf8<'id>) {
        let field_val: u64 = flags.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .reg_exp_literal
                .flags = field_val;
        }
    }
}
impl<'id> TemplateLiteral<'id> {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TemplateElement<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_literal
                .quasis
        })
    }
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Expr<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_literal
                .expressions
        })
    }
    #[inline]
    pub fn set_quasis<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        quasis: SubRange<'id, TemplateElement<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .template_literal
                .quasis = quasis.raw();
        }
    }
    #[inline]
    pub fn set_expressions<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        expressions: SubRange<'id, Expr<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .template_literal
                .expressions = expressions.raw();
        }
    }
}
impl<'id> TemplateElement<'id> {
    #[inline]
    pub fn cooked<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstWtf8<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_element
                .cooked
        };
        AstWtf8::from_raw(raw)
    }
    #[inline]
    pub fn tail<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).template_element.tail }
    }
    #[inline]
    pub fn is_cooked_undefined<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_element
                .is_cooked_undefined
        }
    }
    #[inline]
    pub fn raw<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_cooked<'a>(self, ast: &mut crate::Ast<'a, 'id>, cooked: AstWtf8<'id>) {
        let field_val: u64 = cooked.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .template_element
                .cooked = field_val;
        }
    }
    #[inline]
    pub fn set_tail<'a>(self, ast: &mut crate::Ast<'a, 'id>, tail: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .template_element
                .tail = tail;
        }
    }
    #[inline]
    pub fn set_is_cooked_undefined<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        is_cooked_undefined: bool,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .template_element
                .is_cooked_undefined = is_cooked_undefined;
        }
    }
    #[inline]
    pub fn set_raw<'a>(self, ast: &mut crate::Ast<'a, 'id>, raw: AstUtf8<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .template_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = raw.into_extra_data();
        };
    }
}
impl<'id> IdentifierReference<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .identifier_reference
                .name
        };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .identifier_reference
                .name = field_val;
        }
    }
}
impl<'id> PrivateIdentifier<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .private_identifier
                .name
        };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .private_identifier
                .name = field_val;
        }
    }
}
impl<'id> BindingIdentifier<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binding_identifier
                .name
        };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_identifier
                .name = field_val;
        }
    }
}
impl<'id> IdentifierName<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).identifier_name.name };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .identifier_name
                .name = field_val;
        }
    }
}
impl<'id> LabelIdentifier<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).label_identifier.name };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .label_identifier
                .name = field_val;
        }
    }
}
impl<'id> AssignmentPattern<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .assignment_pattern
                .left
        };
        BindingPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .assignment_pattern
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: BindingPattern<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .assignment_pattern
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .assignment_pattern
                .right = field_val;
        }
    }
}
impl<'id> BindingRestElement<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binding_rest_element
                .argument
        };
        BindingPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: BindingPattern<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_rest_element
                .argument = field_val;
        }
    }
}
impl<'id> ArrayPattern<'id> {
    #[inline]
    pub fn elements<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, Option<BindingPattern<'id>>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .array_pattern
                .elements
        })
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingRestElement<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).array_pattern.rest };
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .array_pattern
                .elements = elements.raw();
        }
    }
    #[inline]
    pub fn set_rest<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        rest: Option<BindingRestElement<'id>>,
    ) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .array_pattern
                .rest = field_val;
        }
    }
}
impl<'id> ObjectPattern<'id> {
    #[inline]
    pub fn properties<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, BindingProperty<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .object_pattern
                .properties
        })
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingRestElement<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).object_pattern.rest };
        (raw != u32::MAX).then(|| BindingRestElement(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        properties: SubRange<'id, BindingProperty<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_pattern
                .properties = properties.raw();
        }
    }
    #[inline]
    pub fn set_rest<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        rest: Option<BindingRestElement<'id>>,
    ) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_pattern
                .rest = field_val;
        }
    }
}
impl<'id> BindingProperty<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).binding_property.key };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binding_property
                .value
        };
        BindingPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binding_property
                .shorthand
        }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binding_property
                .computed
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_property
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: BindingPattern<'id>) {
        let field_val: u32 = value.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_property
                .value = field_val;
        }
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a, 'id>, shorthand: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_property
                .shorthand = shorthand;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binding_property
                .computed = computed;
        }
    }
}
impl<'id> BinaryExpression<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binary_expression
                .left
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binary_expression
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> BinaryOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .binary_expression
                .operator
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: Expr<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binary_expression
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binary_expression
                .right = field_val;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: BinaryOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .binary_expression
                .operator = operator;
        }
    }
}
impl<'id> LogicalExpression<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .logical_expression
                .left
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .logical_expression
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> LogicalOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .logical_expression
                .operator
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: Expr<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .logical_expression
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .logical_expression
                .right = field_val;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: LogicalOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .logical_expression
                .operator = operator;
        }
    }
}
impl<'id> ConditionalExpression<'id> {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .conditional_expression
                .test
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .conditional_expression
                .consequent
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .conditional_expression
                .alternate
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Expr<'id>) {
        let field_val: u32 = test.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .conditional_expression
                .test = field_val;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a, 'id>, consequent: Expr<'id>) {
        let field_val: u32 = consequent.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .conditional_expression
                .consequent = field_val;
        }
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a, 'id>, alternate: Expr<'id>) {
        let field_val: u32 = alternate.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .conditional_expression
                .alternate = field_val;
        }
    }
}
impl<'id> UnaryExpression<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .unary_expression
                .argument
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> UnaryOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .unary_expression
                .operator
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Expr<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .unary_expression
                .argument = field_val;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: UnaryOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .unary_expression
                .operator = operator;
        }
    }
}
impl<'id> UpdateExpression<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> SimpleAssignmentTarget<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .update_expression
                .argument
        };
        SimpleAssignmentTarget(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> UpdateOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .update_expression
                .operator
        }
    }
    #[inline]
    pub fn prefix<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .update_expression
                .prefix
        }
    }
    #[inline]
    pub fn set_argument<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        argument: SimpleAssignmentTarget<'id>,
    ) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .update_expression
                .argument = field_val;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: UpdateOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .update_expression
                .operator = operator;
        }
    }
    #[inline]
    pub fn set_prefix<'a>(self, ast: &mut crate::Ast<'a, 'id>, prefix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .update_expression
                .prefix = prefix;
        }
    }
}
impl<'id> AssignmentExpression<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> AssignmentTarget<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .assignment_expression
                .left
        };
        AssignmentTarget(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .assignment_expression
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> AssignmentOperator {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .assignment_expression
                .operator
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: AssignmentTarget<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .assignment_expression
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .assignment_expression
                .right = field_val;
        }
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: AssignmentOperator) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .assignment_expression
                .operator = operator;
        }
    }
}
impl<'id> VariableDeclaration<'id> {
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> VariableKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declaration
                .kind
        }
    }
    #[inline]
    pub fn declarators<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, VariableDeclarator<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declaration
                .declarators
        })
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: VariableKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declaration
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_declarators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declaration
                .declarators = declarators.raw();
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declaration
                .declare = declare;
        }
    }
}
impl<'id> VariableDeclarator<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declarator
                .id
        };
        BindingPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declarator
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declarator
                .init
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .variable_declarator
                .definite
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: BindingPattern<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declarator
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declarator
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a, 'id>, init: Option<Expr<'id>>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declarator
                .init = field_val;
        }
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a, 'id>, definite: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .variable_declarator
                .definite = definite;
        }
    }
}
impl<'id> IfStatement<'id> {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).if_statement.test };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .if_statement
                .consequent
        };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn alternate<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Stmt<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .if_statement
                .alternate
        };
        (raw != u32::MAX).then(|| Stmt(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Expr<'id>) {
        let field_val: u32 = test.node_id().index() as u32;
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).if_statement.test = field_val;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(self, ast: &mut crate::Ast<'a, 'id>, consequent: Stmt<'id>) {
        let field_val: u32 = consequent.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .if_statement
                .consequent = field_val;
        }
    }
    #[inline]
    pub fn set_alternate<'a>(self, ast: &mut crate::Ast<'a, 'id>, alternate: Option<Stmt<'id>>) {
        let field_val: u32 = alternate.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .if_statement
                .alternate = field_val;
        }
    }
}
impl<'id> SwitchStatement<'id> {
    #[inline]
    pub fn discriminant<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .switch_statement
                .discriminant
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn cases<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, SwitchCase<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .switch_statement
                .cases
        })
    }
    #[inline]
    pub fn set_discriminant<'a>(self, ast: &mut crate::Ast<'a, 'id>, discriminant: Expr<'id>) {
        let field_val: u32 = discriminant.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .switch_statement
                .discriminant = field_val;
        }
    }
    #[inline]
    pub fn set_cases<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        cases: SubRange<'id, SwitchCase<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .switch_statement
                .cases = cases.raw();
        }
    }
}
impl<'id> SwitchCase<'id> {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).switch_case.test };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn consequent<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .switch_case
                .consequent
        })
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Option<Expr<'id>>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).switch_case.test = field_val;
        }
    }
    #[inline]
    pub fn set_consequent<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        consequent: SubRange<'id, Stmt<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .switch_case
                .consequent = consequent.raw();
        }
    }
}
impl<'id> ForStatement<'id> {
    #[inline]
    pub fn init<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<ForStatementInit<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_statement.init };
        (raw != u32::MAX).then(|| ForStatementInit(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_statement.test };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn update<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_statement.update };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_statement.body };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_init<'a>(self, ast: &mut crate::Ast<'a, 'id>, init: Option<ForStatementInit<'id>>) {
        let field_val: u32 = init.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_statement
                .init = field_val;
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Option<Expr<'id>>) {
        let field_val: u32 = test.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_statement
                .test = field_val;
        }
    }
    #[inline]
    pub fn set_update<'a>(self, ast: &mut crate::Ast<'a, 'id>, update: Option<Expr<'id>>) {
        let field_val: u32 = update.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_statement
                .update = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_statement
                .body = field_val;
        }
    }
}
impl<'id> ForInStatement<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> ForStatementLeft<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_in_statement.left };
        ForStatementLeft(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .for_in_statement
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_in_statement.body };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: ForStatementLeft<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_in_statement
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_in_statement
                .right = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_in_statement
                .body = field_val;
        }
    }
}
impl<'id> ForOfStatement<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> ForStatementLeft<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_of_statement.left };
        ForStatementLeft(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .for_of_statement
                .right
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).for_of_statement.body };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn r#await<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .for_of_statement
                .r#await
        }
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: ForStatementLeft<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_of_statement
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: Expr<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_of_statement
                .right = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_of_statement
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_await<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#await: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .for_of_statement
                .r#await = r#await;
        }
    }
}
impl<'id> TryStatement<'id> {
    #[inline]
    pub fn block<'a>(self, ast: &crate::Ast<'a, 'id>) -> BlockStatement<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).try_statement.block };
        BlockStatement(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn handler<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<CatchClause<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).try_statement.handler };
        (raw != u32::MAX).then(|| CatchClause(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn finalizer<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BlockStatement<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .try_statement
                .finalizer
        };
        (raw != u32::MAX).then(|| BlockStatement(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_block<'a>(self, ast: &mut crate::Ast<'a, 'id>, block: BlockStatement<'id>) {
        let field_val: u32 = block.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .try_statement
                .block = field_val;
        }
    }
    #[inline]
    pub fn set_handler<'a>(self, ast: &mut crate::Ast<'a, 'id>, handler: Option<CatchClause<'id>>) {
        let field_val: u32 = handler.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .try_statement
                .handler = field_val;
        }
    }
    #[inline]
    pub fn set_finalizer<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        finalizer: Option<BlockStatement<'id>>,
    ) {
        let field_val: u32 = finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .try_statement
                .finalizer = field_val;
        }
    }
}
impl<'id> CatchClause<'id> {
    #[inline]
    pub fn param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingPattern<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).catch_clause.param };
        (raw != u32::MAX).then(|| BindingPattern(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .catch_clause
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> BlockStatement<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).catch_clause.body };
        BlockStatement(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_param<'a>(self, ast: &mut crate::Ast<'a, 'id>, param: Option<BindingPattern<'id>>) {
        let field_val: u32 = param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .catch_clause
                .param = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .catch_clause
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: BlockStatement<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).catch_clause.body = field_val;
        }
    }
}
impl<'id> BreakStatement<'id> {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<LabelIdentifier<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).break_statement.label };
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a, 'id>, label: Option<LabelIdentifier<'id>>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .break_statement
                .label = field_val;
        }
    }
}
impl<'id> ContinueStatement<'id> {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<LabelIdentifier<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .continue_statement
                .label
        };
        (raw != u32::MAX).then(|| LabelIdentifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a, 'id>, label: Option<LabelIdentifier<'id>>) {
        let field_val: u32 = label.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .continue_statement
                .label = field_val;
        }
    }
}
impl<'id> LabeledStatement<'id> {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a, 'id>) -> LabelIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .labeled_statement
                .label
        };
        LabelIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .labeled_statement
                .body
        };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a, 'id>, label: LabelIdentifier<'id>) {
        let field_val: u32 = label.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .labeled_statement
                .label = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .labeled_statement
                .body = field_val;
        }
    }
}
impl<'id> ExpressionStatement<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .expression_statement
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .expression_statement
                .expression = field_val;
        }
    }
}
impl<'id> ReturnStatement<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .return_statement
                .argument
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Option<Expr<'id>>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .return_statement
                .argument = field_val;
        }
    }
}
impl<'id> ThrowStatement<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .throw_statement
                .argument
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Expr<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .throw_statement
                .argument = field_val;
        }
    }
}
impl<'id> DebuggerStatement<'id> {}
impl<'id> EmptyStatement<'id> {}
impl<'id> Directive<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).directive.expression };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).directive.value };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: StringLiteral<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .directive
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: AstUtf8<'id>) {
        let field_val: u64 = value.raw();
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).directive.value = field_val;
        }
    }
}
impl<'id> Decorator<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).decorator.expression };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .decorator
                .expression = field_val;
        }
    }
}
impl<'id> Class<'id> {
    #[inline]
    pub fn class_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> ClassType {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).class.class_type }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingIdentifier<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).class.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).class.type_parameters };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).class.r#abstract }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).class.declare }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn super_class<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn super_type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn implements<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, TsClassImplements<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> ClassBody<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(4usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_class_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, class_type: ClassType) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).class.class_type = class_type;
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: Option<BindingIdentifier<'id>>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).class.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .class
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#abstract: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).class.r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).class.declare = declare;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_super_class<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        super_class: Option<Expr<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_class.into_extra_data();
        };
    }
    #[inline]
    pub fn set_super_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = super_type_arguments.into_extra_data();
        };
    }
    #[inline]
    pub fn set_implements<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        implements: SubRange<'id, TsClassImplements<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = implements.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: ClassBody<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .class
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(4usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl<'id> ClassBody<'id> {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, ClassElement<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).class_body.body })
    }
    #[inline]
    pub fn set_body<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        body: SubRange<'id, ClassElement<'id>>,
    ) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).class_body.body = body.raw();
        }
    }
}
impl<'id> MethodDefinition<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).method_definition.key };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> MethodDefinitionKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .kind
        }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .computed
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .r#static
        }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .optional
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a, 'id>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> Function<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: MethodDefinitionKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        accessibility: Accessibility,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .method_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: Function<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl<'id> PropertyDefinition<'id> {
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .computed
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .r#static
        }
    }
    #[inline]
    pub fn accessor<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .accessor
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .declare
        }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .optional
        }
    }
    #[inline]
    pub fn definite<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .definite
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .readonly
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a, 'id>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_accessor<'a>(self, ast: &mut crate::Ast<'a, 'id>, accessor: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .accessor = accessor;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .declare = declare;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_definite<'a>(self, ast: &mut crate::Ast<'a, 'id>, definite: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .definite = definite;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a, 'id>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        accessibility: Accessibility,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .property_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = key.into_extra_data();
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: Option<Expr<'id>>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .property_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl<'id> StaticBlock<'id> {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).static_block.body })
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: SubRange<'id, Stmt<'id>>) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).static_block.body = body.raw();
        }
    }
}
impl<'id> Function<'id> {
    #[inline]
    pub fn function_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> FunctionType {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .function_type
        }
    }
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingIdentifier<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).function.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).function.generator }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).function.r#async }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsThisParameter<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> FunctionBody<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_function_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, function_type: FunctionType) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .function
                .function_type = function_type;
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: Option<BindingIdentifier<'id>>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).function.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .function
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a, 'id>, generator: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .function
                .generator = generator;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#async: bool) {
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).function.r#async = r#async;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        this_param: Option<TsThisParameter<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: FunctionBody<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(3usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl<'id> FunctionBody<'id> {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Directive<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .function_body
                .directives
        })
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).function_body.body })
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        directives: SubRange<'id, Directive<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .function_body
                .directives = directives.raw();
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: SubRange<'id, Stmt<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .function_body
                .body = body.raw();
        }
    }
}
impl<'id> BlockStatement<'id> {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).block_statement.body })
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: SubRange<'id, Stmt<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .block_statement
                .body = body.raw();
        }
    }
}
impl<'id> WhileStatement<'id> {
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).while_statement.test };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).while_statement.body };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Expr<'id>) {
        let field_val: u32 = test.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .while_statement
                .test = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .while_statement
                .body = field_val;
        }
    }
}
impl<'id> DoWhileStatement<'id> {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .do_while_statement
                .body
        };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn test<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .do_while_statement
                .test
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .do_while_statement
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_test<'a>(self, ast: &mut crate::Ast<'a, 'id>, test: Expr<'id>) {
        let field_val: u32 = test.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .do_while_statement
                .test = field_val;
        }
    }
}
impl<'id> WithStatement<'id> {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).with_statement.object };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Stmt<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).with_statement.body };
        Stmt(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a, 'id>, object: Expr<'id>) {
        let field_val: u32 = object.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .with_statement
                .object = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Stmt<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .with_statement
                .body = field_val;
        }
    }
}
impl<'id> FormalParameters<'id> {
    #[inline]
    pub fn items<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, FormalParameterItem<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameters
                .items
        })
    }
    #[inline]
    pub fn rest<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<FormalParameterRest<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameters
                .rest
        };
        (raw != u32::MAX).then(|| FormalParameterRest(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameterKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameters
                .kind
        }
    }
    #[inline]
    pub fn set_items<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        items: SubRange<'id, FormalParameterItem<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameters
                .items = items.raw();
        }
    }
    #[inline]
    pub fn set_rest<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        rest: Option<FormalParameterRest<'id>>,
    ) {
        let field_val: u32 = rest.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameters
                .rest = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: FormalParameterKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameters
                .kind = kind;
        }
    }
}
impl<'id> FormalParameter<'id> {
    #[inline]
    pub fn pattern<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameterPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .pattern
        };
        FormalParameterPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .optional
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_pattern<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        pattern: FormalParameterPattern<'id>,
    ) {
        let field_val: u32 = pattern.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter
                .pattern = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_initializer<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        initializer: Option<Expr<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = initializer.into_extra_data();
        };
    }
}
impl<'id> FormalParameterRest<'id> {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter_rest
                .decorators
        })
    }
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameterPattern<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter_rest
                .argument
        };
        FormalParameterPattern(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .formal_parameter_rest
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter_rest
                .decorators = decorators.raw();
        }
    }
    #[inline]
    pub fn set_argument<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        argument: FormalParameterPattern<'id>,
    ) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter_rest
                .argument = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .formal_parameter_rest
                .type_annotation = field_val;
        }
    }
}
impl<'id> ArrayExpression<'id> {
    #[inline]
    pub fn elements<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Option<Argument<'id>>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .array_expression
                .elements
        })
    }
    #[inline]
    pub fn set_elements<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .array_expression
                .elements = elements.raw();
        }
    }
}
impl<'id> ObjectExpression<'id> {
    #[inline]
    pub fn properties<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, ObjectPropertyKind<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .object_expression
                .properties
        })
    }
    #[inline]
    pub fn set_properties<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_expression
                .properties = properties.raw();
        }
    }
}
impl<'id> SpreadElement<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .spread_element
                .argument
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Expr<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .spread_element
                .argument = field_val;
        }
    }
}
impl<'id> ObjectProperty<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).object_property.key };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).object_property.value };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKind {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).object_property.kind }
    }
    #[inline]
    pub fn method<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .object_property
                .method
        }
    }
    #[inline]
    pub fn shorthand<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .object_property
                .shorthand
        }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .object_property
                .computed
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: Expr<'id>) {
        let field_val: u32 = value.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .value = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: PropertyKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_method<'a>(self, ast: &mut crate::Ast<'a, 'id>, method: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .method = method;
        }
    }
    #[inline]
    pub fn set_shorthand<'a>(self, ast: &mut crate::Ast<'a, 'id>, shorthand: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .shorthand = shorthand;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .object_property
                .computed = computed;
        }
    }
}
impl<'id> ParenthesizedExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .parenthesized_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .parenthesized_expression
                .expression = field_val;
        }
    }
}
impl<'id> ArrowFunctionExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .expression
        }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .r#async
        }
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .params
        };
        FormalParameters(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> ArrowFunctionBody<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .arrow_function_expression
                .expression = expression;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#async: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .arrow_function_expression
                .r#async = r#async;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .arrow_function_expression
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let field_val: u32 = params.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .arrow_function_expression
                .params = field_val;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: ArrowFunctionBody<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .arrow_function_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl<'id> SequenceExpression<'id> {
    #[inline]
    pub fn expressions<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Expr<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .sequence_expression
                .expressions
        })
    }
    #[inline]
    pub fn set_expressions<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        expressions: SubRange<'id, Expr<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .sequence_expression
                .expressions = expressions.raw();
        }
    }
}
impl<'id> MemberExpression<'id> {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .member_expression
                .object
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .member_expression
                .property
        };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .member_expression
                .computed
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .member_expression
                .optional
        }
    }
    #[inline]
    pub fn set_object<'a>(self, ast: &mut crate::Ast<'a, 'id>, object: Expr<'id>) {
        let field_val: u32 = object.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .member_expression
                .object = field_val;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a, 'id>, property: PropertyKey<'id>) {
        let field_val: u32 = property.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .member_expression
                .property = field_val;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .member_expression
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .member_expression
                .optional = optional;
        }
    }
}
impl<'id> CallExpression<'id> {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .call_expression
                .callee
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .call_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .call_expression
                .optional
        }
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Argument<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .call_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a, 'id>, callee: Expr<'id>) {
        let field_val: u32 = callee.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .call_expression
                .callee = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .call_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .call_expression
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        arguments: SubRange<'id, Argument<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .call_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = arguments.into_extra_data();
        };
    }
}
impl<'id> ChainExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .chain_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .chain_expression
                .expression = field_val;
        }
    }
}
impl<'id> TaggedTemplateExpression<'id> {
    #[inline]
    pub fn tag<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .tagged_template_expression
                .tag
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .tagged_template_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn quasi<'a>(self, ast: &crate::Ast<'a, 'id>) -> TemplateLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .tagged_template_expression
                .quasi
        };
        TemplateLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_tag<'a>(self, ast: &mut crate::Ast<'a, 'id>, tag: Expr<'id>) {
        let field_val: u32 = tag.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .tagged_template_expression
                .tag = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .tagged_template_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_quasi<'a>(self, ast: &mut crate::Ast<'a, 'id>, quasi: TemplateLiteral<'id>) {
        let field_val: u32 = quasi.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .tagged_template_expression
                .quasi = field_val;
        }
    }
}
impl<'id> NewExpression<'id> {
    #[inline]
    pub fn callee<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).new_expression.callee };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .new_expression
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn arguments<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Argument<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .new_expression
                .arguments
        })
    }
    #[inline]
    pub fn set_callee<'a>(self, ast: &mut crate::Ast<'a, 'id>, callee: Expr<'id>) {
        let field_val: u32 = callee.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .new_expression
                .callee = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .new_expression
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        arguments: SubRange<'id, Argument<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .new_expression
                .arguments = arguments.raw();
        }
    }
}
impl<'id> AwaitExpression<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .await_expression
                .argument
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Expr<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .await_expression
                .argument = field_val;
        }
    }
}
impl<'id> YieldExpression<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .yield_expression
                .argument
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn delegate<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .yield_expression
                .delegate
        }
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Option<Expr<'id>>) {
        let field_val: u32 = argument.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .yield_expression
                .argument = field_val;
        }
    }
    #[inline]
    pub fn set_delegate<'a>(self, ast: &mut crate::Ast<'a, 'id>, delegate: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .yield_expression
                .delegate = delegate;
        }
    }
}
impl<'id> MetaProperty<'id> {
    #[inline]
    pub fn meta<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).meta_property.meta };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .meta_property
                .property
        };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_meta<'a>(self, ast: &mut crate::Ast<'a, 'id>, meta: IdentifierName<'id>) {
        let field_val: u32 = meta.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .meta_property
                .meta = field_val;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a, 'id>, property: IdentifierName<'id>) {
        let field_val: u32 = property.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .meta_property
                .property = field_val;
        }
    }
}
impl<'id> ImportExpression<'id> {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_expression
                .source
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_expression
                .options
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn sync<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_expression
                .sync
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<ImportPhase> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a, 'id>, source: Expr<'id>) {
        let field_val: u32 = source.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_expression
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a, 'id>, options: Option<Expr<'id>>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_expression
                .options = field_val;
        }
    }
    #[inline]
    pub fn set_sync<'a>(self, ast: &mut crate::Ast<'a, 'id>, sync: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_expression
                .sync = sync;
        }
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a, 'id>, phase: Option<ImportPhase>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_expression
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.into_extra_data();
        };
    }
}
impl<'id> ImportDeclaration<'id> {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .source
        };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .import_kind
        }
    }
    #[inline]
    pub fn specifiers<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, ImportDeclarationSpecifier<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, ImportAttribute<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn phase<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<ImportPhase> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a, 'id>, source: StringLiteral<'id>) {
        let field_val: u32 = source.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_declaration
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        import_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_declaration
                .import_kind = import_kind;
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        specifiers: SubRange<'id, ImportDeclarationSpecifier<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.into_extra_data();
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
    #[inline]
    pub fn set_phase<'a>(self, ast: &mut crate::Ast<'a, 'id>, phase: Option<ImportPhase>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = phase.into_extra_data();
        };
    }
}
impl<'id> ImportSpecifier<'id> {
    #[inline]
    pub fn imported<'a>(self, ast: &crate::Ast<'a, 'id>) -> ModuleExportName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_specifier
                .imported
        };
        ModuleExportName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_specifier
                .local
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_specifier
                .import_kind
        }
    }
    #[inline]
    pub fn set_imported<'a>(self, ast: &mut crate::Ast<'a, 'id>, imported: ModuleExportName<'id>) {
        let field_val: u32 = imported.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_specifier
                .imported = field_val;
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a, 'id>, local: BindingIdentifier<'id>) {
        let field_val: u32 = local.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_specifier
                .local = field_val;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        import_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_specifier
                .import_kind = import_kind;
        }
    }
}
impl<'id> ImportDefaultSpecifier<'id> {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_default_specifier
                .local
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a, 'id>, local: BindingIdentifier<'id>) {
        let field_val: u32 = local.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_default_specifier
                .local = field_val;
        }
    }
}
impl<'id> ImportNamespaceSpecifier<'id> {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_namespace_specifier
                .local
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a, 'id>, local: BindingIdentifier<'id>) {
        let field_val: u32 = local.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_namespace_specifier
                .local = field_val;
        }
    }
}
impl<'id> ImportAttribute<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> ModuleExportName<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).import_attribute.key };
        ModuleExportName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .import_attribute
                .value
        };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: ModuleExportName<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_attribute
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: StringLiteral<'id>) {
        let field_val: u32 = value.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .import_attribute
                .value = field_val;
        }
    }
}
impl<'id> ExportNamedDeclaration<'id> {
    #[inline]
    pub fn declaration<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Decl<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .declaration
        };
        (raw != u32::MAX).then(|| Decl(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<StringLiteral<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .source
        };
        (raw != u32::MAX).then(|| StringLiteral(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .export_kind
        }
    }
    #[inline]
    pub fn specifiers<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, ExportSpecifier<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, ImportAttribute<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_declaration<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        declaration: Option<Decl<'id>>,
    ) {
        let field_val: u32 = declaration.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_named_declaration
                .declaration = field_val;
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a, 'id>, source: Option<StringLiteral<'id>>) {
        let field_val: u32 = source.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_named_declaration
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        export_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_named_declaration
                .export_kind = export_kind;
        }
    }
    #[inline]
    pub fn set_specifiers<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        specifiers: SubRange<'id, ExportSpecifier<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = specifiers.into_extra_data();
        };
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_named_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl<'id> ExportDefaultDeclaration<'id> {
    #[inline]
    pub fn declaration<'a>(self, ast: &crate::Ast<'a, 'id>) -> ExportDefaultDeclarationKind<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_default_declaration
                .declaration
        };
        ExportDefaultDeclarationKind(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_declaration<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        declaration: ExportDefaultDeclarationKind<'id>,
    ) {
        let field_val: u32 = declaration.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_default_declaration
                .declaration = field_val;
        }
    }
}
impl<'id> ExportAllDeclaration<'id> {
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<ModuleExportName<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_all_declaration
                .exported
        };
        (raw != u32::MAX).then(|| ModuleExportName(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_all_declaration
                .source
        };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_all_declaration
                .export_kind
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, ImportAttribute<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_all_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_exported<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        exported: Option<ModuleExportName<'id>>,
    ) {
        let field_val: u32 = exported.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_all_declaration
                .exported = field_val;
        }
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a, 'id>, source: StringLiteral<'id>) {
        let field_val: u32 = source.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_all_declaration
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        export_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_all_declaration
                .export_kind = export_kind;
        }
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_all_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl<'id> ExportSpecifier<'id> {
    #[inline]
    pub fn local<'a>(self, ast: &crate::Ast<'a, 'id>) -> ModuleExportName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_specifier
                .local
        };
        ModuleExportName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn exported<'a>(self, ast: &crate::Ast<'a, 'id>) -> ModuleExportName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_specifier
                .exported
        };
        ModuleExportName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn export_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .export_specifier
                .export_kind
        }
    }
    #[inline]
    pub fn set_local<'a>(self, ast: &mut crate::Ast<'a, 'id>, local: ModuleExportName<'id>) {
        let field_val: u32 = local.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_specifier
                .local = field_val;
        }
    }
    #[inline]
    pub fn set_exported<'a>(self, ast: &mut crate::Ast<'a, 'id>, exported: ModuleExportName<'id>) {
        let field_val: u32 = exported.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_specifier
                .exported = field_val;
        }
    }
    #[inline]
    pub fn set_export_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        export_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .export_specifier
                .export_kind = export_kind;
        }
    }
}
impl<'id> Hashbang<'id> {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).hashbang.value };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: AstUtf8<'id>) {
        let field_val: u64 = value.raw();
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).hashbang.value = field_val;
        }
    }
}
impl<'id> TsType<'id> {
    #[inline]
    pub fn is_ts_any_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsAnyKeyword
    }
    #[inline]
    pub fn is_ts_unknown_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUnknownKeyword
    }
    #[inline]
    pub fn is_ts_never_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNeverKeyword
    }
    #[inline]
    pub fn is_ts_void_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsVoidKeyword
    }
    #[inline]
    pub fn is_ts_null_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNullKeyword
    }
    #[inline]
    pub fn is_ts_undefined_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUndefinedKeyword
    }
    #[inline]
    pub fn is_ts_string_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsStringKeyword
    }
    #[inline]
    pub fn is_ts_number_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNumberKeyword
    }
    #[inline]
    pub fn is_ts_big_int_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsBigIntKeyword
    }
    #[inline]
    pub fn is_ts_boolean_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsBooleanKeyword
    }
    #[inline]
    pub fn is_ts_symbol_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsSymbolKeyword
    }
    #[inline]
    pub fn is_ts_object_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsObjectKeyword
    }
    #[inline]
    pub fn is_ts_intrinsic_keyword(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIntrinsicKeyword
    }
    #[inline]
    pub fn is_ts_this_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsThisType
    }
    #[inline]
    pub fn is_ts_type_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeReference
    }
    #[inline]
    pub fn is_ts_type_query(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeQuery
    }
    #[inline]
    pub fn is_ts_import_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportType
    }
    #[inline]
    pub fn is_ts_literal_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsLiteralType
    }
    #[inline]
    pub fn is_ts_template_literal_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTemplateLiteralType
    }
    #[inline]
    pub fn is_ts_array_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsArrayType
    }
    #[inline]
    pub fn is_ts_indexed_access_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexedAccessType
    }
    #[inline]
    pub fn is_ts_tuple_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTupleType
    }
    #[inline]
    pub fn is_ts_optional_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsOptionalType
    }
    #[inline]
    pub fn is_ts_rest_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsRestType
    }
    #[inline]
    pub fn is_ts_js_doc_nullable_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocNullableType
    }
    #[inline]
    pub fn is_ts_js_doc_non_nullable_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocNonNullableType
    }
    #[inline]
    pub fn is_ts_js_doc_unknown_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsJsDocUnknownType
    }
    #[inline]
    pub fn is_ts_union_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsUnionType
    }
    #[inline]
    pub fn is_ts_intersection_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIntersectionType
    }
    #[inline]
    pub fn is_ts_conditional_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConditionalType
    }
    #[inline]
    pub fn is_ts_infer_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsInferType
    }
    #[inline]
    pub fn is_ts_type_operator(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeOperator
    }
    #[inline]
    pub fn is_ts_parenthesized_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsParenthesizedType
    }
    #[inline]
    pub fn is_ts_function_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsFunctionType
    }
    #[inline]
    pub fn is_ts_constructor_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConstructorType
    }
    #[inline]
    pub fn is_ts_type_predicate(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypePredicate
    }
    #[inline]
    pub fn is_ts_type_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsTypeLiteral
    }
    #[inline]
    pub fn is_ts_mapped_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMappedType
    }
    #[inline]
    pub fn as_ts_any_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsAnyKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsAnyKeyword) {
            Some(TsAnyKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_unknown_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsUnknownKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsUnknownKeyword) {
            Some(TsUnknownKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_never_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsNeverKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNeverKeyword) {
            Some(TsNeverKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_void_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsVoidKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsVoidKeyword) {
            Some(TsVoidKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_null_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsNullKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNullKeyword) {
            Some(TsNullKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_undefined_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsUndefinedKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsUndefinedKeyword) {
            Some(TsUndefinedKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_string_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsStringKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsStringKeyword) {
            Some(TsStringKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_number_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsNumberKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNumberKeyword) {
            Some(TsNumberKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_big_int_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsBigIntKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsBigIntKeyword) {
            Some(TsBigIntKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_boolean_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsBooleanKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsBooleanKeyword) {
            Some(TsBooleanKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_symbol_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsSymbolKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsSymbolKeyword) {
            Some(TsSymbolKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_object_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsObjectKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsObjectKeyword) {
            Some(TsObjectKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_intrinsic_keyword(self, ast: &Ast<'_, 'id>) -> Option<TsIntrinsicKeyword<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsIntrinsicKeyword) {
            Some(TsIntrinsicKeyword(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_, 'id>) -> Option<TsThisType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsThisType) {
            Some(TsThisType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_reference(self, ast: &Ast<'_, 'id>) -> Option<TsTypeReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeReference) {
            Some(TsTypeReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_query(self, ast: &Ast<'_, 'id>) -> Option<TsTypeQuery<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeQuery) {
            Some(TsTypeQuery(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_, 'id>) -> Option<TsImportType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsImportType) {
            Some(TsImportType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_literal_type(self, ast: &Ast<'_, 'id>) -> Option<TsLiteralType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsLiteralType) {
            Some(TsLiteralType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_template_literal_type(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsTemplateLiteralType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTemplateLiteralType) {
            Some(TsTemplateLiteralType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_array_type(self, ast: &Ast<'_, 'id>) -> Option<TsArrayType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsArrayType) {
            Some(TsArrayType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_indexed_access_type(self, ast: &Ast<'_, 'id>) -> Option<TsIndexedAccessType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsIndexedAccessType) {
            Some(TsIndexedAccessType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_tuple_type(self, ast: &Ast<'_, 'id>) -> Option<TsTupleType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTupleType) {
            Some(TsTupleType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_optional_type(self, ast: &Ast<'_, 'id>) -> Option<TsOptionalType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsOptionalType) {
            Some(TsOptionalType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_rest_type(self, ast: &Ast<'_, 'id>) -> Option<TsRestType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsRestType) {
            Some(TsRestType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_nullable_type(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsJsDocNullableType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsJsDocNullableType) {
            Some(TsJsDocNullableType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_non_nullable_type(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsJsDocNonNullableType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsJsDocNonNullableType) {
            Some(TsJsDocNonNullableType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_js_doc_unknown_type(self, ast: &Ast<'_, 'id>) -> Option<TsJsDocUnknownType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsJsDocUnknownType) {
            Some(TsJsDocUnknownType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_union_type(self, ast: &Ast<'_, 'id>) -> Option<TsUnionType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsUnionType) {
            Some(TsUnionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_intersection_type(self, ast: &Ast<'_, 'id>) -> Option<TsIntersectionType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsIntersectionType) {
            Some(TsIntersectionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_conditional_type(self, ast: &Ast<'_, 'id>) -> Option<TsConditionalType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsConditionalType) {
            Some(TsConditionalType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_infer_type(self, ast: &Ast<'_, 'id>) -> Option<TsInferType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsInferType) {
            Some(TsInferType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_operator(self, ast: &Ast<'_, 'id>) -> Option<TsTypeOperator<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeOperator) {
            Some(TsTypeOperator(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_parenthesized_type(self, ast: &Ast<'_, 'id>) -> Option<TsParenthesizedType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsParenthesizedType) {
            Some(TsParenthesizedType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_function_type(self, ast: &Ast<'_, 'id>) -> Option<TsFunctionType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsFunctionType) {
            Some(TsFunctionType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_constructor_type(self, ast: &Ast<'_, 'id>) -> Option<TsConstructorType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsConstructorType) {
            Some(TsConstructorType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_predicate(self, ast: &Ast<'_, 'id>) -> Option<TsTypePredicate<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypePredicate) {
            Some(TsTypePredicate(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_type_literal(self, ast: &Ast<'_, 'id>) -> Option<TsTypeLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsTypeLiteral) {
            Some(TsTypeLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_mapped_type(self, ast: &Ast<'_, 'id>) -> Option<TsMappedType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsMappedType) {
            Some(TsMappedType(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsTypeName<'id> {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_, 'id>) -> Option<IdentifierReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> Option<TsQualifiedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsQualifiedNameLeft<'id> {
    #[inline]
    pub fn is_identifier_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierReference
    }
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_reference(self, ast: &Ast<'_, 'id>) -> Option<IdentifierReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierReference) {
            Some(IdentifierReference(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_, 'id>) -> Option<BindingIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_, 'id>) -> Option<ThisExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> Option<TsQualifiedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsImportTypeQualifier<'id> {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> Option<TsQualifiedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsTypeQueryExprName<'id> {
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        )
    }
    #[inline]
    pub fn is_this_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::ThisExpression
    }
    #[inline]
    pub fn is_ts_import_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsImportType
    }
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_, 'id>) -> Option<TsTypeName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        ) {
            Some(TsTypeName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_this_expression(self, ast: &Ast<'_, 'id>) -> Option<ThisExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::ThisExpression) {
            Some(ThisExpression(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_import_type(self, ast: &Ast<'_, 'id>) -> Option<TsImportType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsImportType) {
            Some(TsImportType(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsLiteral<'id> {
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_numeric_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::NumericLiteral
    }
    #[inline]
    pub fn is_big_int_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BigIntLiteral
    }
    #[inline]
    pub fn is_boolean_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BooleanLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn is_unary_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::UnaryExpression
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_numeric_literal(self, ast: &Ast<'_, 'id>) -> Option<NumericLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::NumericLiteral) {
            Some(NumericLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_big_int_literal(self, ast: &Ast<'_, 'id>) -> Option<BigIntLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BigIntLiteral) {
            Some(BigIntLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_boolean_literal(self, ast: &Ast<'_, 'id>) -> Option<BooleanLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BooleanLiteral) {
            Some(BooleanLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_, 'id>) -> Option<TemplateLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_unary_expression(self, ast: &Ast<'_, 'id>) -> Option<UnaryExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::UnaryExpression) {
            Some(UnaryExpression(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsTupleElement<'id> {
    #[inline]
    pub fn is_ts_type(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::TsMappedType
        )
    }
    #[inline]
    pub fn is_ts_named_tuple_member(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsNamedTupleMember
    }
    #[inline]
    pub fn as_ts_type(self, ast: &Ast<'_, 'id>) -> Option<TsType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::TsMappedType
        ) {
            Some(TsType(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_named_tuple_member(self, ast: &Ast<'_, 'id>) -> Option<TsNamedTupleMember<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsNamedTupleMember) {
            Some(TsNamedTupleMember(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsTypeElement<'id> {
    #[inline]
    pub fn is_ts_property_signature(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsPropertySignature
    }
    #[inline]
    pub fn is_ts_method_signature(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsMethodSignature
    }
    #[inline]
    pub fn is_ts_call_signature_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsCallSignatureDeclaration
    }
    #[inline]
    pub fn is_ts_construct_signature_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsConstructSignatureDeclaration
    }
    #[inline]
    pub fn is_ts_index_signature(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsIndexSignature
    }
    #[inline]
    pub fn as_ts_property_signature(self, ast: &Ast<'_, 'id>) -> Option<TsPropertySignature<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsPropertySignature) {
            Some(TsPropertySignature(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_method_signature(self, ast: &Ast<'_, 'id>) -> Option<TsMethodSignature<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsMethodSignature) {
            Some(TsMethodSignature(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_call_signature_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsCallSignatureDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsCallSignatureDeclaration) {
            Some(TsCallSignatureDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_construct_signature_declaration(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsConstructSignatureDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsConstructSignatureDeclaration) {
            Some(TsConstructSignatureDeclaration(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_index_signature(self, ast: &Ast<'_, 'id>) -> Option<TsIndexSignature<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsIndexSignature) {
            Some(TsIndexSignature(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsTypePredicateName<'id> {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_ts_this_type(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsThisType
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_this_type(self, ast: &Ast<'_, 'id>) -> Option<TsThisType<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsThisType) {
            Some(TsThisType(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsEnumMemberName<'id> {
    #[inline]
    pub fn is_identifier_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::IdentifierName
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_template_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TemplateLiteral
    }
    #[inline]
    pub fn as_identifier_name(self, ast: &Ast<'_, 'id>) -> Option<IdentifierName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::IdentifierName) {
            Some(IdentifierName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_template_literal(self, ast: &Ast<'_, 'id>) -> Option<TemplateLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TemplateLiteral) {
            Some(TemplateLiteral(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsModuleName<'id> {
    #[inline]
    pub fn is_binding_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::BindingIdentifier
    }
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsQualifiedName
    }
    #[inline]
    pub fn as_binding_identifier(self, ast: &Ast<'_, 'id>) -> Option<BindingIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::BindingIdentifier) {
            Some(BindingIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_qualified_name(self, ast: &Ast<'_, 'id>) -> Option<TsQualifiedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsQualifiedName) {
            Some(TsQualifiedName(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsModuleBody<'id> {
    #[inline]
    pub fn is_ts_module_block(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleBlock
    }
    #[inline]
    pub fn is_ts_module_declaration(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsModuleDeclaration
    }
    #[inline]
    pub fn as_ts_module_block(self, ast: &Ast<'_, 'id>) -> Option<TsModuleBlock<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsModuleBlock) {
            Some(TsModuleBlock(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_module_declaration(self, ast: &Ast<'_, 'id>) -> Option<TsModuleDeclaration<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsModuleDeclaration) {
            Some(TsModuleDeclaration(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsModuleReference<'id> {
    #[inline]
    pub fn is_ts_type_name(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        )
    }
    #[inline]
    pub fn is_ts_external_module_reference(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::TsExternalModuleReference
    }
    #[inline]
    pub fn as_ts_type_name(self, ast: &Ast<'_, 'id>) -> Option<TsTypeName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
            NodeKind::IdentifierReference | NodeKind::TsQualifiedName
        ) {
            Some(TsTypeName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_ts_external_module_reference(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<TsExternalModuleReference<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::TsExternalModuleReference) {
            Some(TsExternalModuleReference(self.0))
        } else {
            None
        }
    }
}
impl<'id> TsFunction<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<BindingIdentifier<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_function.id };
        (raw != u32::MAX).then(|| BindingIdentifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn generator<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_function.generator }
    }
    #[inline]
    pub fn r#async<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_function.r#async }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_function.declare }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsThisParameter<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: Option<BindingIdentifier<'id>>) {
        let field_val: u32 = id.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).ts_function.id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_generator<'a>(self, ast: &mut crate::Ast<'a, 'id>, generator: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function
                .generator = generator;
        }
    }
    #[inline]
    pub fn set_async<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#async: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function
                .r#async = r#async;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function
                .declare = declare;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        this_param: Option<TsThisParameter<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
}
impl<'id> TsMethodDefinition<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .key
        };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> MethodDefinitionKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .kind
        }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .computed
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .r#static
        }
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .r#override
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .optional
        }
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .r#abstract
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a, 'id>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .accessibility
        }
    }
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsFunction<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: MethodDefinitionKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .r#static = r#static;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .r#abstract = r#abstract;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        accessibility: Accessibility,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_definition
                .accessibility = accessibility;
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = decorators.into_extra_data();
        };
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: TsFunction<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_definition
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = value.into_extra_data();
        };
    }
}
impl<'id> TsTypeAnnotation<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_annotation
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_annotation
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsAnyKeyword<'id> {}
impl<'id> TsUnknownKeyword<'id> {}
impl<'id> TsNeverKeyword<'id> {}
impl<'id> TsVoidKeyword<'id> {}
impl<'id> TsNullKeyword<'id> {}
impl<'id> TsUndefinedKeyword<'id> {}
impl<'id> TsStringKeyword<'id> {}
impl<'id> TsNumberKeyword<'id> {}
impl<'id> TsBigIntKeyword<'id> {}
impl<'id> TsBooleanKeyword<'id> {}
impl<'id> TsSymbolKeyword<'id> {}
impl<'id> TsObjectKeyword<'id> {}
impl<'id> TsIntrinsicKeyword<'id> {}
impl<'id> TsThisType<'id> {}
impl<'id> TsTypeReference<'id> {
    #[inline]
    pub fn type_name<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_reference
                .type_name
        };
        TsTypeName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_reference
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_type_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, type_name: TsTypeName<'id>) {
        let field_val: u32 = type_name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_reference
                .type_name = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_reference
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsQualifiedName<'id> {
    #[inline]
    pub fn left<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsQualifiedNameLeft<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_qualified_name
                .left
        };
        TsQualifiedNameLeft(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn right<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_qualified_name
                .right
        };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_left<'a>(self, ast: &mut crate::Ast<'a, 'id>, left: TsQualifiedNameLeft<'id>) {
        let field_val: u32 = left.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_qualified_name
                .left = field_val;
        }
    }
    #[inline]
    pub fn set_right<'a>(self, ast: &mut crate::Ast<'a, 'id>, right: IdentifierName<'id>) {
        let field_val: u32 = right.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_qualified_name
                .right = field_val;
        }
    }
}
impl<'id> TsTypeQuery<'id> {
    #[inline]
    pub fn expr_name<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeQueryExprName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_query
                .expr_name
        };
        TsTypeQueryExprName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_query
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_expr_name<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        expr_name: TsTypeQueryExprName<'id>,
    ) {
        let field_val: u32 = expr_name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_query
                .expr_name = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_query
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsImportType<'id> {
    #[inline]
    pub fn source<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_import_type.source };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn options<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_type
                .options
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn qualifier<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsImportTypeQualifier<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_type
                .qualifier
        };
        (raw != u32::MAX).then(|| TsImportTypeQualifier(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_type
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_source<'a>(self, ast: &mut crate::Ast<'a, 'id>, source: StringLiteral<'id>) {
        let field_val: u32 = source.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_type
                .source = field_val;
        }
    }
    #[inline]
    pub fn set_options<'a>(self, ast: &mut crate::Ast<'a, 'id>, options: Option<Expr<'id>>) {
        let field_val: u32 = options.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_type
                .options = field_val;
        }
    }
    #[inline]
    pub fn set_qualifier<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        qualifier: Option<TsImportTypeQualifier<'id>>,
    ) {
        let field_val: u32 = qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_type
                .qualifier = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_type
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsTypeParameter<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter
                .name
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsType<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter
                .constraint
        };
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn default<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsType<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter
                .default
        };
        (raw != u32::MAX).then(|| TsType(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn r#in<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter
                .r#in
        }
    }
    #[inline]
    pub fn out<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_type_parameter.out }
    }
    #[inline]
    pub fn r#const<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter
                .r#const
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: BindingIdentifier<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .name = field_val;
        }
    }
    #[inline]
    pub fn set_constraint<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        constraint: Option<TsType<'id>>,
    ) {
        let field_val: u32 = constraint.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .constraint = field_val;
        }
    }
    #[inline]
    pub fn set_default<'a>(self, ast: &mut crate::Ast<'a, 'id>, default: Option<TsType<'id>>) {
        let field_val: u32 = default.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .default = field_val;
        }
    }
    #[inline]
    pub fn set_in<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#in: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .r#in = r#in;
        }
    }
    #[inline]
    pub fn set_out<'a>(self, ast: &mut crate::Ast<'a, 'id>, out: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .out = out;
        }
    }
    #[inline]
    pub fn set_const<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#const: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter
                .r#const = r#const;
        }
    }
}
impl<'id> TsTypeParameterDeclaration<'id> {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsTypeParameter<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter_declaration
                .params
        })
    }
    #[inline]
    pub fn set_params<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        params: SubRange<'id, TsTypeParameter<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter_declaration
                .params = params.raw();
        }
    }
}
impl<'id> TsTypeParameterInstantiation<'id> {
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsType<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_parameter_instantiation
                .params
        })
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: SubRange<'id, TsType<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_parameter_instantiation
                .params = params.raw();
        }
    }
}
impl<'id> TsLiteralType<'id> {
    #[inline]
    pub fn literal<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_literal_type
                .literal
        };
        TsLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_literal<'a>(self, ast: &mut crate::Ast<'a, 'id>, literal: TsLiteral<'id>) {
        let field_val: u32 = literal.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_literal_type
                .literal = field_val;
        }
    }
}
impl<'id> TsTemplateLiteralType<'id> {
    #[inline]
    pub fn quasis<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TemplateElement<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_template_literal_type
                .quasis
        })
    }
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsType<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_template_literal_type
                .types
        })
    }
    #[inline]
    pub fn set_quasis<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        quasis: SubRange<'id, TemplateElement<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_template_literal_type
                .quasis = quasis.raw();
        }
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a, 'id>, types: SubRange<'id, TsType<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_template_literal_type
                .types = types.raw();
        }
    }
}
impl<'id> TsArrayType<'id> {
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_array_type
                .element_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, element_type: TsType<'id>) {
        let field_val: u32 = element_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_array_type
                .element_type = field_val;
        }
    }
}
impl<'id> TsIndexedAccessType<'id> {
    #[inline]
    pub fn object_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_indexed_access_type
                .object_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn index_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_indexed_access_type
                .index_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_object_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, object_type: TsType<'id>) {
        let field_val: u32 = object_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_indexed_access_type
                .object_type = field_val;
        }
    }
    #[inline]
    pub fn set_index_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, index_type: TsType<'id>) {
        let field_val: u32 = index_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_indexed_access_type
                .index_type = field_val;
        }
    }
}
impl<'id> TsTupleType<'id> {
    #[inline]
    pub fn element_types<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, TsTupleElement<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_tuple_type
                .element_types
        })
    }
    #[inline]
    pub fn set_element_types<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        element_types: SubRange<'id, TsTupleElement<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_tuple_type
                .element_types = element_types.raw();
        }
    }
}
impl<'id> TsNamedTupleMember<'id> {
    #[inline]
    pub fn label<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_named_tuple_member
                .label
        };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn element_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_named_tuple_member
                .element_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_named_tuple_member
                .optional
        }
    }
    #[inline]
    pub fn set_label<'a>(self, ast: &mut crate::Ast<'a, 'id>, label: IdentifierName<'id>) {
        let field_val: u32 = label.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_named_tuple_member
                .label = field_val;
        }
    }
    #[inline]
    pub fn set_element_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, element_type: TsType<'id>) {
        let field_val: u32 = element_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_named_tuple_member
                .element_type = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_named_tuple_member
                .optional = optional;
        }
    }
}
impl<'id> TsOptionalType<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_optional_type
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_optional_type
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsRestType<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTupleElement<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_rest_type
                .type_annotation
        };
        TsTupleElement(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsTupleElement<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_rest_type
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsJsDocNullableType<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_js_doc_nullable_type
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_js_doc_nullable_type
                .postfix
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_js_doc_nullable_type
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a, 'id>, postfix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_js_doc_nullable_type
                .postfix = postfix;
        }
    }
}
impl<'id> TsJsDocNonNullableType<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_js_doc_non_nullable_type
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn postfix<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_js_doc_non_nullable_type
                .postfix
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_js_doc_non_nullable_type
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_postfix<'a>(self, ast: &mut crate::Ast<'a, 'id>, postfix: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_js_doc_non_nullable_type
                .postfix = postfix;
        }
    }
}
impl<'id> TsJsDocUnknownType<'id> {}
impl<'id> TsUnionType<'id> {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsType<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_union_type.types })
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a, 'id>, types: SubRange<'id, TsType<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_union_type
                .types = types.raw();
        }
    }
}
impl<'id> TsIntersectionType<'id> {
    #[inline]
    pub fn types<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsType<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_intersection_type
                .types
        })
    }
    #[inline]
    pub fn set_types<'a>(self, ast: &mut crate::Ast<'a, 'id>, types: SubRange<'id, TsType<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_intersection_type
                .types = types.raw();
        }
    }
}
impl<'id> TsConditionalType<'id> {
    #[inline]
    pub fn check_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_conditional_type
                .check_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn extends_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_conditional_type
                .extends_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn true_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_conditional_type
                .true_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn false_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_conditional_type
                .false_type
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_check_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, check_type: TsType<'id>) {
        let field_val: u32 = check_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_conditional_type
                .check_type = field_val;
        }
    }
    #[inline]
    pub fn set_extends_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, extends_type: TsType<'id>) {
        let field_val: u32 = extends_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_conditional_type
                .extends_type = field_val;
        }
    }
    #[inline]
    pub fn set_true_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, true_type: TsType<'id>) {
        let field_val: u32 = true_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_conditional_type
                .true_type = field_val;
        }
    }
    #[inline]
    pub fn set_false_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, false_type: TsType<'id>) {
        let field_val: u32 = false_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_conditional_type
                .false_type = field_val;
        }
    }
}
impl<'id> TsInferType<'id> {
    #[inline]
    pub fn type_parameter<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeParameter<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_infer_type
                .type_parameter
        };
        TsTypeParameter(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_parameter<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameter: TsTypeParameter<'id>,
    ) {
        let field_val: u32 = type_parameter.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_infer_type
                .type_parameter = field_val;
        }
    }
}
impl<'id> TsTypeOperator<'id> {
    #[inline]
    pub fn operator<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeOperatorKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_operator
                .operator
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_operator
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_operator<'a>(self, ast: &mut crate::Ast<'a, 'id>, operator: TsTypeOperatorKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_operator
                .operator = operator;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_operator
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsParenthesizedType<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parenthesized_type
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parenthesized_type
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsFunctionType<'id> {
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function_type
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsThisParameter<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function_type
                .this_param
        };
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function_type
                .params
        };
        FormalParameters(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeAnnotation<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_function_type
                .return_type
        };
        TsTypeAnnotation(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function_type
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        this_param: Option<TsThisParameter<'id>>,
    ) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function_type
                .this_param = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let field_val: u32 = params.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function_type
                .params = field_val;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: TsTypeAnnotation<'id>,
    ) {
        let field_val: u32 = return_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_function_type
                .return_type = field_val;
        }
    }
}
impl<'id> TsConstructorType<'id> {
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_constructor_type
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_constructor_type
                .params
        };
        FormalParameters(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeAnnotation<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_constructor_type
                .return_type
        };
        TsTypeAnnotation(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn r#abstract<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_constructor_type
                .r#abstract
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_constructor_type
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let field_val: u32 = params.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_constructor_type
                .params = field_val;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: TsTypeAnnotation<'id>,
    ) {
        let field_val: u32 = return_type.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_constructor_type
                .return_type = field_val;
        }
    }
    #[inline]
    pub fn set_abstract<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#abstract: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_constructor_type
                .r#abstract = r#abstract;
        }
    }
}
impl<'id> TsTypePredicate<'id> {
    #[inline]
    pub fn parameter_name<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypePredicateName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_predicate
                .parameter_name
        };
        TsTypePredicateName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_predicate
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn asserts<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_predicate
                .asserts
        }
    }
    #[inline]
    pub fn set_parameter_name<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        parameter_name: TsTypePredicateName<'id>,
    ) {
        let field_val: u32 = parameter_name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_predicate
                .parameter_name = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_predicate
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_asserts<'a>(self, ast: &mut crate::Ast<'a, 'id>, asserts: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_predicate
                .asserts = asserts;
        }
    }
}
impl<'id> TsTypeLiteral<'id> {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsTypeElement<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_literal
                .members
        })
    }
    #[inline]
    pub fn set_members<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        members: SubRange<'id, TsTypeElement<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_literal
                .members = members.raw();
        }
    }
}
impl<'id> TsMappedType<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_mapped_type.key };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn constraint<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .constraint
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsMappedTypeModifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .optional
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsMappedTypeModifier {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .readonly
        }
    }
    #[inline]
    pub fn name_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsType<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsType<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: BindingIdentifier<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_mapped_type
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_constraint<'a>(self, ast: &mut crate::Ast<'a, 'id>, constraint: TsType<'id>) {
        let field_val: u32 = constraint.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_mapped_type
                .constraint = field_val;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: TsMappedTypeModifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_mapped_type
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a, 'id>, readonly: TsMappedTypeModifier) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_mapped_type
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_name_type<'a>(self, ast: &mut crate::Ast<'a, 'id>, name_type: Option<TsType<'id>>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = name_type.into_extra_data();
        };
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsType<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_mapped_type
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = type_annotation.into_extra_data();
        };
    }
}
impl<'id> TsPropertySignature<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_property_signature
                .key
        };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_property_signature
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_property_signature
                .computed
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_property_signature
                .optional
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_property_signature
                .readonly
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_property_signature
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_property_signature
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_property_signature
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_property_signature
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a, 'id>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_property_signature
                .readonly = readonly;
        }
    }
}
impl<'id> TsMethodSignature<'id> {
    #[inline]
    pub fn key<'a>(self, ast: &crate::Ast<'a, 'id>) -> PropertyKey<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .key
        };
        PropertyKey(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsMethodSignatureKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .kind
        }
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .computed
        }
    }
    #[inline]
    pub fn optional<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .optional
        }
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsThisParameter<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_key<'a>(self, ast: &mut crate::Ast<'a, 'id>, key: PropertyKey<'id>) {
        let field_val: u32 = key.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_signature
                .key = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_signature
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: TsMethodSignatureKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_signature
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_signature
                .computed = computed;
        }
    }
    #[inline]
    pub fn set_optional<'a>(self, ast: &mut crate::Ast<'a, 'id>, optional: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_method_signature
                .optional = optional;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        this_param: Option<TsThisParameter<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = this_param.into_extra_data();
        };
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = params.into_extra_data();
        };
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_method_signature
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(2usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = return_type.into_extra_data();
        };
    }
}
impl<'id> TsCallSignatureDeclaration<'id> {
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn this_param<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsThisParameter<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .this_param
        };
        (raw != u32::MAX).then(|| TsThisParameter(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .params
        };
        FormalParameters(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .return_type
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_this_param<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        this_param: Option<TsThisParameter<'id>>,
    ) {
        let field_val: u32 = this_param.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .this_param = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let field_val: u32 = params.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .params = field_val;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_call_signature_declaration
                .return_type = field_val;
        }
    }
}
impl<'id> TsConstructSignatureDeclaration<'id> {
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn params<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameters<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .params
        };
        FormalParameters(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn return_type<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .return_type
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_params<'a>(self, ast: &mut crate::Ast<'a, 'id>, params: FormalParameters<'id>) {
        let field_val: u32 = params.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .params = field_val;
        }
    }
    #[inline]
    pub fn set_return_type<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = return_type.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_construct_signature_declaration
                .return_type = field_val;
        }
    }
}
impl<'id> TsIndexSignature<'id> {
    #[inline]
    pub fn parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> SubRange<'id, TsIndexSignatureParameter<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature
                .parameters
        })
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeAnnotation<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature
                .type_annotation
        };
        TsTypeAnnotation(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature
                .readonly
        }
    }
    #[inline]
    pub fn r#static<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature
                .r#static
        }
    }
    #[inline]
    pub fn set_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        parameters: SubRange<'id, TsIndexSignatureParameter<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature
                .parameters = parameters.raw();
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsTypeAnnotation<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a, 'id>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_static<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#static: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature
                .r#static = r#static;
        }
    }
}
impl<'id> TsTypeAliasDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .id
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: BindingIdentifier<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_alias_declaration
                .declare = declare;
        }
    }
}
impl<'id> TsInterfaceDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .id
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_parameters<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterDeclaration<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .type_parameters
        };
        (raw != u32::MAX).then(|| TsTypeParameterDeclaration(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .declare
        }
    }
    #[inline]
    pub fn extends<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsInterfaceHeritage<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsInterfaceBody<'id> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: BindingIdentifier<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_type_parameters<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
    ) {
        let field_val: u32 = type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_declaration
                .type_parameters = field_val;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_declaration
                .declare = declare;
        }
    }
    #[inline]
    pub fn set_extends<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        extends: SubRange<'id, TsInterfaceHeritage<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = extends.into_extra_data();
        };
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: TsInterfaceBody<'id>) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_declaration
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(1usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = body.into_extra_data();
        };
    }
}
impl<'id> TsInterfaceBody<'id> {
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsTypeElement<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_body
                .body
        })
    }
    #[inline]
    pub fn set_body<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        body: SubRange<'id, TsTypeElement<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_body
                .body = body.raw();
        }
    }
}
impl<'id> TsInterfaceHeritage<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_heritage
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_interface_heritage
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_heritage
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_interface_heritage
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsClassImplements<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_class_implements
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_class_implements
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_class_implements
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_class_implements
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsEnumDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_declaration
                .id
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsEnumBody<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_declaration
                .body
        };
        TsEnumBody(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn is_const<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_declaration
                .is_const
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: BindingIdentifier<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: TsEnumBody<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_declaration
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_is_const<'a>(self, ast: &mut crate::Ast<'a, 'id>, is_const: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_declaration
                .is_const = is_const;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_declaration
                .declare = declare;
        }
    }
}
impl<'id> TsEnumBody<'id> {
    #[inline]
    pub fn members<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, TsEnumMember<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_enum_body.members })
    }
    #[inline]
    pub fn set_members<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        members: SubRange<'id, TsEnumMember<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_body
                .members = members.raw();
        }
    }
}
impl<'id> TsEnumMember<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsEnumMemberName<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_enum_member.id };
        TsEnumMemberName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn initializer<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<Expr<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_member
                .initializer
        };
        (raw != u32::MAX).then(|| Expr(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn computed<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_enum_member
                .computed
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: TsEnumMemberName<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).ts_enum_member.id = field_val;
        }
    }
    #[inline]
    pub fn set_initializer<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        initializer: Option<Expr<'id>>,
    ) {
        let field_val: u32 = initializer.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_member
                .initializer = field_val;
        }
    }
    #[inline]
    pub fn set_computed<'a>(self, ast: &mut crate::Ast<'a, 'id>, computed: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_enum_member
                .computed = computed;
        }
    }
}
impl<'id> TsModuleDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsModuleName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_module_declaration
                .id
        };
        TsModuleName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsModuleBody<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_module_declaration
                .body
        };
        (raw != u32::MAX).then(|| TsModuleBody(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsModuleDeclarationKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_module_declaration
                .kind
        }
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_module_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: TsModuleName<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: Option<TsModuleBody<'id>>) {
        let field_val: u32 = body.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_declaration
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_kind<'a>(self, ast: &mut crate::Ast<'a, 'id>, kind: TsModuleDeclarationKind) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_declaration
                .kind = kind;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_declaration
                .declare = declare;
        }
    }
}
impl<'id> TsModuleBlock<'id> {
    #[inline]
    pub fn directives<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Directive<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_module_block
                .directives
        })
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Stmt<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).ts_module_block.body })
    }
    #[inline]
    pub fn set_directives<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        directives: SubRange<'id, Directive<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_block
                .directives = directives.raw();
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: SubRange<'id, Stmt<'id>>) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_module_block
                .body = body.raw();
        }
    }
}
impl<'id> TsGlobalDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_global_declaration
                .id
        };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn body<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsModuleBlock<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_global_declaration
                .body
        };
        TsModuleBlock(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn declare<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_global_declaration
                .declare
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: IdentifierName<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_global_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_body<'a>(self, ast: &mut crate::Ast<'a, 'id>, body: TsModuleBlock<'id>) {
        let field_val: u32 = body.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_global_declaration
                .body = field_val;
        }
    }
    #[inline]
    pub fn set_declare<'a>(self, ast: &mut crate::Ast<'a, 'id>, declare: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_global_declaration
                .declare = declare;
        }
    }
}
impl<'id> TsParameterProperty<'id> {
    #[inline]
    pub fn decorators<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, Decorator<'id>> {
        SubRange::from_raw(unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parameter_property
                .decorators
        })
    }
    #[inline]
    pub fn parameter<'a>(self, ast: &crate::Ast<'a, 'id>) -> FormalParameter<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parameter_property
                .parameter
        };
        FormalParameter(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn r#override<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parameter_property
                .r#override
        }
    }
    #[inline]
    pub fn readonly<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parameter_property
                .readonly
        }
    }
    #[inline]
    pub fn accessibility<'a>(self, ast: &crate::Ast<'a, 'id>) -> Accessibility {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_parameter_property
                .accessibility
        }
    }
    #[inline]
    pub fn set_decorators<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        decorators: SubRange<'id, Decorator<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parameter_property
                .decorators = decorators.raw();
        }
    }
    #[inline]
    pub fn set_parameter<'a>(self, ast: &mut crate::Ast<'a, 'id>, parameter: FormalParameter<'id>) {
        let field_val: u32 = parameter.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parameter_property
                .parameter = field_val;
        }
    }
    #[inline]
    pub fn set_override<'a>(self, ast: &mut crate::Ast<'a, 'id>, r#override: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parameter_property
                .r#override = r#override;
        }
    }
    #[inline]
    pub fn set_readonly<'a>(self, ast: &mut crate::Ast<'a, 'id>, readonly: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parameter_property
                .readonly = readonly;
        }
    }
    #[inline]
    pub fn set_accessibility<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        accessibility: Accessibility,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_parameter_property
                .accessibility = accessibility;
        }
    }
}
impl<'id> TsThisParameter<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<TsTypeAnnotation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_this_parameter
                .type_annotation
        };
        (raw != u32::MAX).then(|| TsTypeAnnotation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) {
        let field_val: u32 = type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_this_parameter
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsAsExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_as_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_as_expression
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_as_expression
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_as_expression
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsSatisfiesExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_satisfies_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_satisfies_expression
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_satisfies_expression
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_satisfies_expression
                .type_annotation = field_val;
        }
    }
}
impl<'id> TsTypeAssertion<'id> {
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsType<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_assertion
                .type_annotation
        };
        TsType(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_type_assertion
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsType<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_assertion
                .type_annotation = field_val;
        }
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_type_assertion
                .expression = field_val;
        }
    }
}
impl<'id> TsNonNullExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_non_null_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_non_null_expression
                .expression = field_val;
        }
    }
}
impl<'id> TsInstantiationExpression<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_instantiation_expression
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> TsTypeParameterInstantiation<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_instantiation_expression
                .type_arguments
        };
        TsTypeParameterInstantiation(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_instantiation_expression
                .expression = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) {
        let field_val: u32 = type_arguments.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_instantiation_expression
                .type_arguments = field_val;
        }
    }
}
impl<'id> TsExportAssignment<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_export_assignment
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_export_assignment
                .expression = field_val;
        }
    }
}
impl<'id> TsNamespaceExportDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> IdentifierName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_namespace_export_declaration
                .id
        };
        IdentifierName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: IdentifierName<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_namespace_export_declaration
                .id = field_val;
        }
    }
}
impl<'id> TsImportEqualsDeclaration<'id> {
    #[inline]
    pub fn id<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .id
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn module_reference<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsModuleReference<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .module_reference
        };
        TsModuleReference(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn import_kind<'a>(self, ast: &crate::Ast<'a, 'id>) -> ImportOrExportKind {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .import_kind
        }
    }
    #[inline]
    pub fn set_id<'a>(self, ast: &mut crate::Ast<'a, 'id>, id: BindingIdentifier<'id>) {
        let field_val: u32 = id.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .id = field_val;
        }
    }
    #[inline]
    pub fn set_module_reference<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        module_reference: TsModuleReference<'id>,
    ) {
        let field_val: u32 = module_reference.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .module_reference = field_val;
        }
    }
    #[inline]
    pub fn set_import_kind<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        import_kind: ImportOrExportKind,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_import_equals_declaration
                .import_kind = import_kind;
        }
    }
}
impl<'id> TsExternalModuleReference<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> StringLiteral<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_external_module_reference
                .expression
        };
        StringLiteral(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: StringLiteral<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_external_module_reference
                .expression = field_val;
        }
    }
}
impl<'id> TsIndexSignatureParameter<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> BindingIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature_parameter
                .name
        };
        BindingIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_annotation<'a>(self, ast: &crate::Ast<'a, 'id>) -> TsTypeAnnotation<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .ts_index_signature_parameter
                .type_annotation
        };
        TsTypeAnnotation(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: BindingIdentifier<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature_parameter
                .name = field_val;
        }
    }
    #[inline]
    pub fn set_type_annotation<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_annotation: TsTypeAnnotation<'id>,
    ) {
        let field_val: u32 = type_annotation.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .ts_index_signature_parameter
                .type_annotation = field_val;
        }
    }
}
impl<'id> JsxElementName<'id> {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxNamespacedName
    }
    #[inline]
    pub fn is_jsx_member_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxMemberExpression
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_, 'id>) -> Option<JsxIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_, 'id>) -> Option<JsxNamespacedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxNamespacedName) {
            Some(JsxNamespacedName(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_, 'id>) -> Option<JsxMemberExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxMemberExpression) {
            Some(JsxMemberExpression(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxMemberExpressionObject<'id> {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_member_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxMemberExpression
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_, 'id>) -> Option<JsxIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_member_expression(self, ast: &Ast<'_, 'id>) -> Option<JsxMemberExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxMemberExpression) {
            Some(JsxMemberExpression(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxAttributeName<'id> {
    #[inline]
    pub fn is_jsx_identifier(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxIdentifier
    }
    #[inline]
    pub fn is_jsx_namespaced_name(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxNamespacedName
    }
    #[inline]
    pub fn as_jsx_identifier(self, ast: &Ast<'_, 'id>) -> Option<JsxIdentifier<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxIdentifier) {
            Some(JsxIdentifier(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_namespaced_name(self, ast: &Ast<'_, 'id>) -> Option<JsxNamespacedName<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxNamespacedName) {
            Some(JsxNamespacedName(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxAttributeItem<'id> {
    #[inline]
    pub fn is_jsx_attribute(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxAttribute
    }
    #[inline]
    pub fn is_jsx_spread_attribute(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxSpreadAttribute
    }
    #[inline]
    pub fn as_jsx_attribute(self, ast: &Ast<'_, 'id>) -> Option<JsxAttribute<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxAttribute) {
            Some(JsxAttribute(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_spread_attribute(self, ast: &Ast<'_, 'id>) -> Option<JsxSpreadAttribute<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxSpreadAttribute) {
            Some(JsxSpreadAttribute(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxAttributeValue<'id> {
    #[inline]
    pub fn is_string_literal(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::StringLiteral
    }
    #[inline]
    pub fn is_jsx_expression_container(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxExpressionContainer
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_string_literal(self, ast: &Ast<'_, 'id>) -> Option<StringLiteral<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::StringLiteral) {
            Some(StringLiteral(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<JsxExpressionContainer<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxExpressionContainer) {
            Some(JsxExpressionContainer(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_, 'id>) -> Option<JsxElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_, 'id>) -> Option<JsxFragment<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxExpression<'id> {
    #[inline]
    pub fn is_expr(self, ast: &Ast<'_, 'id>) -> bool {
        matches!(
            ast.node_kind(self.0),
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
                | NodeKind::JsxFragment
        )
    }
    #[inline]
    pub fn is_jsx_empty_expression(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxEmptyExpression
    }
    #[inline]
    pub fn as_expr(self, ast: &Ast<'_, 'id>) -> Option<Expr<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(
            kind,
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
                | NodeKind::JsxFragment
        ) {
            Some(Expr(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_empty_expression(self, ast: &Ast<'_, 'id>) -> Option<JsxEmptyExpression<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxEmptyExpression) {
            Some(JsxEmptyExpression(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxChild<'id> {
    #[inline]
    pub fn is_jsx_text(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxText
    }
    #[inline]
    pub fn is_jsx_expression_container(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxExpressionContainer
    }
    #[inline]
    pub fn is_jsx_spread_child(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxSpreadChild
    }
    #[inline]
    pub fn is_jsx_element(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxElement
    }
    #[inline]
    pub fn is_jsx_fragment(self, ast: &Ast<'_, 'id>) -> bool {
        ast.node_kind(self.0) == NodeKind::JsxFragment
    }
    #[inline]
    pub fn as_jsx_text(self, ast: &Ast<'_, 'id>) -> Option<JsxText<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxText) {
            Some(JsxText(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_expression_container(
        self,
        ast: &Ast<'_, 'id>,
    ) -> Option<JsxExpressionContainer<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxExpressionContainer) {
            Some(JsxExpressionContainer(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_spread_child(self, ast: &Ast<'_, 'id>) -> Option<JsxSpreadChild<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxSpreadChild) {
            Some(JsxSpreadChild(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_element(self, ast: &Ast<'_, 'id>) -> Option<JsxElement<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxElement) {
            Some(JsxElement(self.0))
        } else {
            None
        }
    }
    #[inline]
    pub fn as_jsx_fragment(self, ast: &Ast<'_, 'id>) -> Option<JsxFragment<'id>> {
        let kind = unsafe { *ast.nodes.kind_unchecked(self.0.raw()) };
        if matches!(kind, NodeKind::JsxFragment) {
            Some(JsxFragment(self.0))
        } else {
            None
        }
    }
}
impl<'id> JsxElement<'id> {
    #[inline]
    pub fn opening_element<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxOpeningElement<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_element
                .opening_element
        };
        JsxOpeningElement(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, JsxChild<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_element.children })
    }
    #[inline]
    pub fn closing_element<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<JsxClosingElement<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_element
                .closing_element
        };
        (raw != u32::MAX).then(|| JsxClosingElement(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_opening_element<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        opening_element: JsxOpeningElement<'id>,
    ) {
        let field_val: u32 = opening_element.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_element
                .opening_element = field_val;
        }
    }
    #[inline]
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        children: SubRange<'id, JsxChild<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_element
                .children = children.raw();
        }
    }
    #[inline]
    pub fn set_closing_element<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) {
        let field_val: u32 = closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_element
                .closing_element = field_val;
        }
    }
}
impl<'id> JsxOpeningElement<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxElementName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_opening_element
                .name
        };
        JsxElementName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn type_arguments<'a>(
        self,
        ast: &crate::Ast<'a, 'id>,
    ) -> Option<TsTypeParameterInstantiation<'id>> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_opening_element
                .type_arguments
        };
        (raw != u32::MAX).then(|| TsTypeParameterInstantiation(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn self_closing<'a>(self, ast: &crate::Ast<'a, 'id>) -> bool {
        unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_opening_element
                .self_closing
        }
    }
    #[inline]
    pub fn attributes<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, JsxAttributeItem<'id>> {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_opening_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            ExtraDataValue::from_extra_data(ast.extra_data.as_slice().get_unchecked(offset.index()))
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: JsxElementName<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_opening_element
                .name = field_val;
        }
    }
    #[inline]
    pub fn set_type_arguments<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) {
        let field_val: u32 = type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_opening_element
                .type_arguments = field_val;
        }
    }
    #[inline]
    pub fn set_self_closing<'a>(self, ast: &mut crate::Ast<'a, 'id>, self_closing: bool) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_opening_element
                .self_closing = self_closing;
        }
    }
    #[inline]
    pub fn set_attributes<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        attributes: SubRange<'id, JsxAttributeItem<'id>>,
    ) {
        let start = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_opening_element
                .extra_data_start
                .index()
        };
        let offset = unsafe { start.unchecked_add(0usize) };
        debug_assert!(offset < ast.extra_data.len());
        let offset = ExtraDataId::from_usize_unchecked(offset);
        unsafe {
            *ast.extra_data
                .as_mut_slice()
                .get_unchecked_mut(offset.index()) = attributes.into_extra_data();
        };
    }
}
impl<'id> JsxClosingElement<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxElementName<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_closing_element
                .name
        };
        JsxElementName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: JsxElementName<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_closing_element
                .name = field_val;
        }
    }
}
impl<'id> JsxFragment<'id> {
    #[inline]
    pub fn opening_fragment<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxOpeningFragment<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_fragment
                .opening_fragment
        };
        JsxOpeningFragment(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn children<'a>(self, ast: &crate::Ast<'a, 'id>) -> SubRange<'id, JsxChild<'id>> {
        SubRange::from_raw(unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_fragment.children })
    }
    #[inline]
    pub fn closing_fragment<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxClosingFragment<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_fragment
                .closing_fragment
        };
        JsxClosingFragment(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_opening_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        opening_fragment: JsxOpeningFragment<'id>,
    ) {
        let field_val: u32 = opening_fragment.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_fragment
                .opening_fragment = field_val;
        }
    }
    #[inline]
    pub fn set_children<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        children: SubRange<'id, JsxChild<'id>>,
    ) {
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_fragment
                .children = children.raw();
        }
    }
    #[inline]
    pub fn set_closing_fragment<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        closing_fragment: JsxClosingFragment<'id>,
    ) {
        let field_val: u32 = closing_fragment.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_fragment
                .closing_fragment = field_val;
        }
    }
}
impl<'id> JsxOpeningFragment<'id> {}
impl<'id> JsxClosingFragment<'id> {}
impl<'id> JsxIdentifier<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_identifier.name };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: AstUtf8<'id>) {
        let field_val: u64 = name.raw();
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_identifier
                .name = field_val;
        }
    }
}
impl<'id> JsxNamespacedName<'id> {
    #[inline]
    pub fn namespace<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_namespaced_name
                .namespace
        };
        JsxIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_namespaced_name
                .name
        };
        JsxIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_namespace<'a>(self, ast: &mut crate::Ast<'a, 'id>, namespace: JsxIdentifier<'id>) {
        let field_val: u32 = namespace.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_namespaced_name
                .namespace = field_val;
        }
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: JsxIdentifier<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_namespaced_name
                .name = field_val;
        }
    }
}
impl<'id> JsxMemberExpression<'id> {
    #[inline]
    pub fn object<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxMemberExpressionObject<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_member_expression
                .object
        };
        JsxMemberExpressionObject(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn property<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxIdentifier<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_member_expression
                .property
        };
        JsxIdentifier(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_object<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        object: JsxMemberExpressionObject<'id>,
    ) {
        let field_val: u32 = object.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_member_expression
                .object = field_val;
        }
    }
    #[inline]
    pub fn set_property<'a>(self, ast: &mut crate::Ast<'a, 'id>, property: JsxIdentifier<'id>) {
        let field_val: u32 = property.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_member_expression
                .property = field_val;
        }
    }
}
impl<'id> JsxAttribute<'id> {
    #[inline]
    pub fn name<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxAttributeName<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_attribute.name };
        JsxAttributeName(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> Option<JsxAttributeValue<'id>> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_attribute.value };
        (raw != u32::MAX).then(|| JsxAttributeValue(NodeId::from_raw_index(raw)))
    }
    #[inline]
    pub fn set_name<'a>(self, ast: &mut crate::Ast<'a, 'id>, name: JsxAttributeName<'id>) {
        let field_val: u32 = name.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_attribute
                .name = field_val;
        }
    }
    #[inline]
    pub fn set_value<'a>(
        self,
        ast: &mut crate::Ast<'a, 'id>,
        value: Option<JsxAttributeValue<'id>>,
    ) {
        let field_val: u32 = value.map_or(u32::MAX, |node| node.node_id().index() as u32);
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_attribute
                .value = field_val;
        }
    }
}
impl<'id> JsxSpreadAttribute<'id> {
    #[inline]
    pub fn argument<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_spread_attribute
                .argument
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_argument<'a>(self, ast: &mut crate::Ast<'a, 'id>, argument: Expr<'id>) {
        let field_val: u32 = argument.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_spread_attribute
                .argument = field_val;
        }
    }
}
impl<'id> JsxExpressionContainer<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> JsxExpression<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_expression_container
                .expression
        };
        JsxExpression(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: JsxExpression<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_expression_container
                .expression = field_val;
        }
    }
}
impl<'id> JsxEmptyExpression<'id> {}
impl<'id> JsxText<'id> {
    #[inline]
    pub fn value<'a>(self, ast: &crate::Ast<'a, 'id>) -> AstUtf8<'id> {
        let raw = unsafe { ast.nodes.data_unchecked(self.0.raw()).jsx_text.value };
        AstUtf8::from_raw(raw)
    }
    #[inline]
    pub fn set_value<'a>(self, ast: &mut crate::Ast<'a, 'id>, value: AstUtf8<'id>) {
        let field_val: u64 = value.raw();
        unsafe {
            ast.nodes.data_mut_unchecked(self.0.raw()).jsx_text.value = field_val;
        }
    }
}
impl<'id> JsxSpreadChild<'id> {
    #[inline]
    pub fn expression<'a>(self, ast: &crate::Ast<'a, 'id>) -> Expr<'id> {
        let raw = unsafe {
            ast.nodes
                .data_unchecked(self.0.raw())
                .jsx_spread_child
                .expression
        };
        Expr(NodeId::from_raw_index(raw))
    }
    #[inline]
    pub fn set_expression<'a>(self, ast: &mut crate::Ast<'a, 'id>, expression: Expr<'id>) {
        let field_val: u32 = expression.node_id().index() as u32;
        unsafe {
            ast.nodes
                .data_mut_unchecked(self.0.raw())
                .jsx_spread_child
                .expression = field_val;
        }
    }
}
