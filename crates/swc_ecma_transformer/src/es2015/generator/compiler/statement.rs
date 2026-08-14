use super::*;

impl<'facts, 'names, 'ast> GeneratorCompiler<'facts, 'names, 'ast> {
    pub(super) fn compile_stmt(&mut self, statement: Stmt) {
        match self.ast.kind_data(statement.node_id()) {
            NodeKindData::BlockStatement(block) => {
                for child in self.nodes(block.body(self.ast)) {
                    self.compile_stmt(child);
                }
            }
            NodeKindData::ExpressionStatement(expression) => {
                let value = self.compile_expr(expression.expression(self.ast));
                let statement = self.ast.stmt_expression_statement(DUMMY_SP, value);
                self.emit(statement);
            }
            NodeKindData::ReturnStatement(ret) => {
                let value = ret.argument(self.ast).map(|expr| self.compile_expr(expr));
                let statement = self.return_op(2, value);
                self.emit(statement);
            }
            NodeKindData::ThrowStatement(throw) => {
                let value = self.compile_expr(throw.argument(self.ast));
                let statement = self.return_op(6, Some(value));
                self.emit(statement);
            }
            NodeKindData::VariableDeclaration(declaration) => {
                self.compile_variable_declaration(declaration);
            }
            NodeKindData::Function(_) => self.hoisted_functions.push(statement),
            NodeKindData::IfStatement(if_statement) => self.compile_if(if_statement),
            NodeKindData::WhileStatement(while_statement) => {
                self.compile_while(
                    while_statement.test(self.ast),
                    while_statement.body(self.ast),
                    None,
                );
            }
            NodeKindData::DoWhileStatement(do_while) => {
                self.compile_do_while(do_while.body(self.ast), do_while.test(self.ast), None);
            }
            NodeKindData::ForStatement(for_statement) => self.compile_for(for_statement, None),
            NodeKindData::ForInStatement(for_in_statement) => {
                self.compile_for_in(for_in_statement, None)
            }
            NodeKindData::LabeledStatement(labeled_statement) => {
                self.compile_labeled(labeled_statement)
            }
            NodeKindData::SwitchStatement(switch_statement) => {
                self.compile_switch(switch_statement)
            }
            NodeKindData::TryStatement(try_statement) => self.compile_try(try_statement),
            NodeKindData::WithStatement(with_statement) => {
                let object = self.compile_expr(with_statement.object(self.ast));
                let object = self.cache(object);
                let NodeKindData::IdentifierReference(object) =
                    self.ast.kind_data(object.node_id())
                else {
                    unreachable!("cached with object is an identifier")
                };
                self.with_stack.push(object.name(self.ast));
                self.compile_stmt(with_statement.body(self.ast));
                self.with_stack.pop();
            }
            NodeKindData::BreakStatement(break_statement) => {
                let name = break_statement
                    .label(self.ast)
                    .map(|label| self.ast.get_utf8(label.name(self.ast)).as_bytes().to_vec());
                let target = self
                    .control
                    .iter()
                    .rev()
                    .find(|target| {
                        name.as_ref()
                            .is_none_or(|name| target.name.as_ref() == Some(name))
                    })
                    .expect("break statement has a control-flow target")
                    .break_label;
                let jump = self.jump(target);
                self.emit(jump);
            }
            NodeKindData::ContinueStatement(continue_statement) => {
                let name = continue_statement
                    .label(self.ast)
                    .map(|label| self.ast.get_utf8(label.name(self.ast)).as_bytes().to_vec());
                let target = self
                    .control
                    .iter()
                    .rev()
                    .find(|target| {
                        name.as_ref()
                            .is_none_or(|name| target.name.as_ref() == Some(name))
                    })
                    .and_then(|target| target.continue_label)
                    .expect("continue statement has a loop target");
                let jump = self.jump(target);
                self.emit(jump);
            }
            _ if !self.facts.contains(statement) => self.emit(statement),
            _ => panic!(
                "generator lowering is missing statement support for {:?}",
                self.ast.node_kind(statement.node_id())
            ),
        }
    }

    fn compile_if(&mut self, statement: swc_next_ecma_ast::IfStatement) {
        let test = self.compile_expr(statement.test(self.ast));
        let then_label = self.new_case();
        let else_label = self.new_case();
        let end_label = self.new_case();
        self.emit_conditional_jump(test, else_label, true);

        self.current = then_label;
        self.compile_stmt(statement.consequent(self.ast));
        if !self.case_terminated(self.current) {
            let jump = self.jump(end_label);
            self.emit(jump);
        }

        self.current = else_label;
        if let Some(alternate) = statement.alternate(self.ast) {
            self.compile_stmt(alternate);
        }
        if !self.case_terminated(self.current) {
            let jump = self.jump(end_label);
            self.emit(jump);
        }
        self.current = end_label;
    }

    fn compile_while(&mut self, test_node: Expr, body: Stmt, name: Option<Vec<u8>>) {
        let head = self.new_case();
        let loop_body = self.new_case();
        let end = self.new_case();
        let jump = self.jump(head);
        self.emit(jump);

        self.current = head;
        let test = self.compile_expr(test_node);
        self.emit_conditional_jump(test, end, true);

        self.control.push(ControlTarget {
            break_label: end,
            continue_label: Some(head),
            name,
        });
        self.current = loop_body;
        self.compile_stmt(body);
        self.control.pop();
        if !self.case_terminated(self.current) {
            let jump = self.jump(head);
            self.emit(jump);
        }
        self.current = end;
    }

    fn compile_do_while(&mut self, body: Stmt, test_node: Expr, name: Option<Vec<u8>>) {
        let loop_body = self.new_case();
        let test_label = self.new_case();
        let end = self.new_case();
        let jump = self.jump(loop_body);
        self.emit(jump);

        self.control.push(ControlTarget {
            break_label: end,
            continue_label: Some(test_label),
            name,
        });
        self.current = loop_body;
        self.compile_stmt(body);
        self.control.pop();
        if !self.case_terminated(self.current) {
            let jump = self.jump(test_label);
            self.emit(jump);
        }

        self.current = test_label;
        let test = self.compile_expr(test_node);
        self.emit_conditional_jump(test, loop_body, false);
        if !self.case_terminated(self.current) {
            let jump = self.jump(end);
            self.emit(jump);
        }
        self.current = end;
    }

    fn compile_for(&mut self, statement: swc_next_ecma_ast::ForStatement, name: Option<Vec<u8>>) {
        if let Some(init) = statement.init(self.ast) {
            match self.ast.for_statement_init_data(init) {
                ForStatementInitData::VariableDeclaration(declaration) => {
                    self.compile_variable_declaration(declaration);
                }
                ForStatementInitData::Expr(expression) => {
                    let value = self.compile_expr(expression);
                    let statement = self.ast.stmt_expression_statement(DUMMY_SP, value);
                    self.emit(statement);
                }
            }
        }
        let head = self.new_case();
        let loop_body = self.new_case();
        let update = self.new_case();
        let end = self.new_case();
        let jump = self.jump(head);
        self.emit(jump);

        self.current = head;
        if let Some(test_node) = statement.test(self.ast) {
            let test = self.compile_expr(test_node);
            self.emit_conditional_jump(test, end, true);
        }

        self.control.push(ControlTarget {
            break_label: end,
            continue_label: Some(update),
            name,
        });
        self.current = loop_body;
        self.compile_stmt(statement.body(self.ast));
        self.control.pop();
        if !self.case_terminated(self.current) {
            let jump = self.jump(update);
            self.emit(jump);
        }

        self.current = update;
        if let Some(update_node) = statement.update(self.ast) {
            let value = self.compile_expr(update_node);
            let statement = self.ast.stmt_expression_statement(DUMMY_SP, value);
            self.emit(statement);
        }
        if !self.case_terminated(self.current) {
            let jump = self.jump(head);
            self.emit(jump);
        }
        self.current = end;
    }

    fn compile_labeled(&mut self, statement: swc_next_ecma_ast::LabeledStatement) {
        let label = statement.label(self.ast);
        let name = self.ast.get_utf8(label.name(self.ast)).as_bytes().to_vec();
        let body = statement.body(self.ast);
        match self.ast.kind_data(body.node_id()) {
            NodeKindData::WhileStatement(while_statement) => self.compile_while(
                while_statement.test(self.ast),
                while_statement.body(self.ast),
                Some(name),
            ),
            NodeKindData::DoWhileStatement(do_while) => {
                self.compile_do_while(do_while.body(self.ast), do_while.test(self.ast), Some(name))
            }
            NodeKindData::ForStatement(for_statement) => {
                self.compile_for(for_statement, Some(name));
            }
            NodeKindData::ForInStatement(for_in_statement) => {
                self.compile_for_in(for_in_statement, Some(name));
            }
            _ => {
                let end = self.new_case();
                self.control.push(ControlTarget {
                    break_label: end,
                    continue_label: None,
                    name: Some(name),
                });
                self.compile_stmt(body);
                self.control.pop();
                if !self.case_terminated(self.current) {
                    let jump = self.jump(end);
                    self.emit(jump);
                }
                self.current = end;
            }
        }
    }

    fn compile_for_in(
        &mut self,
        statement: swc_next_ecma_ast::ForInStatement,
        name: Option<Vec<u8>>,
    ) {
        let target = match self.ast.for_statement_left_data(statement.left(self.ast)) {
            ForStatementLeftData::VariableDeclaration(declaration) => {
                let declarators = self.nodes(declaration.declarators(self.ast));
                assert_eq!(declarators.len(), 1, "for-in has one binding");
                let pattern = declarators[0].id(self.ast);
                let declaration = self.ast.variable_declarator(DUMMY_SP, pattern, None, false);
                self.hoisted_vars.push(declaration);
                self.binding_to_assignment(pattern)
            }
            ForStatementLeftData::AssignmentTarget(target) => target,
        };
        let right = self.compile_expr(statement.right(self.ast));
        let keys = self.new_temp();
        let key = self.new_temp();
        let index = self.new_temp();
        let empty = self
            .ast
            .expr_array_expression(DUMMY_SP, TypedSubRange::empty());
        self.emit_assignment(keys, empty);

        let keys_expr = self.private_reference(keys);
        let push_name = self.ast.add_utf8("push");
        let push_key = self.ast.property_key_identifier_name(DUMMY_SP, push_name);
        let push = self
            .ast
            .expr_member_expression(DUMMY_SP, keys_expr, push_key, false, false);
        let key_expr = self.private_reference(key);
        let arguments = self.ast.add_typed_sub_range([Argument::Expr(key_expr)]);
        let push = self
            .ast
            .expr_call_expression(DUMMY_SP, push, arguments, false, None);
        let push = self.ast.stmt_expression_statement(DUMMY_SP, push);
        let collection_left = self
            .ast
            .for_statement_left_assignment_target_simple_assignment_target_identifier_reference(
                DUMMY_SP, key.text,
            );
        self.ast
            .set_syntax_context(collection_left.node_id(), key.context);
        let collection = self
            .ast
            .stmt_for_in_statement(DUMMY_SP, collection_left, right, push);
        self.emit(collection);
        let zero = self.number(0);
        self.emit_assignment(index, zero);

        let head = self.new_case();
        let loop_body = self.new_case();
        let update = self.new_case();
        let end = self.new_case();
        let jump = self.jump(head);
        self.emit(jump);

        self.current = head;
        let index_expr = self.private_reference(index);
        let keys_expr = self.private_reference(keys);
        let length_name = self.ast.add_utf8("length");
        let length_key = self.ast.property_key_identifier_name(DUMMY_SP, length_name);
        let length = self
            .ast
            .expr_member_expression(DUMMY_SP, keys_expr, length_key, false, false);
        let test =
            self.ast
                .expr_binary_expression(DUMMY_SP, index_expr, length, BinaryOperator::LessThan);
        self.emit_conditional_jump(test, end, true);

        self.current = loop_body;
        let keys_expr = self.private_reference(keys);
        let index_expr = self.private_reference(index);
        let key_value = self.ast.expr_member_expression(
            DUMMY_SP,
            keys_expr,
            PropertyKey::Expr(index_expr),
            true,
            false,
        );
        let assignment = self.ast.expr_assignment_expression(
            DUMMY_SP,
            target,
            key_value,
            AssignmentOperator::Assign,
        );
        let assignment = self.ast.stmt_expression_statement(DUMMY_SP, assignment);
        self.emit(assignment);
        self.control.push(ControlTarget {
            break_label: end,
            continue_label: Some(update),
            name,
        });
        self.compile_stmt(statement.body(self.ast));
        self.control.pop();
        if !self.case_terminated(self.current) {
            let jump = self.jump(update);
            self.emit(jump);
        }

        self.current = update;
        let index_expr = self.private_reference(index);
        let one = self.number(1);
        let next = self
            .ast
            .expr_binary_expression(DUMMY_SP, index_expr, one, BinaryOperator::Add);
        self.emit_assignment(index, next);
        let jump = self.jump(head);
        self.emit(jump);
        self.current = end;
    }

    fn compile_switch(&mut self, statement: swc_next_ecma_ast::SwitchStatement) {
        let discriminant = self.compile_expr(statement.discriminant(self.ast));
        let discriminant = self.cache(discriminant);
        let cases = self.nodes(statement.cases(self.ast));
        let body_labels: Vec<_> = (0..cases.len()).map(|_| self.new_case()).collect();
        let end = self.new_case();
        let default_label = cases
            .iter()
            .position(|case| case.test(self.ast).is_none())
            .map_or(end, |index| body_labels[index]);

        for (index, case) in cases.iter().copied().enumerate() {
            let Some(test_node) = case.test(self.ast) else {
                continue;
            };
            let test = self.compile_expr(test_node);
            let equal = self.ast.expr_binary_expression(
                DUMMY_SP,
                discriminant,
                test,
                BinaryOperator::StrictEqual,
            );
            self.emit_conditional_jump(equal, body_labels[index], false);
        }
        let jump = self.jump(default_label);
        self.emit(jump);

        self.control.push(ControlTarget {
            break_label: end,
            continue_label: None,
            name: None,
        });
        for (index, case) in cases.iter().copied().enumerate() {
            self.current = body_labels[index];
            for statement in self.nodes(case.consequent(self.ast)) {
                self.compile_stmt(statement);
            }
            if !self.case_terminated(self.current) {
                let next = body_labels.get(index + 1).copied().unwrap_or(end);
                let jump = self.jump(next);
                self.emit(jump);
            }
        }
        self.control.pop();
        self.current = end;
    }

    fn compile_try(&mut self, statement: swc_next_ecma_ast::TryStatement) {
        let try_label = self.new_case();
        let catch_label = statement.handler(self.ast).map(|_| self.new_case());
        let finally_label = statement.finalizer(self.ast).map(|_| self.new_case());
        let end = self.new_case();
        self.emit_try_push(try_label, catch_label, finally_label, end);
        let jump = self.jump(try_label);
        self.emit(jump);

        self.current = try_label;
        let block = statement.block(self.ast);
        for child in self.nodes(block.body(self.ast)) {
            self.compile_stmt(child);
        }
        if !self.case_terminated(self.current) {
            let jump = self.jump(finally_label.unwrap_or(end));
            self.emit(jump);
        }

        if let (Some(handler), Some(catch_label)) = (statement.handler(self.ast), catch_label) {
            self.current = catch_label;
            if let Some(pattern) = handler.param(self.ast) {
                let source = self.state_call("sent");
                let target = self.binding_to_assignment(pattern);
                let assignment = self.ast.expr_assignment_expression(
                    DUMMY_SP,
                    target,
                    source,
                    AssignmentOperator::Assign,
                );
                let statement = self.ast.stmt_expression_statement(DUMMY_SP, assignment);
                self.emit(statement);
                let declaration = self.ast.variable_declarator(DUMMY_SP, pattern, None, false);
                self.hoisted_vars.push(declaration);
            } else {
                let sent = self.state_call("sent");
                let statement = self.ast.stmt_expression_statement(DUMMY_SP, sent);
                self.emit(statement);
            }
            for child in self.nodes(handler.body(self.ast).body(self.ast)) {
                self.compile_stmt(child);
            }
            if !self.case_terminated(self.current) {
                let jump = self.jump(finally_label.unwrap_or(end));
                self.emit(jump);
            }
        }

        if let (Some(finalizer), Some(finally_label)) =
            (statement.finalizer(self.ast), finally_label)
        {
            self.current = finally_label;
            for child in self.nodes(finalizer.body(self.ast)) {
                self.compile_stmt(child);
            }
            if !self.case_terminated(self.current) {
                let end_finally = self.return_op(7, None);
                self.emit(end_finally);
            }
        }
        self.current = end;
    }

    fn compile_variable_declaration(
        &mut self,
        declaration: swc_next_ecma_ast::VariableDeclaration,
    ) {
        for id in declaration.declarators(self.ast).iter() {
            let declarator = self.ast.get_node_in_sub_range(id);
            let pattern = declarator.id(self.ast);
            let outer = self.ast.variable_declarator(
                DUMMY_SP,
                pattern,
                None,
                declarator.definite(self.ast),
            );
            self.hoisted_vars.push(outer);
            if let Some(init) = declarator.init(self.ast) {
                let right = self.compile_expr(init);
                let left = self.binding_to_assignment(pattern);
                let assignment = self.ast.expr_assignment_expression(
                    DUMMY_SP,
                    left,
                    right,
                    AssignmentOperator::Assign,
                );
                let statement = self.ast.stmt_expression_statement(DUMMY_SP, assignment);
                self.emit(statement);
            }
        }
    }

    fn binding_to_assignment(&mut self, pattern: BindingPattern) -> AssignmentTarget {
        match self.ast.kind_data(pattern.node_id()) {
            NodeKindData::BindingIdentifier(identifier) => self
                .ast
                .assignment_target_simple_assignment_target_identifier_reference(
                    DUMMY_SP,
                    identifier.name(self.ast),
                ),
            _ => panic!("destructuring must run before generator lowering"),
        }
    }
}
