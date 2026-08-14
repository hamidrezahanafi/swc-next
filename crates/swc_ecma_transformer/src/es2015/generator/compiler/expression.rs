use super::*;

impl<'names, 'ast> GeneratorCompiler<'names, 'ast> {
    pub(super) fn compile_expr(&mut self, expression: Expr) -> Expr {
        match self.ast.kind_data(expression.node_id()) {
            NodeKindData::YieldExpression(yield_expression) => {
                let value = yield_expression
                    .argument(self.ast)
                    .map(|argument| self.compile_expr(argument));
                let opcode = if yield_expression.delegate(self.ast) {
                    5
                } else {
                    4
                };
                let statement = self.return_op(opcode, value);
                self.emit(statement);
                self.start_new_case();
                self.state_call("sent")
            }
            NodeKindData::ParenthesizedExpression(parenthesized) => {
                let inner = self.compile_expr(parenthesized.expression(self.ast));
                parenthesized.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::BinaryExpression(binary) => self.compile_binary(expression, binary),
            NodeKindData::LogicalExpression(logical) => self.compile_logical(
                logical.left(self.ast),
                logical.right(self.ast),
                logical.operator(self.ast),
            ),
            NodeKindData::ConditionalExpression(conditional) => self.compile_conditional(
                conditional.test(self.ast),
                conditional.consequent(self.ast),
                conditional.alternate(self.ast),
            ),
            NodeKindData::UnaryExpression(unary) => {
                let argument = self.compile_expr(unary.argument(self.ast));
                unary.set_argument(self.ast, argument);
                expression
            }
            NodeKindData::UpdateExpression(update) => {
                if let NodeKindData::MemberExpression(member) =
                    self.ast.kind_data(update.argument(self.ast).node_id())
                {
                    let object = self.compile_expr(member.object(self.ast));
                    let property = member.property(self.ast);
                    let object = if self.contains_yield(property) {
                        self.cache(object)
                    } else {
                        object
                    };
                    member.set_object(self.ast, object);
                    if let PropertyKeyData::Expr(property_expr) =
                        self.ast.property_key_data(property)
                    {
                        let property_expr = self.compile_expr(property_expr);
                        member.set_property(self.ast, PropertyKey::Expr(property_expr));
                    }
                }
                expression
            }
            NodeKindData::AssignmentExpression(assignment) => {
                let right_node = assignment.right(self.ast);
                let left = self.compile_assignment_target(
                    assignment.left(self.ast),
                    self.contains_yield(right_node),
                );
                let right = self.compile_expr(right_node);
                assignment.set_left(self.ast, left);
                assignment.set_right(self.ast, right);
                expression
            }
            NodeKindData::MemberExpression(member) => {
                let object = self.compile_expr(member.object(self.ast));
                let property = member.property(self.ast);
                let object = if self.contains_yield(property) {
                    self.cache(object)
                } else {
                    object
                };
                member.set_object(self.ast, object);
                if let PropertyKeyData::Expr(property_expr) = self.ast.property_key_data(property) {
                    let property_expr = self.compile_expr(property_expr);
                    member.set_property(self.ast, PropertyKey::Expr(property_expr));
                }
                expression
            }
            NodeKindData::CallExpression(call) => {
                let arguments = self.nodes(call.arguments(self.ast));
                let callee = self.compile_expr(call.callee(self.ast));
                let callee = if arguments
                    .iter()
                    .any(|argument| self.contains_yield(*argument))
                {
                    self.preserve_callee(callee)
                } else {
                    callee
                };
                call.set_callee(self.ast, callee);
                let arguments = self.compile_arguments(arguments);
                let arguments = self.ast.add_typed_sub_range(arguments);
                call.set_arguments(self.ast, arguments);
                expression
            }
            NodeKindData::NewExpression(new_expression) => {
                let arguments = self.nodes(new_expression.arguments(self.ast));
                let callee = self.compile_expr(new_expression.callee(self.ast));
                let callee = if arguments
                    .iter()
                    .any(|argument| self.contains_yield(*argument))
                {
                    self.cache(callee)
                } else {
                    callee
                };
                new_expression.set_callee(self.ast, callee);
                let arguments = self.compile_arguments(arguments);
                let arguments = self.ast.add_typed_sub_range(arguments);
                new_expression.set_arguments(self.ast, arguments);
                expression
            }
            NodeKindData::ArrayExpression(array) => {
                let elements = self.nodes(array.elements(self.ast));
                let mut compiled = Vec::with_capacity(elements.len());
                for (index, element) in elements.iter().copied().enumerate() {
                    let later_yield = elements[index + 1..]
                        .iter()
                        .flatten()
                        .any(|argument| self.contains_yield(*argument));
                    let element =
                        element.map(|argument| self.compile_argument(argument, later_yield));
                    compiled.push(element);
                }
                let compiled = self.ast.add_typed_sub_range(compiled);
                array.set_elements(self.ast, compiled);
                expression
            }
            NodeKindData::ObjectExpression(object) => {
                let properties = self.nodes(object.properties(self.ast));
                let mut compiled = Vec::with_capacity(properties.len());
                for (index, property) in properties.iter().copied().enumerate() {
                    let later_yield = properties[index + 1..]
                        .iter()
                        .any(|property| self.contains_yield(*property));
                    compiled.push(self.compile_object_property(property, later_yield));
                }
                let compiled = self.ast.add_typed_sub_range(compiled);
                object.set_properties(self.ast, compiled);
                expression
            }
            NodeKindData::Class(class) => {
                self.compile_class(class);
                expression
            }
            NodeKindData::TemplateLiteral(template) => {
                self.compile_template(template);
                expression
            }
            NodeKindData::TaggedTemplateExpression(tagged) => {
                let quasi = tagged.quasi(self.ast);
                let tag = self.compile_expr(tagged.tag(self.ast));
                let tag = if self.contains_yield(quasi) {
                    self.preserve_callee(tag)
                } else {
                    tag
                };
                tagged.set_tag(self.ast, tag);
                self.compile_template(quasi);
                expression
            }
            NodeKindData::ChainExpression(chain) => {
                let inner = self.compile_expr(chain.expression(self.ast));
                chain.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::AwaitExpression(await_expression) => {
                let argument = self.compile_expr(await_expression.argument(self.ast));
                await_expression.set_argument(self.ast, argument);
                expression
            }
            NodeKindData::TsAsExpression(wrapper) => {
                let inner = self.compile_expr(wrapper.expression(self.ast));
                wrapper.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::TsSatisfiesExpression(wrapper) => {
                let inner = self.compile_expr(wrapper.expression(self.ast));
                wrapper.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::TsTypeAssertion(wrapper) => {
                let inner = self.compile_expr(wrapper.expression(self.ast));
                wrapper.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::TsNonNullExpression(wrapper) => {
                let inner = self.compile_expr(wrapper.expression(self.ast));
                wrapper.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::TsInstantiationExpression(wrapper) => {
                let inner = self.compile_expr(wrapper.expression(self.ast));
                wrapper.set_expression(self.ast, inner);
                expression
            }
            NodeKindData::SequenceExpression(sequence) => {
                let expressions = self.nodes(sequence.expressions(self.ast));
                let zero = self.ast.add_utf8("0");
                let mut last =
                    self.ast
                        .expr_numeric_literal(DUMMY_SP, NumericLiteralKind::Decimal, zero);
                for (index, child) in expressions.iter().copied().enumerate() {
                    let value = self.compile_expr(child);
                    if index + 1 == expressions.len() {
                        last = value;
                    } else {
                        let statement = self.ast.stmt_expression_statement(DUMMY_SP, value);
                        self.emit(statement);
                    }
                }
                last
            }
            _ if !self.contains_yield(expression) => expression,
            _ => panic!(
                "generator lowering is missing expression support for {:?}",
                self.ast.node_kind(expression.node_id())
            ),
        }
    }

    fn compile_binary(&mut self, expression: Expr, binary: BinaryExpression) -> Expr {
        let left = self.compile_expr(binary.left(self.ast));
        let right_node = binary.right(self.ast);
        let left = if self.contains_yield(right_node) {
            self.cache(left)
        } else {
            left
        };
        let right = self.compile_expr(right_node);
        binary.set_left(self.ast, left);
        binary.set_right(self.ast, right);
        expression
    }

    fn compile_class(&mut self, class: swc_next_ecma_ast::Class) {
        self.compile_decorators(self.nodes(class.decorators(self.ast)));
        if let Some(super_class) = class.super_class(self.ast) {
            let super_class = self.compile_expr(super_class);
            class.set_super_class(self.ast, Some(super_class));
        }

        let body = class.body(self.ast);
        for element in self.nodes(body.body(self.ast)) {
            match self.ast.class_element_data(element) {
                ClassElementData::MethodDefinition(method) => {
                    self.compile_decorators(self.nodes(method.decorators(self.ast)));
                    self.compile_property_key(method.key(self.ast), |compiler, key| {
                        method.set_key(compiler.ast, key);
                    });
                }
                ClassElementData::PropertyDefinition(property) => {
                    self.compile_decorators(self.nodes(property.decorators(self.ast)));
                    self.compile_property_key(property.key(self.ast), |compiler, key| {
                        property.set_key(compiler.ast, key);
                    });
                }
                ClassElementData::TsMethodDefinition(method) => {
                    self.compile_decorators(self.nodes(method.decorators(self.ast)));
                    self.compile_property_key(method.key(self.ast), |compiler, key| {
                        method.set_key(compiler.ast, key);
                    });
                }
                ClassElementData::StaticBlock(_) | ClassElementData::TsIndexSignature(_) => {}
            }
        }
    }

    fn compile_decorators(&mut self, decorators: Vec<Decorator>) {
        for decorator in decorators {
            let expression = self.compile_expr(decorator.expression(self.ast));
            decorator.set_expression(self.ast, expression);
        }
    }

    fn compile_property_key(&mut self, key: PropertyKey, set: impl FnOnce(&mut Self, PropertyKey)) {
        if let PropertyKeyData::Expr(expression) = self.ast.property_key_data(key) {
            let expression = self.compile_expr(expression);
            set(self, PropertyKey::Expr(expression));
        }
    }

    fn compile_template(&mut self, template: swc_next_ecma_ast::TemplateLiteral) {
        let expressions = self.nodes(template.expressions(self.ast));
        let mut compiled = Vec::with_capacity(expressions.len());
        for (index, expression) in expressions.iter().copied().enumerate() {
            let later_yield = expressions[index + 1..]
                .iter()
                .any(|expression| self.contains_yield(*expression));
            let expression = self.compile_expr(expression);
            compiled.push(if later_yield {
                self.cache(expression)
            } else {
                expression
            });
        }
        let compiled = self.ast.add_typed_sub_range(compiled);
        template.set_expressions(self.ast, compiled);
    }

    fn compile_assignment_target(
        &mut self,
        target: AssignmentTarget,
        preserve: bool,
    ) -> AssignmentTarget {
        if let NodeKindData::MemberExpression(member) = self.ast.kind_data(target.node_id()) {
            let object = self.compile_expr(member.object(self.ast));
            let property = member.property(self.ast);
            let object = if preserve || self.contains_yield(property) {
                self.cache(object)
            } else {
                object
            };
            member.set_object(self.ast, object);
            if let PropertyKeyData::Expr(property_expr) = self.ast.property_key_data(property) {
                let property_expr = self.compile_expr(property_expr);
                let property_expr = if preserve {
                    self.cache(property_expr)
                } else {
                    property_expr
                };
                member.set_property(self.ast, PropertyKey::Expr(property_expr));
            }
        } else if self.contains_yield(target) {
            panic!(
                "generator lowering requires destructuring assignment targets to be lowered first"
            );
        }
        target
    }

    fn compile_conditional(&mut self, test_node: Expr, consequent: Expr, alternate: Expr) -> Expr {
        let result = self.new_temp();
        let test = self.compile_expr(test_node);
        let then_label = self.new_case();
        let else_label = self.new_case();
        let end_label = self.new_case();
        self.emit_conditional_jump(test, else_label, true);

        self.current = then_label;
        let value = self.compile_expr(consequent);
        self.emit_assignment(result, value);
        let jump = self.jump(end_label);
        self.emit(jump);

        self.current = else_label;
        let value = self.compile_expr(alternate);
        self.emit_assignment(result, value);
        let jump = self.jump(end_label);
        self.emit(jump);

        self.current = end_label;
        self.private_reference(result)
    }

    fn compile_logical(&mut self, left: Expr, right: Expr, operator: LogicalOperator) -> Expr {
        let left = self.compile_expr(left);
        let result = self.new_temp();
        self.emit_assignment(result, left);
        let right_label = self.new_case();
        let end_label = self.new_case();
        let result_expr = self.private_reference(result);
        match operator {
            LogicalOperator::And => self.emit_conditional_jump(result_expr, end_label, true),
            LogicalOperator::Or => self.emit_conditional_jump(result_expr, end_label, false),
            LogicalOperator::NullishCoalescing => {
                let null = self.ast.expr_null_literal(DUMMY_SP);
                let test = self.ast.expr_binary_expression(
                    DUMMY_SP,
                    result_expr,
                    null,
                    BinaryOperator::NotEqual,
                );
                self.emit_conditional_jump(test, end_label, false);
            }
        }
        self.current = right_label;
        let right = self.compile_expr(right);
        self.emit_assignment(result, right);
        let jump = self.jump(end_label);
        self.emit(jump);
        self.current = end_label;
        self.private_reference(result)
    }

    fn compile_arguments(&mut self, arguments: Vec<Argument>) -> Vec<Argument> {
        let mut compiled = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.iter().copied().enumerate() {
            let later_yield = arguments[index + 1..]
                .iter()
                .any(|argument| self.contains_yield(*argument));
            compiled.push(self.compile_argument(argument, later_yield));
        }
        compiled
    }

    fn compile_argument(&mut self, argument: Argument, preserve: bool) -> Argument {
        match self.ast.argument_data(argument) {
            ArgumentData::Expr(expression) => {
                let expression = self.compile_expr(expression);
                let expression = if preserve {
                    self.cache(expression)
                } else {
                    expression
                };
                Argument::Expr(expression)
            }
            ArgumentData::SpreadElement(spread) => {
                let expression = self.compile_expr(spread.argument(self.ast));
                let expression = if preserve {
                    self.cache(expression)
                } else {
                    expression
                };
                spread.set_argument(self.ast, expression);
                Argument::SpreadElement(spread)
            }
        }
    }

    fn compile_object_property(
        &mut self,
        property: ObjectPropertyKind,
        preserve: bool,
    ) -> ObjectPropertyKind {
        match self.ast.object_property_kind_data(property) {
            ObjectPropertyKindData::SpreadElement(spread) => {
                let value = self.compile_expr(spread.argument(self.ast));
                let value = if preserve { self.cache(value) } else { value };
                spread.set_argument(self.ast, value);
                ObjectPropertyKind::SpreadElement(spread)
            }
            ObjectPropertyKindData::ObjectProperty(object_property) => {
                let key = object_property.key(self.ast);
                let mut key = key;
                if let PropertyKeyData::Expr(key_expr) = self.ast.property_key_data(key) {
                    let key_expr = self.compile_expr(key_expr);
                    let key_expr =
                        if self.contains_yield(object_property.value(self.ast)) || preserve {
                            self.cache(key_expr)
                        } else {
                            key_expr
                        };
                    key = PropertyKey::Expr(key_expr);
                    object_property.set_key(self.ast, key);
                }
                let value = self.compile_expr(object_property.value(self.ast));
                let value = if preserve { self.cache(value) } else { value };
                object_property.set_value(self.ast, value);
                ObjectPropertyKind::ObjectProperty(object_property)
            }
        }
    }

    fn preserve_callee(&mut self, callee: Expr) -> Expr {
        if let NodeKindData::MemberExpression(member) = self.ast.kind_data(callee.node_id()) {
            let object = self.cache(member.object(self.ast));
            member.set_object(self.ast, object);
            let property = member.property(self.ast);
            if let PropertyKeyData::Expr(property_expr) = self.ast.property_key_data(property) {
                let property_expr = self.cache(property_expr);
                member.set_property(self.ast, PropertyKey::Expr(property_expr));
            }
            callee
        } else {
            self.cache(callee)
        }
    }
}
