use std::borrow::Cow;

use swc_next_ecma_ast::{
    Ast, Diagnostic, ImportOrExportKind, Label, NodeId, NodeKind, NodeKindData, Span, Traverse,
    TsClassImplements, TsEnumMember, TsExportAssignment, TsGlobalDeclaration, TsModuleDeclaration,
    TsModuleReference, TsQualifiedName, TsType,
};

use crate::{
    ReferenceSpace, Semantic, SemanticBuilder as JsSemanticBuilder, SymbolFlags,
    builder::ReportOptions,
    scope::ScopeKind,
    state::{Cx, State},
    ts::check,
};

pub struct SemanticBuilder<'t, 'a> {
    pub(crate) base: JsSemanticBuilder<'t, 'a>,

    ts_namespace_depth: u32,
    in_ts_module_name: bool,
}

impl<'t, 'a> SemanticBuilder<'t, 'a> {
    pub(crate) fn ast(&self) -> &'t Ast<'a> {
        &self.base.ast
    }

    pub(crate) fn check_syntax(&self) -> bool {
        self.base.check_syntax
    }

    /// True when the walker is currently inside a TS namespace body.
    pub(crate) fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    fn is_conditional_extends(&self, state: &State<'a>, node: TsType) -> bool {
        state.parent().is_some_and(|parent| {
            matches!(
                self.ast().kind_data(parent),
                NodeKindData::TsConditionalType(conditional)
                    if conditional.extends_type(self.ast()).node_id() == node.node_id()
            )
        })
    }
}

impl<'t, 'a> SemanticBuilder<'t, 'a> {
    #[inline(always)]
    fn check(&mut self, node_data: NodeKindData) {
        match node_data {
            NodeKindData::TsExportAssignment(node) => check::check_ts_export_assignment(self, node),
            _ => {}
        }
    }

    pub fn finish(mut self, mut state: State<'a>) -> (Semantic<'a>, Vec<Diagnostic<'a>>) {
        let check_syntax = self.base.check_syntax;
        let (export_alias_issues, import_alias_issues) =
            state
                .symbols
                .resolve_ts_aliases(self.ast(), &state.scope, check_syntax);
        if self.base.check_syntax {
            check::check_ts_alias_issues(&mut self, export_alias_issues, import_alias_issues);
        }
        self.base.finish(state)
    }
}

impl<'a> Traverse<'a, State<'a>> for SemanticBuilder<'_, 'a> {
    fn enter_node(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        self.base.enter_node(node, ctx);
    }

    #[inline(always)]
    fn exit_node(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        if self.base.check_syntax {
            self.check(node);
        }
        self.base.exit_node(node, ctx);
    }

    #[inline(always)]
    fn enter_scope(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        let st = &mut ctx.state;
        match node {
            NodeKindData::TsFunction(node) => {
                st.scope.enter_ts_function(node.node_id());
                st.symbols.enter_ts_function(self.ast(), node, &st.scope);
            }
            NodeKindData::TsFunctionType(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_ts_function_type(&st.scope);
            }
            NodeKindData::TsConstructorType(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_ts_constructor_type(&st.scope);
            }
            NodeKindData::TsMethodSignature(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_ts_method_signature(&st.scope);
            }
            NodeKindData::TsIndexSignature(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_ts_index_signature(&st.scope);
            }
            NodeKindData::TsMappedType(node) => {
                st.scope.enter_block_scope(node.node_id());
                let parent = st.parent();
                st.symbols
                    .enter_ts_mapped_type(self.ast(), parent, &st.scope);
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_ts_call_signature_declaration(&st.scope);
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols
                    .enter_ts_construct_signature_declaration(&st.scope);
            }
            NodeKindData::TsEnumBody(node) => {
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    st.parent().map(|parent| self.ast().kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = st
                    .symbols
                    .symbol_of_node(declaration.id(self.ast()).node_id())
                    .expect("an enum declaration has a symbol");
                st.symbols.enter_ts_enum_members(owner);
                st.scope.enter_ts_enum_body(node.node_id());
                st.symbols.enter_ts_enum_body(&st.scope);
            }
            NodeKindData::TsModuleDeclaration(node) => {
                self.in_ts_module_name = false;
                let declarations = st.symbols.enter_ts_module_body(self.ast(), node);
                if self.check_syntax() {
                    for prepared in declarations {
                        check::check_prepared_redeclaration(self, prepared);
                    }
                }
            }
            NodeKindData::TsModuleBlock(node) => {
                self.ts_namespace_depth += 1;
                st.scope.enter_ts_module_block(node.node_id());
                st.symbols.enter_ts_module_block(&st.scope);
            }
            NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsConditionalType(_) => st.scope.enter_block_scope(node.node_id()),
            NodeKindData::TsTypeParameter(parameter)
                if st.parent().is_some_and(|parent| {
                    self.ast().node_kind(parent) == NodeKind::TsInferType
                }) =>
            {
                // The name is bound before this constraint/default region.
                st.scope
                    .enter_ts_lookup_view(node.node_id(), ScopeKind::TsInferConstraint);
                st.symbols.expose_ts_infer_parameter(
                    self.ast(),
                    parameter.name(self.ast()),
                    &st.scope,
                );
            }
            _ => {}
        }
    }

    #[inline(always)]
    fn leave_scope(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        let st = &mut ctx.state;
        match node {
            NodeKindData::TsFunction(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsFunctionType(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructorType(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsMethodSignature(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsIndexSignature(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsMappedType(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsEnumBody(node) => {
                st.scope.exit_scope(node.node_id());
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    st.parent().map(|parent| self.ast().kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = st
                    .symbols
                    .symbol_of_node(declaration.id(self.ast()).node_id())
                    .expect("an enum declaration has a symbol");
                st.symbols.exit_ts_enum_members(owner);
            }
            NodeKindData::TsModuleDeclaration(_) => {
                st.symbols.exit_ts_module_body();
                st.symbols.exit_saved_context();
            }
            NodeKindData::TsModuleBlock(_) => {
                debug_assert!(self.ts_namespace_depth > 0);
                st.scope.exit_scope(node.node_id());
                self.ts_namespace_depth -= 1;
            }
            NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsConditionalType(_) => st.scope.exit_scope(node.node_id()),
            NodeKindData::TsTypeParameter(_)
                if st.parent().is_some_and(|parent| {
                    self.ast().node_kind(parent) == NodeKind::TsInferType
                }) =>
            {
                st.scope.exit_ts_lookup_view(node.node_id());
            }
            _ => {}
        }
    }

    fn enter_ts_qualified_name(&mut self, _node: TsQualifiedName, _ctx: &mut Cx<'_, 'a>) {
        let flags = self.reference_flags;
        let space = if !self.in_ts_module_name
            && matches!(
                flags.space,
                ReferenceSpace::Value | ReferenceSpace::Type | ReferenceSpace::Namespace
            ) {
            ReferenceSpace::Namespace
        } else {
            flags.space
        };
        self.push_reference_flags(flags.with_space(space));
    }

    fn exit_ts_qualified_name(&mut self, _node: TsQualifiedName, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_ts_export_assignment(&mut self, node: TsExportAssignment, _ctx: &mut Cx<'_, 'a>) {
        let expression = node.expression(self.ast()).node_id();
        self.push_reference_flags(
            self.flags_with_space(export_expression_space(self.ast, expression)),
        );
    }

    fn exit_ts_export_assignment(&mut self, _node: TsExportAssignment, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_ts_class_implements(&mut self, node: TsClassImplements, _ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
        let expression = node.expression(self.ast).node_id();
        self.reference_flags = self.heritage_reference_flags(expression);
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .symbols
            .enter_ts_import_equals_declaration(node.import_kind(self.ast), &ctx.state.scope);
        // Import-equals narrows lookup without erasing runtime reads.
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if node.import_kind(self.ast) == ImportOrExportKind::Type {
            flags = flags.no_access();
        }
        self.push_reference_flags(flags);
    }

    fn enter_ts_module_reference(&mut self, _node: TsModuleReference, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags.space = ReferenceSpace::Namespace;
    }

    fn enter_ts_enum_member(&mut self, node: TsEnumMember, ctx: &mut Cx<'_, 'a>) {
        let prepared = ctx
            .state
            .symbols
            .prepare_ts_enum_member(self.ast, node, &ctx.state.scope);
        if let Some(prepared) = prepared {
            if self.check_syntax {
                ts::check::check_prepared_redeclaration(self, prepared);
            }
            ctx.state
                .symbols
                .declare_prepared_ts_enum_member(self.ast, prepared);
        }
    }

    fn enter_ts_module_declaration(&mut self, node: TsModuleDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .enter_ts_module_declaration(self.ast, node, &ctx.state.scope);
        self.in_ts_module_name = true;
    }

    fn enter_ts_global_declaration(&mut self, node: TsGlobalDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .enter_ts_global_declaration(self.ast, node);
    }

    fn exit_ts_global_declaration(&mut self, _node: TsGlobalDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_ts_global_declaration();
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .symbols
            .enter_ts_type_alias_declaration(self.ast, node, &ctx.state.scope);
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        _node: TsTypeAliasDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .symbols
            .enter_ts_interface_declaration(self.ast, node, &ctx.state.scope);
    }

    fn exit_ts_interface_declaration(
        &mut self,
        _node: TsInterfaceDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_ts_interface_heritage(&mut self, node: TsInterfaceHeritage, _ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
        let expression = node.expression(self.ast).node_id();
        self.reference_flags = self.heritage_reference_flags(expression);
    }

    fn enter_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_query(&mut self, _node: TsTypeQuery, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.flags_with_space(ReferenceSpace::Typeof);
    }

    fn enter_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut Cx<'_, 'a>) {
        // Also reached through TsTypeQueryExprName, without a TsType hook.
        self.enter_type_position();
    }

    fn exit_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_parameter(&mut self, _node: TsTypeParameter, ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
        let parent = ctx.state.parent();
        ctx.state
            .symbols
            .enter_ts_type_parameter(self.ast, parent, &ctx.state.scope);
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .scope
            .enter_ts_lookup_view(node.node_id(), ScopeKind::TsTypeParameters);
    }

    fn exit_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state.scope.exit_ts_lookup_view(node.node_id());
    }

    fn enter_ts_type_predicate(&mut self, node: TsTypePredicate, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .add_ts_type_predicate_reference(self.ast, node, &ctx.state.scope);
    }

    fn enter_ts_method_signature(&mut self, _node: TsMethodSignature, _ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_method_signature(&mut self, _node: TsMethodSignature, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_index_signature(&mut self, _node: TsIndexSignature, _ctx: &mut Cx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_index_signature(&mut self, _node: TsIndexSignature, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_annotation(&mut self, node: TsTypeAnnotation, ctx: &mut Cx<'_, 'a>) {
        let signature_return = ctx.state.parent().is_some_and(|parent| {
            matches!(
                self.ast.node_kind(parent),
                NodeKind::Function
                    | NodeKind::TsFunction
                    | NodeKind::ArrowFunctionExpression
                    | NodeKind::TsFunctionType
                    | NodeKind::TsConstructorType
                    | NodeKind::TsMethodSignature
                    | NodeKind::TsCallSignatureDeclaration
                    | NodeKind::TsConstructSignatureDeclaration
                    | NodeKind::TsIndexSignature
            )
        });
        let view = signature_return.then_some(ScopeKind::TsSignatureReturn);
        if let Some(view) = view {
            ctx.state.scope.enter_ts_lookup_view(node.node_id(), view);
        }
    }

    fn exit_ts_type_annotation(&mut self, node: TsTypeAnnotation, ctx: &mut Cx<'_, 'a>) {
        let signature_return = ctx.state.parent().is_some_and(|parent| {
            matches!(
                self.ast.node_kind(parent),
                NodeKind::Function
                    | NodeKind::TsFunction
                    | NodeKind::ArrowFunctionExpression
                    | NodeKind::TsFunctionType
                    | NodeKind::TsConstructorType
                    | NodeKind::TsMethodSignature
                    | NodeKind::TsCallSignatureDeclaration
                    | NodeKind::TsConstructSignatureDeclaration
                    | NodeKind::TsIndexSignature
            )
        });
        let view = signature_return.then_some(ScopeKind::TsSignatureReturn);
        if view.is_some() {
            ctx.state.scope.exit_ts_lookup_view(node.node_id());
        }
    }

    fn enter_ts_enum_declaration(&mut self, node: TsEnumDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .enter_ts_enum_declaration(self.ast, node, &ctx.state.scope);
    }

    fn exit_ts_enum_declaration(&mut self, _node: TsEnumDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        if ctx.state.symbols.in_unsupported_ts_external_module() {
            return;
        }
        ctx.state
            .symbols
            .record_ts_global_export(self.ast, node, &ctx.state.scope);
    }

    fn exit_ts_class_implements(&mut self, _node: TsClassImplements, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn exit_ts_interface_heritage(&mut self, _node: TsInterfaceHeritage, _ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
    }

    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .symbols
            .record_ts_import_equals_target(self.ast, node);
        ctx.state.symbols.exit_saved_context();
        self.pop_reference_flags();
    }

    fn exit_ts_type_parameter(&mut self, _node: TsTypeParameter, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_saved_context();
        self.exit_type_position();
    }

    fn enter_ts_instantiation_expression(
        &mut self,
        _node: TsInstantiationExpression,
        _ctx: &mut Cx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_ts_type(&mut self, node: TsType, ctx: &mut Cx<'_, 'a>) {
        // The conditional scope spans extends and true, but infer bindings
        // must remain hidden from references elsewhere in the extends subtree.
        if self.is_conditional_extends(&ctx.state, node) {
            ctx.state
                .scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsConditionalExtends);
        }
        // Covers concrete type nodes, including nested types inside typeof.
        self.enter_type_position();
    }

    fn exit_ts_type(&mut self, node: TsType, ctx: &mut Cx<'_, 'a>) {
        self.exit_type_position();
        if self.is_conditional_extends(&ctx.state, node) {
            ctx.state.scope.exit_ts_lookup_view(node.node_id());
        }
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    pub fn report(
        &mut self,
        span: Span,
        message: impl Into<Cow<'a, str>>,
        options: ReportOptions<'a>,
    ) {
        self.base.report(span, message, options);
    }

    pub fn report_redeclaration(
        &mut self,
        name: &[u8],
        node: NodeId,
        existing_node: NodeId,
        existing_flags: SymbolFlags,
    ) {
        self.base
            .report_redeclaration(name, node, existing_node, existing_flags);
    }

    pub fn report_ts_namespace_export_alias_conflict(
        &mut self,
        name: &[u8],
        node: NodeId,
        previous: NodeId,
        duplicate: bool,
    ) {
        let name = String::from_utf8_lossy(name);
        let message = if duplicate {
            format!("Export '{}' has already been declared", name)
        } else {
            format!(
                "Export declaration conflicts with exported declaration of '{}'",
                name
            )
        };
        let current_span = self.ast().span(node);
        let previous_span = self.ast().span(previous);
        let labels = vec![
            Label::new(
                previous_span,
                Cow::Borrowed("the existing export is declared here"),
            ),
            Label::new(current_span, Cow::Borrowed("this export conflicts with it")),
        ];
        self.report(
            current_span,
            message,
            ReportOptions {
                labels,
                ..ReportOptions::default()
            },
        );
    }

    pub fn report_ts_import_alias_type_only_export(&mut self, node: NodeId, export: NodeId) {
        let current_span = self.ast().span(node);
        let export_span = self.ast().span(export);
        let labels = vec![
            Label::new(
                export_span,
                Cow::Borrowed("the declaration was exported as type-only here"),
            ),
            Label::new(
                current_span,
                Cow::Borrowed("this import alias references it here"),
            ),
        ];
        self.report(
            current_span,
            "An import alias cannot reference a declaration that was exported using 'export type'",
            ReportOptions {
                labels,
                ..ReportOptions::default()
            },
        );
    }
}
