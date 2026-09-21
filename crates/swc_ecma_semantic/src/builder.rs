//! The semantic builder, a visitor that builds the semantic model and reports
//! the scope-dependent early errors the specification requires.

mod declarations;
mod js;
mod ts;

use std::{borrow::Cow, collections::hash_map::Entry};

use rustc_hash::FxHashMap;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AssignmentPattern, Ast, AwaitExpression, BinaryExpression, BinaryOperator, BindingIdentifier,
    BindingPattern, CallExpression, ChainExpression, Class, ClassBody, ClassType,
    ConditionalExpression, Decorator, Diagnostic, Directive, ExportAllDeclaration,
    ExportDefaultDeclaration, ExportDefaultDeclarationKindData, ExportNamedDeclaration,
    ExportSpecifier, ForInStatement, ForOfStatement, ForStatementLeft, FormalParameters, Function,
    FunctionType, IdentifierReference, ImportDeclaration, ImportExpression, ImportOrExportKind,
    ImportSpecifier, JsxClosingElement, JsxElement, JsxFragment, JsxOpeningElement, Label,
    LogicalExpression, MemberExpression, NewExpression, NodeId, NodeKind, NodeKindData,
    ObjectExpression, PropertyKey, PropertyKeyData, ScopeId, SequenceExpression, Severity,
    SimpleAssignmentTarget, Span, SwitchStatement, TaggedTemplateExpression, TemplateLiteral,
    Traverse, TraverseCtx, TsClassImplements, TsEnumDeclaration, TsEnumMember, TsExportAssignment,
    TsGlobalDeclaration, TsImportEqualsDeclaration, TsImportType, TsIndexSignature,
    TsInstantiationExpression, TsInterfaceBody, TsInterfaceDeclaration, TsInterfaceHeritage,
    TsMethodSignature, TsModuleDeclaration, TsModuleNameData, TsModuleReference,
    TsNamespaceExportDeclaration, TsQualifiedName, TsType, TsTypeAliasDeclaration,
    TsTypeAnnotation, TsTypeParameter, TsTypeParameterDeclaration, TsTypePredicate, TsTypeQuery,
    UnaryExpression, UnaryOperator, UpdateExpression, VariableDeclaration, VariableDeclarator,
    VariableKind, WithStatement, YieldExpression,
};

use crate::{
    ReferenceFlags, ReferenceSpace, Semantic, SymbolFlags,
    binder::{ExportState, PreparedDeclaration, SymbolTracker},
    ecmascript,
    module_record::ModuleRecordBuilder,
    scope::{DecoratorSite, ScopeKind, ScopeTracker},
};

pub struct SemanticBuilder<'traverse, 'a> {
    exported_names: FxHashMap<&'traverse [u8], NodeId>,
    export_specifiers: Vec<ExportSpecifier>,
    diagnostics: Vec<Diagnostic<'a>>,
    scope: ScopeTracker<'a>,
    symbols: SymbolTracker<'a>,
    module_record: ModuleRecordBuilder<'a>,
    ts_namespace_depth: u32,
    ast: &'traverse Ast<'a>,
    path: Vec<NodeId>,
    // Nested type syntax saves and restores this default through the visitor.
    default_reference_flags: ReferenceFlags,
    // Value expressions consume the incoming context and reset these flags.
    // Target, key, and type boundaries save context when later siblings need it.
    reference_flags: ReferenceFlags,
    reference_flags_stack: Vec<ReferenceFlags>,
    type_positions: Vec<ReferenceFlags>,
    arrow_exports: Vec<ExportState>,
    check_syntax: bool,
    in_ts_module_name: bool,
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    pub fn new(ast: &'traverse Ast<'a>, check_syntax: bool, build_module_record: bool) -> Self {
        Self {
            exported_names: FxHashMap::default(),
            export_specifiers: Vec::new(),
            diagnostics: Vec::new(),
            scope: ScopeTracker::init(ast),
            symbols: SymbolTracker::init(ast),
            module_record: ModuleRecordBuilder::new(ast.allocator(), build_module_record),
            default_reference_flags: ReferenceFlags::default(),
            ts_namespace_depth: 0,
            ast,
            path: Vec::with_capacity(64),
            reference_flags: ReferenceFlags::default(),
            reference_flags_stack: Vec::new(),
            type_positions: Vec::new(),
            arrow_exports: Vec::new(),
            check_syntax,
            in_ts_module_name: false,
        }
    }

    pub fn finish(mut self) -> (Semantic<'a>, Vec<Diagnostic<'a>>) {
        let check_syntax = self.check_syntax;
        let (export_alias_issues, import_alias_issues) =
            self.symbols
                .resolve_ts_aliases(self.ast, &self.scope, check_syntax);
        if check_syntax {
            ts::check_ts_alias_issues(&mut self, export_alias_issues, import_alias_issues);
        }
        debug_assert!(self.path.is_empty());
        debug_assert!(self.reference_flags_stack.is_empty());
        debug_assert!(self.type_positions.is_empty());
        debug_assert!(self.arrow_exports.is_empty());
        let (scopes, has_with_statement) = self.scope.into_scope_tree();
        let semantic =
            self.symbols
                .finalize(self.ast, scopes, has_with_statement, self.module_record);
        (semantic, self.diagnostics)
    }

    fn parent(&self) -> Option<NodeId> {
        self.path.iter().rev().nth(1).copied()
    }

    fn current_node(&self) -> Option<NodeId> {
        self.path.last().copied()
    }

    fn ancestors(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.path.iter().rev().copied()
    }

    fn record_ts_expando_assignment(&mut self, node: swc_next_ecma_ast::AssignmentExpression) {
        self.symbols.record_ts_expando_assignment(
            self.ast,
            node,
            &self.scope,
            self.path.iter().rev().copied(),
        );
    }

    #[inline]
    fn record_identifier_reference(&mut self, node: IdentifierReference, flags: ReferenceFlags) {
        self.symbols
            .add_identifier_reference(self.ast, node, &self.scope, flags);
    }

    /// True when the walker is currently inside a TS type-only subtree.
    fn in_type_position(&self) -> bool {
        self.default_reference_flags.space == ReferenceSpace::Type
    }

    /// True when the walker is currently inside a TS namespace body.
    fn in_ts_namespace(&self) -> bool {
        self.ts_namespace_depth > 0
    }

    fn is_conditional_extends(&self, node: TsType) -> bool {
        self.parent().is_some_and(|parent| {
            matches!(
                self.ast.kind_data(parent),
                NodeKindData::TsConditionalType(conditional)
                    if conditional.extends_type(self.ast).node_id() == node.node_id()
            )
        })
    }

    fn enter_type_position(&mut self) {
        let flags = ReferenceFlags::default()
            .with_space(ReferenceSpace::Type)
            .no_access();
        self.type_positions
            .push(std::mem::replace(&mut self.default_reference_flags, flags));
        self.push_reference_flags(flags);
    }

    fn exit_type_position(&mut self) {
        let previous = self
            .type_positions
            .pop()
            .expect("a type position was entered");
        self.pop_reference_flags();
        debug_assert!(self.in_type_position());
        self.default_reference_flags = previous;
    }

    #[inline]
    fn assignment_reference_flags(&self, read: bool) -> ReferenceFlags {
        let flags = self.default_reference_flags;
        if !flags.is_read() {
            flags
        } else if read {
            flags.read_write()
        } else {
            flags.write_only()
        }
    }

    #[inline]
    fn flags_with_space(&self, space: ReferenceSpace) -> ReferenceFlags {
        let flags = self.default_reference_flags.with_space(space);
        if space.in_type_position() {
            flags.no_access()
        } else {
            flags
        }
    }

    fn push_reference_flags(&mut self, flags: ReferenceFlags) {
        self.reference_flags_stack
            .push(std::mem::replace(&mut self.reference_flags, flags));
    }

    fn pop_reference_flags(&mut self) {
        self.reference_flags = self
            .reference_flags_stack
            .pop()
            .expect("a reference context was entered");
    }

    fn heritage_reference_flags(&self, expression: NodeId) -> ReferenceFlags {
        let space = if self.ast.node_kind(expression) == NodeKind::MemberExpression {
            ReferenceSpace::Namespace
        } else {
            ReferenceSpace::Type
        };
        self.flags_with_space(space)
    }
}

impl SemanticBuilder<'_, '_> {
    #[inline(always)]
    fn check(&mut self, node_data: NodeKindData) {
        match node_data {
            NodeKindData::Program(_) => js::check_program(self),
            NodeKindData::StringLiteral(node) => js::check_string_literal(self, node),
            NodeKindData::NumericLiteral(node) => js::check_numeric_literal(self, node),
            NodeKindData::RegExpLiteral(node) => js::check_reg_exp_literal(self, node),
            NodeKindData::IdentifierReference(node) => js::check_identifier_reference(self, node),
            NodeKindData::LabelIdentifier(node) => js::check_label_identifier(self, node),
            NodeKindData::BinaryExpression(node) => js::check_binary_expression(self, node),
            NodeKindData::UnaryExpression(node) => js::check_unary_expression(self, node),
            NodeKindData::UpdateExpression(node) => js::check_update_expression(self, node),
            NodeKindData::AssignmentExpression(node) => js::check_assignment_expression(self, node),
            NodeKindData::VariableDeclaration(node) => js::check_variable_declaration(self, node),
            NodeKindData::SwitchStatement(node) => js::check_switch_statement(self, node),
            NodeKindData::ForInStatement(node) => js::check_for_in_statement(self, node),
            NodeKindData::ForOfStatement(node) => js::check_for_of_statement(self, node),
            NodeKindData::BreakStatement(node) => js::check_break_statement(self, node),
            NodeKindData::ContinueStatement(node) => js::check_continue_statement(self, node),
            NodeKindData::LabeledStatement(node) => js::check_labeled_statement(self, node),
            NodeKindData::Directive(node) => js::check_directive(self, node),
            NodeKindData::ClassBody(node) => js::check_class_body(self, node),
            NodeKindData::Function(node) => js::check_function(self, node),
            NodeKindData::WithStatement(node) => js::check_with_statement(self, node),
            NodeKindData::ObjectExpression(node) => js::check_object_expression(self, node),
            NodeKindData::MemberExpression(node) => js::check_member_expression(self, node),
            NodeKindData::CallExpression(node) => js::check_call_expression(self, node),
            NodeKindData::AwaitExpression(node) => js::check_await_expression(self, node),
            NodeKindData::YieldExpression(node) => js::check_yield_expression(self, node),
            NodeKindData::MetaProperty(node) => js::check_meta_property(self, node),
            NodeKindData::ImportDeclaration(node) => js::check_import_declaration(self, node),
            NodeKindData::ExportNamedDeclaration(node) => {
                js::check_export_named_declaration(self, node)
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                js::check_export_default_declaration(self, node)
            }
            NodeKindData::TsExportAssignment(node) => ts::check_ts_export_assignment(self, node),
            NodeKindData::ExportAllDeclaration(node) => {
                js::check_export_all_declaration(self, node)
            }
            NodeKindData::ExportSpecifier(node) => js::check_export_specifier(self, node),
            _ => {}
        }
    }
}

/// Builds scopes and bindings at Traverse node and scope boundaries.
impl<'a> Traverse<'a> for SemanticBuilder<'_, 'a> {
    fn enter_node(&mut self, node: NodeKindData, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.path.push(node.node_id());
    }

    #[inline(always)]
    fn exit_node(&mut self, node: NodeKindData, _ctx: &mut TraverseCtx<'_, 'a>) {
        if self.check_syntax {
            self.check(node);
        }
        let exited = self.path.pop();
        debug_assert_eq!(exited, Some(node.node_id()));
    }

    #[inline(always)]
    fn enter_scope(&mut self, node: NodeKindData, _ctx: &mut TraverseCtx<'_, 'a>) {
        match node {
            NodeKindData::Class(node) => {
                self.scope.enter_class(self.ast, node);
                self.symbols.enter_class(self.ast, node, &self.scope);
            }
            NodeKindData::BlockStatement(node) => {
                self.scope.enter_block_statement(self.ast, node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                if !node.cases(self.ast).is_empty() {
                    self.scope.enter_switch_case(Some(node.node_id()));
                }
            }
            NodeKindData::ForStatement(node) => {
                self.scope.enter_block_scope(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols.enter_catch_clause(&self.scope);
            }
            NodeKindData::WithStatement(node) => {
                self.scope.enter_with_scope(node.node_id());
            }
            NodeKindData::Function(node) => {
                let annex_b_if_body = is_annex_b_if_function(self.ast, self.parent(), node);
                self.scope.enter_function(self.ast, node, annex_b_if_body);
                self.symbols.enter_function(self.ast, node, &self.scope);
            }
            NodeKindData::FunctionBody(node) => {
                let parent = self.parent();
                self.scope.enter_function_body(self.ast, node, parent);
            }
            NodeKindData::TsFunction(node) => {
                self.scope.enter_ts_function(node.node_id());
                self.symbols.enter_ts_function(self.ast, node, &self.scope);
            }
            NodeKindData::StaticBlock(node) => {
                self.scope.enter_static_block(node.node_id());
            }
            NodeKindData::TsFunctionType(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols.enter_ts_function_type(&self.scope);
            }
            NodeKindData::TsConstructorType(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols.enter_ts_constructor_type(&self.scope);
            }
            NodeKindData::TsMethodSignature(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols.enter_ts_method_signature(&self.scope);
            }
            NodeKindData::TsIndexSignature(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols.enter_ts_index_signature(&self.scope);
            }
            NodeKindData::TsMappedType(node) => {
                self.scope.enter_block_scope(node.node_id());
                let parent = self.parent();
                self.symbols
                    .enter_ts_mapped_type(self.ast, parent, &self.scope);
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols
                    .enter_ts_call_signature_declaration(&self.scope);
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                self.scope.enter_block_scope(node.node_id());
                self.symbols
                    .enter_ts_construct_signature_declaration(&self.scope);
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                self.scope.enter_arrow_function_expression(self.ast, node);
                self.arrow_exports
                    .push(self.symbols.enter_arrow_function_expression());
            }
            NodeKindData::TsEnumBody(node) => {
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    self.parent().map(|parent| self.ast.kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = self
                    .symbols
                    .symbol_of_node(declaration.id(self.ast).node_id())
                    .expect("an enum declaration has a symbol");
                self.symbols.enter_ts_enum_members(owner);
                self.scope.enter_ts_enum_body(node.node_id());
                self.symbols.enter_ts_enum_body(&self.scope);
            }
            NodeKindData::ForInStatement(node) => self.scope.enter_block_scope(node.node_id()),
            NodeKindData::ForOfStatement(node) => self.scope.enter_block_scope(node.node_id()),
            NodeKindData::TsModuleDeclaration(node) => {
                self.in_ts_module_name = false;
                let declarations = self.symbols.enter_ts_module_body(self.ast, node);
                if self.check_syntax {
                    for prepared in declarations {
                        declarations::check_prepared_redeclaration(self, prepared);
                    }
                }
            }
            NodeKindData::TsModuleBlock(node) => {
                self.ts_namespace_depth += 1;
                self.scope.enter_ts_module_block(node.node_id());
                self.symbols.enter_ts_module_block(&self.scope);
            }
            NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsConditionalType(_) => self.scope.enter_block_scope(node.node_id()),
            NodeKindData::TsTypeParameter(parameter)
                if self
                    .parent()
                    .is_some_and(|parent| self.ast.node_kind(parent) == NodeKind::TsInferType) =>
            {
                // The name is bound before this constraint/default region.
                self.scope
                    .enter_ts_lookup_view(node.node_id(), ScopeKind::TsInferConstraint);
                self.symbols.expose_ts_infer_parameter(
                    self.ast,
                    parameter.name(self.ast),
                    &self.scope,
                );
            }
            // The root scope is initialized by SemanticBuilder::new. Global
            // container contexts use node hooks below.
            _ => {}
        }
    }

    #[inline(always)]
    fn leave_scope(&mut self, node: NodeKindData, _ctx: &mut TraverseCtx<'_, 'a>) {
        match node {
            NodeKindData::Class(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::BlockStatement(node) => {
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::ForStatement(node) => {
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::WithStatement(node) => {
                self.scope.exit_with_scope(node.node_id());
            }
            NodeKindData::Function(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::FunctionBody(node) => {
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsFunction(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::StaticBlock(node) => {
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsFunctionType(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructorType(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsMethodSignature(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsIndexSignature(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsMappedType(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                self.symbols.exit_saved_context();
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                let export_state = self
                    .arrow_exports
                    .pop()
                    .expect("an arrow saved its export state");
                self.symbols.exit_arrow_function_expression(export_state);
                self.scope.exit_scope(node.node_id());
            }
            NodeKindData::TsEnumBody(node) => {
                self.scope.exit_scope(node.node_id());
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    self.parent().map(|parent| self.ast.kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = self
                    .symbols
                    .symbol_of_node(declaration.id(self.ast).node_id())
                    .expect("an enum declaration has a symbol");
                self.symbols.exit_ts_enum_members(owner);
            }
            NodeKindData::ForInStatement(node) => self.scope.exit_scope(node.node_id()),
            NodeKindData::ForOfStatement(node) => self.scope.exit_scope(node.node_id()),
            NodeKindData::TsModuleDeclaration(_) => {
                self.symbols.exit_ts_module_body();
                self.symbols.exit_saved_context();
            }
            NodeKindData::TsModuleBlock(_) => {
                debug_assert!(self.ts_namespace_depth > 0);
                self.scope.exit_scope(node.node_id());
                self.ts_namespace_depth -= 1;
            }
            NodeKindData::TsTypeAliasDeclaration(_)
            | NodeKindData::TsInterfaceDeclaration(_)
            | NodeKindData::TsConditionalType(_) => self.scope.exit_scope(node.node_id()),
            NodeKindData::TsTypeParameter(_)
                if self
                    .parent()
                    .is_some_and(|parent| self.ast.node_kind(parent) == NodeKind::TsInferType) =>
            {
                self.scope.exit_ts_lookup_view(node.node_id());
            }
            _ => {}
        }
    }

    fn enter_binary_expression(&mut self, _node: BinaryExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_logical_expression(
        &mut self,
        _node: LogicalExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_conditional_expression(
        &mut self,
        _node: ConditionalExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_sequence_expression(
        &mut self,
        _node: SequenceExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_chain_expression(&mut self, _node: ChainExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_new_expression(&mut self, _node: NewExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_tagged_template_expression(
        &mut self,
        _node: TaggedTemplateExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_array_expression(&mut self, _node: ArrayExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_object_expression(&mut self, _node: ObjectExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_import_expression(&mut self, _node: ImportExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_await_expression(&mut self, _node: AwaitExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_yield_expression(&mut self, _node: YieldExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_ts_instantiation_expression(
        &mut self,
        _node: TsInstantiationExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_jsx_element(&mut self, _node: JsxElement, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_jsx_fragment(&mut self, _node: JsxFragment, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_template_literal(&mut self, _node: TemplateLiteral, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_function(&mut self, _node: Function, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_call_expression(&mut self, node: CallExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
        if !self.symbols.in_ambient_context()
            && self.default_reference_flags.is_read()
            && is_direct_eval_call(self.ast, node)
        {
            self.scope.mark_direct_eval();
        }
    }

    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
        self.record_ts_expando_assignment(node);
    }

    fn enter_simple_assignment_target(
        &mut self,
        _node: SimpleAssignmentTarget,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let read = match self.parent().map(|parent| self.ast.kind_data(parent)) {
            Some(NodeKindData::AssignmentExpression(parent)) => {
                parent.operator(self.ast) != AssignmentOperator::Assign
            }
            Some(NodeKindData::UpdateExpression(_)) => true,
            _ => false,
        };
        self.reference_flags = self.assignment_reference_flags(read);
    }

    fn exit_simple_assignment_target(
        &mut self,
        _node: SimpleAssignmentTarget,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // Each target establishes its own flags; following expressions read.
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_unary_expression(&mut self, node: UnaryExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        let flags = self.default_reference_flags;
        let flags = if node.operator(self.ast) == UnaryOperator::Delete {
            flags.no_access()
        } else {
            flags
        };
        self.push_reference_flags(flags);
    }

    fn exit_unary_expression(&mut self, _node: UnaryExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_member_expression(&mut self, _node: MemberExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.push_reference_flags(self.flags_with_space(self.reference_flags.space));
    }

    fn exit_member_expression(&mut self, _node: MemberExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_ts_type(&mut self, node: TsType, _ctx: &mut TraverseCtx<'_, 'a>) {
        // The conditional scope spans extends and true, but infer bindings
        // must remain hidden from references elsewhere in the extends subtree.
        if self.is_conditional_extends(node) {
            self.scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsConditionalExtends);
        }
        // Covers concrete type nodes, including nested types inside typeof.
        self.enter_type_position();
    }

    fn exit_ts_type(&mut self, node: TsType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
        if self.is_conditional_extends(node) {
            self.scope.exit_ts_lookup_view(node.node_id());
        }
    }

    fn enter_assignment_pattern(
        &mut self,
        _node: AssignmentPattern,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // The default value reads, but later destructuring targets still write.
        self.push_reference_flags(self.reference_flags);
    }

    fn exit_assignment_pattern(
        &mut self,
        _node: AssignmentPattern,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_reference_flags();
    }

    fn enter_variable_declarator(
        &mut self,
        _node: VariableDeclarator,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // A for-in/of declaration still needs its write context while binding
        // the target; the initializer switches to the default context below.
        self.push_reference_flags(self.reference_flags);
    }

    fn exit_variable_declarator(
        &mut self,
        _node: VariableDeclarator,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_reference_flags();
    }

    fn enter_binding_pattern(&mut self, _node: BindingPattern, _ctx: &mut TraverseCtx<'_, 'a>) {
        if let Some(NodeKindData::VariableDeclarator(declarator)) =
            self.parent().map(|parent| self.ast.kind_data(parent))
        {
            let is_var = self.path.iter().rev().nth(2).is_some_and(|parent| {
                matches!(
                    self.ast.kind_data(*parent),
                    NodeKindData::VariableDeclaration(declaration)
                        if declaration.kind(self.ast) == VariableKind::Var
                )
            });
            let flags = if is_var
                && self.scope.current().flags.var_write_crosses_with
                && (declarator.init(self.ast).is_some() || self.reference_flags.is_write())
            {
                self.assignment_reference_flags(false)
            } else {
                self.default_reference_flags
            };
            self.reference_flags = flags;
        }
    }

    fn exit_binding_pattern(&mut self, _node: BindingPattern, _ctx: &mut TraverseCtx<'_, 'a>) {
        if let Some(NodeKind::VariableDeclarator | NodeKind::AssignmentPattern) =
            self.parent().map(|parent| self.ast.node_kind(parent))
        {
            self.reference_flags = self.default_reference_flags;
        }
    }

    fn enter_property_key(&mut self, node: PropertyKey, _ctx: &mut TraverseCtx<'_, 'a>) {
        if matches!(
            self.ast.property_key_data(node),
            PropertyKeyData::IdentifierName(_)
        ) {
            return;
        }
        let type_query = match self.parent().map(|parent| self.ast.kind_data(parent)) {
            Some(NodeKindData::PropertyDefinition(node)) => {
                (node.declare(self.ast) || node.r#abstract(self.ast))
                    && node.decorators(self.ast).is_empty()
                    && node.computed(self.ast)
            }
            Some(NodeKindData::TsMethodDefinition(node)) => {
                node.decorators(self.ast).is_empty() && node.computed(self.ast)
            }
            Some(NodeKindData::TsPropertySignature(node)) => node.computed(self.ast),
            Some(NodeKindData::TsMethodSignature(node)) => node.computed(self.ast),
            _ => false,
        };
        let flags = if type_query {
            self.flags_with_space(ReferenceSpace::Typeof)
        } else {
            self.default_reference_flags
        };
        self.push_reference_flags(flags);
    }

    fn exit_property_key(&mut self, node: PropertyKey, _ctx: &mut TraverseCtx<'_, 'a>) {
        if !matches!(
            self.ast.property_key_data(node),
            PropertyKeyData::IdentifierName(_)
        ) {
            self.pop_reference_flags();
        }
    }

    fn enter_for_statement_left(
        &mut self,
        _node: ForStatementLeft,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.push_reference_flags(self.assignment_reference_flags(false));
    }

    fn exit_for_statement_left(&mut self, _node: ForStatementLeft, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_ts_qualified_name(&mut self, _node: TsQualifiedName, _ctx: &mut TraverseCtx<'_, 'a>) {
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

    fn exit_ts_qualified_name(&mut self, _node: TsQualifiedName, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_ts_export_assignment(
        &mut self,
        node: TsExportAssignment,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let expression = node.expression(self.ast).node_id();
        self.push_reference_flags(
            self.flags_with_space(export_expression_space(self.ast, expression)),
        );
    }

    fn exit_ts_export_assignment(
        &mut self,
        _node: TsExportAssignment,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.pop_reference_flags();
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.record_identifier_reference(node, self.reference_flags);
    }

    fn enter_directive(&mut self, node: Directive, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.scope.enter_directive(self.ast, node);
    }

    fn enter_class(&mut self, _node: Class, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_class_body(&mut self, _node: ClassBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.symbols.enter_class_body();
    }

    fn enter_decorator(&mut self, node: Decorator, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TS resolves members and parameters at the class, skipping method
        // locals. Only a class declaration's own decorators skip the class;
        // class expressions retain their type parameters and name environment.
        let site = match self.parent().map(|parent| self.ast.kind_data(parent)) {
            Some(NodeKindData::Class(class))
                if class.class_type(self.ast) == ClassType::ClassDeclaration =>
            {
                DecoratorSite::ClassDeclaration
            }
            Some(NodeKindData::Class(_)) => DecoratorSite::ClassExpression,
            _ => DecoratorSite::Member,
        };
        self.push_reference_flags(self.default_reference_flags);
        self.scope.enter_decorator(node.node_id(), site);
    }

    fn exit_decorator(&mut self, _node: Decorator, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.scope.exit_decorator();
        self.pop_reference_flags();
    }

    fn enter_ts_class_implements(
        &mut self,
        node: TsClassImplements,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
        let expression = node.expression(self.ast).node_id();
        self.reference_flags = self.heritage_reference_flags(expression);
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_import_declaration(self.ast, node, &self.scope);
    }

    fn exit_import_declaration(
        &mut self,
        _node: ImportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_saved_context();
    }

    fn enter_import_specifier(&mut self, node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.symbols.enter_import_specifier(self.ast, node);
    }

    fn exit_import_specifier(&mut self, _node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.symbols.exit_saved_context();
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_ts_import_equals_declaration(node.import_kind(self.ast), &self.scope);
        // Import-equals narrows lookup without erasing runtime reads.
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if node.import_kind(self.ast) == ImportOrExportKind::Type {
            flags = flags.no_access();
        }
        self.push_reference_flags(flags);
    }

    fn enter_ts_module_reference(
        &mut self,
        _node: TsModuleReference,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.reference_flags.space = ReferenceSpace::Namespace;
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.enter_export_named_declaration(self.ast, node);
    }

    fn exit_export_named_declaration(
        &mut self,
        _node: ExportNamedDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_export_declaration();
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if !self.in_ts_namespace() {
            self.module_record.add_default_export(self.ast, node);
        }
        self.symbols.enter_export_default_declaration();
        let declaration = node.declaration(self.ast);
        let space = declaration
            .as_expr(self.ast)
            .map_or(ReferenceSpace::Value, |expression| {
                export_expression_space(self.ast, expression.node_id())
            });
        self.push_reference_flags(self.flags_with_space(space));
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if !self.in_ts_namespace() {
            self.module_record.add_export_all(self.ast, node);
        }
    }

    fn enter_export_specifier(&mut self, node: ExportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        let declaration = self.parent().and_then(|parent| {
            let NodeKindData::ExportNamedDeclaration(declaration) = self.ast.kind_data(parent)
            else {
                return None;
            };
            Some(declaration)
        });
        if !self.in_ts_namespace()
            && let Some(declaration) = declaration
        {
            self.module_record
                .add_specifier(self.ast, declaration, node);
        }
        let type_only = node.export_kind(self.ast) == ImportOrExportKind::Type
            || declaration.is_some_and(|declaration| {
                declaration.export_kind(self.ast) == ImportOrExportKind::Type
            });
        let mut flags = self.flags_with_space(ReferenceSpace::Any);
        if type_only {
            flags = flags.no_access();
        }
        if declaration.is_some_and(|declaration| declaration.source(self.ast).is_none()) {
            self.symbols
                .add_local_export_reference(self.ast, node, &self.scope, flags, type_only);
        }
    }

    fn enter_ts_enum_member(&mut self, node: TsEnumMember, _ctx: &mut TraverseCtx<'_, 'a>) {
        let prepared = self
            .symbols
            .prepare_ts_enum_member(self.ast, node, &self.scope);
        if let Some(prepared) = prepared {
            if self.check_syntax {
                declarations::check_prepared_redeclaration(self, prepared);
            }
            self.symbols
                .declare_prepared_ts_enum_member(self.ast, prepared);
        }
    }

    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_ts_module_declaration(self.ast, node, &self.scope);
        self.in_ts_module_name = true;
    }

    fn enter_ts_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.enter_ts_global_declaration(self.ast, node);
    }

    fn exit_ts_global_declaration(
        &mut self,
        _node: TsGlobalDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_ts_global_declaration();
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_ts_type_alias_declaration(self.ast, node, &self.scope);
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        _node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_saved_context();
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_ts_interface_declaration(self.ast, node, &self.scope);
    }

    fn exit_ts_interface_declaration(
        &mut self,
        _node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_saved_context();
    }

    fn enter_ts_interface_heritage(
        &mut self,
        node: TsInterfaceHeritage,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
        let expression = node.expression(self.ast).node_id();
        self.reference_flags = self.heritage_reference_flags(expression);
    }

    fn enter_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_query(&mut self, _node: TsTypeQuery, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.reference_flags = self.flags_with_space(ReferenceSpace::Typeof);
    }

    fn enter_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut TraverseCtx<'_, 'a>) {
        // Also reached through TsTypeQueryExprName, without a TsType hook.
        self.enter_type_position();
    }

    fn exit_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_parameter(&mut self, _node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
        let parent = self.parent();
        self.symbols
            .enter_ts_type_parameter(self.ast, parent, &self.scope);
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.scope
            .enter_ts_lookup_view(node.node_id(), ScopeKind::TsTypeParameters);
    }

    fn exit_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.scope.exit_ts_lookup_view(node.node_id());
    }

    fn enter_ts_type_predicate(&mut self, node: TsTypePredicate, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.symbols
            .add_ts_type_predicate_reference(self.ast, node, &self.scope);
    }

    fn enter_ts_method_signature(
        &mut self,
        _node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_method_signature(
        &mut self,
        _node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_index_signature(
        &mut self,
        _node: TsIndexSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_index_signature(&mut self, _node: TsIndexSignature, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_jsx_opening_element(
        &mut self,
        node: JsxOpeningElement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .add_jsx_opening_element_reference(self.ast, node, &self.scope);
    }

    fn enter_jsx_closing_element(
        &mut self,
        node: JsxClosingElement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .add_jsx_closing_element_reference(self.ast, node, &self.scope);
    }

    fn exit_binding_identifier(&mut self, node: BindingIdentifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        let flags = self.reference_flags;
        let prepared = if self.check_syntax {
            js::check_binding_identifier(self, node)
        } else {
            None
        };
        let direct_export = self.symbols.export_state.named();
        let direct_export = direct_export.filter(|_| !self.in_ts_namespace());
        let declaration = if let Some(prepared) = prepared {
            self.symbols
                .declare_prepared_binding_identifier(self.ast, node, &self.scope, prepared)
        } else {
            self.symbols
                .declare_binding_identifier(self.ast, node, &self.scope)
        };
        if declaration.is_some() && flags.is_write_only() {
            self.symbols
                .add_var_write_reference(self.ast, node, &self.scope, flags);
        }
        if declaration.is_some()
            && let Some(statement) = direct_export
        {
            self.module_record
                .add_direct_export(self.ast, node.node_id(), statement);
        }
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_variable_declaration(self.ast, node, &self.scope);
    }

    fn exit_variable_declaration(
        &mut self,
        _node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_saved_context();
    }

    fn enter_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            self.symbols.export_state = Default::default();
            return;
        }
        self.symbols.enter_formal_parameters(&self.scope);
        if self.ast.is_ts() {
            self.scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsSignatureParameter);
        }
    }

    fn exit_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            return;
        }
        if self.ast.is_ts() {
            self.scope.exit_ts_lookup_view(node.node_id());
        }
        self.symbols.exit_formal_parameters();
    }

    fn enter_ts_type_annotation(&mut self, node: TsTypeAnnotation, _ctx: &mut TraverseCtx<'_, 'a>) {
        let signature_return = self.parent().is_some_and(|parent| {
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
            self.scope.enter_ts_lookup_view(node.node_id(), view);
        }
    }

    fn exit_ts_type_annotation(&mut self, node: TsTypeAnnotation, _ctx: &mut TraverseCtx<'_, 'a>) {
        let signature_return = self.parent().is_some_and(|parent| {
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
            self.scope.exit_ts_lookup_view(node.node_id());
        }
    }

    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols
            .enter_ts_enum_declaration(self.ast, node, &self.scope);
    }

    fn exit_ts_enum_declaration(
        &mut self,
        _node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_saved_context();
    }

    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if self.symbols.in_unsupported_ts_external_module() {
            return;
        }
        self.symbols
            .record_ts_global_export(self.ast, node, &self.scope);
    }

    fn exit_ts_class_implements(
        &mut self,
        _node: TsClassImplements,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn exit_ts_interface_heritage(
        &mut self,
        _node: TsInterfaceHeritage,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.record_ts_import_equals_target(self.ast, node);
        self.symbols.exit_saved_context();
        self.pop_reference_flags();
    }

    fn exit_export_default_declaration(
        &mut self,
        _node: ExportDefaultDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.symbols.exit_export_declaration();
        self.pop_reference_flags();
    }

    fn exit_ts_type_parameter(&mut self, _node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.symbols.exit_saved_context();
        self.exit_type_position();
    }
}

fn is_annex_b_if_function(tree: &Ast<'_>, parent: Option<NodeId>, function: Function) -> bool {
    if function.function_type(tree) != FunctionType::FunctionDeclaration {
        return false;
    }
    let Some(parent) = parent else {
        return false;
    };
    let NodeKindData::IfStatement(statement) = tree.kind_data(parent) else {
        return false;
    };
    let function = function.node_id();
    statement.consequent(tree).node_id() == function
        || statement
            .alternate(tree)
            .is_some_and(|alternate| alternate.node_id() == function)
}

fn export_expression_space(tree: &Ast<'_>, expression: NodeId) -> ReferenceSpace {
    let expression = crate::ecmascript::unwrap_parentheses(tree, expression);
    if tree.node_kind(expression) == NodeKind::IdentifierReference {
        ReferenceSpace::Any
    } else {
        ReferenceSpace::Value
    }
}

fn is_direct_eval_call(tree: &Ast<'_>, call: CallExpression) -> bool {
    if call.optional(tree) {
        return false;
    }

    let mut callee = call.callee(tree).node_id();
    loop {
        callee = match tree.kind_data(callee) {
            NodeKindData::IdentifierReference(identifier) => {
                return tree.get_utf8(identifier.name(tree)).as_bytes() == b"eval";
            }
            NodeKindData::ParenthesizedExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsAsExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsSatisfiesExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsTypeAssertion(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsNonNullExpression(wrapper) => wrapper.expression(tree).node_id(),
            NodeKindData::TsInstantiationExpression(wrapper) => wrapper.expression(tree).node_id(),
            _ => return false,
        };
    }
}

/// Extracts the name from a ModuleExportName: an IdentifierName,
/// IdentifierReference, or StringLiteral.
fn module_export_name<'b>(tree: &'b Ast<'_>, node: NodeId) -> &'b [u8] {
    match tree.kind_data(node) {
        NodeKindData::IdentifierName(identifier) => tree.get_utf8(identifier.name(tree)).as_bytes(),
        NodeKindData::IdentifierReference(identifier) => {
            tree.get_utf8(identifier.name(tree)).as_bytes()
        }
        NodeKindData::StringLiteral(literal) => tree.get_wtf8(literal.value(tree)).as_bytes(),
        _ => b"",
    }
}

struct ReportOptions<'a> {
    severity: Severity,
    help: Option<Cow<'a, str>>,
    labels: Vec<Label<'a>>,
}

impl Default for ReportOptions<'_> {
    fn default() -> Self {
        Self {
            severity: Severity::Error,
            help: None,
            labels: Vec::new(),
        }
    }
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    fn report_unresolved_export(&mut self, tree: &Ast<'a>, name: &[u8], node: NodeId) {
        let message = self.unresolved_export_message(name);
        self.report(tree.span(node), message, ReportOptions::default());
    }

    fn report_non_local_export(&mut self, tree: &Ast<'a>, name: &[u8], node: NodeId) {
        let message = self.fmt(format_args!(
            "Cannot export '{}'. Only local declarations can be exported from a module",
            String::from_utf8_lossy(name)
        ));
        self.report(tree.span(node), message, ReportOptions::default());
    }

    fn unresolved_export_message(&self, name: &[u8]) -> String {
        self.fmt(format_args!(
            "Export '{}' is not defined",
            String::from_utf8_lossy(name)
        ))
    }

    #[cold]
    fn report_redeclaration(
        &mut self,
        tree: &Ast<'a>,
        name: &[u8],
        node: NodeId,
        existing_node: NodeId,
        existing_flags: SymbolFlags,
    ) {
        let name = String::from_utf8_lossy(name);
        let current_span = tree.span(node);
        let existing_span = tree.span(existing_node);
        let kind = existing_flags.diagnostic_kind();
        let article = if matches!(kind.as_bytes()[0], b'a' | b'e' | b'i' | b'o' | b'u') {
            "an"
        } else {
            "a"
        };
        let name = name.as_ref();
        let message = ["Identifier '", name, "' has already been declared"].concat();
        let first = [
            "'",
            name,
            "' was first declared as ",
            article,
            " ",
            kind,
            " here",
        ]
        .concat();
        let help = [
            "Consider removing or renaming this declaration of '",
            name,
            "'",
        ]
        .concat();
        self.report(
            current_span,
            message,
            ReportOptions {
                help: Some(Cow::Owned(help)),
                labels: vec![
                    self.label(existing_span, first),
                    self.label(current_span, "cannot be redeclared here"),
                ],
                ..ReportOptions::default()
            },
        );
    }

    #[cold]
    fn report(&mut self, span: Span, message: impl Into<Cow<'a, str>>, options: ReportOptions<'a>) {
        let message = message.into();
        debug_assert!(span.start <= span.end);
        debug_assert!(!message.is_empty());
        self.diagnostics.push(Diagnostic {
            severity: options.severity,
            message,
            span,
            help: options.help,
            labels: options.labels,
        });
    }

    fn label(&self, span: Span, message: impl Into<Cow<'a, str>>) -> Label<'a> {
        Label {
            span,
            message: message.into(),
        }
    }

    fn labels(&self, items: &[Label<'a>]) -> Vec<Label<'a>> {
        items.to_vec()
    }

    fn fmt(&self, arguments: std::fmt::Arguments<'_>) -> String {
        arguments.to_string()
    }
}
