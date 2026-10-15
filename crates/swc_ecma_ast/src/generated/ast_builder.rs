#![allow(
    unused,
    clippy::useless_conversion,
    clippy::identity_op,
    clippy::too_many_arguments
)]
use crate::generated::ast_payload::*;
use crate::*;
use oxc_index::Idx;
impl<'a, 'id> Ast<'a, 'id> {
    #[inline]
    pub fn program(
        &mut self,
        span: Span,
        directives: SubRange<'id, Directive<'id>>,
        body: SubRange<'id, Stmt<'id>>,
        hashbang: Option<Hashbang<'id>>,
    ) -> Program<'id> {
        let _f0 = self.add_extra(body);
        Program(self.add_node(
            span,
            NodeKind::Program,
            NodePayload {
                program: ProgramPayload {
                    directives: directives.raw(),
                    hashbang: hashbang.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn stmt_block_statement(
        &mut self,
        span: Span,
        body: SubRange<'id, Stmt<'id>>,
    ) -> Stmt<'id> {
        Stmt::BlockStatement(self.block_statement(span, body).into())
    }
    #[inline]
    pub fn stmt_break_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier<'id>>,
    ) -> Stmt<'id> {
        Stmt::BreakStatement(self.break_statement(span, label).into())
    }
    #[inline]
    pub fn stmt_continue_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier<'id>>,
    ) -> Stmt<'id> {
        Stmt::ContinueStatement(self.continue_statement(span, label).into())
    }
    #[inline]
    pub fn stmt_debugger_statement(&mut self, span: Span) -> Stmt<'id> {
        Stmt::DebuggerStatement(self.debugger_statement(span).into())
    }
    #[inline]
    pub fn stmt_do_while_statement(
        &mut self,
        span: Span,
        body: Stmt<'id>,
        test: Expr<'id>,
    ) -> Stmt<'id> {
        Stmt::DoWhileStatement(self.do_while_statement(span, body, test).into())
    }
    #[inline]
    pub fn stmt_empty_statement(&mut self, span: Span) -> Stmt<'id> {
        Stmt::EmptyStatement(self.empty_statement(span).into())
    }
    #[inline]
    pub fn stmt_expression_statement(&mut self, span: Span, expression: Expr<'id>) -> Stmt<'id> {
        Stmt::ExpressionStatement(self.expression_statement(span, expression).into())
    }
    #[inline]
    pub fn stmt_for_in_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft<'id>,
        right: Expr<'id>,
        body: Stmt<'id>,
    ) -> Stmt<'id> {
        Stmt::ForInStatement(self.for_in_statement(span, left, right, body).into())
    }
    #[inline]
    pub fn stmt_for_of_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft<'id>,
        right: Expr<'id>,
        body: Stmt<'id>,
        r#await: bool,
    ) -> Stmt<'id> {
        Stmt::ForOfStatement(
            self.for_of_statement(span, left, right, body, r#await)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_for_statement(
        &mut self,
        span: Span,
        init: Option<ForStatementInit<'id>>,
        test: Option<Expr<'id>>,
        update: Option<Expr<'id>>,
        body: Stmt<'id>,
    ) -> Stmt<'id> {
        Stmt::ForStatement(self.for_statement(span, init, test, update, body).into())
    }
    #[inline]
    pub fn stmt_if_statement(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Stmt<'id>,
        alternate: Option<Stmt<'id>>,
    ) -> Stmt<'id> {
        Stmt::IfStatement(self.if_statement(span, test, consequent, alternate).into())
    }
    #[inline]
    pub fn stmt_labeled_statement(
        &mut self,
        span: Span,
        label: LabelIdentifier<'id>,
        body: Stmt<'id>,
    ) -> Stmt<'id> {
        Stmt::LabeledStatement(self.labeled_statement(span, label, body).into())
    }
    #[inline]
    pub fn stmt_return_statement(&mut self, span: Span, argument: Option<Expr<'id>>) -> Stmt<'id> {
        Stmt::ReturnStatement(self.return_statement(span, argument).into())
    }
    #[inline]
    pub fn stmt_switch_statement(
        &mut self,
        span: Span,
        discriminant: Expr<'id>,
        cases: SubRange<'id, SwitchCase<'id>>,
    ) -> Stmt<'id> {
        Stmt::SwitchStatement(self.switch_statement(span, discriminant, cases).into())
    }
    #[inline]
    pub fn stmt_throw_statement(&mut self, span: Span, argument: Expr<'id>) -> Stmt<'id> {
        Stmt::ThrowStatement(self.throw_statement(span, argument).into())
    }
    #[inline]
    pub fn stmt_try_statement(
        &mut self,
        span: Span,
        block: BlockStatement<'id>,
        handler: Option<CatchClause<'id>>,
        finalizer: Option<BlockStatement<'id>>,
    ) -> Stmt<'id> {
        Stmt::TryStatement(self.try_statement(span, block, handler, finalizer).into())
    }
    #[inline]
    pub fn stmt_while_statement(
        &mut self,
        span: Span,
        test: Expr<'id>,
        body: Stmt<'id>,
    ) -> Stmt<'id> {
        Stmt::WhileStatement(self.while_statement(span, test, body).into())
    }
    #[inline]
    pub fn stmt_with_statement(
        &mut self,
        span: Span,
        object: Expr<'id>,
        body: Stmt<'id>,
    ) -> Stmt<'id> {
        Stmt::WithStatement(self.with_statement(span, object, body).into())
    }
    #[inline]
    pub fn stmt_decl_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsFunction(
            self.ts_function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        type_annotation: TsType<'id>,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsTypeAliasDeclaration(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        extends: SubRange<'id, TsInterfaceHeritage<'id>>,
        body: TsInterfaceBody<'id>,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        body: TsEnumBody<'id>,
        is_const: bool,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsEnumDeclaration(
            self.ts_enum_declaration(span, id, body, is_const, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName<'id>,
        body: Option<TsModuleBody<'id>>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName<'id>,
        body: TsModuleBlock<'id>,
        declare: bool,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsGlobalDeclaration(
            self.ts_global_declaration(span, id, body, declare).into(),
        ))
    }
    #[inline]
    pub fn stmt_decl_ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        module_reference: TsModuleReference<'id>,
        import_kind: ImportOrExportKind,
    ) -> Stmt<'id> {
        Stmt::Declaration(Decl::TsImportEqualsDeclaration(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind)
                .into(),
        ))
    }
    #[inline]
    pub fn stmt_import_declaration(
        &mut self,
        span: Span,
        specifiers: SubRange<'id, ImportDeclarationSpecifier<'id>>,
        source: StringLiteral<'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        phase: Option<ImportPhase>,
        import_kind: ImportOrExportKind,
    ) -> Stmt<'id> {
        Stmt::ImportDeclaration(
            self.import_declaration(span, specifiers, source, attributes, phase, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Decl<'id>>,
        specifiers: SubRange<'id, ExportSpecifier<'id>>,
        source: Option<StringLiteral<'id>>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        export_kind: ImportOrExportKind,
    ) -> Stmt<'id> {
        Stmt::ExportNamedDeclaration(
            self.export_named_declaration(
                span,
                declaration,
                specifiers,
                source,
                attributes,
                export_kind,
            )
            .into(),
        )
    }
    #[inline]
    pub fn stmt_export_default_declaration(
        &mut self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'id>,
    ) -> Stmt<'id> {
        Stmt::ExportDefaultDeclaration(self.export_default_declaration(span, declaration).into())
    }
    #[inline]
    pub fn stmt_export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName<'id>>,
        source: StringLiteral<'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        export_kind: ImportOrExportKind,
    ) -> Stmt<'id> {
        Stmt::ExportAllDeclaration(
            self.export_all_declaration(span, exported, source, attributes, export_kind)
                .into(),
        )
    }
    #[inline]
    pub fn stmt_ts_export_assignment(&mut self, span: Span, expression: Expr<'id>) -> Stmt<'id> {
        Stmt::TsExportAssignment(self.ts_export_assignment(span, expression).into())
    }
    #[inline]
    pub fn stmt_ts_namespace_export_declaration(
        &mut self,
        span: Span,
        id: IdentifierName<'id>,
    ) -> Stmt<'id> {
        Stmt::TsNamespaceExportDeclaration(self.ts_namespace_export_declaration(span, id).into())
    }
    #[inline]
    pub fn decl_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
        declare: bool,
    ) -> Decl<'id> {
        Decl::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> Decl<'id> {
        Decl::Function(
            self.function(
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
            .into(),
        )
    }
    #[inline]
    pub fn decl_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> Decl<'id> {
        Decl::Class(
            self.class(
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
            .into(),
        )
    }
    #[inline]
    pub fn decl_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsFunction(
            self.ts_function(
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
            .into(),
        )
    }
    #[inline]
    pub fn decl_ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        type_annotation: TsType<'id>,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsTypeAliasDeclaration(
            self.ts_type_alias_declaration(span, id, type_parameters, type_annotation, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        extends: SubRange<'id, TsInterfaceHeritage<'id>>,
        body: TsInterfaceBody<'id>,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        body: TsEnumBody<'id>,
        is_const: bool,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsEnumDeclaration(
            self.ts_enum_declaration(span, id, body, is_const, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName<'id>,
        body: Option<TsModuleBody<'id>>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        )
    }
    #[inline]
    pub fn decl_ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName<'id>,
        body: TsModuleBlock<'id>,
        declare: bool,
    ) -> Decl<'id> {
        Decl::TsGlobalDeclaration(self.ts_global_declaration(span, id, body, declare).into())
    }
    #[inline]
    pub fn decl_ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        module_reference: TsModuleReference<'id>,
        import_kind: ImportOrExportKind,
    ) -> Decl<'id> {
        Decl::TsImportEqualsDeclaration(
            self.ts_import_equals_declaration(span, id, module_reference, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn expr_identifier_reference(&mut self, span: Span, name: AstUtf8<'id>) -> Expr<'id> {
        Expr::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn expr_this_expression(&mut self, span: Span) -> Expr<'id> {
        Expr::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn expr_super(&mut self, span: Span) -> Expr<'id> {
        Expr::Super(self.super_(span).into())
    }
    #[inline]
    pub fn expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> Expr<'id> {
        Expr::MetaProperty(self.meta_property(span, meta, property).into())
    }
    #[inline]
    pub fn expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> Expr<'id> {
        Expr::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> Expr<'id> {
        Expr::NumericLiteral(self.numeric_literal(span, value, raw).into())
    }
    #[inline]
    pub fn expr_big_int_literal(&mut self, span: Span, raw: AstUtf8<'id>) -> Expr<'id> {
        Expr::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn expr_boolean_literal(&mut self, span: Span, value: bool) -> Expr<'id> {
        Expr::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn expr_null_literal(&mut self, span: Span) -> Expr<'id> {
        Expr::NullLiteral(self.null_literal(span).into())
    }
    #[inline]
    pub fn expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> Expr<'id> {
        Expr::RegExpLiteral(self.reg_exp_literal(span, pattern, flags).into())
    }
    #[inline]
    pub fn expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> Expr<'id> {
        Expr::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> Expr<'id> {
        Expr::BinaryExpression(self.binary_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> Expr<'id> {
        Expr::LogicalExpression(self.logical_expression(span, left, right, operator).into())
    }
    #[inline]
    pub fn expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> Expr<'id> {
        Expr::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> Expr<'id> {
        Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        )
    }
    #[inline]
    pub fn expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> Expr<'id> {
        Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        )
    }
    #[inline]
    pub fn expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> Expr<'id> {
        Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        )
    }
    #[inline]
    pub fn expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> Expr<'id> {
        Expr::SequenceExpression(self.sequence_expression(span, expressions).into())
    }
    #[inline]
    pub fn expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> Expr<'id> {
        Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> Expr<'id> {
        Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        )
    }
    #[inline]
    pub fn expr_chain_expression(&mut self, span: Span, expression: Expr<'id>) -> Expr<'id> {
        Expr::ChainExpression(self.chain_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> Expr<'id> {
        Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> Expr<'id> {
        Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        )
    }
    #[inline]
    pub fn expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> Expr<'id> {
        Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        )
    }
    #[inline]
    pub fn expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> Expr<'id> {
        Expr::ArrayExpression(self.array_expression(span, elements).into())
    }
    #[inline]
    pub fn expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> Expr<'id> {
        Expr::ObjectExpression(self.object_expression(span, properties).into())
    }
    #[inline]
    pub fn expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> Expr<'id> {
        Expr::ParenthesizedExpression(self.parenthesized_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> Expr<'id> {
        Expr::Function(
            self.function(
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
            .into(),
        )
    }
    #[inline]
    pub fn expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> Expr<'id> {
        Expr::Class(
            self.class(
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
            .into(),
        )
    }
    #[inline]
    pub fn expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> Expr<'id> {
        Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        )
    }
    #[inline]
    pub fn expr_await_expression(&mut self, span: Span, argument: Expr<'id>) -> Expr<'id> {
        Expr::AwaitExpression(self.await_expression(span, argument).into())
    }
    #[inline]
    pub fn expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> Expr<'id> {
        Expr::YieldExpression(self.yield_expression(span, argument, delegate).into())
    }
    #[inline]
    pub fn expr_private_identifier(&mut self, span: Span, name: AstUtf8<'id>) -> Expr<'id> {
        Expr::PrivateIdentifier(self.private_identifier(span, name).into())
    }
    #[inline]
    pub fn expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> Expr<'id> {
        Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> Expr<'id> {
        Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> Expr<'id> {
        Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn expr_ts_non_null_expression(&mut self, span: Span, expression: Expr<'id>) -> Expr<'id> {
        Expr::TsNonNullExpression(self.ts_non_null_expression(span, expression).into())
    }
    #[inline]
    pub fn expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> Expr<'id> {
        Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> Expr<'id> {
        Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> Expr<'id> {
        Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn binding_pattern_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::SimpleAssignmentTarget(SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn binding_pattern_assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern<'id>,
        right: Expr<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::AssignmentPattern(self.assignment_pattern(span, left, right).into())
    }
    #[inline]
    pub fn binding_pattern_binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern<'id>,
    ) -> BindingPattern<'id> {
        BindingPattern::BindingRestElement(self.binding_rest_element(span, argument).into())
    }
    #[inline]
    pub fn binding_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> BindingPattern<'id> {
        BindingPattern::ArrayPattern(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn binding_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<'id, BindingProperty<'id>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> BindingPattern<'id> {
        BindingPattern::ObjectPattern(self.object_pattern(span, properties, rest).into())
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn assignment_target_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::SimpleAssignmentTarget(SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn assignment_target_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::ArrayAssignmentTarget(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn assignment_target_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<'id, BindingProperty<'id>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> AssignmentTarget<'id> {
        AssignmentTarget::ObjectAssignmentTarget(self.object_pattern(span, properties, rest).into())
    }
    #[inline]
    pub fn simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        )
    }
    #[inline]
    pub fn simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> SimpleAssignmentTarget<'id> {
        SimpleAssignmentTarget::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        )
    }
    #[inline]
    pub fn property_key_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn property_key_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_this_expression(&mut self, span: Span) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn property_key_expr_super(&mut self, span: Span) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn property_key_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn property_key_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_null_literal(&mut self, span: Span) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn property_key_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn property_key_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> PropertyKey<'id> {
        PropertyKey::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn module_export_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ModuleExportName<'id> {
        ModuleExportName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn module_export_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> ModuleExportName<'id> {
        ModuleExportName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn argument_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_this_expression(&mut self, span: Span) -> Argument<'id> {
        Argument::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn argument_expr_super(&mut self, span: Span) -> Argument<'id> {
        Argument::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn argument_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn argument_expr_boolean_literal(&mut self, span: Span, value: bool) -> Argument<'id> {
        Argument::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_null_literal(&mut self, span: Span) -> Argument<'id> {
        Argument::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn argument_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> Argument<'id> {
        Argument::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> Argument<'id> {
        Argument::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> Argument<'id> {
        Argument::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> Argument<'id> {
        Argument::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> Argument<'id> {
        Argument::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn argument_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> Argument<'id> {
        Argument::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn argument_spread_element(&mut self, span: Span, argument: Expr<'id>) -> Argument<'id> {
        Argument::SpreadElement(self.spread_element(span, argument).into())
    }
    #[inline]
    pub fn object_property_kind_object_property(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        value: Expr<'id>,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectPropertyKind<'id> {
        ObjectPropertyKind::ObjectProperty(
            self.object_property(span, key, value, kind, method, shorthand, computed)
                .into(),
        )
    }
    #[inline]
    pub fn object_property_kind_spread_element(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> ObjectPropertyKind<'id> {
        ObjectPropertyKind::SpreadElement(self.spread_element(span, argument).into())
    }
    #[inline]
    pub fn for_statement_init_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
        declare: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_init_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_this_expression(&mut self, span: Span) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_super(&mut self, span: Span) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_null_literal(&mut self, span: Span) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn for_statement_init_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_init_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> ForStatementInit<'id> {
        ForStatementInit::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn for_statement_left_variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
        declare: bool,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::VariableDeclaration(
            self.variable_declaration(span, kind, declarators, declare)
                .into(),
        )
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::IdentifierReference(
                self.identifier_reference(span, name).into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::MemberExpression(
                self.member_expression(span, object, property, computed, optional)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsAsExpression(
                self.ts_as_expression(span, expression, type_annotation)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsSatisfiesExpression(
                self.ts_satisfies_expression(span, expression, type_annotation)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsTypeAssertion(
                self.ts_type_assertion(span, type_annotation, expression)
                    .into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_simple_assignment_target_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::SimpleAssignmentTarget(
            SimpleAssignmentTarget::TsNonNullExpression(
                self.ts_non_null_expression(span, expression).into(),
            ),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::ArrayAssignmentTarget(
            self.array_pattern(span, elements, rest).into(),
        ))
    }
    #[inline]
    pub fn for_statement_left_assignment_target_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<'id, BindingProperty<'id>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> ForStatementLeft<'id> {
        ForStatementLeft::AssignmentTarget(AssignmentTarget::ObjectAssignmentTarget(
            self.object_pattern(span, properties, rest).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_function_body(
        &mut self,
        span: Span,
        directives: SubRange<'id, Directive<'id>>,
        body: SubRange<'id, Stmt<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::FunctionBody(self.function_body(span, directives, body).into())
    }
    #[inline]
    pub fn arrow_function_body_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_this_expression(
        &mut self,
        span: Span,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_super(&mut self, span: Span) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_null_literal(&mut self, span: Span) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn arrow_function_body_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn arrow_function_body_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> ArrowFunctionBody<'id> {
        ArrowFunctionBody::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn formal_parameter_item_formal_parameter(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        pattern: FormalParameterPattern<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        initializer: Option<Expr<'id>>,
        optional: bool,
    ) -> FormalParameterItem<'id> {
        FormalParameterItem::FormalParameter(
            self.formal_parameter(
                span,
                decorators,
                pattern,
                type_annotation,
                initializer,
                optional,
            )
            .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_item_ts_parameter_property(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        parameter: FormalParameter<'id>,
        r#override: bool,
        readonly: bool,
        accessibility: Accessibility,
    ) -> FormalParameterItem<'id> {
        FormalParameterItem::TsParameterProperty(
            self.ts_parameter_property(
                span,
                decorators,
                parameter,
                r#override,
                readonly,
                accessibility,
            )
            .into(),
        )
    }
    #[inline]
    pub fn formal_parameter_pattern_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> FormalParameterPattern<'id> {
        FormalParameterPattern::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn formal_parameter_pattern_array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> FormalParameterPattern<'id> {
        FormalParameterPattern::ArrayPattern(self.array_pattern(span, elements, rest).into())
    }
    #[inline]
    pub fn formal_parameter_pattern_object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<'id, BindingProperty<'id>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> FormalParameterPattern<'id> {
        FormalParameterPattern::ObjectPattern(self.object_pattern(span, properties, rest).into())
    }
    #[inline]
    pub fn class_element_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        value: Function<'id>,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement<'id> {
        ClassElement::MethodDefinition(
            self.method_definition(
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
            .into(),
        )
    }
    #[inline]
    pub fn class_element_property_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        value: Option<Expr<'id>>,
        computed: bool,
        r#static: bool,
        accessor: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement<'id> {
        ClassElement::PropertyDefinition(
            self.property_definition(
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
            .into(),
        )
    }
    #[inline]
    pub fn class_element_static_block(
        &mut self,
        span: Span,
        body: SubRange<'id, Stmt<'id>>,
    ) -> ClassElement<'id> {
        ClassElement::StaticBlock(self.static_block(span, body).into())
    }
    #[inline]
    pub fn class_element_ts_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        value: TsFunction<'id>,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> ClassElement<'id> {
        ClassElement::TsMethodDefinition(
            self.ts_method_definition(
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
            .into(),
        )
    }
    #[inline]
    pub fn class_element_ts_index_signature(
        &mut self,
        span: Span,
        parameters: SubRange<'id, TsIndexSignatureParameter<'id>>,
        type_annotation: TsTypeAnnotation<'id>,
        readonly: bool,
        r#static: bool,
    ) -> ClassElement<'id> {
        ClassElement::TsIndexSignature(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
                .into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_specifier(
        &mut self,
        span: Span,
        imported: ModuleExportName<'id>,
        local: BindingIdentifier<'id>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclarationSpecifier<'id> {
        ImportDeclarationSpecifier::ImportSpecifier(
            self.import_specifier(span, imported, local, import_kind)
                .into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_default_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier<'id>,
    ) -> ImportDeclarationSpecifier<'id> {
        ImportDeclarationSpecifier::ImportDefaultSpecifier(
            self.import_default_specifier(span, local).into(),
        )
    }
    #[inline]
    pub fn import_declaration_specifier_import_namespace_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier<'id>,
    ) -> ImportDeclarationSpecifier<'id> {
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(
            self.import_namespace_specifier(span, local).into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_this_expression(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_super(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::BigIntLiteral(
            self.big_int_literal(span, raw).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_null_literal(
        &mut self,
        span: Span,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn export_default_declaration_kind_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Function(
            self.function(
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
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::Class(
            self.class(
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
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::TsFunction(
            self.ts_function(
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
            .into(),
        )
    }
    #[inline]
    pub fn export_default_declaration_kind_ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        extends: SubRange<'id, TsInterfaceHeritage<'id>>,
        body: TsInterfaceBody<'id>,
        declare: bool,
    ) -> ExportDefaultDeclarationKind<'id> {
        ExportDefaultDeclarationKind::TsInterfaceDeclaration(
            self.ts_interface_declaration(span, id, type_parameters, extends, body, declare)
                .into(),
        )
    }
    #[inline]
    pub fn super_(&mut self, span: Span) -> Super<'id> {
        Super(self.add_node(span, NodeKind::Super, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn this_expression(&mut self, span: Span) -> ThisExpression<'id> {
        ThisExpression(self.add_node(span, NodeKind::ThisExpression, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> StringLiteral<'id> {
        StringLiteral(self.add_node(
            span,
            NodeKind::StringLiteral,
            NodePayload {
                string_literal: StringLiteralPayload {
                    value: value.raw(),
                    raw: raw.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> NumericLiteral<'id> {
        NumericLiteral(self.add_node(
            span,
            NodeKind::NumericLiteral,
            NodePayload {
                numeric_literal: NumericLiteralPayload {
                    value,
                    raw: raw.map_or(u64::from(u32::MAX), AstUtf8::raw),
                },
            },
        ))
    }
    #[inline]
    pub fn big_int_literal(&mut self, span: Span, raw: AstUtf8<'id>) -> BigIntLiteral<'id> {
        BigIntLiteral(self.add_node(
            span,
            NodeKind::BigIntLiteral,
            NodePayload {
                big_int_literal: BigIntLiteralPayload {
                    raw: raw.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn boolean_literal(&mut self, span: Span, value: bool) -> BooleanLiteral<'id> {
        BooleanLiteral(self.add_node(
            span,
            NodeKind::BooleanLiteral,
            NodePayload {
                boolean_literal: BooleanLiteralPayload {
                    value,
                    _pad0: [0; 15usize],
                },
            },
        ))
    }
    #[inline]
    pub fn null_literal(&mut self, span: Span) -> NullLiteral<'id> {
        NullLiteral(self.add_node(span, NodeKind::NullLiteral, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> RegExpLiteral<'id> {
        RegExpLiteral(self.add_node(
            span,
            NodeKind::RegExpLiteral,
            NodePayload {
                reg_exp_literal: RegExpLiteralPayload {
                    pattern: pattern.raw(),
                    flags: flags.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> TemplateLiteral<'id> {
        TemplateLiteral(self.add_node(
            span,
            NodeKind::TemplateLiteral,
            NodePayload {
                template_literal: TemplateLiteralPayload {
                    quasis: quasis.raw(),
                    expressions: expressions.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn template_element(
        &mut self,
        span: Span,
        cooked: AstWtf8<'id>,
        raw: AstUtf8<'id>,
        tail: bool,
        is_cooked_undefined: bool,
    ) -> TemplateElement<'id> {
        let _f0 = self.add_extra(raw);
        TemplateElement(self.add_node(
            span,
            NodeKind::TemplateElement,
            NodePayload {
                template_element: TemplateElementPayload {
                    cooked: cooked.raw(),
                    tail,
                    is_cooked_undefined,
                    _pad0: [0; 2usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> IdentifierReference<'id> {
        IdentifierReference(self.add_node(
            span,
            NodeKind::IdentifierReference,
            NodePayload {
                identifier_reference: IdentifierReferencePayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn private_identifier(&mut self, span: Span, name: AstUtf8<'id>) -> PrivateIdentifier<'id> {
        PrivateIdentifier(self.add_node(
            span,
            NodeKind::PrivateIdentifier,
            NodePayload {
                private_identifier: PrivateIdentifierPayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binding_identifier(&mut self, span: Span, name: AstUtf8<'id>) -> BindingIdentifier<'id> {
        BindingIdentifier(self.add_node(
            span,
            NodeKind::BindingIdentifier,
            NodePayload {
                binding_identifier: BindingIdentifierPayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn identifier_name(&mut self, span: Span, name: AstUtf8<'id>) -> IdentifierName<'id> {
        IdentifierName(self.add_node(
            span,
            NodeKind::IdentifierName,
            NodePayload {
                identifier_name: IdentifierNamePayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn label_identifier(&mut self, span: Span, name: AstUtf8<'id>) -> LabelIdentifier<'id> {
        LabelIdentifier(self.add_node(
            span,
            NodeKind::LabelIdentifier,
            NodePayload {
                label_identifier: LabelIdentifierPayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn assignment_pattern(
        &mut self,
        span: Span,
        left: BindingPattern<'id>,
        right: Expr<'id>,
    ) -> AssignmentPattern<'id> {
        AssignmentPattern(self.add_node(
            span,
            NodeKind::AssignmentPattern,
            NodePayload {
                assignment_pattern: AssignmentPatternPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binding_rest_element(
        &mut self,
        span: Span,
        argument: BindingPattern<'id>,
    ) -> BindingRestElement<'id> {
        BindingRestElement(self.add_node(
            span,
            NodeKind::BindingRestElement,
            NodePayload {
                binding_rest_element: BindingRestElementPayload {
                    argument: argument.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn array_pattern(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<BindingPattern<'id>>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> ArrayPattern<'id> {
        ArrayPattern(self.add_node(
            span,
            NodeKind::ArrayPattern,
            NodePayload {
                array_pattern: ArrayPatternPayload {
                    elements: elements.raw(),
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn object_pattern(
        &mut self,
        span: Span,
        properties: SubRange<'id, BindingProperty<'id>>,
        rest: Option<BindingRestElement<'id>>,
    ) -> ObjectPattern<'id> {
        ObjectPattern(self.add_node(
            span,
            NodeKind::ObjectPattern,
            NodePayload {
                object_pattern: ObjectPatternPayload {
                    properties: properties.raw(),
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binding_property(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        value: BindingPattern<'id>,
        shorthand: bool,
        computed: bool,
    ) -> BindingProperty<'id> {
        BindingProperty(self.add_node(
            span,
            NodeKind::BindingProperty,
            NodePayload {
                binding_property: BindingPropertyPayload {
                    key: key.node_id().index() as u32,
                    value: value.node_id().index() as u32,
                    shorthand,
                    computed,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> BinaryExpression<'id> {
        BinaryExpression(self.add_node(
            span,
            NodeKind::BinaryExpression,
            NodePayload {
                binary_expression: BinaryExpressionPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> LogicalExpression<'id> {
        LogicalExpression(self.add_node(
            span,
            NodeKind::LogicalExpression,
            NodePayload {
                logical_expression: LogicalExpressionPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> ConditionalExpression<'id> {
        ConditionalExpression(self.add_node(
            span,
            NodeKind::ConditionalExpression,
            NodePayload {
                conditional_expression: ConditionalExpressionPayload {
                    test: test.node_id().index() as u32,
                    consequent: consequent.node_id().index() as u32,
                    alternate: alternate.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> UnaryExpression<'id> {
        UnaryExpression(self.add_node(
            span,
            NodeKind::UnaryExpression,
            NodePayload {
                unary_expression: UnaryExpressionPayload {
                    argument: argument.node_id().index() as u32,
                    operator,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> UpdateExpression<'id> {
        UpdateExpression(self.add_node(
            span,
            NodeKind::UpdateExpression,
            NodePayload {
                update_expression: UpdateExpressionPayload {
                    argument: argument.node_id().index() as u32,
                    operator,
                    prefix,
                    _pad0: [0; 10usize],
                },
            },
        ))
    }
    #[inline]
    pub fn assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> AssignmentExpression<'id> {
        AssignmentExpression(self.add_node(
            span,
            NodeKind::AssignmentExpression,
            NodePayload {
                assignment_expression: AssignmentExpressionPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    operator,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn variable_declaration(
        &mut self,
        span: Span,
        kind: VariableKind,
        declarators: SubRange<'id, VariableDeclarator<'id>>,
        declare: bool,
    ) -> VariableDeclaration<'id> {
        VariableDeclaration(self.add_node(
            span,
            NodeKind::VariableDeclaration,
            NodePayload {
                variable_declaration: VariableDeclarationPayload {
                    declarators: declarators.raw(),
                    kind,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn variable_declarator(
        &mut self,
        span: Span,
        id: BindingPattern<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        init: Option<Expr<'id>>,
        definite: bool,
    ) -> VariableDeclarator<'id> {
        VariableDeclarator(self.add_node(
            span,
            NodeKind::VariableDeclarator,
            NodePayload {
                variable_declarator: VariableDeclaratorPayload {
                    id: id.node_id().index() as u32,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    init: init.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    definite,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn if_statement(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Stmt<'id>,
        alternate: Option<Stmt<'id>>,
    ) -> IfStatement<'id> {
        IfStatement(self.add_node(
            span,
            NodeKind::IfStatement,
            NodePayload {
                if_statement: IfStatementPayload {
                    test: test.node_id().index() as u32,
                    consequent: consequent.node_id().index() as u32,
                    alternate: alternate.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn switch_statement(
        &mut self,
        span: Span,
        discriminant: Expr<'id>,
        cases: SubRange<'id, SwitchCase<'id>>,
    ) -> SwitchStatement<'id> {
        SwitchStatement(self.add_node(
            span,
            NodeKind::SwitchStatement,
            NodePayload {
                switch_statement: SwitchStatementPayload {
                    cases: cases.raw(),
                    discriminant: discriminant.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn switch_case(
        &mut self,
        span: Span,
        test: Option<Expr<'id>>,
        consequent: SubRange<'id, Stmt<'id>>,
    ) -> SwitchCase<'id> {
        SwitchCase(self.add_node(
            span,
            NodeKind::SwitchCase,
            NodePayload {
                switch_case: SwitchCasePayload {
                    consequent: consequent.raw(),
                    test: test.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn for_statement(
        &mut self,
        span: Span,
        init: Option<ForStatementInit<'id>>,
        test: Option<Expr<'id>>,
        update: Option<Expr<'id>>,
        body: Stmt<'id>,
    ) -> ForStatement<'id> {
        ForStatement(self.add_node(
            span,
            NodeKind::ForStatement,
            NodePayload {
                for_statement: ForStatementPayload {
                    init: init.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    test: test.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    update: update.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    body: body.node_id().index() as u32,
                },
            },
        ))
    }
    #[inline]
    pub fn for_in_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft<'id>,
        right: Expr<'id>,
        body: Stmt<'id>,
    ) -> ForInStatement<'id> {
        ForInStatement(self.add_node(
            span,
            NodeKind::ForInStatement,
            NodePayload {
                for_in_statement: ForInStatementPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn for_of_statement(
        &mut self,
        span: Span,
        left: ForStatementLeft<'id>,
        right: Expr<'id>,
        body: Stmt<'id>,
        r#await: bool,
    ) -> ForOfStatement<'id> {
        ForOfStatement(self.add_node(
            span,
            NodeKind::ForOfStatement,
            NodePayload {
                for_of_statement: ForOfStatementPayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    r#await,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn try_statement(
        &mut self,
        span: Span,
        block: BlockStatement<'id>,
        handler: Option<CatchClause<'id>>,
        finalizer: Option<BlockStatement<'id>>,
    ) -> TryStatement<'id> {
        TryStatement(self.add_node(
            span,
            NodeKind::TryStatement,
            NodePayload {
                try_statement: TryStatementPayload {
                    block: block.node_id().index() as u32,
                    handler: handler.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    finalizer: finalizer.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn catch_clause(
        &mut self,
        span: Span,
        param: Option<BindingPattern<'id>>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        body: BlockStatement<'id>,
    ) -> CatchClause<'id> {
        CatchClause(self.add_node(
            span,
            NodeKind::CatchClause,
            NodePayload {
                catch_clause: CatchClausePayload {
                    param: param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    body: body.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn break_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier<'id>>,
    ) -> BreakStatement<'id> {
        BreakStatement(self.add_node(
            span,
            NodeKind::BreakStatement,
            NodePayload {
                break_statement: BreakStatementPayload {
                    label: label.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn continue_statement(
        &mut self,
        span: Span,
        label: Option<LabelIdentifier<'id>>,
    ) -> ContinueStatement<'id> {
        ContinueStatement(self.add_node(
            span,
            NodeKind::ContinueStatement,
            NodePayload {
                continue_statement: ContinueStatementPayload {
                    label: label.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn labeled_statement(
        &mut self,
        span: Span,
        label: LabelIdentifier<'id>,
        body: Stmt<'id>,
    ) -> LabeledStatement<'id> {
        LabeledStatement(self.add_node(
            span,
            NodeKind::LabeledStatement,
            NodePayload {
                labeled_statement: LabeledStatementPayload {
                    label: label.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn expression_statement(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ExpressionStatement<'id> {
        ExpressionStatement(self.add_node(
            span,
            NodeKind::ExpressionStatement,
            NodePayload {
                expression_statement: ExpressionStatementPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn return_statement(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
    ) -> ReturnStatement<'id> {
        ReturnStatement(self.add_node(
            span,
            NodeKind::ReturnStatement,
            NodePayload {
                return_statement: ReturnStatementPayload {
                    argument: argument.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn throw_statement(&mut self, span: Span, argument: Expr<'id>) -> ThrowStatement<'id> {
        ThrowStatement(self.add_node(
            span,
            NodeKind::ThrowStatement,
            NodePayload {
                throw_statement: ThrowStatementPayload {
                    argument: argument.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn debugger_statement(&mut self, span: Span) -> DebuggerStatement<'id> {
        DebuggerStatement(self.add_node(span, NodeKind::DebuggerStatement, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn empty_statement(&mut self, span: Span) -> EmptyStatement<'id> {
        EmptyStatement(self.add_node(span, NodeKind::EmptyStatement, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn directive(
        &mut self,
        span: Span,
        expression: StringLiteral<'id>,
        value: AstUtf8<'id>,
    ) -> Directive<'id> {
        Directive(self.add_node(
            span,
            NodeKind::Directive,
            NodePayload {
                directive: DirectivePayload {
                    value: value.raw(),
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn decorator(&mut self, span: Span, expression: Expr<'id>) -> Decorator<'id> {
        Decorator(self.add_node(
            span,
            NodeKind::Decorator,
            NodePayload {
                decorator: DecoratorPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> Class<'id> {
        let _extra_data = [
            (decorators).into_extra_data(),
            (super_class).into_extra_data(),
            (super_type_arguments).into_extra_data(),
            (implements).into_extra_data(),
            (body).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        Class(self.add_node(
            span,
            NodeKind::Class,
            NodePayload {
                class: ClassPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    class_type,
                    r#abstract,
                    declare,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn class_body(
        &mut self,
        span: Span,
        body: SubRange<'id, ClassElement<'id>>,
    ) -> ClassBody<'id> {
        ClassBody(self.add_node(
            span,
            NodeKind::ClassBody,
            NodePayload {
                class_body: ClassBodyPayload {
                    body: body.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        value: Function<'id>,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> MethodDefinition<'id> {
        let _extra_data = [(decorators).into_extra_data(), (value).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        MethodDefinition(self.add_node(
            span,
            NodeKind::MethodDefinition,
            NodePayload {
                method_definition: MethodDefinitionPayload {
                    key: key.node_id().index() as u32,
                    kind,
                    computed,
                    r#static,
                    r#override,
                    optional,
                    r#abstract,
                    accessibility,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn property_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        value: Option<Expr<'id>>,
        computed: bool,
        r#static: bool,
        accessor: bool,
        declare: bool,
        r#override: bool,
        optional: bool,
        definite: bool,
        readonly: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> PropertyDefinition<'id> {
        let _extra_data = [
            (decorators).into_extra_data(),
            (key).into_extra_data(),
            (type_annotation).into_extra_data(),
            (value).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        PropertyDefinition(self.add_node(
            span,
            NodeKind::PropertyDefinition,
            NodePayload {
                property_definition: PropertyDefinitionPayload {
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
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn static_block(&mut self, span: Span, body: SubRange<'id, Stmt<'id>>) -> StaticBlock<'id> {
        StaticBlock(self.add_node(
            span,
            NodeKind::StaticBlock,
            NodePayload {
                static_block: StaticBlockPayload {
                    body: body.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> Function<'id> {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
            (body).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        Function(self.add_node(
            span,
            NodeKind::Function,
            NodePayload {
                function: FunctionPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    function_type,
                    generator,
                    r#async,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn function_body(
        &mut self,
        span: Span,
        directives: SubRange<'id, Directive<'id>>,
        body: SubRange<'id, Stmt<'id>>,
    ) -> FunctionBody<'id> {
        FunctionBody(self.add_node(
            span,
            NodeKind::FunctionBody,
            NodePayload {
                function_body: FunctionBodyPayload {
                    directives: directives.raw(),
                    body: body.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn block_statement(
        &mut self,
        span: Span,
        body: SubRange<'id, Stmt<'id>>,
    ) -> BlockStatement<'id> {
        BlockStatement(self.add_node(
            span,
            NodeKind::BlockStatement,
            NodePayload {
                block_statement: BlockStatementPayload {
                    body: body.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn while_statement(
        &mut self,
        span: Span,
        test: Expr<'id>,
        body: Stmt<'id>,
    ) -> WhileStatement<'id> {
        WhileStatement(self.add_node(
            span,
            NodeKind::WhileStatement,
            NodePayload {
                while_statement: WhileStatementPayload {
                    test: test.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn do_while_statement(
        &mut self,
        span: Span,
        body: Stmt<'id>,
        test: Expr<'id>,
    ) -> DoWhileStatement<'id> {
        DoWhileStatement(self.add_node(
            span,
            NodeKind::DoWhileStatement,
            NodePayload {
                do_while_statement: DoWhileStatementPayload {
                    body: body.node_id().index() as u32,
                    test: test.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn with_statement(
        &mut self,
        span: Span,
        object: Expr<'id>,
        body: Stmt<'id>,
    ) -> WithStatement<'id> {
        WithStatement(self.add_node(
            span,
            NodeKind::WithStatement,
            NodePayload {
                with_statement: WithStatementPayload {
                    object: object.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameters(
        &mut self,
        span: Span,
        items: SubRange<'id, FormalParameterItem<'id>>,
        rest: Option<FormalParameterRest<'id>>,
        kind: FormalParameterKind,
    ) -> FormalParameters<'id> {
        FormalParameters(self.add_node(
            span,
            NodeKind::FormalParameters,
            NodePayload {
                formal_parameters: FormalParametersPayload {
                    items: items.raw(),
                    rest: rest.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameter(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        pattern: FormalParameterPattern<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        initializer: Option<Expr<'id>>,
        optional: bool,
    ) -> FormalParameter<'id> {
        let _extra_data = [
            (decorators).into_extra_data(),
            (initializer).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        FormalParameter(self.add_node(
            span,
            NodeKind::FormalParameter,
            NodePayload {
                formal_parameter: FormalParameterPayload {
                    pattern: pattern.node_id().index() as u32,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    optional,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn formal_parameter_rest(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        argument: FormalParameterPattern<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) -> FormalParameterRest<'id> {
        FormalParameterRest(self.add_node(
            span,
            NodeKind::FormalParameterRest,
            NodePayload {
                formal_parameter_rest: FormalParameterRestPayload {
                    decorators: decorators.raw(),
                    argument: argument.node_id().index() as u32,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> ArrayExpression<'id> {
        ArrayExpression(self.add_node(
            span,
            NodeKind::ArrayExpression,
            NodePayload {
                array_expression: ArrayExpressionPayload {
                    elements: elements.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> ObjectExpression<'id> {
        ObjectExpression(self.add_node(
            span,
            NodeKind::ObjectExpression,
            NodePayload {
                object_expression: ObjectExpressionPayload {
                    properties: properties.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn spread_element(&mut self, span: Span, argument: Expr<'id>) -> SpreadElement<'id> {
        SpreadElement(self.add_node(
            span,
            NodeKind::SpreadElement,
            NodePayload {
                spread_element: SpreadElementPayload {
                    argument: argument.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn object_property(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        value: Expr<'id>,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
    ) -> ObjectProperty<'id> {
        ObjectProperty(self.add_node(
            span,
            NodeKind::ObjectProperty,
            NodePayload {
                object_property: ObjectPropertyPayload {
                    key: key.node_id().index() as u32,
                    value: value.node_id().index() as u32,
                    kind,
                    method,
                    shorthand,
                    computed,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> ParenthesizedExpression<'id> {
        ParenthesizedExpression(self.add_node(
            span,
            NodeKind::ParenthesizedExpression,
            NodePayload {
                parenthesized_expression: ParenthesizedExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> ArrowFunctionExpression<'id> {
        let _extra_data = [(return_type).into_extra_data(), (body).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ArrowFunctionExpression(self.add_node(
            span,
            NodeKind::ArrowFunctionExpression,
            NodePayload {
                arrow_function_expression: ArrowFunctionExpressionPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params: params.node_id().index() as u32,
                    expression,
                    r#async,
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> SequenceExpression<'id> {
        SequenceExpression(self.add_node(
            span,
            NodeKind::SequenceExpression,
            NodePayload {
                sequence_expression: SequenceExpressionPayload {
                    expressions: expressions.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> MemberExpression<'id> {
        MemberExpression(self.add_node(
            span,
            NodeKind::MemberExpression,
            NodePayload {
                member_expression: MemberExpressionPayload {
                    object: object.node_id().index() as u32,
                    property: property.node_id().index() as u32,
                    computed,
                    optional,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> CallExpression<'id> {
        let _f0 = self.add_extra(arguments);
        CallExpression(self.add_node(
            span,
            NodeKind::CallExpression,
            NodePayload {
                call_expression: CallExpressionPayload {
                    callee: callee.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    optional,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn chain_expression(&mut self, span: Span, expression: Expr<'id>) -> ChainExpression<'id> {
        ChainExpression(self.add_node(
            span,
            NodeKind::ChainExpression,
            NodePayload {
                chain_expression: ChainExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> TaggedTemplateExpression<'id> {
        TaggedTemplateExpression(self.add_node(
            span,
            NodeKind::TaggedTemplateExpression,
            NodePayload {
                tagged_template_expression: TaggedTemplateExpressionPayload {
                    tag: tag.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    quasi: quasi.node_id().index() as u32,
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> NewExpression<'id> {
        NewExpression(self.add_node(
            span,
            NodeKind::NewExpression,
            NodePayload {
                new_expression: NewExpressionPayload {
                    arguments: arguments.raw(),
                    callee: callee.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn await_expression(&mut self, span: Span, argument: Expr<'id>) -> AwaitExpression<'id> {
        AwaitExpression(self.add_node(
            span,
            NodeKind::AwaitExpression,
            NodePayload {
                await_expression: AwaitExpressionPayload {
                    argument: argument.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> YieldExpression<'id> {
        YieldExpression(self.add_node(
            span,
            NodeKind::YieldExpression,
            NodePayload {
                yield_expression: YieldExpressionPayload {
                    argument: argument.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    delegate,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> MetaProperty<'id> {
        MetaProperty(self.add_node(
            span,
            NodeKind::MetaProperty,
            NodePayload {
                meta_property: MetaPropertyPayload {
                    meta: meta.node_id().index() as u32,
                    property: property.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> ImportExpression<'id> {
        let _f0 = self.add_extra(phase);
        ImportExpression(self.add_node(
            span,
            NodeKind::ImportExpression,
            NodePayload {
                import_expression: ImportExpressionPayload {
                    source: source.node_id().index() as u32,
                    options: options.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    sync,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn import_declaration(
        &mut self,
        span: Span,
        specifiers: SubRange<'id, ImportDeclarationSpecifier<'id>>,
        source: StringLiteral<'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        phase: Option<ImportPhase>,
        import_kind: ImportOrExportKind,
    ) -> ImportDeclaration<'id> {
        let _extra_data = [
            (specifiers).into_extra_data(),
            (attributes).into_extra_data(),
            (phase).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ImportDeclaration(self.add_node(
            span,
            NodeKind::ImportDeclaration,
            NodePayload {
                import_declaration: ImportDeclarationPayload {
                    source: source.node_id().index() as u32,
                    import_kind,
                    _pad0: [0; 7usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn import_specifier(
        &mut self,
        span: Span,
        imported: ModuleExportName<'id>,
        local: BindingIdentifier<'id>,
        import_kind: ImportOrExportKind,
    ) -> ImportSpecifier<'id> {
        ImportSpecifier(self.add_node(
            span,
            NodeKind::ImportSpecifier,
            NodePayload {
                import_specifier: ImportSpecifierPayload {
                    imported: imported.node_id().index() as u32,
                    local: local.node_id().index() as u32,
                    import_kind,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn import_default_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier<'id>,
    ) -> ImportDefaultSpecifier<'id> {
        ImportDefaultSpecifier(self.add_node(
            span,
            NodeKind::ImportDefaultSpecifier,
            NodePayload {
                import_default_specifier: ImportDefaultSpecifierPayload {
                    local: local.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn import_namespace_specifier(
        &mut self,
        span: Span,
        local: BindingIdentifier<'id>,
    ) -> ImportNamespaceSpecifier<'id> {
        ImportNamespaceSpecifier(self.add_node(
            span,
            NodeKind::ImportNamespaceSpecifier,
            NodePayload {
                import_namespace_specifier: ImportNamespaceSpecifierPayload {
                    local: local.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn import_attribute(
        &mut self,
        span: Span,
        key: ModuleExportName<'id>,
        value: StringLiteral<'id>,
    ) -> ImportAttribute<'id> {
        ImportAttribute(self.add_node(
            span,
            NodeKind::ImportAttribute,
            NodePayload {
                import_attribute: ImportAttributePayload {
                    key: key.node_id().index() as u32,
                    value: value.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn export_named_declaration(
        &mut self,
        span: Span,
        declaration: Option<Decl<'id>>,
        specifiers: SubRange<'id, ExportSpecifier<'id>>,
        source: Option<StringLiteral<'id>>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        export_kind: ImportOrExportKind,
    ) -> ExportNamedDeclaration<'id> {
        let _extra_data = [
            (specifiers).into_extra_data(),
            (attributes).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        ExportNamedDeclaration(self.add_node(
            span,
            NodeKind::ExportNamedDeclaration,
            NodePayload {
                export_named_declaration: ExportNamedDeclarationPayload {
                    declaration: declaration.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    source: source.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    export_kind,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn export_default_declaration(
        &mut self,
        span: Span,
        declaration: ExportDefaultDeclarationKind<'id>,
    ) -> ExportDefaultDeclaration<'id> {
        ExportDefaultDeclaration(self.add_node(
            span,
            NodeKind::ExportDefaultDeclaration,
            NodePayload {
                export_default_declaration: ExportDefaultDeclarationPayload {
                    declaration: declaration.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn export_all_declaration(
        &mut self,
        span: Span,
        exported: Option<ModuleExportName<'id>>,
        source: StringLiteral<'id>,
        attributes: SubRange<'id, ImportAttribute<'id>>,
        export_kind: ImportOrExportKind,
    ) -> ExportAllDeclaration<'id> {
        let _f0 = self.add_extra(attributes);
        ExportAllDeclaration(self.add_node(
            span,
            NodeKind::ExportAllDeclaration,
            NodePayload {
                export_all_declaration: ExportAllDeclarationPayload {
                    exported: exported.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    source: source.node_id().index() as u32,
                    export_kind,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn export_specifier(
        &mut self,
        span: Span,
        local: ModuleExportName<'id>,
        exported: ModuleExportName<'id>,
        export_kind: ImportOrExportKind,
    ) -> ExportSpecifier<'id> {
        ExportSpecifier(self.add_node(
            span,
            NodeKind::ExportSpecifier,
            NodePayload {
                export_specifier: ExportSpecifierPayload {
                    local: local.node_id().index() as u32,
                    exported: exported.node_id().index() as u32,
                    export_kind,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn hashbang(&mut self, span: Span, value: AstUtf8<'id>) -> Hashbang<'id> {
        Hashbang(self.add_node(
            span,
            NodeKind::Hashbang,
            NodePayload {
                hashbang: HashbangPayload {
                    value: value.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_ts_any_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsAnyKeyword(self.ts_any_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_unknown_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsUnknownKeyword(self.ts_unknown_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_never_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsNeverKeyword(self.ts_never_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_void_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsVoidKeyword(self.ts_void_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_null_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsNullKeyword(self.ts_null_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_undefined_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsUndefinedKeyword(self.ts_undefined_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_string_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsStringKeyword(self.ts_string_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_number_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsNumberKeyword(self.ts_number_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_big_int_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsBigIntKeyword(self.ts_big_int_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_boolean_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsBooleanKeyword(self.ts_boolean_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_symbol_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsSymbolKeyword(self.ts_symbol_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_object_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsObjectKeyword(self.ts_object_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_intrinsic_keyword(&mut self, span: Span) -> TsType<'id> {
        TsType::TsIntrinsicKeyword(self.ts_intrinsic_keyword(span).into())
    }
    #[inline]
    pub fn ts_type_ts_this_type(&mut self, span: Span) -> TsType<'id> {
        TsType::TsThisType(self.ts_this_type(span).into())
    }
    #[inline]
    pub fn ts_type_ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsType<'id> {
        TsType::TsTypeReference(
            self.ts_type_reference(span, type_name, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsType<'id> {
        TsType::TsTypeQuery(self.ts_type_query(span, expr_name, type_arguments).into())
    }
    #[inline]
    pub fn ts_type_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral<'id>,
        options: Option<Expr<'id>>,
        qualifier: Option<TsImportTypeQualifier<'id>>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsType<'id> {
        TsType::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_literal_type(&mut self, span: Span, literal: TsLiteral<'id>) -> TsType<'id> {
        TsType::TsLiteralType(self.ts_literal_type(span, literal).into())
    }
    #[inline]
    pub fn ts_type_ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsType<'id> {
        TsType::TsTemplateLiteralType(self.ts_template_literal_type(span, quasis, types).into())
    }
    #[inline]
    pub fn ts_type_ts_array_type(&mut self, span: Span, element_type: TsType<'id>) -> TsType<'id> {
        TsType::TsArrayType(self.ts_array_type(span, element_type).into())
    }
    #[inline]
    pub fn ts_type_ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType<'id>,
        index_type: TsType<'id>,
    ) -> TsType<'id> {
        TsType::TsIndexedAccessType(
            self.ts_indexed_access_type(span, object_type, index_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_tuple_type(
        &mut self,
        span: Span,
        element_types: SubRange<'id, TsTupleElement<'id>>,
    ) -> TsType<'id> {
        TsType::TsTupleType(self.ts_tuple_type(span, element_types).into())
    }
    #[inline]
    pub fn ts_type_ts_optional_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsType<'id> {
        TsType::TsOptionalType(self.ts_optional_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_rest_type(
        &mut self,
        span: Span,
        type_annotation: TsTupleElement<'id>,
    ) -> TsType<'id> {
        TsType::TsRestType(self.ts_rest_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsType<'id> {
        TsType::TsJsDocNullableType(
            self.ts_js_doc_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsType<'id> {
        TsType::TsJsDocNonNullableType(
            self.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_js_doc_unknown_type(&mut self, span: Span) -> TsType<'id> {
        TsType::TsJsDocUnknownType(self.ts_js_doc_unknown_type(span).into())
    }
    #[inline]
    pub fn ts_type_ts_union_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsType<'id> {
        TsType::TsUnionType(self.ts_union_type(span, types).into())
    }
    #[inline]
    pub fn ts_type_ts_intersection_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsType<'id> {
        TsType::TsIntersectionType(self.ts_intersection_type(span, types).into())
    }
    #[inline]
    pub fn ts_type_ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType<'id>,
        extends_type: TsType<'id>,
        true_type: TsType<'id>,
        false_type: TsType<'id>,
    ) -> TsType<'id> {
        TsType::TsConditionalType(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_infer_type(
        &mut self,
        span: Span,
        type_parameter: TsTypeParameter<'id>,
    ) -> TsType<'id> {
        TsType::TsInferType(self.ts_infer_type(span, type_parameter).into())
    }
    #[inline]
    pub fn ts_type_ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType<'id>,
    ) -> TsType<'id> {
        TsType::TsTypeOperator(
            self.ts_type_operator(span, operator, type_annotation)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsType<'id> {
        TsType::TsParenthesizedType(self.ts_parenthesized_type(span, type_annotation).into())
    }
    #[inline]
    pub fn ts_type_ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
    ) -> TsType<'id> {
        TsType::TsFunctionType(
            self.ts_function_type(span, type_parameters, this_param, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
        r#abstract: bool,
    ) -> TsType<'id> {
        TsType::TsConstructorType(
            self.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        asserts: bool,
    ) -> TsType<'id> {
        TsType::TsTypePredicate(
            self.ts_type_predicate(span, parameter_name, type_annotation, asserts)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_ts_type_literal(
        &mut self,
        span: Span,
        members: SubRange<'id, TsTypeElement<'id>>,
    ) -> TsType<'id> {
        TsType::TsTypeLiteral(self.ts_type_literal(span, members).into())
    }
    #[inline]
    pub fn ts_type_ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier<'id>,
        constraint: TsType<'id>,
        name_type: Option<TsType<'id>>,
        type_annotation: Option<TsType<'id>>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsType<'id> {
        TsType::TsMappedType(
            self.ts_mapped_type(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_type_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsTypeName<'id> {
        TsTypeName::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsTypeName<'id> {
        TsTypeName::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsQualifiedNameLeft<'id> {
        TsQualifiedNameLeft::IdentifierReference(self.identifier_reference(span, name).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsQualifiedNameLeft<'id> {
        TsQualifiedNameLeft::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsQualifiedNameLeft<'id> {
        TsQualifiedNameLeft::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_this_expression(
        &mut self,
        span: Span,
    ) -> TsQualifiedNameLeft<'id> {
        TsQualifiedNameLeft::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn ts_qualified_name_left_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsQualifiedNameLeft<'id> {
        TsQualifiedNameLeft::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_import_type_qualifier_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsImportTypeQualifier<'id> {
        TsImportTypeQualifier::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_import_type_qualifier_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsImportTypeQualifier<'id> {
        TsImportTypeQualifier::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_type_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsTypeQueryExprName<'id> {
        TsTypeQueryExprName::TsTypeName(TsTypeName::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsTypeQueryExprName<'id> {
        TsTypeQueryExprName::TsTypeName(TsTypeName::TsQualifiedName(
            self.ts_qualified_name(span, left, right).into(),
        ))
    }
    #[inline]
    pub fn ts_type_query_expr_name_this_expression(
        &mut self,
        span: Span,
    ) -> TsTypeQueryExprName<'id> {
        TsTypeQueryExprName::ThisExpression(self.this_expression(span).into())
    }
    #[inline]
    pub fn ts_type_query_expr_name_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral<'id>,
        options: Option<Expr<'id>>,
        qualifier: Option<TsImportTypeQualifier<'id>>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTypeQueryExprName<'id> {
        TsTypeQueryExprName::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        )
    }
    #[inline]
    pub fn ts_literal_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> TsLiteral<'id> {
        TsLiteral::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_literal_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> TsLiteral<'id> {
        TsLiteral::NumericLiteral(self.numeric_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_literal_big_int_literal(&mut self, span: Span, raw: AstUtf8<'id>) -> TsLiteral<'id> {
        TsLiteral::BigIntLiteral(self.big_int_literal(span, raw).into())
    }
    #[inline]
    pub fn ts_literal_boolean_literal(&mut self, span: Span, value: bool) -> TsLiteral<'id> {
        TsLiteral::BooleanLiteral(self.boolean_literal(span, value).into())
    }
    #[inline]
    pub fn ts_literal_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> TsLiteral<'id> {
        TsLiteral::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn ts_literal_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> TsLiteral<'id> {
        TsLiteral::UnaryExpression(self.unary_expression(span, argument, operator).into())
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_any_keyword(&mut self, span: Span) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsAnyKeyword(self.ts_any_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_unknown_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsUnknownKeyword(
            self.ts_unknown_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_never_keyword(&mut self, span: Span) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsNeverKeyword(self.ts_never_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_void_keyword(&mut self, span: Span) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsVoidKeyword(self.ts_void_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_null_keyword(&mut self, span: Span) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsNullKeyword(self.ts_null_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_undefined_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsUndefinedKeyword(
            self.ts_undefined_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_string_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsStringKeyword(self.ts_string_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_number_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsNumberKeyword(self.ts_number_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_big_int_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsBigIntKeyword(
            self.ts_big_int_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_boolean_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsBooleanKeyword(
            self.ts_boolean_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_symbol_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsSymbolKeyword(self.ts_symbol_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_object_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsObjectKeyword(self.ts_object_keyword(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_intrinsic_keyword(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsIntrinsicKeyword(
            self.ts_intrinsic_keyword(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_this_type(&mut self, span: Span) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsThisType(self.ts_this_type(span).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTypeReference(
            self.ts_type_reference(span, type_name, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTypeQuery(
            self.ts_type_query(span, expr_name, type_arguments).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral<'id>,
        options: Option<Expr<'id>>,
        qualifier: Option<TsImportTypeQualifier<'id>>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsImportType(
            self.ts_import_type(span, source, options, qualifier, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_literal_type(
        &mut self,
        span: Span,
        literal: TsLiteral<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsLiteralType(
            self.ts_literal_type(span, literal).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTemplateLiteralType(
            self.ts_template_literal_type(span, quasis, types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_array_type(
        &mut self,
        span: Span,
        element_type: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsArrayType(
            self.ts_array_type(span, element_type).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType<'id>,
        index_type: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsIndexedAccessType(
            self.ts_indexed_access_type(span, object_type, index_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_tuple_type(
        &mut self,
        span: Span,
        element_types: SubRange<'id, TsTupleElement<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTupleType(
            self.ts_tuple_type(span, element_types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_optional_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsOptionalType(
            self.ts_optional_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_rest_type(
        &mut self,
        span: Span,
        type_annotation: TsTupleElement<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsRestType(
            self.ts_rest_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsJsDocNullableType(
            self.ts_js_doc_nullable_type(span, type_annotation, postfix)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsJsDocNonNullableType(
            self.ts_js_doc_non_nullable_type(span, type_annotation, postfix)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_js_doc_unknown_type(
        &mut self,
        span: Span,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsJsDocUnknownType(
            self.ts_js_doc_unknown_type(span).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_union_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsUnionType(self.ts_union_type(span, types).into()))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_intersection_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsIntersectionType(
            self.ts_intersection_type(span, types).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType<'id>,
        extends_type: TsType<'id>,
        true_type: TsType<'id>,
        false_type: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsConditionalType(
            self.ts_conditional_type(span, check_type, extends_type, true_type, false_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_infer_type(
        &mut self,
        span: Span,
        type_parameter: TsTypeParameter<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsInferType(
            self.ts_infer_type(span, type_parameter).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTypeOperator(
            self.ts_type_operator(span, operator, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsParenthesizedType(
            self.ts_parenthesized_type(span, type_annotation).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsFunctionType(
            self.ts_function_type(span, type_parameters, this_param, params, return_type)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
        r#abstract: bool,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsConstructorType(
            self.ts_constructor_type(span, type_parameters, params, return_type, r#abstract)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        asserts: bool,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTypePredicate(
            self.ts_type_predicate(span, parameter_name, type_annotation, asserts)
                .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_type_literal(
        &mut self,
        span: Span,
        members: SubRange<'id, TsTypeElement<'id>>,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsTypeLiteral(
            self.ts_type_literal(span, members).into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_type_ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier<'id>,
        constraint: TsType<'id>,
        name_type: Option<TsType<'id>>,
        type_annotation: Option<TsType<'id>>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsType(TsType::TsMappedType(
            self.ts_mapped_type(
                span,
                key,
                constraint,
                name_type,
                type_annotation,
                optional,
                readonly,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn ts_tuple_element_ts_named_tuple_member(
        &mut self,
        span: Span,
        label: IdentifierName<'id>,
        element_type: TsType<'id>,
        optional: bool,
    ) -> TsTupleElement<'id> {
        TsTupleElement::TsNamedTupleMember(
            self.ts_named_tuple_member(span, label, element_type, optional)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_property_signature(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        computed: bool,
        optional: bool,
        readonly: bool,
    ) -> TsTypeElement<'id> {
        TsTypeElement::TsPropertySignature(
            self.ts_property_signature(span, key, type_annotation, computed, optional, readonly)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_method_signature(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> TsTypeElement<'id> {
        TsTypeElement::TsMethodSignature(
            self.ts_method_signature(
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
            .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_call_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) -> TsTypeElement<'id> {
        TsTypeElement::TsCallSignatureDeclaration(
            self.ts_call_signature_declaration(
                span,
                type_parameters,
                this_param,
                params,
                return_type,
            )
            .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_construct_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) -> TsTypeElement<'id> {
        TsTypeElement::TsConstructSignatureDeclaration(
            self.ts_construct_signature_declaration(span, type_parameters, params, return_type)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_element_ts_index_signature(
        &mut self,
        span: Span,
        parameters: SubRange<'id, TsIndexSignatureParameter<'id>>,
        type_annotation: TsTypeAnnotation<'id>,
        readonly: bool,
        r#static: bool,
    ) -> TsTypeElement<'id> {
        TsTypeElement::TsIndexSignature(
            self.ts_index_signature(span, parameters, type_annotation, readonly, r#static)
                .into(),
        )
    }
    #[inline]
    pub fn ts_type_predicate_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsTypePredicateName<'id> {
        TsTypePredicateName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_type_predicate_name_ts_this_type(&mut self, span: Span) -> TsTypePredicateName<'id> {
        TsTypePredicateName::TsThisType(self.ts_this_type(span).into())
    }
    #[inline]
    pub fn ts_enum_member_name_identifier_name(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsEnumMemberName<'id> {
        TsEnumMemberName::IdentifierName(self.identifier_name(span, name).into())
    }
    #[inline]
    pub fn ts_enum_member_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> TsEnumMemberName<'id> {
        TsEnumMemberName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_enum_member_name_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> TsEnumMemberName<'id> {
        TsEnumMemberName::TemplateLiteral(self.template_literal(span, quasis, expressions).into())
    }
    #[inline]
    pub fn ts_module_name_binding_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsModuleName<'id> {
        TsModuleName::BindingIdentifier(self.binding_identifier(span, name).into())
    }
    #[inline]
    pub fn ts_module_name_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> TsModuleName<'id> {
        TsModuleName::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn ts_module_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsModuleName<'id> {
        TsModuleName::TsQualifiedName(self.ts_qualified_name(span, left, right).into())
    }
    #[inline]
    pub fn ts_module_body_ts_module_block(
        &mut self,
        span: Span,
        directives: SubRange<'id, Directive<'id>>,
        body: SubRange<'id, Stmt<'id>>,
    ) -> TsModuleBody<'id> {
        TsModuleBody::TsModuleBlock(self.ts_module_block(span, directives, body).into())
    }
    #[inline]
    pub fn ts_module_body_ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName<'id>,
        body: Option<TsModuleBody<'id>>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> TsModuleBody<'id> {
        TsModuleBody::TsModuleDeclaration(
            self.ts_module_declaration(span, id, body, kind, declare)
                .into(),
        )
    }
    #[inline]
    pub fn ts_module_reference_ts_type_name_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> TsModuleReference<'id> {
        TsModuleReference::TsTypeName(TsTypeName::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn ts_module_reference_ts_type_name_ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsModuleReference<'id> {
        TsModuleReference::TsTypeName(TsTypeName::TsQualifiedName(
            self.ts_qualified_name(span, left, right).into(),
        ))
    }
    #[inline]
    pub fn ts_module_reference_ts_external_module_reference(
        &mut self,
        span: Span,
        expression: StringLiteral<'id>,
    ) -> TsModuleReference<'id> {
        TsModuleReference::TsExternalModuleReference(
            self.ts_external_module_reference(span, expression).into(),
        )
    }
    #[inline]
    pub fn ts_function(
        &mut self,
        span: Span,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        generator: bool,
        r#async: bool,
        declare: bool,
    ) -> TsFunction<'id> {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsFunction(self.add_node(
            span,
            NodeKind::TsFunction,
            NodePayload {
                ts_function: TsFunctionPayload {
                    id: id.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    generator,
                    r#async,
                    declare,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_method_definition(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        key: PropertyKey<'id>,
        value: TsFunction<'id>,
        kind: MethodDefinitionKind,
        computed: bool,
        r#static: bool,
        r#override: bool,
        optional: bool,
        r#abstract: bool,
        accessibility: Accessibility,
    ) -> TsMethodDefinition<'id> {
        let _extra_data = [(decorators).into_extra_data(), (value).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMethodDefinition(self.add_node(
            span,
            NodeKind::TsMethodDefinition,
            NodePayload {
                ts_method_definition: TsMethodDefinitionPayload {
                    key: key.node_id().index() as u32,
                    kind,
                    computed,
                    r#static,
                    r#override,
                    optional,
                    r#abstract,
                    accessibility,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_annotation(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsTypeAnnotation<'id> {
        TsTypeAnnotation(self.add_node(
            span,
            NodeKind::TsTypeAnnotation,
            NodePayload {
                ts_type_annotation: TsTypeAnnotationPayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_any_keyword(&mut self, span: Span) -> TsAnyKeyword<'id> {
        TsAnyKeyword(self.add_node(span, NodeKind::TsAnyKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_unknown_keyword(&mut self, span: Span) -> TsUnknownKeyword<'id> {
        TsUnknownKeyword(self.add_node(span, NodeKind::TsUnknownKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_never_keyword(&mut self, span: Span) -> TsNeverKeyword<'id> {
        TsNeverKeyword(self.add_node(span, NodeKind::TsNeverKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_void_keyword(&mut self, span: Span) -> TsVoidKeyword<'id> {
        TsVoidKeyword(self.add_node(span, NodeKind::TsVoidKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_null_keyword(&mut self, span: Span) -> TsNullKeyword<'id> {
        TsNullKeyword(self.add_node(span, NodeKind::TsNullKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_undefined_keyword(&mut self, span: Span) -> TsUndefinedKeyword<'id> {
        TsUndefinedKeyword(self.add_node(
            span,
            NodeKind::TsUndefinedKeyword,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_string_keyword(&mut self, span: Span) -> TsStringKeyword<'id> {
        TsStringKeyword(self.add_node(span, NodeKind::TsStringKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_number_keyword(&mut self, span: Span) -> TsNumberKeyword<'id> {
        TsNumberKeyword(self.add_node(span, NodeKind::TsNumberKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_big_int_keyword(&mut self, span: Span) -> TsBigIntKeyword<'id> {
        TsBigIntKeyword(self.add_node(span, NodeKind::TsBigIntKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_boolean_keyword(&mut self, span: Span) -> TsBooleanKeyword<'id> {
        TsBooleanKeyword(self.add_node(span, NodeKind::TsBooleanKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_symbol_keyword(&mut self, span: Span) -> TsSymbolKeyword<'id> {
        TsSymbolKeyword(self.add_node(span, NodeKind::TsSymbolKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_object_keyword(&mut self, span: Span) -> TsObjectKeyword<'id> {
        TsObjectKeyword(self.add_node(span, NodeKind::TsObjectKeyword, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_intrinsic_keyword(&mut self, span: Span) -> TsIntrinsicKeyword<'id> {
        TsIntrinsicKeyword(self.add_node(
            span,
            NodeKind::TsIntrinsicKeyword,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_this_type(&mut self, span: Span) -> TsThisType<'id> {
        TsThisType(self.add_node(span, NodeKind::TsThisType, NodePayload { raw: 0 }))
    }
    #[inline]
    pub fn ts_type_reference(
        &mut self,
        span: Span,
        type_name: TsTypeName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTypeReference<'id> {
        TsTypeReference(self.add_node(
            span,
            NodeKind::TsTypeReference,
            NodePayload {
                ts_type_reference: TsTypeReferencePayload {
                    type_name: type_name.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_qualified_name(
        &mut self,
        span: Span,
        left: TsQualifiedNameLeft<'id>,
        right: IdentifierName<'id>,
    ) -> TsQualifiedName<'id> {
        TsQualifiedName(self.add_node(
            span,
            NodeKind::TsQualifiedName,
            NodePayload {
                ts_qualified_name: TsQualifiedNamePayload {
                    left: left.node_id().index() as u32,
                    right: right.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_query(
        &mut self,
        span: Span,
        expr_name: TsTypeQueryExprName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsTypeQuery<'id> {
        TsTypeQuery(self.add_node(
            span,
            NodeKind::TsTypeQuery,
            NodePayload {
                ts_type_query: TsTypeQueryPayload {
                    expr_name: expr_name.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_import_type(
        &mut self,
        span: Span,
        source: StringLiteral<'id>,
        options: Option<Expr<'id>>,
        qualifier: Option<TsImportTypeQualifier<'id>>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsImportType<'id> {
        TsImportType(self.add_node(
            span,
            NodeKind::TsImportType,
            NodePayload {
                ts_import_type: TsImportTypePayload {
                    source: source.node_id().index() as u32,
                    options: options.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    qualifier: qualifier.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_parameter(
        &mut self,
        span: Span,
        name: BindingIdentifier<'id>,
        constraint: Option<TsType<'id>>,
        default: Option<TsType<'id>>,
        r#in: bool,
        out: bool,
        r#const: bool,
    ) -> TsTypeParameter<'id> {
        TsTypeParameter(self.add_node(
            span,
            NodeKind::TsTypeParameter,
            NodePayload {
                ts_type_parameter: TsTypeParameterPayload {
                    name: name.node_id().index() as u32,
                    constraint: constraint.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    default: default.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    r#in,
                    out,
                    r#const,
                    _pad0: [0; 1usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_parameter_declaration(
        &mut self,
        span: Span,
        params: SubRange<'id, TsTypeParameter<'id>>,
    ) -> TsTypeParameterDeclaration<'id> {
        TsTypeParameterDeclaration(self.add_node(
            span,
            NodeKind::TsTypeParameterDeclaration,
            NodePayload {
                ts_type_parameter_declaration: TsTypeParameterDeclarationPayload {
                    params: params.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_parameter_instantiation(
        &mut self,
        span: Span,
        params: SubRange<'id, TsType<'id>>,
    ) -> TsTypeParameterInstantiation<'id> {
        TsTypeParameterInstantiation(self.add_node(
            span,
            NodeKind::TsTypeParameterInstantiation,
            NodePayload {
                ts_type_parameter_instantiation: TsTypeParameterInstantiationPayload {
                    params: params.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_literal_type(&mut self, span: Span, literal: TsLiteral<'id>) -> TsLiteralType<'id> {
        TsLiteralType(self.add_node(
            span,
            NodeKind::TsLiteralType,
            NodePayload {
                ts_literal_type: TsLiteralTypePayload {
                    literal: literal.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_template_literal_type(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsTemplateLiteralType<'id> {
        TsTemplateLiteralType(self.add_node(
            span,
            NodeKind::TsTemplateLiteralType,
            NodePayload {
                ts_template_literal_type: TsTemplateLiteralTypePayload {
                    quasis: quasis.raw(),
                    types: types.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn ts_array_type(&mut self, span: Span, element_type: TsType<'id>) -> TsArrayType<'id> {
        TsArrayType(self.add_node(
            span,
            NodeKind::TsArrayType,
            NodePayload {
                ts_array_type: TsArrayTypePayload {
                    element_type: element_type.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_indexed_access_type(
        &mut self,
        span: Span,
        object_type: TsType<'id>,
        index_type: TsType<'id>,
    ) -> TsIndexedAccessType<'id> {
        TsIndexedAccessType(self.add_node(
            span,
            NodeKind::TsIndexedAccessType,
            NodePayload {
                ts_indexed_access_type: TsIndexedAccessTypePayload {
                    object_type: object_type.node_id().index() as u32,
                    index_type: index_type.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_tuple_type(
        &mut self,
        span: Span,
        element_types: SubRange<'id, TsTupleElement<'id>>,
    ) -> TsTupleType<'id> {
        TsTupleType(self.add_node(
            span,
            NodeKind::TsTupleType,
            NodePayload {
                ts_tuple_type: TsTupleTypePayload {
                    element_types: element_types.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_named_tuple_member(
        &mut self,
        span: Span,
        label: IdentifierName<'id>,
        element_type: TsType<'id>,
        optional: bool,
    ) -> TsNamedTupleMember<'id> {
        TsNamedTupleMember(self.add_node(
            span,
            NodeKind::TsNamedTupleMember,
            NodePayload {
                ts_named_tuple_member: TsNamedTupleMemberPayload {
                    label: label.node_id().index() as u32,
                    element_type: element_type.node_id().index() as u32,
                    optional,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_optional_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsOptionalType<'id> {
        TsOptionalType(self.add_node(
            span,
            NodeKind::TsOptionalType,
            NodePayload {
                ts_optional_type: TsOptionalTypePayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_rest_type(
        &mut self,
        span: Span,
        type_annotation: TsTupleElement<'id>,
    ) -> TsRestType<'id> {
        TsRestType(self.add_node(
            span,
            NodeKind::TsRestType,
            NodePayload {
                ts_rest_type: TsRestTypePayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_js_doc_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsJsDocNullableType<'id> {
        TsJsDocNullableType(self.add_node(
            span,
            NodeKind::TsJsDocNullableType,
            NodePayload {
                ts_js_doc_nullable_type: TsJsDocNullableTypePayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    postfix,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_js_doc_non_nullable_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        postfix: bool,
    ) -> TsJsDocNonNullableType<'id> {
        TsJsDocNonNullableType(self.add_node(
            span,
            NodeKind::TsJsDocNonNullableType,
            NodePayload {
                ts_js_doc_non_nullable_type: TsJsDocNonNullableTypePayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    postfix,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_js_doc_unknown_type(&mut self, span: Span) -> TsJsDocUnknownType<'id> {
        TsJsDocUnknownType(self.add_node(
            span,
            NodeKind::TsJsDocUnknownType,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn ts_union_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsUnionType<'id> {
        TsUnionType(self.add_node(
            span,
            NodeKind::TsUnionType,
            NodePayload {
                ts_union_type: TsUnionTypePayload {
                    types: types.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_intersection_type(
        &mut self,
        span: Span,
        types: SubRange<'id, TsType<'id>>,
    ) -> TsIntersectionType<'id> {
        TsIntersectionType(self.add_node(
            span,
            NodeKind::TsIntersectionType,
            NodePayload {
                ts_intersection_type: TsIntersectionTypePayload {
                    types: types.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_conditional_type(
        &mut self,
        span: Span,
        check_type: TsType<'id>,
        extends_type: TsType<'id>,
        true_type: TsType<'id>,
        false_type: TsType<'id>,
    ) -> TsConditionalType<'id> {
        TsConditionalType(self.add_node(
            span,
            NodeKind::TsConditionalType,
            NodePayload {
                ts_conditional_type: TsConditionalTypePayload {
                    check_type: check_type.node_id().index() as u32,
                    extends_type: extends_type.node_id().index() as u32,
                    true_type: true_type.node_id().index() as u32,
                    false_type: false_type.node_id().index() as u32,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_infer_type(
        &mut self,
        span: Span,
        type_parameter: TsTypeParameter<'id>,
    ) -> TsInferType<'id> {
        TsInferType(self.add_node(
            span,
            NodeKind::TsInferType,
            NodePayload {
                ts_infer_type: TsInferTypePayload {
                    type_parameter: type_parameter.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_operator(
        &mut self,
        span: Span,
        operator: TsTypeOperatorKind,
        type_annotation: TsType<'id>,
    ) -> TsTypeOperator<'id> {
        TsTypeOperator(self.add_node(
            span,
            NodeKind::TsTypeOperator,
            NodePayload {
                ts_type_operator: TsTypeOperatorPayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    operator,
                    _pad0: [0; 11usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_parenthesized_type(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
    ) -> TsParenthesizedType<'id> {
        TsParenthesizedType(self.add_node(
            span,
            NodeKind::TsParenthesizedType,
            NodePayload {
                ts_parenthesized_type: TsParenthesizedTypePayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_function_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
    ) -> TsFunctionType<'id> {
        TsFunctionType(self.add_node(
            span,
            NodeKind::TsFunctionType,
            NodePayload {
                ts_function_type: TsFunctionTypePayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    this_param: this_param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params: params.node_id().index() as u32,
                    return_type: return_type.node_id().index() as u32,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: TsTypeAnnotation<'id>,
        r#abstract: bool,
    ) -> TsConstructorType<'id> {
        TsConstructorType(self.add_node(
            span,
            NodeKind::TsConstructorType,
            NodePayload {
                ts_constructor_type: TsConstructorTypePayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params: params.node_id().index() as u32,
                    return_type: return_type.node_id().index() as u32,
                    r#abstract,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_predicate(
        &mut self,
        span: Span,
        parameter_name: TsTypePredicateName<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        asserts: bool,
    ) -> TsTypePredicate<'id> {
        TsTypePredicate(self.add_node(
            span,
            NodeKind::TsTypePredicate,
            NodePayload {
                ts_type_predicate: TsTypePredicatePayload {
                    parameter_name: parameter_name.node_id().index() as u32,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    asserts,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_literal(
        &mut self,
        span: Span,
        members: SubRange<'id, TsTypeElement<'id>>,
    ) -> TsTypeLiteral<'id> {
        TsTypeLiteral(self.add_node(
            span,
            NodeKind::TsTypeLiteral,
            NodePayload {
                ts_type_literal: TsTypeLiteralPayload {
                    members: members.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_mapped_type(
        &mut self,
        span: Span,
        key: BindingIdentifier<'id>,
        constraint: TsType<'id>,
        name_type: Option<TsType<'id>>,
        type_annotation: Option<TsType<'id>>,
        optional: TsMappedTypeModifier,
        readonly: TsMappedTypeModifier,
    ) -> TsMappedType<'id> {
        let _extra_data = [
            (name_type).into_extra_data(),
            (type_annotation).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMappedType(self.add_node(
            span,
            NodeKind::TsMappedType,
            NodePayload {
                ts_mapped_type: TsMappedTypePayload {
                    key: key.node_id().index() as u32,
                    constraint: constraint.node_id().index() as u32,
                    optional,
                    readonly,
                    _pad0: [0; 2usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_property_signature(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        type_annotation: Option<TsTypeAnnotation<'id>>,
        computed: bool,
        optional: bool,
        readonly: bool,
    ) -> TsPropertySignature<'id> {
        TsPropertySignature(self.add_node(
            span,
            NodeKind::TsPropertySignature,
            NodePayload {
                ts_property_signature: TsPropertySignaturePayload {
                    key: key.node_id().index() as u32,
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    computed,
                    optional,
                    readonly,
                    _pad0: [0; 5usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_method_signature(
        &mut self,
        span: Span,
        key: PropertyKey<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        kind: TsMethodSignatureKind,
        computed: bool,
        optional: bool,
    ) -> TsMethodSignature<'id> {
        let _extra_data = [
            (this_param).into_extra_data(),
            (params).into_extra_data(),
            (return_type).into_extra_data(),
        ];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsMethodSignature(self.add_node(
            span,
            NodeKind::TsMethodSignature,
            NodePayload {
                ts_method_signature: TsMethodSignaturePayload {
                    key: key.node_id().index() as u32,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    computed,
                    optional,
                    _pad0: [0; 1usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_call_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) -> TsCallSignatureDeclaration<'id> {
        TsCallSignatureDeclaration(self.add_node(
            span,
            NodeKind::TsCallSignatureDeclaration,
            NodePayload {
                ts_call_signature_declaration: TsCallSignatureDeclarationPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    this_param: this_param.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params: params.node_id().index() as u32,
                    return_type: return_type.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn ts_construct_signature_declaration(
        &mut self,
        span: Span,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
    ) -> TsConstructSignatureDeclaration<'id> {
        TsConstructSignatureDeclaration(self.add_node(
            span,
            NodeKind::TsConstructSignatureDeclaration,
            NodePayload {
                ts_construct_signature_declaration: TsConstructSignatureDeclarationPayload {
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    params: params.node_id().index() as u32,
                    return_type: return_type.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 4usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_index_signature(
        &mut self,
        span: Span,
        parameters: SubRange<'id, TsIndexSignatureParameter<'id>>,
        type_annotation: TsTypeAnnotation<'id>,
        readonly: bool,
        r#static: bool,
    ) -> TsIndexSignature<'id> {
        TsIndexSignature(self.add_node(
            span,
            NodeKind::TsIndexSignature,
            NodePayload {
                ts_index_signature: TsIndexSignaturePayload {
                    parameters: parameters.raw(),
                    type_annotation: type_annotation.node_id().index() as u32,
                    readonly,
                    r#static,
                    _pad0: [0; 2usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_alias_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        type_annotation: TsType<'id>,
        declare: bool,
    ) -> TsTypeAliasDeclaration<'id> {
        TsTypeAliasDeclaration(self.add_node(
            span,
            NodeKind::TsTypeAliasDeclaration,
            NodePayload {
                ts_type_alias_declaration: TsTypeAliasDeclarationPayload {
                    id: id.node_id().index() as u32,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    type_annotation: type_annotation.node_id().index() as u32,
                    declare,
                    _pad0: [0; 3usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_interface_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        extends: SubRange<'id, TsInterfaceHeritage<'id>>,
        body: TsInterfaceBody<'id>,
        declare: bool,
    ) -> TsInterfaceDeclaration<'id> {
        let _extra_data = [(extends).into_extra_data(), (body).into_extra_data()];
        let _extra_data_start = self.add_extra_data_slice(&_extra_data);
        TsInterfaceDeclaration(self.add_node(
            span,
            NodeKind::TsInterfaceDeclaration,
            NodePayload {
                ts_interface_declaration: TsInterfaceDeclarationPayload {
                    id: id.node_id().index() as u32,
                    type_parameters:
                        type_parameters.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    declare,
                    _pad0: [0; 3usize],
                    extra_data_start: _extra_data_start,
                },
            },
        ))
    }
    #[inline]
    pub fn ts_interface_body(
        &mut self,
        span: Span,
        body: SubRange<'id, TsTypeElement<'id>>,
    ) -> TsInterfaceBody<'id> {
        TsInterfaceBody(self.add_node(
            span,
            NodeKind::TsInterfaceBody,
            NodePayload {
                ts_interface_body: TsInterfaceBodyPayload {
                    body: body.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_interface_heritage(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsInterfaceHeritage<'id> {
        TsInterfaceHeritage(self.add_node(
            span,
            NodeKind::TsInterfaceHeritage,
            NodePayload {
                ts_interface_heritage: TsInterfaceHeritagePayload {
                    expression: expression.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_class_implements(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
    ) -> TsClassImplements<'id> {
        TsClassImplements(self.add_node(
            span,
            NodeKind::TsClassImplements,
            NodePayload {
                ts_class_implements: TsClassImplementsPayload {
                    expression: expression.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_enum_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        body: TsEnumBody<'id>,
        is_const: bool,
        declare: bool,
    ) -> TsEnumDeclaration<'id> {
        TsEnumDeclaration(self.add_node(
            span,
            NodeKind::TsEnumDeclaration,
            NodePayload {
                ts_enum_declaration: TsEnumDeclarationPayload {
                    id: id.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    is_const,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_enum_body(
        &mut self,
        span: Span,
        members: SubRange<'id, TsEnumMember<'id>>,
    ) -> TsEnumBody<'id> {
        TsEnumBody(self.add_node(
            span,
            NodeKind::TsEnumBody,
            NodePayload {
                ts_enum_body: TsEnumBodyPayload {
                    members: members.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_enum_member(
        &mut self,
        span: Span,
        id: TsEnumMemberName<'id>,
        initializer: Option<Expr<'id>>,
        computed: bool,
    ) -> TsEnumMember<'id> {
        TsEnumMember(self.add_node(
            span,
            NodeKind::TsEnumMember,
            NodePayload {
                ts_enum_member: TsEnumMemberPayload {
                    id: id.node_id().index() as u32,
                    initializer: initializer.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    computed,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_module_declaration(
        &mut self,
        span: Span,
        id: TsModuleName<'id>,
        body: Option<TsModuleBody<'id>>,
        kind: TsModuleDeclarationKind,
        declare: bool,
    ) -> TsModuleDeclaration<'id> {
        TsModuleDeclaration(self.add_node(
            span,
            NodeKind::TsModuleDeclaration,
            NodePayload {
                ts_module_declaration: TsModuleDeclarationPayload {
                    id: id.node_id().index() as u32,
                    body: body.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    kind,
                    declare,
                    _pad0: [0; 6usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_module_block(
        &mut self,
        span: Span,
        directives: SubRange<'id, Directive<'id>>,
        body: SubRange<'id, Stmt<'id>>,
    ) -> TsModuleBlock<'id> {
        TsModuleBlock(self.add_node(
            span,
            NodeKind::TsModuleBlock,
            NodePayload {
                ts_module_block: TsModuleBlockPayload {
                    directives: directives.raw(),
                    body: body.raw(),
                },
            },
        ))
    }
    #[inline]
    pub fn ts_global_declaration(
        &mut self,
        span: Span,
        id: IdentifierName<'id>,
        body: TsModuleBlock<'id>,
        declare: bool,
    ) -> TsGlobalDeclaration<'id> {
        TsGlobalDeclaration(self.add_node(
            span,
            NodeKind::TsGlobalDeclaration,
            NodePayload {
                ts_global_declaration: TsGlobalDeclarationPayload {
                    id: id.node_id().index() as u32,
                    body: body.node_id().index() as u32,
                    declare,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_parameter_property(
        &mut self,
        span: Span,
        decorators: SubRange<'id, Decorator<'id>>,
        parameter: FormalParameter<'id>,
        r#override: bool,
        readonly: bool,
        accessibility: Accessibility,
    ) -> TsParameterProperty<'id> {
        TsParameterProperty(self.add_node(
            span,
            NodeKind::TsParameterProperty,
            NodePayload {
                ts_parameter_property: TsParameterPropertyPayload {
                    decorators: decorators.raw(),
                    parameter: parameter.node_id().index() as u32,
                    r#override,
                    readonly,
                    accessibility,
                    _pad0: [0; 1usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_this_parameter(
        &mut self,
        span: Span,
        type_annotation: Option<TsTypeAnnotation<'id>>,
    ) -> TsThisParameter<'id> {
        TsThisParameter(self.add_node(
            span,
            NodeKind::TsThisParameter,
            NodePayload {
                ts_this_parameter: TsThisParameterPayload {
                    type_annotation:
                        type_annotation.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> TsAsExpression<'id> {
        TsAsExpression(self.add_node(
            span,
            NodeKind::TsAsExpression,
            NodePayload {
                ts_as_expression: TsAsExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> TsSatisfiesExpression<'id> {
        TsSatisfiesExpression(self.add_node(
            span,
            NodeKind::TsSatisfiesExpression,
            NodePayload {
                ts_satisfies_expression: TsSatisfiesExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> TsTypeAssertion<'id> {
        TsTypeAssertion(self.add_node(
            span,
            NodeKind::TsTypeAssertion,
            NodePayload {
                ts_type_assertion: TsTypeAssertionPayload {
                    type_annotation: type_annotation.node_id().index() as u32,
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> TsNonNullExpression<'id> {
        TsNonNullExpression(self.add_node(
            span,
            NodeKind::TsNonNullExpression,
            NodePayload {
                ts_non_null_expression: TsNonNullExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> TsInstantiationExpression<'id> {
        TsInstantiationExpression(self.add_node(
            span,
            NodeKind::TsInstantiationExpression,
            NodePayload {
                ts_instantiation_expression: TsInstantiationExpressionPayload {
                    expression: expression.node_id().index() as u32,
                    type_arguments: type_arguments.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_export_assignment(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> TsExportAssignment<'id> {
        TsExportAssignment(self.add_node(
            span,
            NodeKind::TsExportAssignment,
            NodePayload {
                ts_export_assignment: TsExportAssignmentPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_namespace_export_declaration(
        &mut self,
        span: Span,
        id: IdentifierName<'id>,
    ) -> TsNamespaceExportDeclaration<'id> {
        TsNamespaceExportDeclaration(self.add_node(
            span,
            NodeKind::TsNamespaceExportDeclaration,
            NodePayload {
                ts_namespace_export_declaration: TsNamespaceExportDeclarationPayload {
                    id: id.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_import_equals_declaration(
        &mut self,
        span: Span,
        id: BindingIdentifier<'id>,
        module_reference: TsModuleReference<'id>,
        import_kind: ImportOrExportKind,
    ) -> TsImportEqualsDeclaration<'id> {
        TsImportEqualsDeclaration(self.add_node(
            span,
            NodeKind::TsImportEqualsDeclaration,
            NodePayload {
                ts_import_equals_declaration: TsImportEqualsDeclarationPayload {
                    id: id.node_id().index() as u32,
                    module_reference: module_reference.node_id().index() as u32,
                    import_kind,
                    _pad0: [0; 7usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_external_module_reference(
        &mut self,
        span: Span,
        expression: StringLiteral<'id>,
    ) -> TsExternalModuleReference<'id> {
        TsExternalModuleReference(self.add_node(
            span,
            NodeKind::TsExternalModuleReference,
            NodePayload {
                ts_external_module_reference: TsExternalModuleReferencePayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn ts_index_signature_parameter(
        &mut self,
        span: Span,
        name: BindingIdentifier<'id>,
        type_annotation: TsTypeAnnotation<'id>,
    ) -> TsIndexSignatureParameter<'id> {
        TsIndexSignatureParameter(self.add_node(
            span,
            NodeKind::TsIndexSignatureParameter,
            NodePayload {
                ts_index_signature_parameter: TsIndexSignatureParameterPayload {
                    name: name.node_id().index() as u32,
                    type_annotation: type_annotation.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_element_name_jsx_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> JsxElementName<'id> {
        JsxElementName::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier<'id>,
        name: JsxIdentifier<'id>,
    ) -> JsxElementName<'id> {
        JsxElementName::JsxNamespacedName(self.jsx_namespaced_name(span, namespace, name).into())
    }
    #[inline]
    pub fn jsx_element_name_jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject<'id>,
        property: JsxIdentifier<'id>,
    ) -> JsxElementName<'id> {
        JsxElementName::JsxMemberExpression(
            self.jsx_member_expression(span, object, property).into(),
        )
    }
    #[inline]
    pub fn jsx_member_expression_object_jsx_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> JsxMemberExpressionObject<'id> {
        JsxMemberExpressionObject::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_member_expression_object_jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject<'id>,
        property: JsxIdentifier<'id>,
    ) -> JsxMemberExpressionObject<'id> {
        JsxMemberExpressionObject::JsxMemberExpression(
            self.jsx_member_expression(span, object, property).into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_name_jsx_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> JsxAttributeName<'id> {
        JsxAttributeName::JsxIdentifier(self.jsx_identifier(span, name).into())
    }
    #[inline]
    pub fn jsx_attribute_name_jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier<'id>,
        name: JsxIdentifier<'id>,
    ) -> JsxAttributeName<'id> {
        JsxAttributeName::JsxNamespacedName(self.jsx_namespaced_name(span, namespace, name).into())
    }
    #[inline]
    pub fn jsx_attribute_item_jsx_attribute(
        &mut self,
        span: Span,
        name: JsxAttributeName<'id>,
        value: Option<JsxAttributeValue<'id>>,
    ) -> JsxAttributeItem<'id> {
        JsxAttributeItem::JsxAttribute(self.jsx_attribute(span, name, value).into())
    }
    #[inline]
    pub fn jsx_attribute_item_jsx_spread_attribute(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> JsxAttributeItem<'id> {
        JsxAttributeItem::JsxSpreadAttribute(self.jsx_spread_attribute(span, argument).into())
    }
    #[inline]
    pub fn jsx_attribute_value_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> JsxAttributeValue<'id> {
        JsxAttributeValue::StringLiteral(self.string_literal(span, value, raw).into())
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression<'id>,
    ) -> JsxAttributeValue<'id> {
        JsxAttributeValue::JsxExpressionContainer(
            self.jsx_expression_container(span, expression).into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> JsxAttributeValue<'id> {
        JsxAttributeValue::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_attribute_value_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> JsxAttributeValue<'id> {
        JsxAttributeValue::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_expression_expr_identifier_reference(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::IdentifierReference(
            self.identifier_reference(span, name).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_this_expression(&mut self, span: Span) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ThisExpression(self.this_expression(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_super(&mut self, span: Span) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::Super(self.super_(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_meta_property(
        &mut self,
        span: Span,
        meta: IdentifierName<'id>,
        property: IdentifierName<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::MetaProperty(
            self.meta_property(span, meta, property).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_string_literal(
        &mut self,
        span: Span,
        value: AstWtf8<'id>,
        raw: AstUtf8<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::StringLiteral(
            self.string_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_numeric_literal(
        &mut self,
        span: Span,
        value: f64,
        raw: Option<AstUtf8<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::NumericLiteral(
            self.numeric_literal(span, value, raw).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_big_int_literal(
        &mut self,
        span: Span,
        raw: AstUtf8<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::BigIntLiteral(self.big_int_literal(span, raw).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_boolean_literal(
        &mut self,
        span: Span,
        value: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::BooleanLiteral(
            self.boolean_literal(span, value).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_null_literal(&mut self, span: Span) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::NullLiteral(self.null_literal(span).into()))
    }
    #[inline]
    pub fn jsx_expression_expr_reg_exp_literal(
        &mut self,
        span: Span,
        pattern: AstUtf8<'id>,
        flags: AstUtf8<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::RegExpLiteral(
            self.reg_exp_literal(span, pattern, flags).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_template_literal(
        &mut self,
        span: Span,
        quasis: SubRange<'id, TemplateElement<'id>>,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TemplateLiteral(
            self.template_literal(span, quasis, expressions).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_binary_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: BinaryOperator,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::BinaryExpression(
            self.binary_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_logical_expression(
        &mut self,
        span: Span,
        left: Expr<'id>,
        right: Expr<'id>,
        operator: LogicalOperator,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::LogicalExpression(
            self.logical_expression(span, left, right, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_unary_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
        operator: UnaryOperator,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::UnaryExpression(
            self.unary_expression(span, argument, operator).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_update_expression(
        &mut self,
        span: Span,
        argument: SimpleAssignmentTarget<'id>,
        operator: UpdateOperator,
        prefix: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::UpdateExpression(
            self.update_expression(span, argument, operator, prefix)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_assignment_expression(
        &mut self,
        span: Span,
        left: AssignmentTarget<'id>,
        right: Expr<'id>,
        operator: AssignmentOperator,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::AssignmentExpression(
            self.assignment_expression(span, left, right, operator)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_conditional_expression(
        &mut self,
        span: Span,
        test: Expr<'id>,
        consequent: Expr<'id>,
        alternate: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ConditionalExpression(
            self.conditional_expression(span, test, consequent, alternate)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_sequence_expression(
        &mut self,
        span: Span,
        expressions: SubRange<'id, Expr<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::SequenceExpression(
            self.sequence_expression(span, expressions).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_member_expression(
        &mut self,
        span: Span,
        object: Expr<'id>,
        property: PropertyKey<'id>,
        computed: bool,
        optional: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::MemberExpression(
            self.member_expression(span, object, property, computed, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_call_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
        optional: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::CallExpression(
            self.call_expression(span, callee, type_arguments, arguments, optional)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_chain_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ChainExpression(
            self.chain_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_new_expression(
        &mut self,
        span: Span,
        callee: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        arguments: SubRange<'id, Argument<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::NewExpression(
            self.new_expression(span, callee, type_arguments, arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_tagged_template_expression(
        &mut self,
        span: Span,
        tag: Expr<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        quasi: TemplateLiteral<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TaggedTemplateExpression(
            self.tagged_template_expression(span, tag, type_arguments, quasi)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_arrow_function_expression(
        &mut self,
        span: Span,
        expression: bool,
        r#async: bool,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: ArrowFunctionBody<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ArrowFunctionExpression(
            self.arrow_function_expression(
                span,
                expression,
                r#async,
                type_parameters,
                params,
                return_type,
                body,
            )
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_array_expression(
        &mut self,
        span: Span,
        elements: SubRange<'id, Option<Argument<'id>>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ArrayExpression(
            self.array_expression(span, elements).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_object_expression(
        &mut self,
        span: Span,
        properties: SubRange<'id, ObjectPropertyKind<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ObjectExpression(
            self.object_expression(span, properties).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_parenthesized_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ParenthesizedExpression(
            self.parenthesized_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_function(
        &mut self,
        span: Span,
        function_type: FunctionType,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        this_param: Option<TsThisParameter<'id>>,
        params: FormalParameters<'id>,
        return_type: Option<TsTypeAnnotation<'id>>,
        body: FunctionBody<'id>,
        generator: bool,
        r#async: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::Function(
            self.function(
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
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_class(
        &mut self,
        span: Span,
        class_type: ClassType,
        decorators: SubRange<'id, Decorator<'id>>,
        id: Option<BindingIdentifier<'id>>,
        type_parameters: Option<TsTypeParameterDeclaration<'id>>,
        super_class: Option<Expr<'id>>,
        super_type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        implements: SubRange<'id, TsClassImplements<'id>>,
        body: ClassBody<'id>,
        r#abstract: bool,
        declare: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::Class(
            self.class(
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
            .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_import_expression(
        &mut self,
        span: Span,
        source: Expr<'id>,
        options: Option<Expr<'id>>,
        phase: Option<ImportPhase>,
        sync: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::ImportExpression(
            self.import_expression(span, source, options, phase, sync)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_await_expression(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::AwaitExpression(
            self.await_expression(span, argument).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_yield_expression(
        &mut self,
        span: Span,
        argument: Option<Expr<'id>>,
        delegate: bool,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::YieldExpression(
            self.yield_expression(span, argument, delegate).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_private_identifier(
        &mut self,
        span: Span,
        name: AstUtf8<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::PrivateIdentifier(
            self.private_identifier(span, name).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_as_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TsAsExpression(
            self.ts_as_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_satisfies_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_annotation: TsType<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TsSatisfiesExpression(
            self.ts_satisfies_expression(span, expression, type_annotation)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_type_assertion(
        &mut self,
        span: Span,
        type_annotation: TsType<'id>,
        expression: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TsTypeAssertion(
            self.ts_type_assertion(span, type_annotation, expression)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_non_null_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TsNonNullExpression(
            self.ts_non_null_expression(span, expression).into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_ts_instantiation_expression(
        &mut self,
        span: Span,
        expression: Expr<'id>,
        type_arguments: TsTypeParameterInstantiation<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::TsInstantiationExpression(
            self.ts_instantiation_expression(span, expression, type_arguments)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_expr_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> JsxExpression<'id> {
        JsxExpression::Expr(Expr::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        ))
    }
    #[inline]
    pub fn jsx_expression_jsx_empty_expression(&mut self, span: Span) -> JsxExpression<'id> {
        JsxExpression::JsxEmptyExpression(self.jsx_empty_expression(span).into())
    }
    #[inline]
    pub fn jsx_child_jsx_text(&mut self, span: Span, value: AstUtf8<'id>) -> JsxChild<'id> {
        JsxChild::JsxText(self.jsx_text(span, value).into())
    }
    #[inline]
    pub fn jsx_child_jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression<'id>,
    ) -> JsxChild<'id> {
        JsxChild::JsxExpressionContainer(self.jsx_expression_container(span, expression).into())
    }
    #[inline]
    pub fn jsx_child_jsx_spread_child(
        &mut self,
        span: Span,
        expression: Expr<'id>,
    ) -> JsxChild<'id> {
        JsxChild::JsxSpreadChild(self.jsx_spread_child(span, expression).into())
    }
    #[inline]
    pub fn jsx_child_jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> JsxChild<'id> {
        JsxChild::JsxElement(
            self.jsx_element(span, opening_element, children, closing_element)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_child_jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> JsxChild<'id> {
        JsxChild::JsxFragment(
            self.jsx_fragment(span, opening_fragment, children, closing_fragment)
                .into(),
        )
    }
    #[inline]
    pub fn jsx_element(
        &mut self,
        span: Span,
        opening_element: JsxOpeningElement<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_element: Option<JsxClosingElement<'id>>,
    ) -> JsxElement<'id> {
        JsxElement(self.add_node(
            span,
            NodeKind::JsxElement,
            NodePayload {
                jsx_element: JsxElementPayload {
                    children: children.raw(),
                    opening_element: opening_element.node_id().index() as u32,
                    closing_element:
                        closing_element.map_or(u32::MAX, |node| node.node_id().index() as u32),
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_opening_element(
        &mut self,
        span: Span,
        name: JsxElementName<'id>,
        type_arguments: Option<TsTypeParameterInstantiation<'id>>,
        attributes: SubRange<'id, JsxAttributeItem<'id>>,
        self_closing: bool,
    ) -> JsxOpeningElement<'id> {
        let _f0 = self.add_extra(attributes);
        JsxOpeningElement(self.add_node(
            span,
            NodeKind::JsxOpeningElement,
            NodePayload {
                jsx_opening_element: JsxOpeningElementPayload {
                    name: name.node_id().index() as u32,
                    type_arguments:
                        type_arguments.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    self_closing,
                    _pad0: [0; 3usize],
                    extra_data_start: _f0,
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_closing_element(
        &mut self,
        span: Span,
        name: JsxElementName<'id>,
    ) -> JsxClosingElement<'id> {
        JsxClosingElement(self.add_node(
            span,
            NodeKind::JsxClosingElement,
            NodePayload {
                jsx_closing_element: JsxClosingElementPayload {
                    name: name.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_fragment(
        &mut self,
        span: Span,
        opening_fragment: JsxOpeningFragment<'id>,
        children: SubRange<'id, JsxChild<'id>>,
        closing_fragment: JsxClosingFragment<'id>,
    ) -> JsxFragment<'id> {
        JsxFragment(self.add_node(
            span,
            NodeKind::JsxFragment,
            NodePayload {
                jsx_fragment: JsxFragmentPayload {
                    children: children.raw(),
                    opening_fragment: opening_fragment.node_id().index() as u32,
                    closing_fragment: closing_fragment.node_id().index() as u32,
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_opening_fragment(&mut self, span: Span) -> JsxOpeningFragment<'id> {
        JsxOpeningFragment(self.add_node(
            span,
            NodeKind::JsxOpeningFragment,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_closing_fragment(&mut self, span: Span) -> JsxClosingFragment<'id> {
        JsxClosingFragment(self.add_node(
            span,
            NodeKind::JsxClosingFragment,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_identifier(&mut self, span: Span, name: AstUtf8<'id>) -> JsxIdentifier<'id> {
        JsxIdentifier(self.add_node(
            span,
            NodeKind::JsxIdentifier,
            NodePayload {
                jsx_identifier: JsxIdentifierPayload {
                    name: name.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_namespaced_name(
        &mut self,
        span: Span,
        namespace: JsxIdentifier<'id>,
        name: JsxIdentifier<'id>,
    ) -> JsxNamespacedName<'id> {
        JsxNamespacedName(self.add_node(
            span,
            NodeKind::JsxNamespacedName,
            NodePayload {
                jsx_namespaced_name: JsxNamespacedNamePayload {
                    namespace: namespace.node_id().index() as u32,
                    name: name.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_member_expression(
        &mut self,
        span: Span,
        object: JsxMemberExpressionObject<'id>,
        property: JsxIdentifier<'id>,
    ) -> JsxMemberExpression<'id> {
        JsxMemberExpression(self.add_node(
            span,
            NodeKind::JsxMemberExpression,
            NodePayload {
                jsx_member_expression: JsxMemberExpressionPayload {
                    object: object.node_id().index() as u32,
                    property: property.node_id().index() as u32,
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_attribute(
        &mut self,
        span: Span,
        name: JsxAttributeName<'id>,
        value: Option<JsxAttributeValue<'id>>,
    ) -> JsxAttribute<'id> {
        JsxAttribute(self.add_node(
            span,
            NodeKind::JsxAttribute,
            NodePayload {
                jsx_attribute: JsxAttributePayload {
                    name: name.node_id().index() as u32,
                    value: value.map_or(u32::MAX, |node| node.node_id().index() as u32),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_spread_attribute(
        &mut self,
        span: Span,
        argument: Expr<'id>,
    ) -> JsxSpreadAttribute<'id> {
        JsxSpreadAttribute(self.add_node(
            span,
            NodeKind::JsxSpreadAttribute,
            NodePayload {
                jsx_spread_attribute: JsxSpreadAttributePayload {
                    argument: argument.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_expression_container(
        &mut self,
        span: Span,
        expression: JsxExpression<'id>,
    ) -> JsxExpressionContainer<'id> {
        JsxExpressionContainer(self.add_node(
            span,
            NodeKind::JsxExpressionContainer,
            NodePayload {
                jsx_expression_container: JsxExpressionContainerPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_empty_expression(&mut self, span: Span) -> JsxEmptyExpression<'id> {
        JsxEmptyExpression(self.add_node(
            span,
            NodeKind::JsxEmptyExpression,
            NodePayload { raw: 0 },
        ))
    }
    #[inline]
    pub fn jsx_text(&mut self, span: Span, value: AstUtf8<'id>) -> JsxText<'id> {
        JsxText(self.add_node(
            span,
            NodeKind::JsxText,
            NodePayload {
                jsx_text: JsxTextPayload {
                    value: value.raw(),
                    _pad0: [0; 8usize],
                },
            },
        ))
    }
    #[inline]
    pub fn jsx_spread_child(&mut self, span: Span, expression: Expr<'id>) -> JsxSpreadChild<'id> {
        JsxSpreadChild(self.add_node(
            span,
            NodeKind::JsxSpreadChild,
            NodePayload {
                jsx_spread_child: JsxSpreadChildPayload {
                    expression: expression.node_id().index() as u32,
                    _pad0: [0; 12usize],
                },
            },
        ))
    }
}
