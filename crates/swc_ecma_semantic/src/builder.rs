//! The semantic builder, a visitor that builds the semantic model and reports
//! the scope-dependent early errors the specification requires.

use std::borrow::Cow;

use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    AssignmentPattern, Ast, AwaitExpression, BinaryExpression, BindingIdentifier, BindingPattern,
    CallExpression, ChainExpression, Class, ClassBody, ClassType, ConditionalExpression, Decorator,
    Diagnostic, Directive, ExportAllDeclaration, ExportDefaultDeclaration, ExportNamedDeclaration,
    ExportSpecifier, ForStatementLeft, FormalParameters, Function, FunctionType,
    IdentifierReference, ImportDeclaration, ImportExpression, ImportOrExportKind, ImportSpecifier,
    JsxClosingElement, JsxElement, JsxFragment, JsxOpeningElement, Label, LogicalExpression,
    MemberExpression, NewExpression, NodeId, NodeKind, NodeKindData, ObjectExpression, PropertyKey,
    PropertyKeyData, SequenceExpression, Severity, SimpleAssignmentTarget, Span,
    TaggedTemplateExpression, TemplateLiteral, Traverse, UnaryExpression, UnaryOperator,
    VariableDeclaration, VariableDeclarator, VariableKind, YieldExpression,
};

use crate::{
    ReferenceFlags, ReferenceSpace, Semantic, SymbolFlags,
    binder::ExportState,
    check,
    module_record::ModuleRecordBuilder,
    scope::{DecoratorSite, ScopeKind},
    state::{Cx, State},
};

pub struct SemanticBuilder<'traverse, 'a> {
    pub(crate) ast: &'traverse Ast<'a>,
    pub(crate) exported_names: FxHashMap<&'traverse [u8], NodeId>,
    pub(crate) export_specifiers: Vec<ExportSpecifier>,
    diagnostics: Vec<Diagnostic<'a>>,
    module_record: ModuleRecordBuilder<'a>,
    // Nested type syntax saves and restores this default through the visitor.
    default_reference_flags: ReferenceFlags,
    // Value expressions consume the incoming context and reset these flags.
    // Target, key, and type boundaries save context when later siblings need it.
    reference_flags: ReferenceFlags,
    reference_flags_stack: Vec<ReferenceFlags>,
    type_positions: Vec<ReferenceFlags>,
    arrow_exports: Vec<ExportState>,

    pub(crate) check_syntax: bool,
}

impl<'traverse, 'a> SemanticBuilder<'traverse, 'a> {
    pub fn new(ast: &'traverse Ast<'a>, check_syntax: bool, build_module_record: bool) -> Self {
        Self {
            exported_names: FxHashMap::default(),
            export_specifiers: Vec::new(),
            diagnostics: Vec::new(),
            module_record: ModuleRecordBuilder::new(ast.allocator(), build_module_record),
            default_reference_flags: ReferenceFlags::default(),
            ast,
            reference_flags: ReferenceFlags::default(),
            reference_flags_stack: Vec::new(),
            type_positions: Vec::new(),
            arrow_exports: Vec::new(),
            check_syntax,
        }
    }

    pub fn finish(self, state: State<'a>) -> (Semantic<'a>, Vec<Diagnostic<'a>>) {
        debug_assert!(self.reference_flags_stack.is_empty());
        debug_assert!(self.type_positions.is_empty());
        debug_assert!(self.arrow_exports.is_empty());
        let (scopes, has_with_statement) = state.scope.into_scope_tree();
        let semantic =
            state
                .symbols
                .finalize(self.ast, scopes, has_with_statement, self.module_record);
        (semantic, self.diagnostics)
    }

    /// True when the walker is currently inside a TS type-only subtree.
    pub(crate) fn in_type_position(&self) -> bool {
        self.default_reference_flags.space == ReferenceSpace::Type
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
            NodeKindData::Program(_) => check::check_program(self),
            NodeKindData::StringLiteral(node) => check::check_string_literal(self, node),
            NodeKindData::NumericLiteral(node) => check::check_numeric_literal(self, node),
            NodeKindData::RegExpLiteral(node) => check::check_reg_exp_literal(self, node),
            NodeKindData::IdentifierReference(node) => {
                check::check_identifier_reference(self, node)
            }
            NodeKindData::LabelIdentifier(node) => check::check_label_identifier(self, node),
            NodeKindData::BinaryExpression(node) => check::check_binary_expression(self, node),
            NodeKindData::UnaryExpression(node) => check::check_unary_expression(self, node),
            NodeKindData::UpdateExpression(node) => check::check_update_expression(self, node),
            NodeKindData::AssignmentExpression(node) => {
                check::check_assignment_expression(self, node)
            }
            NodeKindData::VariableDeclaration(node) => {
                check::check_variable_declaration(self, node)
            }
            NodeKindData::SwitchStatement(node) => check::check_switch_statement(self, node),
            NodeKindData::ForInStatement(node) => check::check_for_in_statement(self, node),
            NodeKindData::ForOfStatement(node) => check::check_for_of_statement(self, node),
            NodeKindData::BreakStatement(node) => check::check_break_statement(self, node),
            NodeKindData::ContinueStatement(node) => check::check_continue_statement(self, node),
            NodeKindData::LabeledStatement(node) => check::check_labeled_statement(self, node),
            NodeKindData::Directive(node) => check::check_directive(self, node),
            NodeKindData::ClassBody(node) => check::check_class_body(self, node),
            NodeKindData::Function(node) => check::check_function(self, node),
            NodeKindData::WithStatement(node) => check::check_with_statement(self, node),
            NodeKindData::ObjectExpression(node) => check::check_object_expression(self, node),
            NodeKindData::MemberExpression(node) => check::check_member_expression(self, node),
            NodeKindData::CallExpression(node) => check::check_call_expression(self, node),
            NodeKindData::AwaitExpression(node) => check::check_await_expression(self, node),
            NodeKindData::YieldExpression(node) => check::check_yield_expression(self, node),
            NodeKindData::MetaProperty(node) => check::check_meta_property(self, node),
            NodeKindData::ImportDeclaration(node) => check::check_import_declaration(self, node),
            NodeKindData::ExportNamedDeclaration(node) => {
                check::check_export_named_declaration(self, node)
            }
            NodeKindData::ExportDefaultDeclaration(node) => {
                check::check_export_default_declaration(self, node)
            }
            NodeKindData::ExportAllDeclaration(node) => {
                check::check_export_all_declaration(self, node)
            }
            NodeKindData::ExportSpecifier(node) => check::check_export_specifier(self, node),
            _ => {}
        }
    }
}

/// Builds scopes and bindings at Traverse node and scope boundaries.
impl<'a> Traverse<'a, State<'a>> for SemanticBuilder<'_, 'a> {
    fn enter_node(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        ctx.state.path.push(node.node_id());
    }

    #[inline(always)]
    fn exit_node(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        if self.check_syntax {
            self.check(node);
        }
        let exited = ctx.state.path.pop();
        debug_assert_eq!(exited, Some(node.node_id()));
    }

    #[inline(always)]
    fn enter_scope(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        let st = &mut ctx.state;
        match node {
            NodeKindData::Class(node) => {
                st.scope.enter_class(self.ast, node);
                st.symbols.enter_class(self.ast, node, &st.scope);
            }
            NodeKindData::BlockStatement(node) => {
                st.scope.enter_block_statement(self.ast, node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                if !node.cases(self.ast).is_empty() {
                    st.scope.enter_switch_case(Some(node.node_id()));
                }
            }
            NodeKindData::ForStatement(node) => {
                st.scope.enter_block_scope(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                st.scope.enter_block_scope(node.node_id());
                st.symbols.enter_catch_clause(&st.scope);
            }
            NodeKindData::WithStatement(node) => {
                st.scope.enter_with_scope(node.node_id());
            }
            NodeKindData::Function(node) => {
                let annex_b_if_body = is_annex_b_if_function(self.ast, st.parent(), node);
                st.scope.enter_function(self.ast, node, annex_b_if_body);
                st.symbols.enter_function(self.ast, node, &st.scope);
            }
            NodeKindData::FunctionBody(node) => {
                let parent = st.parent();
                st.scope.enter_function_body(self.ast, node, parent);
            }
            NodeKindData::StaticBlock(node) => {
                st.scope.enter_static_block(node.node_id());
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                st.scope.enter_arrow_function_expression(self.ast, node);
                self.arrow_exports
                    .push(st.symbols.enter_arrow_function_expression());
            }
            NodeKindData::ForInStatement(node) => st.scope.enter_block_scope(node.node_id()),
            NodeKindData::ForOfStatement(node) => st.scope.enter_block_scope(node.node_id()),
            // The root scope is initialized by SemanticBuilder::new. Global
            // container contexts use node hooks below.
            _ => {}
        }
    }

    #[inline(always)]
    fn leave_scope(&mut self, node: NodeKindData, ctx: &mut Cx<'_, 'a>) {
        let st = &mut ctx.state;
        match node {
            NodeKindData::Class(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::BlockStatement(node) => {
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::SwitchStatement(node) => {
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::ForStatement(node) => {
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::CatchClause(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::WithStatement(node) => {
                st.scope.exit_with_scope(node.node_id());
            }
            NodeKindData::Function(node) => {
                st.symbols.exit_saved_context();
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::FunctionBody(node) => {
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::StaticBlock(node) => {
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::ArrowFunctionExpression(node) => {
                let export_state = self
                    .arrow_exports
                    .pop()
                    .expect("an arrow saved its export state");
                st.symbols.exit_arrow_function_expression(export_state);
                st.scope.exit_scope(node.node_id());
            }
            NodeKindData::ForInStatement(node) => st.scope.exit_scope(node.node_id()),
            NodeKindData::ForOfStatement(node) => st.scope.exit_scope(node.node_id()),
            _ => {}
        }
    }

    fn enter_binary_expression(&mut self, _node: BinaryExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_logical_expression(&mut self, _node: LogicalExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_conditional_expression(
        &mut self,
        _node: ConditionalExpression,
        _ctx: &mut Cx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_sequence_expression(&mut self, _node: SequenceExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_chain_expression(&mut self, _node: ChainExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_new_expression(&mut self, _node: NewExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_tagged_template_expression(
        &mut self,
        _node: TaggedTemplateExpression,
        _ctx: &mut Cx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_arrow_function_expression(
        &mut self,
        _node: ArrowFunctionExpression,
        _ctx: &mut Cx<'_, 'a>,
    ) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_array_expression(&mut self, _node: ArrayExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_object_expression(&mut self, _node: ObjectExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_import_expression(&mut self, _node: ImportExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_await_expression(&mut self, _node: AwaitExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_yield_expression(&mut self, _node: YieldExpression, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_jsx_element(&mut self, _node: JsxElement, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_jsx_fragment(&mut self, _node: JsxFragment, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_template_literal(&mut self, _node: TemplateLiteral, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_function(&mut self, _node: Function, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_call_expression(&mut self, node: CallExpression, ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
        if !ctx.state.symbols.in_ambient_context()
            && self.default_reference_flags.is_read()
            && is_direct_eval_call(self.ast, node)
        {
            ctx.state.scope.mark_direct_eval();
        }
    }

    fn enter_assignment_expression(&mut self, node: AssignmentExpression, ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
        ctx.state.record_ts_expando_assignment(ctx.ast, node);
    }

    fn enter_simple_assignment_target(
        &mut self,
        _node: SimpleAssignmentTarget,
        ctx: &mut Cx<'_, 'a>,
    ) {
        let read = match ctx.state.parent().map(|parent| self.ast.kind_data(parent)) {
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
        _ctx: &mut Cx<'_, 'a>,
    ) {
        // Each target establishes its own flags; following expressions read.
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_unary_expression(&mut self, node: UnaryExpression, _ctx: &mut Cx<'_, 'a>) {
        let flags = self.default_reference_flags;
        let flags = if node.operator(self.ast) == UnaryOperator::Delete {
            flags.no_access()
        } else {
            flags
        };
        self.push_reference_flags(flags);
    }

    fn exit_unary_expression(&mut self, _node: UnaryExpression, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_member_expression(&mut self, _node: MemberExpression, _ctx: &mut Cx<'_, 'a>) {
        self.push_reference_flags(self.flags_with_space(self.reference_flags.space));
    }

    fn exit_member_expression(&mut self, _node: MemberExpression, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_assignment_pattern(&mut self, _node: AssignmentPattern, _ctx: &mut Cx<'_, 'a>) {
        // The default value reads, but later destructuring targets still write.
        self.push_reference_flags(self.reference_flags);
    }

    fn exit_assignment_pattern(&mut self, _node: AssignmentPattern, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_variable_declarator(&mut self, _node: VariableDeclarator, _ctx: &mut Cx<'_, 'a>) {
        // A for-in/of declaration still needs its write context while binding
        // the target; the initializer switches to the default context below.
        self.push_reference_flags(self.reference_flags);
    }

    fn exit_variable_declarator(&mut self, _node: VariableDeclarator, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    #[inline(always)]
    fn enter_binding_pattern(&mut self, _node: BindingPattern, ctx: &mut Cx<'_, 'a>) {
        if let Some(NodeKindData::VariableDeclarator(declarator)) =
            ctx.state.parent().map(|parent| self.ast.kind_data(parent))
        {
            let is_var = ctx.state.path.iter().rev().nth(2).is_some_and(|parent| {
                matches!(
                    self.ast.kind_data(*parent),
                    NodeKindData::VariableDeclaration(declaration)
                        if declaration.kind(self.ast) == VariableKind::Var
                )
            });
            let flags = if is_var
                && ctx.state.scope.current().flags.var_write_crosses_with
                && (declarator.init(self.ast).is_some() || self.reference_flags.is_write())
            {
                self.assignment_reference_flags(false)
            } else {
                self.default_reference_flags
            };
            self.reference_flags = flags;
        }
    }

    fn exit_binding_pattern(&mut self, _node: BindingPattern, ctx: &mut Cx<'_, 'a>) {
        if let Some(NodeKind::VariableDeclarator | NodeKind::AssignmentPattern) =
            ctx.state.parent().map(|parent| self.ast.node_kind(parent))
        {
            self.reference_flags = self.default_reference_flags;
        }
    }

    #[inline(always)]
    fn enter_property_key(&mut self, node: PropertyKey, ctx: &mut Cx<'_, 'a>) {
        if matches!(
            self.ast.property_key_data(node),
            PropertyKeyData::IdentifierName(_)
        ) {
            return;
        }
        let type_query = match ctx.state.parent().map(|parent| self.ast.kind_data(parent)) {
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

    fn exit_property_key(&mut self, node: PropertyKey, _ctx: &mut Cx<'_, 'a>) {
        if !matches!(
            self.ast.property_key_data(node),
            PropertyKeyData::IdentifierName(_)
        ) {
            self.pop_reference_flags();
        }
    }

    fn enter_for_statement_left(&mut self, _node: ForStatementLeft, _ctx: &mut Cx<'_, 'a>) {
        self.push_reference_flags(self.assignment_reference_flags(false));
    }

    fn exit_for_statement_left(&mut self, _node: ForStatementLeft, _ctx: &mut Cx<'_, 'a>) {
        self.pop_reference_flags();
    }

    fn enter_identifier_reference(&mut self, node: IdentifierReference, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .record_identifier_reference(ctx.ast, node, self.reference_flags);
    }

    fn enter_directive(&mut self, node: Directive, ctx: &mut Cx<'_, 'a>) {
        ctx.state.scope.enter_directive(self.ast, node);
    }

    fn enter_class(&mut self, _node: Class, _ctx: &mut Cx<'_, 'a>) {
        self.reference_flags = self.default_reference_flags;
    }

    fn enter_class_body(&mut self, _node: ClassBody, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.enter_class_body();
    }

    fn enter_decorator(&mut self, node: Decorator, ctx: &mut Cx<'_, 'a>) {
        // TS resolves members and parameters at the class, skipping method
        // locals. Only a class declaration's own decorators skip the class;
        // class expressions retain their type parameters and name environment.
        let site = match ctx.state.parent().map(|parent| self.ast.kind_data(parent)) {
            Some(NodeKindData::Class(class))
                if class.class_type(self.ast) == ClassType::ClassDeclaration =>
            {
                DecoratorSite::ClassDeclaration
            }
            Some(NodeKindData::Class(_)) => DecoratorSite::ClassExpression,
            _ => DecoratorSite::Member,
        };
        self.push_reference_flags(self.default_reference_flags);
        ctx.state.scope.enter_decorator(node.node_id(), site);
    }

    fn exit_decorator(&mut self, _node: Decorator, ctx: &mut Cx<'_, 'a>) {
        ctx.state.scope.exit_decorator();
        self.pop_reference_flags();
    }

    fn enter_import_declaration(&mut self, node: ImportDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .enter_import_declaration(self.ast, node, &ctx.state.scope);
    }

    fn exit_import_declaration(&mut self, _node: ImportDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_import_specifier(&mut self, node: ImportSpecifier, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.enter_import_specifier(self.ast, node);
    }

    fn exit_import_specifier(&mut self, _node: ImportSpecifier, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_export_named_declaration(
        &mut self,
        node: ExportNamedDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state
            .symbols
            .enter_export_named_declaration(self.ast, node);
    }

    fn exit_export_named_declaration(
        &mut self,
        _node: ExportNamedDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state.symbols.exit_export_declaration();
    }

    fn enter_export_default_declaration(
        &mut self,
        node: ExportDefaultDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        if !self.in_ts_namespace() {
            self.module_record.add_default_export(self.ast, node);
        }
        ctx.state.symbols.enter_export_default_declaration();
        let declaration = node.declaration(self.ast);
        let space = declaration
            .as_expr(self.ast)
            .map_or(ReferenceSpace::Value, |expression| {
                export_expression_space(self.ast, expression.node_id())
            });
        self.push_reference_flags(self.flags_with_space(space));
    }

    fn enter_export_all_declaration(&mut self, node: ExportAllDeclaration, _ctx: &mut Cx<'_, 'a>) {
        if !self.in_ts_namespace() {
            self.module_record.add_export_all(self.ast, node);
        }
    }

    fn enter_export_specifier(&mut self, node: ExportSpecifier, ctx: &mut Cx<'_, 'a>) {
        let declaration = ctx.state.parent().and_then(|parent| {
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
            ctx.state.symbols.add_local_export_reference(
                self.ast,
                node,
                &ctx.state.scope,
                flags,
                type_only,
            );
        }
    }

    fn enter_jsx_opening_element(&mut self, node: JsxOpeningElement, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .add_jsx_opening_element_reference(self.ast, node, &ctx.state.scope);
    }

    fn enter_jsx_closing_element(&mut self, node: JsxClosingElement, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .add_jsx_closing_element_reference(self.ast, node, &ctx.state.scope);
    }

    fn exit_binding_identifier(&mut self, node: BindingIdentifier, ctx: &mut Cx<'_, 'a>) {
        let flags = self.reference_flags;
        let prepared = if self.check_syntax {
            check::check_binding_identifier(self, node)
        } else {
            None
        };
        let direct_export = ctx.state.symbols.export_state.named();
        let direct_export = direct_export.filter(|_| !self.in_ts_namespace());
        let declaration = if let Some(prepared) = prepared {
            ctx.state.symbols.declare_prepared_binding_identifier(
                self.ast,
                node,
                &ctx.state.scope,
                prepared,
            )
        } else {
            ctx.state
                .symbols
                .declare_binding_identifier(self.ast, node, &ctx.state.scope)
        };
        if declaration.is_some() && flags.is_write_only() {
            ctx.state
                .symbols
                .add_var_write_reference(self.ast, node, &ctx.state.scope, flags);
        }
        if declaration.is_some()
            && let Some(statement) = direct_export
        {
            self.module_record
                .add_direct_export(self.ast, node.node_id(), statement);
        }
    }

    fn enter_variable_declaration(&mut self, node: VariableDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state
            .symbols
            .enter_variable_declaration(self.ast, node, &ctx.state.scope);
    }

    fn exit_variable_declaration(&mut self, _node: VariableDeclaration, ctx: &mut Cx<'_, 'a>) {
        ctx.state.symbols.exit_saved_context();
    }

    fn enter_formal_parameters(&mut self, node: FormalParameters, ctx: &mut Cx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            ctx.state.symbols.export_state = Default::default();
            return;
        }
        ctx.state.symbols.enter_formal_parameters(&ctx.state.scope);
        if self.ast.is_ts() {
            ctx.state
                .scope
                .enter_ts_lookup_view(node.node_id(), ScopeKind::TsSignatureParameter);
        }
    }

    fn exit_formal_parameters(&mut self, node: FormalParameters, ctx: &mut Cx<'_, 'a>) {
        if node.items(self.ast).is_empty() && node.rest(self.ast).is_none() {
            return;
        }
        if self.ast.is_ts() {
            ctx.state.scope.exit_ts_lookup_view(node.node_id());
        }
        ctx.state.symbols.exit_formal_parameters();
    }

    fn exit_export_default_declaration(
        &mut self,
        _node: ExportDefaultDeclaration,
        ctx: &mut Cx<'_, 'a>,
    ) {
        ctx.state.symbols.exit_export_declaration();
        self.pop_reference_flags();
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

pub(crate) struct ReportOptions<'a> {
    pub severity: Severity,
    pub help: Option<Cow<'a, str>>,
    pub labels: Vec<Label<'a>>,
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
    pub(crate) fn report_unresolved_export(&mut self, name: &[u8], node: NodeId) {
        let message = self.unresolved_export_message(name);
        self.report(self.ast.span(node), message, ReportOptions::default());
    }

    pub(crate) fn report_non_local_export(&mut self, name: &[u8], node: NodeId) {
        let message = format!(
            "Cannot export '{}'. Only local declarations can be exported from a module",
            String::from_utf8_lossy(name)
        );
        self.report(self.ast.span(node), message, ReportOptions::default());
    }

    pub(crate) fn unresolved_export_message(&self, name: &[u8]) -> String {
        format!("Export '{}' is not defined", String::from_utf8_lossy(name))
    }

    #[cold]
    pub(crate) fn report_redeclaration(
        &mut self,
        name: &[u8],
        node: NodeId,
        existing_node: NodeId,
        existing_flags: SymbolFlags,
    ) {
        let name = String::from_utf8_lossy(name);
        let current_span = self.ast.span(node);
        let existing_span = self.ast.span(existing_node);
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
                    Label::new(existing_span, Cow::Owned(first)),
                    Label::new(current_span, Cow::Borrowed("cannot be redeclared here")),
                ],
                ..ReportOptions::default()
            },
        );
    }

    #[cold]
    pub(crate) fn report(
        &mut self,
        span: Span,
        message: impl Into<Cow<'a, str>>,
        options: ReportOptions<'a>,
    ) {
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
}
