use super::{StatementExpansion, TypeScriptLowerer, erase::update_binding_property_shorthand};
use swc_next_ecma_ast::{
    Accessibility, ArrowFunctionExpression, Ast, BindingProperty, CallExpression, CatchClause,
    Class, ClassElement, DUMMY_SP, Expr, ExprData, FormalParameter, FormalParameterRest, GetSpan,
    JsxClosingElement, JsxOpeningElement, MethodDefinition, NewExpression, ObjectProperty,
    PropertyDefinition, SimpleAssignmentTarget, SimpleAssignmentTargetData, Stmt, SubRange,
    SwitchCase, SwitchStatement, TaggedTemplateExpression, VariableDeclaration, VariableDeclarator,
    VisitMut, VisitMutWith,
};

impl<'id, 'ast> VisitMut<'ast, 'id> for TypeScriptLowerer<'_, 'ast, 'id> {
    fn ast(&mut self) -> &mut Ast<'ast, 'id> {
        self.ast
    }

    fn visit_mut_stmt(&mut self, statement: Stmt<'id>) -> Option<Stmt<'id>> {
        match self.lower_statement(statement)? {
            StatementExpansion::Remove => {
                Some(Stmt::EmptyStatement(self.ast().empty_statement(DUMMY_SP)))
            }
            StatementExpansion::Single(statement) => Some(statement),
            StatementExpansion::Pair(first, second) => {
                let body = self.ast().add_sub_range([first, second]);
                Some(Stmt::BlockStatement(
                    self.ast().block_statement(DUMMY_SP, body),
                ))
            }
        }
    }

    fn visit_mut_stmts(
        &mut self,
        statements: SubRange<'id, Stmt<'id>>,
    ) -> Option<SubRange<'id, Stmt<'id>>> {
        self.lower_statement_list(statements)
    }

    fn visit_mut_switch_statement(
        &mut self,
        statement: SwitchStatement<'id>,
    ) -> Option<SwitchStatement<'id>> {
        let discriminant = statement.discriminant(self.ast_ref());
        if let Some(discriminant) = discriminant.visit_mut_with(self) {
            statement.set_discriminant(self.ast(), discriminant);
        }

        self.enter_binding_scope();
        let cases = statement.cases(self.ast_ref());
        if self.analysis.has_merged_runtime_bindings() {
            for case_id in cases.iter() {
                let case = self.ast_ref().get_node_in_sub_range(case_id);
                self.register_scope_runtime_bindings(case.consequent(self.ast_ref()));
            }
        }
        if let Some(cases) = cases.visit_mut_with(self) {
            statement.set_cases(self.ast(), cases);
        }
        self.exit_binding_scope();
        None
    }

    fn visit_mut_switch_case(&mut self, case: SwitchCase<'id>) -> Option<SwitchCase<'id>> {
        let test = case.test(self.ast_ref());
        if let Some(test) = test.visit_mut_with(self) {
            case.set_test(self.ast(), test);
        }

        let consequent = case.consequent(self.ast_ref());
        if let Some(consequent) = self.lower_statement_list_in_current_scope(consequent) {
            case.set_consequent(self.ast(), consequent);
        }
        None
    }

    fn visit_mut_class_elements(
        &mut self,
        elements: SubRange<'id, ClassElement<'id>>,
    ) -> Option<SubRange<'id, ClassElement<'id>>> {
        self.lower_class_elements(elements)
    }

    fn visit_mut_expr(&mut self, expression: Expr<'id>) -> Option<Expr<'id>> {
        if let Some(value) = self.analysis.inline_const_enum_use(expression.node_id()) {
            return Some(self.inline_enum_constant(expression, value));
        }
        let inner = match self.ast_ref().expr_data(expression) {
            ExprData::IdentifierReference(identifier) => {
                return self.namespace_identifier_reference(identifier);
            }
            ExprData::TsAsExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsSatisfiesExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsTypeAssertion(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsNonNullExpression(wrapper) => Some(wrapper.expression(self.ast())),
            ExprData::TsInstantiationExpression(wrapper) => Some(wrapper.expression(self.ast())),
            _ => None,
        };
        if let Some(inner) = inner {
            let removed_span = expression.span(self.ast_ref());
            Some(self.unwrap_expression(inner, removed_span))
        } else {
            expression.visit_mut_children_with(self)
        }
    }

    fn visit_mut_object_property(
        &mut self,
        property: ObjectProperty<'id>,
    ) -> Option<ObjectProperty<'id>> {
        property.visit_mut_children_with(self);
        let shorthand_value_is_identifier = matches!(
            self.ast_ref().expr_data(property.value(self.ast_ref())),
            ExprData::IdentifierReference(_)
        );
        if property.shorthand(self.ast_ref()) && !shorthand_value_is_identifier {
            property.set_shorthand(self.ast(), false);
        }
        None
    }

    fn visit_mut_binding_property(
        &mut self,
        property: BindingProperty<'id>,
    ) -> Option<BindingProperty<'id>> {
        property.visit_mut_children_with(self);
        update_binding_property_shorthand(self.ast(), property);
        None
    }

    fn visit_mut_simple_assignment_target(
        &mut self,
        target: SimpleAssignmentTarget<'id>,
    ) -> Option<SimpleAssignmentTarget<'id>> {
        if let SimpleAssignmentTargetData::IdentifierReference(identifier) =
            self.ast().simple_assignment_target_data(target)
            && let Some(replacement) = self.namespace_identifier_target(identifier)
        {
            return Some(replacement);
        }
        let inner = match self.ast().simple_assignment_target_data(target) {
            SimpleAssignmentTargetData::TsAsExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsSatisfiesExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsTypeAssertion(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            SimpleAssignmentTargetData::TsNonNullExpression(wrapper) => {
                Some(wrapper.expression(self.ast()))
            }
            _ => None,
        };
        if let Some(inner) = inner {
            let removed_span = target.span(self.ast_ref());
            let replacement = self.unwrap_expression(inner, removed_span);
            Some(self.expression_to_assignment_target(replacement))
        } else {
            target.visit_mut_children_with(self)
        }
    }

    fn visit_mut_formal_parameter(
        &mut self,
        parameter: FormalParameter<'id>,
    ) -> Option<FormalParameter<'id>> {
        parameter.set_type_annotation(self.ast(), None);
        parameter.set_optional(self.ast(), false);
        parameter.visit_mut_children_with(self)
    }

    fn visit_mut_formal_parameter_rest(
        &mut self,
        rest: FormalParameterRest<'id>,
    ) -> Option<FormalParameterRest<'id>> {
        rest.set_type_annotation(self.ast(), None);
        rest.visit_mut_children_with(self)
    }

    fn visit_mut_catch_clause(&mut self, clause: CatchClause<'id>) -> Option<CatchClause<'id>> {
        clause.set_type_annotation(self.ast(), None);
        clause.visit_mut_children_with(self)
    }

    fn visit_mut_variable_declaration(
        &mut self,
        declaration: VariableDeclaration<'id>,
    ) -> Option<VariableDeclaration<'id>> {
        declaration.set_declare(self.ast(), false);
        declaration.visit_mut_children_with(self)
    }

    fn visit_mut_variable_declarator(
        &mut self,
        declarator: VariableDeclarator<'id>,
    ) -> Option<VariableDeclarator<'id>> {
        declarator.set_definite(self.ast(), false);
        declarator.set_type_annotation(self.ast(), None);
        declarator.visit_mut_children_with(self)
    }

    fn visit_mut_class(&mut self, class: Class<'id>) -> Option<Class<'id>> {
        class.set_type_parameters(self.ast(), None);
        class.set_super_type_arguments(self.ast(), None);
        class.set_implements(self.ast(), SubRange::empty());
        class.set_abstract(self.ast(), false);
        class.set_declare(self.ast(), false);
        class.visit_mut_children_with(self);
        self.lower_parameter_properties(class);
        None
    }

    fn visit_mut_method_definition(
        &mut self,
        method: MethodDefinition<'id>,
    ) -> Option<MethodDefinition<'id>> {
        method.set_override(self.ast(), false);
        method.set_optional(self.ast(), false);
        method.set_abstract(self.ast(), false);
        method.set_accessibility(self.ast(), Accessibility::None);
        method.visit_mut_children_with(self)
    }

    fn visit_mut_property_definition(
        &mut self,
        property: PropertyDefinition<'id>,
    ) -> Option<PropertyDefinition<'id>> {
        property.set_type_annotation(self.ast(), None);
        property.set_declare(self.ast(), false);
        property.set_override(self.ast(), false);
        property.set_optional(self.ast(), false);
        property.set_definite(self.ast(), false);
        property.set_readonly(self.ast(), false);
        property.set_abstract(self.ast(), false);
        property.set_accessibility(self.ast(), Accessibility::None);
        property.visit_mut_children_with(self)
    }

    fn visit_mut_function(
        &mut self,
        function: swc_next_ecma_ast::Function<'id>,
    ) -> Option<swc_next_ecma_ast::Function<'id>> {
        function.set_type_parameters(self.ast(), None);
        function.set_this_param(self.ast(), None);
        function.set_return_type(self.ast(), None);
        function.visit_mut_children_with(self)
    }

    fn visit_mut_arrow_function_expression(
        &mut self,
        arrow: ArrowFunctionExpression<'id>,
    ) -> Option<ArrowFunctionExpression<'id>> {
        arrow.set_type_parameters(self.ast(), None);
        arrow.set_return_type(self.ast(), None);
        arrow.visit_mut_children_with(self)
    }

    fn visit_mut_call_expression(
        &mut self,
        call: CallExpression<'id>,
    ) -> Option<CallExpression<'id>> {
        call.set_type_arguments(self.ast(), None);
        call.visit_mut_children_with(self)
    }

    fn visit_mut_new_expression(
        &mut self,
        expression: NewExpression<'id>,
    ) -> Option<NewExpression<'id>> {
        expression.set_type_arguments(self.ast(), None);
        expression.visit_mut_children_with(self)
    }

    fn visit_mut_tagged_template_expression(
        &mut self,
        template: TaggedTemplateExpression<'id>,
    ) -> Option<TaggedTemplateExpression<'id>> {
        template.set_type_arguments(self.ast(), None);
        template.visit_mut_children_with(self)
    }

    fn visit_mut_jsx_opening_element(
        &mut self,
        element: JsxOpeningElement<'id>,
    ) -> Option<JsxOpeningElement<'id>> {
        element.set_type_arguments(self.ast(), None);
        if let Some(name) = self.namespace_jsx_name(element.name(self.ast_ref())) {
            element.set_name(self.ast(), name);
        }
        element.visit_mut_children_with(self)
    }

    fn visit_mut_jsx_closing_element(
        &mut self,
        element: JsxClosingElement<'id>,
    ) -> Option<JsxClosingElement<'id>> {
        if let Some(name) = self.namespace_jsx_name(element.name(self.ast_ref())) {
            element.set_name(self.ast(), name);
        }
        element.visit_mut_children_with(self)
    }
}
