use std::collections::HashSet;

use swc_next_ecma_ast::{
    Argument, ArgumentData, AssignmentOperator, AssignmentTarget, Ast, AstUtf8, BinaryExpression,
    BinaryOperator, BindingPattern, ClassElementData, DUMMY_SP, Decorator, Expr,
    ForStatementInitData, ForStatementLeftData, FormalParameterKind, Function, FunctionType,
    LogicalOperator, NodeKind, NodeKindData, NumericLiteralKind, ObjectPropertyKind,
    ObjectPropertyKindData, PropertyKey, PropertyKeyData, Stmt, TypedSubRange, UnaryOperator,
    VariableKind,
};
use swc_next_ecma_semantic::Semantic;

use super::analysis::{CaptureFacts, GeneratorAnalysis, YieldFacts};

mod expression;
mod statement;

pub(super) fn transform(
    mut analysis: GeneratorAnalysis,
    ast: &mut Ast<'_>,
    semantic: &Semantic<'_>,
) {
    let mut names = NameAllocator::new(ast, semantic);
    for function in analysis.generators {
        let captures = analysis
            .captures
            .remove(&function.node_id())
            .unwrap_or_default();
        GeneratorCompiler::new(ast, &analysis.yields, &mut names, captures).lower(function);
    }
}

struct NameAllocator {
    used: HashSet<Vec<u8>>,
}

impl NameAllocator {
    fn new(ast: &Ast<'_>, semantic: &Semantic<'_>) -> Self {
        let mut used = HashSet::new();
        for entry in semantic.iter_symbols() {
            used.insert(ast.get_wtf8(entry.symbol.name).as_bytes().to_vec());
        }
        for entry in semantic.iter_references() {
            used.insert(ast.get_utf8(entry.reference.name).as_bytes().to_vec());
        }
        Self { used }
    }

    fn fresh(&mut self, base: &str, ast: &mut Ast<'_>) -> AstUtf8 {
        let mut candidate = base.to_owned();
        let mut suffix = 1u32;
        while self.used.contains(candidate.as_bytes()) {
            candidate.clear();
            candidate.push_str(base);
            candidate.push_str(&suffix.to_string());
            suffix += 1;
        }
        self.used.insert(candidate.as_bytes().to_vec());
        ast.add_utf8(&candidate)
    }
}

struct GeneratorCompiler<'facts, 'names, 'ast> {
    ast: &'names mut Ast<'ast>,
    facts: &'facts YieldFacts,
    names: &'names mut NameAllocator,
    state_name: AstUtf8,
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

impl<'facts, 'names, 'ast> GeneratorCompiler<'facts, 'names, 'ast> {
    fn new(
        ast: &'names mut Ast<'ast>,
        facts: &'facts YieldFacts,
        names: &'names mut NameAllocator,
        captures: CaptureFacts,
    ) -> Self {
        let state_name = names.fresh("_state", ast);
        let mut compiler = Self {
            ast,
            facts,
            names,
            state_name,
            cases: vec![Vec::new()],
            current: 0,
            hoisted_vars: Vec::new(),
            capture_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            control: Vec::new(),
            with_stack: Vec::new(),
        };
        compiler.prepare_captures(captures);
        compiler
    }

    fn prepare_captures(&mut self, captures: CaptureFacts) {
        if !captures.arguments.is_empty() {
            let name = self.names.fresh("_arguments", self.ast);
            for reference in captures.arguments {
                reference.set_name(self.ast, name);
            }
            let source_name = self.ast.add_utf8("arguments");
            let initializer = self.ast.expr_identifier_reference(DUMMY_SP, source_name);
            let declaration = self.named_declarator(name, Some(initializer));
            self.capture_vars.push(declaration);
        }
        if !captures.new_targets.is_empty() {
            let name = self.names.fresh("_newtarget", self.ast);
            for meta in captures.new_targets {
                let replacement = self.ast.identifier_reference(DUMMY_SP, name);
                // SAFETY: `MetaProperty` occurs through an `Expr` edge, which
                // also accepts `IdentifierReference`; the temporary is the
                // final unattached node and is consumed immediately.
                unsafe {
                    self.ast.replace_data(meta, replacement);
                }
            }
            let meta_name = self.ast.add_utf8("new");
            let property_name = self.ast.add_utf8("target");
            let meta = self.ast.identifier_name(DUMMY_SP, meta_name);
            let property = self.ast.identifier_name(DUMMY_SP, property_name);
            let initializer = self.ast.expr_meta_property(DUMMY_SP, meta, property);
            let declaration = self.named_declarator(name, Some(initializer));
            self.capture_vars.push(declaration);
        }
    }

    fn named_declarator(
        &mut self,
        name: AstUtf8,
        initializer: Option<Expr>,
    ) -> swc_next_ecma_ast::VariableDeclarator {
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name,
            TypedSubRange::empty(),
            None,
            false,
        );
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
                self.state_name,
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

impl<'facts, 'names, 'ast> GeneratorCompiler<'facts, 'names, 'ast> {
    fn cache(&mut self, expression: Expr) -> Expr {
        let name = self.new_temp();
        self.emit_assignment(name, expression);
        self.ast.expr_identifier_reference(DUMMY_SP, name)
    }

    fn new_temp(&mut self) -> AstUtf8 {
        let name = self.names.fresh("_tmp", self.ast);
        let pattern = self.ast.binding_pattern_binding_identifier(
            DUMMY_SP,
            name,
            TypedSubRange::empty(),
            None,
            false,
        );
        let declarator = self.ast.variable_declarator(DUMMY_SP, pattern, None, false);
        self.hoisted_vars.push(declarator);
        name
    }

    fn emit_assignment(&mut self, name: AstUtf8, expression: Expr) {
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(DUMMY_SP, name);
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
        let object = self
            .ast
            .expr_identifier_reference(DUMMY_SP, self.state_name);
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

        let state = self
            .ast
            .expr_identifier_reference(DUMMY_SP, self.state_name);
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
            let object = self
                .ast
                .expr_identifier_reference(DUMMY_SP, self.state_name);
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
