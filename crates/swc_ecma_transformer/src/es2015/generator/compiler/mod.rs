use swc_next_ecma_ast::{
    Argument, ArgumentData, AssignmentOperator, AssignmentTarget, Ast, AstNode, AstUtf8,
    BinaryExpression, BinaryOperator, BindingPattern, ClassElementData, DUMMY_SP, Decorator, Expr,
    ForStatementInitData, ForStatementLeftData, FormalParameterKind, Function, FunctionType,
    LogicalOperator, NodeKind, NodeKindData, NumericLiteralKind, ObjectPropertyKind,
    ObjectPropertyKindData, PropertyKey, PropertyKeyData, ScopeId, Stmt, SymbolId, TypedSubRange,
    UnaryOperator, VariableKind,
};
use swc_next_ecma_semantic::{
    FunctionCaptures, ReferenceFlags, Semantic, SubtreeEffect, SymbolFlags,
};

use crate::state::{PrivateName, reserve_private_name};

mod expression;
mod statement;

pub(super) fn transform<'a>(ast: &mut Ast<'a>, semantic: &mut Semantic<'a>) {
    let generators = semantic
        .generator_functions_post_order()
        .expect("generator transform requires GENERATOR_FUNCTIONS semantic feature")
        .to_vec();
    for function in generators {
        let captures = semantic
            .function_captures(function.node_id())
            .expect("generator transform requires FUNCTION_ENV_CAPTURE semantic feature");
        let arguments = captures.arguments.to_vec();
        let new_targets = captures.new_targets.to_vec();
        let body_scope = semantic.scope_of(function.body(ast).node_id());
        let scope = semantic.scope(body_scope).hoist_target;
        GeneratorCompiler::new(ast, semantic, scope, &arguments, &new_targets).lower(function);
        semantic.clear_generator_yield_effects(function.node_id());
    }
    semantic.finish_generator_transform();
}

struct GeneratorCompiler<'semantic, 'ast> {
    ast: &'semantic mut Ast<'ast>,
    semantic: &'semantic mut Semantic<'ast>,
    scope: ScopeId,
    state_name: PrivateName,
    cases: Vec<Vec<Stmt>>,
    current: usize,
    hoisted_vars: Vec<swc_next_ecma_ast::VariableDeclarator>,
    capture_vars: Vec<swc_next_ecma_ast::VariableDeclarator>,
    hoisted_functions: Vec<Stmt>,
    control: Vec<ControlTarget>,
    with_stack: Vec<AstUtf8>,
}

#[derive(Clone)]
struct ControlTarget {
    break_label: usize,
    continue_label: Option<usize>,
    name: Option<Vec<u8>>,
}

impl<'semantic, 'ast> GeneratorCompiler<'semantic, 'ast> {
    fn new(
        ast: &'semantic mut Ast<'ast>,
        semantic: &'semantic mut Semantic<'ast>,
        scope: ScopeId,
        arguments: &[swc_next_ecma_ast::ReferenceId],
        new_targets: &[swc_next_ecma_ast::NodeId],
    ) -> Self {
        let state_name = reserve_private_name(ast, semantic, scope, "_state");
        let mut compiler = Self {
            ast,
            semantic,
            scope,
            state_name,
            cases: vec![Vec::new()],
            current: 0,
            hoisted_vars: Vec::new(),
            capture_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            control: Vec::new(),
            with_stack: Vec::new(),
        };
        compiler.prepare_captures(FunctionCaptures {
            arguments,
            new_targets,
        });
        compiler
    }

    #[inline]
    fn contains_yield<N: AstNode>(&self, node: N) -> bool {
        self.semantic
            .contains_effect(node.node_id(), SubtreeEffect::Yield)
            .expect("generator transform requires the YIELD semantic feature")
    }

    fn prepare_captures(&mut self, captures: FunctionCaptures<'_>) {
        if !captures.arguments.is_empty() {
            let name = reserve_private_name(self.ast, self.semantic, self.scope, "_arguments");
            let source_name = self.ast.add_utf8("arguments");
            let initializer = self.ast.expr_identifier_reference(DUMMY_SP, source_name);
            let declaration = self.named_declarator(name, Some(initializer));
            let symbol = self.register_declarator(declaration, name);
            self.semantic.register_node(
                initializer.node_id(),
                Some(declaration.node_id()),
                self.scope,
            );
            self.semantic.reference_generated(
                self.ast,
                self.scope,
                initializer.node_id(),
                self.ast.canonicalize_utf8(source_name),
                ReferenceFlags::default(),
                None,
            );
            let generated_name = self.ast.canonicalize_utf8(name.text);
            for reference_id in captures.arguments {
                let reference_node = self.semantic.reference(*reference_id).node;
                let NodeKindData::IdentifierReference(reference) =
                    self.ast.kind_data(reference_node)
                else {
                    continue;
                };
                reference.set_name(self.ast, name.text);
                self.ast
                    .set_syntax_context(reference.node_id(), name.context);
                self.semantic
                    .rename_reference(self.ast, *reference_id, generated_name);
                self.semantic.rebind_reference(*reference_id, Some(symbol));
            }
            self.capture_vars.push(declaration);
        }
        if !captures.new_targets.is_empty() {
            let name = reserve_private_name(self.ast, self.semantic, self.scope, "_newtarget");
            let meta_name = self.ast.add_utf8("new");
            let property_name = self.ast.add_utf8("target");
            let meta = self.ast.identifier_name(DUMMY_SP, meta_name);
            let property = self.ast.identifier_name(DUMMY_SP, property_name);
            let initializer = self.ast.expr_meta_property(DUMMY_SP, meta, property);
            let declaration = self.named_declarator(name, Some(initializer));
            let symbol = self.register_declarator(declaration, name);
            self.semantic.register_node(
                initializer.node_id(),
                Some(declaration.node_id()),
                self.scope,
            );
            let generated_name = self.ast.canonicalize_utf8(name.text);
            for meta_node in captures.new_targets {
                let NodeKindData::MetaProperty(meta) = self.ast.kind_data(*meta_node) else {
                    continue;
                };
                let replacement = self.ast.identifier_reference(DUMMY_SP, name.text);
                self.ast
                    .set_syntax_context(replacement.node_id(), name.context);
                unsafe {
                    self.ast.replace_data(meta, replacement);
                }
                self.semantic.reference_generated(
                    self.ast,
                    self.scope,
                    *meta_node,
                    generated_name,
                    ReferenceFlags::default(),
                    Some(symbol),
                );
            }
            self.capture_vars.push(declaration);
        }
    }

    fn register_declarator(
        &mut self,
        declaration: swc_next_ecma_ast::VariableDeclarator,
        name: PrivateName,
    ) -> SymbolId {
        self.semantic
            .register_node(declaration.node_id(), None, self.scope);
        let identifier = declaration
            .id(self.ast)
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        if identifier.node_id() != declaration.node_id() {
            self.semantic.register_node(
                identifier.node_id(),
                Some(declaration.node_id()),
                self.scope,
            );
        }
        self.semantic.declare(
            self.ast,
            self.scope,
            identifier.node_id(),
            self.ast.canonicalize_utf8(name.text),
            SymbolFlags::FUNCTION_SCOPED_VAR,
        )
    }

    fn named_declarator(
        &mut self,
        name: PrivateName,
        initializer: Option<Expr>,
    ) -> swc_next_ecma_ast::VariableDeclarator {
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name.text,
            TypedSubRange::empty(),
            None,
            false,
        );
        let identifier = pattern
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        self.ast
            .variable_declarator(DUMMY_SP, pattern, initializer, false)
    }

    fn lower(mut self, function: Function) {
        let body = function.body(self.ast);
        let statements = self.nodes(body.body(self.ast));
        for statement in statements {
            self.compile_stmt(statement);
        }
        if !self.case_terminated(self.current) {
            let done = self.return_op(2, None);
            self.emit(done);
        }

        let state_body = self.build_state_body();
        let state_pattern = self
            .ast
            .formal_parameter_pattern_binding_pattern_binding_identifier(
                DUMMY_SP,
                self.state_name.text,
                TypedSubRange::empty(),
                None,
                false,
            );
        let state_param = self
            .ast
            .formal_parameter_item_formal_parameter(DUMMY_SP, state_pattern);
        let state_items = self.ast.add_typed_sub_range([state_param]);
        let state_params = self.ast.formal_parameters(
            DUMMY_SP,
            state_items,
            None,
            FormalParameterKind::FormalParameters,
        );
        let state_identifier = state_pattern
            .as_binding_pattern(self.ast)
            .and_then(|pattern| pattern.as_binding_identifier(self.ast))
            .expect("state parameter is an identifier");
        self.ast
            .set_syntax_context(state_identifier.node_id(), self.state_name.context);
        let inner = self.ast.expr_function(
            DUMMY_SP,
            FunctionType::FunctionExpression,
            None,
            false,
            false,
            state_params,
            state_body,
            None,
            None,
        );
        let helper_name = self.ast.add_utf8("_ts_generator");
        let helper = self.ast.expr_identifier_reference(DUMMY_SP, helper_name);
        let this = self.ast.expr_this_expression(DUMMY_SP);
        let arguments = self
            .ast
            .add_typed_sub_range([Argument::Expr(this), Argument::Expr(inner)]);
        let call = self
            .ast
            .expr_call_expression(DUMMY_SP, helper, arguments, false, None);

        let mut outer = Vec::new();
        if !self.capture_vars.is_empty() || !self.hoisted_vars.is_empty() {
            let mut variables = std::mem::take(&mut self.capture_vars);
            variables.append(&mut self.hoisted_vars);
            let declarators = self.ast.add_typed_sub_range(variables);
            outer.push(self.ast.stmt_decl_variable_declaration(
                DUMMY_SP,
                VariableKind::Var,
                declarators,
                false,
            ));
        }
        outer.append(&mut self.hoisted_functions);
        outer.push(self.ast.stmt_return_statement(DUMMY_SP, Some(call)));
        let outer = self.ast.add_typed_sub_range(outer);
        body.set_body(self.ast, outer);
        function.set_generator(self.ast, false);
    }
}

impl<'semantic, 'ast> GeneratorCompiler<'semantic, 'ast> {
    fn cache(&mut self, expression: Expr) -> Expr {
        let name = self.new_temp();
        self.emit_assignment(name, expression);
        self.private_reference(name)
    }

    fn new_temp(&mut self) -> PrivateName {
        let name = reserve_private_name(self.ast, self.semantic, self.scope, "_tmp");
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name.text,
            TypedSubRange::empty(),
            None,
            false,
        );
        let identifier = pattern
            .as_binding_identifier(self.ast)
            .expect("private binding is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        let declarator = self.ast.variable_declarator(DUMMY_SP, pattern, None, false);
        self.hoisted_vars.push(declarator);
        name
    }

    fn private_reference(&mut self, name: PrivateName) -> Expr {
        let expression = self.ast.expr_identifier_reference(DUMMY_SP, name.text);
        let identifier = expression
            .as_identifier_reference(self.ast)
            .expect("private reference is an identifier");
        self.ast
            .set_syntax_context(identifier.node_id(), name.context);
        expression
    }

    fn emit_assignment(&mut self, name: PrivateName, expression: Expr) {
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, name.text);
        self.ast.set_syntax_context(left.node_id(), name.context);
        let assignment = self.ast.expr_assignment_expression(
            DUMMY_SP,
            left,
            expression,
            AssignmentOperator::Assign,
        );
        let statement = self.ast.stmt_expression_statement(DUMMY_SP, assignment);
        self.emit(statement);
    }

    fn state_call(&mut self, property: &str) -> Expr {
        let object = self.private_reference(self.state_name);
        let property_name = self.ast.add_utf8(property);
        let property = self
            .ast
            .property_key_identifier_name(DUMMY_SP, property_name);
        let member = self
            .ast
            .expr_member_expression(DUMMY_SP, object, property, false, false);
        self.ast
            .expr_call_expression(DUMMY_SP, member, TypedSubRange::empty(), false, None)
    }

    fn return_op(&mut self, opcode: u32, value: Option<Expr>) -> Stmt {
        let opcode = self.number(opcode as usize);
        let mut elements = vec![Some(Argument::Expr(opcode))];
        if let Some(value) = value {
            elements.push(Some(Argument::Expr(value)));
        }
        let elements = self.ast.add_typed_sub_range(elements);
        let array = self.ast.expr_array_expression(DUMMY_SP, elements);
        self.ast.stmt_return_statement(DUMMY_SP, Some(array))
    }

    fn jump(&mut self, label: usize) -> Stmt {
        let label = self.number(label);
        self.return_op(3, Some(label))
    }

    fn emit_try_push(
        &mut self,
        start: usize,
        catch: Option<usize>,
        finally: Option<usize>,
        end: usize,
    ) {
        let elements = [
            Some(Argument::Expr(self.number(start))),
            catch.map(|label| Argument::Expr(self.number(label))),
            finally.map(|label| Argument::Expr(self.number(label))),
            Some(Argument::Expr(self.number(end))),
        ];
        let elements = self.ast.add_typed_sub_range(elements);
        let record = self.ast.expr_array_expression(DUMMY_SP, elements);

        let state = self.private_reference(self.state_name);
        let trys_name = self.ast.add_utf8("trys");
        let trys_key = self.ast.property_key_identifier_name(DUMMY_SP, trys_name);
        let trys = self
            .ast
            .expr_member_expression(DUMMY_SP, state, trys_key, false, false);
        let push_name = self.ast.add_utf8("push");
        let push_key = self.ast.property_key_identifier_name(DUMMY_SP, push_name);
        let push = self
            .ast
            .expr_member_expression(DUMMY_SP, trys, push_key, false, false);
        let arguments = self.ast.add_typed_sub_range([Argument::Expr(record)]);
        let call = self
            .ast
            .expr_call_expression(DUMMY_SP, push, arguments, false, None);
        let statement = self.ast.stmt_expression_statement(DUMMY_SP, call);
        self.emit(statement);
    }

    fn emit_conditional_jump(&mut self, test: Expr, label: usize, when_false: bool) {
        let test = if when_false {
            self.ast
                .expr_unary_expression(DUMMY_SP, test, UnaryOperator::LogicalNot)
        } else {
            test
        };
        let jump = self.jump(label);
        let statement = self.ast.stmt_if_statement(DUMMY_SP, test, jump, None);
        self.emit(statement);
    }

    fn number(&mut self, value: usize) -> Expr {
        let raw = self.ast.add_utf8(&value.to_string());
        self.ast
            .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, raw)
    }

    fn build_state_body(&mut self) -> swc_next_ecma_ast::FunctionBody {
        let body = if self.cases.len() == 1 {
            self.ast
                .add_typed_sub_range(std::mem::take(&mut self.cases[0]))
        } else {
            let object = self.private_reference(self.state_name);
            let label_name = self.ast.add_utf8("label");
            let property = self.ast.property_key_identifier_name(DUMMY_SP, label_name);
            let discriminant = self
                .ast
                .expr_member_expression(DUMMY_SP, object, property, false, false);
            let case_bodies = std::mem::take(&mut self.cases);
            let mut cases = Vec::with_capacity(case_bodies.len());
            for (label, statements) in case_bodies.into_iter().enumerate() {
                let raw = self.ast.add_utf8(&label.to_string());
                let test =
                    self.ast
                        .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, raw);
                let consequent = self.ast.add_typed_sub_range(statements);
                cases.push(self.ast.switch_case(DUMMY_SP, Some(test), consequent));
            }
            let cases = self.ast.add_typed_sub_range(cases);
            let switch = self
                .ast
                .stmt_switch_statement(DUMMY_SP, discriminant, cases);
            self.ast.add_typed_sub_range([switch])
        };
        self.ast
            .function_body(DUMMY_SP, TypedSubRange::empty(), body)
    }

    fn emit(&mut self, mut statement: Stmt) {
        for name in self.with_stack.iter().rev().copied() {
            let object = self.ast.expr_identifier_reference(DUMMY_SP, name);
            statement = self.ast.stmt_with_statement(DUMMY_SP, object, statement);
        }
        self.cases[self.current].push(statement);
    }

    fn start_new_case(&mut self) -> usize {
        let label = self.new_case();
        self.current = label;
        label
    }

    fn new_case(&mut self) -> usize {
        let label = self.cases.len();
        self.cases.push(Vec::new());
        label
    }

    fn case_terminated(&self, case: usize) -> bool {
        self.cases[case].last().is_some_and(|statement| {
            matches!(
                self.ast.node_kind(statement.node_id()),
                NodeKind::ReturnStatement | NodeKind::ThrowStatement
            )
        })
    }

    fn nodes<N: swc_next_ecma_ast::ExtraDataCompact<'ast>>(
        &self,
        range: TypedSubRange<N>,
    ) -> Vec<N> {
        range
            .iter()
            .map(|id| self.ast.get_node_in_sub_range(id))
            .collect()
    }
}
