//! The semantic builder, a visitor that builds the semantic model and reports
//! the scope-dependent early errors the specification requires.

mod declarations;
mod js;
mod ts;

use std::{borrow::Cow, collections::hash_map::Entry};

use rustc_hash::FxHashMap;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, Ast, BinaryOperator, BindingIdentifier,
    CallExpression, Class, ClassBody, ClassType, Decorator, Diagnostic, Directive,
    ExportAllDeclaration, ExportDefaultDeclaration, ExportDefaultDeclarationKindData,
    ExportNamedDeclaration, ExportSpecifier, ForInStatement, ForOfStatement, FormalParameters,
    Function, FunctionBody, FunctionType, IdentifierReference, ImportDeclaration,
    ImportOrExportKind, ImportSpecifier, JsxClosingElement, JsxOpeningElement, Label,
    MemberExpression, NodeId, NodeKind, NodeKindData, PropertyKeyData, ScopeId, Severity, Span,
    SwitchStatement, Traverse, TraverseCtx, TsArrayType, TsCallSignatureDeclaration,
    TsClassImplements, TsConditionalType, TsConstructSignatureDeclaration, TsConstructorType,
    TsEnumDeclaration, TsEnumMember, TsExportAssignment, TsFunctionType, TsGlobalDeclaration,
    TsImportEqualsDeclaration, TsImportType, TsIndexSignature, TsIndexedAccessType, TsInferType,
    TsInterfaceBody, TsInterfaceDeclaration, TsInterfaceHeritage, TsIntersectionType,
    TsJsDocNonNullableType, TsJsDocNullableType, TsJsDocUnknownType, TsLiteralType, TsMappedType,
    TsMethodSignature, TsModuleBlock, TsModuleDeclaration, TsModuleNameData, TsNamedTupleMember,
    TsNamespaceExportDeclaration, TsOptionalType, TsParenthesizedType, TsPropertySignature,
    TsRestType, TsTemplateLiteralType, TsTupleType, TsTypeAliasDeclaration, TsTypeAnnotation,
    TsTypeLiteral, TsTypeOperator, TsTypeParameter, TsTypeParameterDeclaration,
    TsTypeParameterInstantiation, TsTypePredicate, TsTypeQuery, TsTypeReference, TsUnionType,
    UnaryExpression, UnaryOperator, UpdateExpression, VariableDeclaration, VariableKind,
    WithStatement,
};

use crate::{
    ReferenceFlags, ReferenceSpace, Semantic, SymbolFlags,
    binder::{ExportState, PreparedDeclaration},
    context::SemanticContext,
    ecmascript,
    scope::{DecoratorSite, ScopeKind},
};

pub struct SemanticBuilder<'traverse, 'a> {
    exported_names: FxHashMap<&'traverse [u8], NodeId>,
    export_specifiers: Vec<ExportSpecifier>,
    diagnostics: Vec<Diagnostic<'a>>,
    state: SemanticContext<'a>,
    ast: &'traverse Ast<'a>,
    path: Vec<NodeId>,
    node_flags: Vec<ReferenceFlags>,
    type_positions: Vec<ReferenceFlags>,
    arrow_exports: Vec<ExportState>,
    dynamic_var_writes: Vec<bool>,
    check_syntax: bool,
    in_ts_module_name: bool,
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    pub fn new(ast: &'traverse Ast<'a>, check_syntax: bool, build_module_record: bool) -> Self {
        Self {
            exported_names: FxHashMap::default(),
            export_specifiers: Vec::new(),
            diagnostics: Vec::new(),
            state: SemanticContext::init(ast, build_module_record),
            ast,
            path: Vec::with_capacity(64),
            node_flags: Vec::with_capacity(64),
            type_positions: Vec::new(),
            arrow_exports: Vec::new(),
            dynamic_var_writes: Vec::new(),
            check_syntax,
            in_ts_module_name: false,
        }
    }

    pub fn finish(mut self) -> (Semantic<'a>, Vec<Diagnostic<'a>>) {
        let check_syntax = self.check_syntax;
        let (export_alias_issues, import_alias_issues) = {
            let state = &mut self.state;
            state
                .symbols
                .resolve_ts_aliases(self.ast, &state.scope, check_syntax)
        };
        if check_syntax {
            ts::check_ts_alias_issues(&mut self, export_alias_issues, import_alias_issues);
        }
        debug_assert!(self.path.is_empty());
        debug_assert!(self.node_flags.is_empty());
        debug_assert!(self.type_positions.is_empty());
        debug_assert!(self.arrow_exports.is_empty());
        debug_assert!(self.dynamic_var_writes.is_empty());
        let semantic = self.state.finalize(self.ast);
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
        self.state.symbols.record_ts_expando_assignment(
            self.ast,
            node,
            &self.state.scope,
            self.path.iter().rev().copied(),
        );
    }

    #[inline]
    fn record_identifier_reference(&mut self, node: IdentifierReference, flags: ReferenceFlags) {
        let state = &mut self.state;
        state
            .symbols
            .add_identifier_reference(self.ast, node, &state.scope, flags);
    }

    fn enter_type_position(&mut self) {
        self.type_positions.push(self.state.enter_type_position());
        self.state
            .replace_reference_flags(self.state.default_reference_flags());
    }

    fn exit_type_position(&mut self) {
        let previous = self
            .type_positions
            .pop()
            .expect("a type position was entered");
        self.state.exit_type_position(previous);
    }

    #[inline]
    fn assignment_reference_flags(&self, read: bool) -> ReferenceFlags {
        let flags = self.state.default_reference_flags();
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
        let flags = self.state.default_reference_flags().with_space(space);
        if space.in_type_position() {
            flags.no_access()
        } else {
            flags
        }
    }

    /// Reference context belongs to a parent/child edge. It is consumed once
    /// on entry; siblings start again with the enclosing default context.
    fn child_reference_flags(&self, child: NodeId) -> ReferenceFlags {
        let Some(parent) = self.current_node() else {
            return self.state.default_reference_flags();
        };
        let flags = *self.node_flags.last().expect("a parent has a node frame");
        let tree = self.ast;
        match tree.kind_data(parent) {
            NodeKindData::AssignmentExpression(node) if node.left(tree).node_id() == child => {
                self.assignment_reference_flags(node.operator(tree) != AssignmentOperator::Assign)
            }
            NodeKindData::UpdateExpression(_) => self.assignment_reference_flags(true),
            NodeKindData::UnaryExpression(node) if node.operator(tree) == UnaryOperator::Delete => {
                self.state.default_reference_flags().no_access()
            }
            NodeKindData::MemberExpression(node) if node.object(tree).node_id() == child => {
                self.flags_with_space(flags.space)
            }
            NodeKindData::ParenthesizedExpression(_) | NodeKindData::TsNonNullExpression(_) => {
                flags
            }
            NodeKindData::TsAsExpression(node) if node.expression(tree).node_id() == child => flags,
            NodeKindData::TsSatisfiesExpression(node)
                if node.expression(tree).node_id() == child =>
            {
                flags
            }
            NodeKindData::TsTypeAssertion(node) if node.expression(tree).node_id() == child => {
                flags
            }
            NodeKindData::AssignmentPattern(node) if node.left(tree).node_id() == child => flags,
            NodeKindData::BindingRestElement(node) if node.argument(tree).node_id() == child => {
                flags
            }
            NodeKindData::ArrayPattern(_) | NodeKindData::ObjectPattern(_)
                if !matches!(
                    tree.node_kind(child),
                    NodeKind::Decorator | NodeKind::TsTypeAnnotation
                ) =>
            {
                flags
            }
            NodeKindData::BindingProperty(node) if node.value(tree).node_id() == child => flags,
            NodeKindData::ForInStatement(node) if node.left(tree).node_id() == child => {
                self.assignment_reference_flags(false)
            }
            NodeKindData::ForOfStatement(node) if node.left(tree).node_id() == child => {
                self.assignment_reference_flags(false)
            }
            NodeKindData::VariableDeclarator(node) if node.id(tree).node_id() == child => {
                let declaration_flags = self
                    .node_flags
                    .iter()
                    .rev()
                    .nth(1)
                    .copied()
                    .unwrap_or_default();
                if self.dynamic_var_writes.last() == Some(&true)
                    && (node.init(tree).is_some() || declaration_flags.is_write())
                {
                    self.assignment_reference_flags(false)
                } else {
                    self.state.default_reference_flags()
                }
            }
            NodeKindData::TsQualifiedName(node)
                if !self.in_ts_module_name && node.left(tree).node_id() == child =>
            {
                let space = match flags.space {
                    ReferenceSpace::Value | ReferenceSpace::Type | ReferenceSpace::Namespace => {
                        ReferenceSpace::Namespace
                    }
                    space => space,
                };
                flags.with_space(space)
            }
            NodeKindData::TsTypeQuery(node) if node.expr_name(tree).node_id() == child => {
                self.flags_with_space(ReferenceSpace::Typeof)
            }
            NodeKindData::TsClassImplements(node) if node.expression(tree).node_id() == child => {
                self.heritage_reference_flags(child)
            }
            NodeKindData::TsInterfaceHeritage(node) if node.expression(tree).node_id() == child => {
                self.heritage_reference_flags(child)
            }
            NodeKindData::PropertyDefinition(node)
                if node.key(tree).node_id() == child
                    && (node.declare(tree) || node.r#abstract(tree))
                    && node.decorators(tree).is_empty()
                    && node.computed(tree) =>
            {
                self.flags_with_space(ReferenceSpace::Typeof)
            }
            NodeKindData::TsMethodDefinition(node)
                if node.key(tree).node_id() == child
                    && node.decorators(tree).is_empty()
                    && node.computed(tree) =>
            {
                self.flags_with_space(ReferenceSpace::Typeof)
            }
            NodeKindData::TsPropertySignature(node)
                if node.key(tree).node_id() == child && node.computed(tree) =>
            {
                self.flags_with_space(ReferenceSpace::Typeof)
            }
            NodeKindData::TsImportEqualsDeclaration(node) => {
                let space = if node.id(tree).node_id() == child {
                    ReferenceSpace::Any
                } else {
                    ReferenceSpace::Namespace
                };
                // Import-equals narrows lookup without erasing runtime reads.
                let flags = self.flags_with_space(ReferenceSpace::Any).with_space(space);
                if node.import_kind(tree) == ImportOrExportKind::Type {
                    flags.no_access()
                } else {
                    flags
                }
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                let space = node
                    .declaration(tree)
                    .as_expr(tree)
                    .map_or(ReferenceSpace::Value, |expression| {
                        export_expression_space(tree, expression.node_id())
                    });
                self.flags_with_space(space)
            }
            NodeKindData::TsExportAssignment(node) => self.flags_with_space(
                export_expression_space(tree, node.expression(tree).node_id()),
            ),
            NodeKindData::ExportSpecifier(node) if node.local(tree).node_id() == child => flags,
            _ => self.state.default_reference_flags(),
        }
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

/// First Traverse migration. Pending hooks retain their original Visit bodies.
impl<'a> Traverse<'a> for SemanticBuilder<'_, 'a> {
    fn enter_node(&mut self, node: NodeId, _ctx: &mut TraverseCtx<'_, 'a>) {
        let flags = self.child_reference_flags(node);
        self.state.replace_reference_flags(flags);
        self.path.push(node);
        self.node_flags.push(self.state.enter_node());
    }

    fn exit_node(&mut self, node: NodeId, _ctx: &mut TraverseCtx<'_, 'a>) {
        if self.check_syntax && self.ast.node_kind(node) != NodeKind::Function {
            self.check(self.ast.kind_data(node));
        }
        let exited = self.path.pop();
        debug_assert_eq!(exited, Some(node));
        self.node_flags.pop();
    }

    fn enter_scope(&mut self, node: NodeId, _ctx: &mut TraverseCtx<'_, 'a>) {
        match self.ast.kind_data(node) {
            NodeKindData::BlockStatement(node) => {
                self.state.enter_block_statement(self.ast, node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                if !node.cases(self.ast).is_empty() {
                    self.state.enter_switch_case(Some(node.node_id()));
                }
            }
            NodeKindData::ForStatement(node) => {
                self.state.enter_for_statement(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                self.state.enter_catch_clause(node.node_id());
                let state = &mut self.state;
                state.symbols.enter_catch_clause(&state.scope);
            }
            NodeKindData::WithStatement(node) => {
                self.state.enter_with_statement(node.node_id());
            }
            NodeKindData::Function(node) => {
                // TODO(traverse): Annex B still uses the function strict flag; preserve this known issue for a separate fix.
                // Original Visit body:
                // let annex_b_if_body = is_annex_b_if_function(self.ast, self.current_node(), node);
                // self.state.enter_function(self.ast, node, annex_b_if_body);
                // self.enter_node(node.node_id());
                // let state = &mut self.state;
                // state.symbols.enter_function(self.ast, node, &state.scope);
                // node.visit_children_with(self);
                // self.state.symbols.exit_saved_context();
                // self.leave_node(NodeKindData::Function(node));
                // self.state.exit_scope(node.node_id());
                let annex_b_if_body = is_annex_b_if_function(self.ast, self.parent(), node);
                self.state.enter_function(self.ast, node, annex_b_if_body);
                let state = &mut self.state;
                state.symbols.enter_function(self.ast, node, &state.scope);
            }
            NodeKindData::TsFunction(node) => {
                self.state.enter_ts_function(node.node_id());
                let state = &mut self.state;
                state
                    .symbols
                    .enter_ts_function(self.ast, node, &state.scope);
            }
            NodeKindData::StaticBlock(node) => {
                self.state.enter_static_block(node.node_id());
            }
            NodeKindData::TsFunctionType(node) => {
                self.state.enter_ts_function_type(node.node_id());
                let state = &mut self.state;
                state.symbols.enter_ts_function_type(&state.scope);
            }
            NodeKindData::TsConstructorType(node) => {
                self.state.enter_ts_constructor_type(node.node_id());
                let state = &mut self.state;
                state.symbols.enter_ts_constructor_type(&state.scope);
            }
            NodeKindData::TsMappedType(node) => {
                self.state.enter_ts_mapped_type(node.node_id());
                let parent = self.parent();
                let state = &mut self.state;
                state
                    .symbols
                    .enter_ts_mapped_type(self.ast, parent, &state.scope);
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                self.state
                    .enter_ts_call_signature_declaration(node.node_id());
                let state = &mut self.state;
                state
                    .symbols
                    .enter_ts_call_signature_declaration(&state.scope);
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                self.state
                    .enter_ts_construct_signature_declaration(node.node_id());
                let state = &mut self.state;
                state
                    .symbols
                    .enter_ts_construct_signature_declaration(&state.scope);
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                self.state.enter_arrow_function_expression(self.ast, node);
                self.arrow_exports
                    .push(self.state.symbols.enter_arrow_function_expression());
            }
            NodeKindData::TsEnumBody(node) => {
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    self.parent().map(|parent| self.ast.kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = self
                    .state
                    .symbols
                    .symbol_of_node(declaration.id(self.ast).node_id())
                    .expect("an enum declaration has a symbol");
                self.state.symbols.enter_ts_enum_members(owner);
                self.state.enter_ts_enum_body(node.node_id());
                let state = &mut self.state;
                state.symbols.enter_ts_enum_body(&state.scope);
            }
            NodeKindData::ForInStatement(node) => self.state.enter_for_in_statement(node.node_id()),
            NodeKindData::ForOfStatement(node) => self.state.enter_for_of_statement(node.node_id()),
            NodeKindData::TsModuleDeclaration(node) => {
                self.in_ts_module_name = false;
                let declarations = self.state.symbols.enter_ts_module_body(self.ast, node);
                if self.check_syntax {
                    for prepared in declarations {
                        declarations::check_prepared_redeclaration(self, prepared);
                    }
                }
            }
            // The root scope is initialized by SemanticContext::init. Class,
            // type alias, interface, and global scopes still use node hooks below.
            _ => {}
        }
    }

    fn leave_scope(&mut self, node: NodeId, _ctx: &mut TraverseCtx<'_, 'a>) {
        match self.ast.kind_data(node) {
            NodeKindData::BlockStatement(node) => {
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::ForStatement(node) => {
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::WithStatement(node) => {
                self.state.exit_with_statement(node.node_id());
            }
            NodeKindData::Function(node) => {
                self.state.symbols.exit_saved_context();
                if self.check_syntax {
                    self.check(NodeKindData::Function(node));
                }
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsFunction(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::StaticBlock(node) => {
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsFunctionType(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructorType(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsMappedType(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsCallSignatureDeclaration(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsConstructSignatureDeclaration(node) => {
                self.state.symbols.exit_saved_context();
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                let export_state = self
                    .arrow_exports
                    .pop()
                    .expect("an arrow saved its export state");
                self.state
                    .symbols
                    .exit_arrow_function_expression(export_state);
                self.state.exit_scope(node.node_id());
            }
            NodeKindData::TsEnumBody(node) => {
                self.state.exit_scope(node.node_id());
                let Some(NodeKindData::TsEnumDeclaration(declaration)) =
                    self.parent().map(|parent| self.ast.kind_data(parent))
                else {
                    unreachable!("an enum body belongs to an enum declaration");
                };
                let owner = self
                    .state
                    .symbols
                    .symbol_of_node(declaration.id(self.ast).node_id())
                    .expect("an enum declaration has a symbol");
                self.state.symbols.exit_ts_enum_members(owner);
            }
            NodeKindData::ForInStatement(node) => self.state.exit_scope(node.node_id()),
            NodeKindData::ForOfStatement(node) => self.state.exit_scope(node.node_id()),
            NodeKindData::TsModuleDeclaration(_) => {
                self.state.symbols.exit_ts_module_body();
                self.state.symbols.exit_saved_context();
            }
            _ => {}
        }
    }

    fn enter_call_expression(&mut self, node: CallExpression, _ctx: &mut TraverseCtx<'_, 'a>) {
        if !self.state.symbols.in_ambient_context()
            && self.state.default_reference_flags().is_read()
            && is_direct_eval_call(self.ast, node)
        {
            self.state.scope.mark_direct_eval();
        }
    }

    fn enter_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.record_ts_expando_assignment(node);
    }

    fn enter_identifier_reference(
        &mut self,
        node: IdentifierReference,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let flags = *self
            .node_flags
            .last()
            .expect("an identifier has a node frame");
        self.record_identifier_reference(node, flags);
    }

    fn enter_directive(&mut self, node: Directive, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.enter_directive(self.ast, node);
    }

    fn enter_function_body(&mut self, node: FunctionBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TODO(traverse): Traverse has no FunctionBody scope annotation; preserve the conditional parameter/body environment.
        // Original Visit body:
        // let parent = self.current_node();
        // self.state.enter_function_body(self.ast, node, parent);
        // self.enter_node(node.node_id());
        // node.visit_children_with(self);
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::FunctionBody(node));
        let parent = self.parent();
        self.state.enter_function_body(self.ast, node, parent);
    }

    fn exit_function_body(&mut self, node: FunctionBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.exit_scope(node.node_id());
    }

    fn enter_class(&mut self, node: Class, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TODO(traverse): Class scope starts after id in Traverse; keep node hooks until binding and decorator contexts are aligned.
        // Original Visit body:
        // self.state.enter_class(self.ast, node);
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state.symbols.enter_class(self.ast, node, &state.scope);
        // node.visit_children_with(self);
        // self.state.symbols.exit_saved_context();
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::Class(node));
        self.state.enter_class(self.ast, node);
        let state = &mut self.state;
        state.symbols.enter_class(self.ast, node, &state.scope);
    }

    fn exit_class(&mut self, node: Class, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
    }

    fn enter_class_body(&mut self, _node: ClassBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.enter_class_body();
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
        self.state.enter_decorator(node.node_id(), site);
    }

    fn exit_decorator(&mut self, _node: Decorator, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.exit_decorator();
    }

    fn enter_ts_class_implements(
        &mut self,
        _node: TsClassImplements,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn enter_import_declaration(
        &mut self,
        node: ImportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        state
            .symbols
            .enter_import_declaration(self.ast, node, &state.scope);
    }

    fn exit_import_declaration(
        &mut self,
        _node: ImportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_saved_context();
    }

    fn enter_import_specifier(&mut self, node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.enter_import_specifier(self.ast, node);
    }

    fn exit_import_specifier(&mut self, _node: ImportSpecifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.exit_saved_context();
    }

    fn enter_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_import_equals_declaration(node.import_kind(self.ast), &state.scope);
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state
            .symbols
            .enter_export_named_declaration(self.ast, node);
    }

    fn exit_export_named_declaration(
        &mut self,
        _node: ExportNamedDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_export_declaration();
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if !self.state.in_ts_namespace() {
            self.state.module_record.add_default_export(self.ast, node);
        }
        self.state.symbols.enter_export_default_declaration();
    }

    fn enter_export_all_declaration(
        &mut self,
        node: ExportAllDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if !self.state.in_ts_namespace() {
            self.state.module_record.add_export_all(self.ast, node);
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
        if !self.state.in_ts_namespace()
            && let Some(declaration) = declaration
        {
            self.state
                .module_record
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
            let state = &mut self.state;
            state.symbols.add_local_export_reference(
                self.ast,
                node,
                &state.scope,
                flags,
                type_only,
            );
        }
        *self
            .node_flags
            .last_mut()
            .expect("a specifier has a node frame") = flags;
    }

    fn enter_ts_enum_member(&mut self, node: TsEnumMember, _ctx: &mut TraverseCtx<'_, 'a>) {
        let state = &mut self.state;
        let prepared = state
            .symbols
            .prepare_ts_enum_member(self.ast, node, &state.scope);
        if let Some(prepared) = prepared {
            if self.check_syntax {
                declarations::check_prepared_redeclaration(self, prepared);
            }
            self.state
                .symbols
                .declare_prepared_ts_enum_member(self.ast, prepared);
        }
    }

    fn enter_ts_module_declaration(
        &mut self,
        node: TsModuleDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // TODO(traverse): The module body still owns its lexical scope; align owner identities separately.
        // Original Visit body:
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state
        //     .symbols
        //     .enter_ts_module_declaration(self.ast, node, &state.scope);
        // let name = node.id(self.ast);
        // let was_in_ts_module_name = std::mem::replace(&mut self.in_ts_module_name, true);
        // name.visit_with(self);
        // self.in_ts_module_name = was_in_ts_module_name;
        // let declarations = self.state.symbols.enter_ts_module_body(self.ast, node);
        // if self.check_syntax {
        //     for prepared in declarations {
        //         declarations::check_prepared_redeclaration(self, prepared);
        //     }
        // }
        // node.body(self.ast).visit_with(self);
        // self.state.symbols.exit_ts_module_body();
        // self.state.symbols.exit_saved_context();
        // self.leave_node(NodeKindData::TsModuleDeclaration(node));
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_module_declaration(self.ast, node, &state.scope);
        self.in_ts_module_name = true;
    }

    fn enter_ts_module_block(&mut self, node: TsModuleBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TODO(traverse): Namespace owner remains the body node until declaration/body scope ownership is aligned.
        // Original Visit body:
        // self.state.enter_ts_module_block(node.node_id());
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state.symbols.enter_ts_module_block(&state.scope);
        // node.visit_children_with(self);
        // self.state.exit_ts_module_block(node.node_id());
        // self.leave_node(NodeKindData::TsModuleBlock(node));
        self.state.enter_ts_module_block(node.node_id());
        let state = &mut self.state;
        state.symbols.enter_ts_module_block(&state.scope);
    }

    fn exit_ts_module_block(&mut self, node: TsModuleBlock, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.exit_ts_module_block(node.node_id());
    }

    fn enter_ts_global_declaration(
        &mut self,
        node: TsGlobalDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state
            .symbols
            .enter_ts_global_declaration(self.ast, node);
    }

    fn exit_ts_global_declaration(
        &mut self,
        _node: TsGlobalDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_ts_global_declaration();
    }

    fn enter_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // TODO(traverse): The binder still derives the declaration target from the inner scope parent.
        // Original Visit body:
        // self.state.enter_ts_type_alias_declaration(node.node_id());
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state
        //     .symbols
        //     .enter_ts_type_alias_declaration(self.ast, node, &state.scope);
        // node.visit_children_with(self);
        // self.state.symbols.exit_saved_context();
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::TsTypeAliasDeclaration(node));
        self.state.enter_ts_type_alias_declaration(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_type_alias_declaration(self.ast, node, &state.scope);
    }

    fn exit_ts_type_alias_declaration(
        &mut self,
        node: TsTypeAliasDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
    }

    fn enter_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // TODO(traverse): The binder still derives the declaration target from the inner scope parent.
        // Original Visit body:
        // self.state.enter_ts_interface_declaration(node.node_id());
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state
        //     .symbols
        //     .enter_ts_interface_declaration(self.ast, node, &state.scope);
        // node.visit_children_with(self);
        // self.state.symbols.exit_saved_context();
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::TsInterfaceDeclaration(node));
        self.state.enter_ts_interface_declaration(node.node_id());
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_interface_declaration(self.ast, node, &state.scope);
    }

    fn exit_ts_interface_declaration(
        &mut self,
        node: TsInterfaceDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
    }

    fn enter_ts_interface_heritage(
        &mut self,
        _node: TsInterfaceHeritage,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn enter_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_interface_body(&mut self, _node: TsInterfaceBody, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_reference(&mut self, _node: TsTypeReference, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_type_reference(&mut self, _node: TsTypeReference, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_query(&mut self, _node: TsTypeQuery, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn enter_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_import_type(&mut self, _node: TsImportType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_parameter(&mut self, _node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TODO(traverse): Infer constraints need a lookup view after the name is bound; ordinary type parameters are migrated below.
        // Original Visit body:
        // let type_position = self.state.enter_type_position();
        // self.enter_node(node.node_id());
        // let parent = self.parent();
        // let state = &mut self.state;
        // state
        //     .symbols
        //     .enter_ts_type_parameter(self.ast, parent, &state.scope);
        // node.name(self.ast).visit_with(self);
        // let infer =
        //     parent.is_some_and(|parent| self.ast.node_kind(parent) == NodeKind::TsInferType);
        // if infer {
        //     let state = &mut self.state;
        //     state
        //         .scope
        //         .enter_ts_lookup_view(node.node_id(), ScopeKind::TsInferConstraint);
        //     state
        //         .symbols
        //         .expose_ts_infer_parameter(self.ast, node.name(self.ast), &state.scope);
        // }
        // node.constraint(self.ast).visit_with(self);
        // node.default(self.ast).visit_with(self);
        // if infer {
        //     self.state.scope.exit_ts_lookup_view(node.node_id());
        // }
        // self.state.symbols.exit_saved_context();
        // self.leave_node(NodeKindData::TsTypeParameter(node));
        // self.state.exit_type_position(type_position);
        self.enter_type_position();
        let parent = self.parent();
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_type_parameter(self.ast, parent, &state.scope);
    }

    fn enter_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
        self.state
            .scope
            .enter_ts_lookup_view(node.node_id(), ScopeKind::TsTypeParameters);
    }

    fn exit_ts_type_parameter_declaration(
        &mut self,
        node: TsTypeParameterDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.scope.exit_ts_lookup_view(node.node_id());
        self.exit_type_position();
    }

    fn enter_ts_type_parameter_instantiation(
        &mut self,
        _node: TsTypeParameterInstantiation,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_type_parameter_instantiation(
        &mut self,
        _node: TsTypeParameterInstantiation,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_literal_type(&mut self, _node: TsLiteralType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_literal_type(&mut self, _node: TsLiteralType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_template_literal_type(
        &mut self,
        _node: TsTemplateLiteralType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_template_literal_type(
        &mut self,
        _node: TsTemplateLiteralType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_array_type(&mut self, _node: TsArrayType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_array_type(&mut self, _node: TsArrayType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_indexed_access_type(
        &mut self,
        _node: TsIndexedAccessType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_indexed_access_type(
        &mut self,
        _node: TsIndexedAccessType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_tuple_type(&mut self, _node: TsTupleType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_tuple_type(&mut self, _node: TsTupleType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_named_tuple_member(
        &mut self,
        _node: TsNamedTupleMember,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_named_tuple_member(
        &mut self,
        _node: TsNamedTupleMember,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_optional_type(&mut self, _node: TsOptionalType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_optional_type(&mut self, _node: TsOptionalType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_rest_type(&mut self, _node: TsRestType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_rest_type(&mut self, _node: TsRestType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_js_doc_nullable_type(
        &mut self,
        _node: TsJsDocNullableType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_js_doc_nullable_type(
        &mut self,
        _node: TsJsDocNullableType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_js_doc_non_nullable_type(
        &mut self,
        _node: TsJsDocNonNullableType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_js_doc_non_nullable_type(
        &mut self,
        _node: TsJsDocNonNullableType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_js_doc_unknown_type(
        &mut self,
        _node: TsJsDocUnknownType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_js_doc_unknown_type(
        &mut self,
        _node: TsJsDocUnknownType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_union_type(&mut self, _node: TsUnionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_union_type(&mut self, _node: TsUnionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_intersection_type(
        &mut self,
        _node: TsIntersectionType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_intersection_type(
        &mut self,
        _node: TsIntersectionType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_infer_type(&mut self, _node: TsInferType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_infer_type(&mut self, _node: TsInferType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_type_operator(&mut self, _node: TsTypeOperator, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_type_operator(&mut self, _node: TsTypeOperator, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_parenthesized_type(
        &mut self,
        _node: TsParenthesizedType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_parenthesized_type(
        &mut self,
        _node: TsParenthesizedType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_type_literal(&mut self, _node: TsTypeLiteral, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_type_literal(&mut self, _node: TsTypeLiteral, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_property_signature(
        &mut self,
        _node: TsPropertySignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn enter_ts_type_predicate(&mut self, node: TsTypePredicate, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
        let state = &mut self.state;
        state
            .symbols
            .add_ts_type_predicate_reference(self.ast, node, &state.scope);
    }

    fn exit_ts_type_predicate(&mut self, _node: TsTypePredicate, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_conditional_type(
        &mut self,
        _node: TsConditionalType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // TODO(traverse): The scope hook covers extends/true, but the extends-only lookup view still needs field boundaries.
        // Original Visit body:
        // let type_position = self.state.enter_type_position();
        // self.enter_node(node.node_id());
        // node.check_type(self.ast).visit_with(self);
        // self.state.enter_ts_conditional_type(node.node_id());
        // let extends = node.extends_type(self.ast);
        // self.state
        //     .scope
        //     .enter_ts_lookup_view(extends.node_id(), ScopeKind::TsConditionalExtends);
        // extends.visit_with(self);
        // self.state.scope.exit_ts_lookup_view(extends.node_id());
        // node.true_type(self.ast).visit_with(self);
        // self.state.exit_scope(node.node_id());
        // node.false_type(self.ast).visit_with(self);
        // self.leave_node(NodeKindData::TsConditionalType(node));
        // self.state.exit_type_position(type_position);
    }

    fn enter_ts_function_type(&mut self, _node: TsFunctionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_function_type(&mut self, _node: TsFunctionType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_constructor_type(
        &mut self,
        _node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_constructor_type(
        &mut self,
        _node: TsConstructorType,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_mapped_type(&mut self, _node: TsMappedType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.enter_type_position();
    }

    fn exit_ts_mapped_type(&mut self, _node: TsMappedType, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn enter_ts_method_signature(
        &mut self,
        _node: TsMethodSignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        // TODO(traverse): The scope hook currently includes the computed key, which must resolve outside signature locals.
        // Original Visit body:
        // let type_position = self.state.enter_type_position();
        // self.enter_node(node.node_id());
        // self.visit_type_member_key(node.key(self.ast), node.computed(self.ast));
        // self.state.enter_ts_method_signature(node.node_id());
        // let state = &mut self.state;
        // state.symbols.enter_ts_method_signature(&state.scope);
        // node.type_parameters(self.ast).visit_with(self);
        // node.params(self.ast).visit_with(self);
        // node.return_type(self.ast).visit_with(self);
        // self.state.symbols.exit_saved_context();
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::TsMethodSignature(node));
        // self.state.exit_type_position(type_position);
    }

    fn enter_ts_call_signature_declaration(
        &mut self,
        _node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_call_signature_declaration(
        &mut self,
        _node: TsCallSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_construct_signature_declaration(
        &mut self,
        _node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.enter_type_position();
    }

    fn exit_ts_construct_signature_declaration(
        &mut self,
        _node: TsConstructSignatureDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn enter_ts_index_signature(&mut self, node: TsIndexSignature, _ctx: &mut TraverseCtx<'_, 'a>) {
        // TODO(traverse): Traverse has no TsIndexSignature scope annotation yet.
        // Original Visit body:
        // self.state.enter_ts_index_signature(node.node_id());
        // let type_position = self.state.enter_type_position();
        // self.enter_node(node.node_id());
        // let state = &mut self.state;
        // state.symbols.enter_ts_index_signature(&state.scope);
        // node.visit_children_with(self);
        // self.state.symbols.exit_saved_context();
        // self.state.exit_scope(node.node_id());
        // self.leave_node(NodeKindData::TsIndexSignature(node));
        // self.state.exit_type_position(type_position);
        self.state.enter_ts_index_signature(node.node_id());
        self.enter_type_position();
        let state = &mut self.state;
        state.symbols.enter_ts_index_signature(&state.scope);
    }

    fn exit_ts_index_signature(&mut self, node: TsIndexSignature, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.exit_saved_context();
        self.state.exit_scope(node.node_id());
        self.exit_type_position();
    }

    fn enter_jsx_opening_element(
        &mut self,
        node: JsxOpeningElement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        state
            .symbols
            .add_jsx_opening_element_reference(self.ast, node, &state.scope);
    }

    fn enter_jsx_closing_element(
        &mut self,
        node: JsxClosingElement,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        state
            .symbols
            .add_jsx_closing_element_reference(self.ast, node, &state.scope);
    }

    fn exit_binding_identifier(&mut self, node: BindingIdentifier, _ctx: &mut TraverseCtx<'_, 'a>) {
        let flags = *self.node_flags.last().expect("a binding has a node frame");
        let prepared = if self.check_syntax {
            js::check_binding_identifier(self, node)
        } else {
            None
        };
        let direct_export = self.state.symbols.export_state.named();
        let direct_export = direct_export.filter(|_| !self.state.in_ts_namespace());
        let state = &mut self.state;
        let declaration = if let Some(prepared) = prepared {
            state.symbols.declare_prepared_binding_identifier(
                self.ast,
                node,
                &state.scope,
                prepared,
            )
        } else {
            state
                .symbols
                .declare_binding_identifier(self.ast, node, &state.scope)
        };
        if declaration.is_some() && flags.is_write_only() {
            state
                .symbols
                .add_var_write_reference(self.ast, node, &state.scope, flags);
        }
        if declaration.is_some()
            && let Some(statement) = direct_export
        {
            state
                .module_record
                .add_direct_export(self.ast, node.node_id(), statement);
        }
    }

    fn enter_variable_declaration(
        &mut self,
        node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        let dynamic_var_write =
            state
                .symbols
                .enter_variable_declaration(self.ast, node, &state.scope);
        self.dynamic_var_writes.push(dynamic_var_write);
    }

    fn exit_variable_declaration(
        &mut self,
        _node: VariableDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.dynamic_var_writes
            .pop()
            .expect("a declaration saved its write context");
        self.state.symbols.exit_saved_context();
    }

    fn enter_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            self.state.symbols.export_state = Default::default();
            return;
        }
        let state = &mut self.state;
        state.symbols.enter_formal_parameters(&state.scope);
        if self.ast.is_ts() {
            state
                .scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsSignatureParameter);
        }
    }

    fn exit_formal_parameters(&mut self, node: FormalParameters, _ctx: &mut TraverseCtx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            return;
        }
        if self.ast.is_ts() {
            self.state.scope.exit_ts_lookup_view(node.node_id());
        }
        self.state.symbols.exit_formal_parameters();
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
            self.state.scope.enter_ts_lookup_view(node.node_id(), view);
        }
        self.enter_type_position();
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
            self.state.scope.exit_ts_lookup_view(node.node_id());
        }
        self.exit_type_position();
    }

    fn enter_ts_enum_declaration(
        &mut self,
        node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        let state = &mut self.state;
        state
            .symbols
            .enter_ts_enum_declaration(self.ast, node, &state.scope);
    }

    fn exit_ts_enum_declaration(
        &mut self,
        _node: TsEnumDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_saved_context();
    }

    fn enter_ts_namespace_export_declaration(
        &mut self,
        node: TsNamespaceExportDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        if self.state.symbols.in_unsupported_ts_external_module() {
            return;
        }
        let state = &mut self.state;
        state
            .symbols
            .record_ts_global_export(self.ast, node, &state.scope);
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

    fn exit_ts_type_query(&mut self, _node: TsTypeQuery, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.exit_type_position();
    }

    fn exit_ts_property_signature(
        &mut self,
        _node: TsPropertySignature,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.exit_type_position();
    }

    fn exit_ts_import_equals_declaration(
        &mut self,
        node: TsImportEqualsDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state
            .symbols
            .record_ts_import_equals_target(self.ast, node);
        self.state.symbols.exit_saved_context();
    }

    fn exit_export_default_declaration(
        &mut self,
        _node: ExportDefaultDeclaration,
        _ctx: &mut TraverseCtx<'_, 'a>,
    ) {
        self.state.symbols.exit_export_declaration();
    }

    fn exit_ts_type_parameter(&mut self, _node: TsTypeParameter, _ctx: &mut TraverseCtx<'_, 'a>) {
        self.state.symbols.exit_saved_context();
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
