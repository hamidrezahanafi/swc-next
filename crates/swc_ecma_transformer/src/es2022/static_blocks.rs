//! ES2022 static blocks transform.
//!
//! Static blocks before a static field are merged into that field's
//! initializer. Trailing blocks are saved in one closure and invoked after
//! class evaluation, matching Babel's standalone transform.

use std::collections::HashMap;

use swc_next_ecma_ast::{
    Accessibility, Argument, ArrowFunctionBody, AssignmentOperator, Ast, AstInternedUtf8, AstWtf8,
    Class, ClassBody, ClassElement, ClassType, DUMMY_SP, Expr, GetSpan, NodeId, NodeKindData,
    PropertyDefinition, PropertyKey, StaticBlock, Stmt, TypedSubRange, UnaryOperator,
};
use swc_next_ecma_semantic::{GeneratedNameState, Semantic};

use crate::state::TransformerCtx;

pub(super) struct StaticBlocks {
    expression_calls: HashMap<NodeId, Expr>,
    closure_names: Option<GeneratedNameState>,
}

struct ClassTransform<'visit, 'a> {
    ast: &'visit mut Ast<'a>,
    semantic: &'visit mut Semantic<'a>,
    closure_names: &'visit mut Option<GeneratedNameState>,
}

impl<'a> ClassTransform<'_, 'a> {
    fn next_closure_name(&mut self) -> AstInternedUtf8 {
        // UsedNameState currently provides one conservative program-wide
        // collision domain. The statement injector places each declaration in
        // its nearest function, static-block, or program scope.
        if self.closure_names.is_none() {
            *self.closure_names = Some(self.semantic.used_names_mut().generated_name_state(
                self.ast,
                "_staticBlock",
                2,
            ));
        }
        self.semantic.used_names_mut().next_name_from_state(
            self.ast,
            self.closure_names
                .as_mut()
                .expect("closure name state was initialized"),
        )
    }

    fn transform_class(
        &mut self,
        class: Class,
        is_expression: bool,
        inferred_name: Option<AstWtf8>,
    ) -> Option<(Expr, AstInternedUtf8)> {
        let body = class.body(self.ast);
        let mut elements = body.body(self.ast);
        let original_len = elements.len();
        let mut write_index = 0;
        let mut pending = Vec::new();
        let mut first_static_property = None;
        let mut last_static_property = None;

        for read_index in 0..original_len {
            let element_id = elements
                .get(read_index)
                .expect("read index must be inside the class body");
            let element = self.ast.get_node_in_sub_range(element_id);
            if let Some(block) = element.as_static_block(self.ast) {
                let is_first_pending = pending.is_empty();
                if let Some(expression) = self.block_to_expression(block, is_first_pending) {
                    pending.push(expression);
                }
                continue;
            }

            if let Some(property) = static_property(element, self.ast) {
                first_static_property.get_or_insert(property);
                last_static_property = Some(property);
                if !pending.is_empty() {
                    self.prepend_to_initializer(
                        property,
                        pending.drain(..).map(|block| block.expression).collect(),
                    );
                }
            }

            if write_index != read_index {
                let output_id = elements
                    .get(write_index)
                    .expect("write index must be inside the class body");
                elements.replace_slot(self.ast, output_id, element);
            }
            write_index += 1;
        }

        if pending.is_empty() {
            if write_index != original_len {
                elements.split_off(write_index);
                body.set_body(self.ast, elements);
            }
            return None;
        }

        if let Some(name) = inferred_name {
            let set_name = self.build_set_function_name(name);
            if let Some(property) = first_static_property {
                self.prepend_to_initializer(property, vec![set_name]);
            } else {
                pending.insert(
                    0,
                    BlockExpression {
                        expression: set_name,
                        block_body: None,
                        span: DUMMY_SP,
                    },
                );
            }
        }

        let closure_name = self.next_closure_name();
        let closure = self.build_trailing_closure(&pending, is_expression);
        let assignment_span = if pending[0].block_body.is_none() {
            pending[0].span
        } else {
            DUMMY_SP
        };
        let assignment = self.build_assignment(closure_name, closure, assignment_span);

        if let Some(property) = last_static_property {
            self.prepend_to_initializer(property, vec![assignment]);
        } else {
            let private_name = self
                .semantic
                .class_mut()
                .next_private_name(self.ast, class.node_id());
            let property = self.build_fallback_property(private_name, assignment);
            let output_id = elements
                .get(write_index)
                .expect("removed static block leaves a class body slot");
            elements.replace_slot(
                self.ast,
                output_id,
                ClassElement::PropertyDefinition(property),
            );
            write_index += 1;
        }

        if write_index != original_len {
            elements.split_off(write_index);
            body.set_body(self.ast, elements);
        }

        Some((self.build_call(closure_name), closure_name))
    }

    fn block_to_expression(
        &mut self,
        block: StaticBlock,
        is_first_pending: bool,
    ) -> Option<BlockExpression> {
        let statements = block.body(self.ast);
        let span = block.span(self.ast);
        let has_attached_comments =
            self.ast.comments().iter().any(|comment| {
                comment.attached_to == span.start || comment.attached_to == span.end
            });
        if statements.is_empty() && !has_attached_comments {
            return None;
        }
        if statements.len() == 1 {
            let statement = statements
                .get_node(self.ast, 0)
                .expect("single statement must exist");
            if let Some(expression_statement) = statement.as_expression_statement(self.ast) {
                return Some(BlockExpression {
                    expression: expression_statement.expression(self.ast),
                    block_body: None,
                    span,
                });
            }
        }

        let mut expression = self.build_iife(span, statements);
        if has_attached_comments {
            // Comments on the first block belong inside its synthetic
            // parentheses. Comments between adjacent blocks stay before the
            // later operand in the generated sequence.
            let wrapper_span = if is_first_pending { DUMMY_SP } else { span };
            expression = self
                .ast
                .expr_parenthesized_expression(wrapper_span, expression);
        }
        Some(BlockExpression {
            expression,
            block_body: Some(statements),
            span,
        })
    }

    fn build_iife(
        &mut self,
        span: swc_next_ecma_ast::Span,
        statements: TypedSubRange<Stmt>,
    ) -> Expr {
        let arrow_body =
            self.ast
                .arrow_function_body_function_body(span, TypedSubRange::empty(), statements);
        let arrow = self.build_arrow(span, arrow_body, false);
        self.ast
            .expr_call_expression(span, arrow, None, TypedSubRange::empty(), false)
    }

    fn build_trailing_closure(
        &mut self,
        pending: &[BlockExpression],
        needs_completion_value: bool,
    ) -> Expr {
        if pending.len() == 1 && pending[0].block_body.is_some() {
            let statements = pending[0].block_body.expect("checked above");
            let statements = if needs_completion_value {
                let mut body = Vec::with_capacity(statements.len() + 1);
                body.extend(
                    statements
                        .iter()
                        .map(|id| self.ast.get_node_in_sub_range(id)),
                );
                let this = self.ast.expr_this_expression(DUMMY_SP);
                body.push(self.ast.stmt_return_statement(DUMMY_SP, Some(this)));
                self.ast.add_typed_sub_range(body)
            } else {
                statements
            };
            let body = self.ast.arrow_function_body_function_body(
                DUMMY_SP,
                TypedSubRange::empty(),
                statements,
            );
            return self.build_arrow(DUMMY_SP, body, false);
        }

        let mut expressions: Vec<_> = pending.iter().map(|block| block.expression).collect();
        if needs_completion_value {
            expressions.push(self.ast.expr_this_expression(DUMMY_SP));
        }
        let expression = self.sequence_or_single(expressions);
        let body = ArrowFunctionBody::Expr(expression);
        self.build_arrow(DUMMY_SP, body, true)
    }

    fn build_arrow(
        &mut self,
        span: swc_next_ecma_ast::Span,
        body: ArrowFunctionBody,
        expression: bool,
    ) -> Expr {
        let params = self.ast.formal_parameters(
            DUMMY_SP,
            TypedSubRange::empty(),
            None,
            swc_next_ecma_ast::FormalParameterKind::FormalParameters,
        );
        self.ast
            .expr_arrow_function_expression(span, expression, false, None, params, None, body)
    }

    fn sequence_or_single(&mut self, expressions: Vec<Expr>) -> Expr {
        if expressions.len() == 1 {
            expressions[0]
        } else {
            let expressions = self.ast.add_typed_sub_range(expressions);
            self.ast.expr_sequence_expression(DUMMY_SP, expressions)
        }
    }

    fn prepend_to_initializer(&mut self, property: PropertyDefinition, mut expressions: Vec<Expr>) {
        if let Some(value) = property.value(self.ast) {
            expressions.push(value);
            let expressions = self.ast.add_typed_sub_range(expressions);
            let sequence = self.ast.expr_sequence_expression(DUMMY_SP, expressions);
            property.set_value(self.ast, Some(sequence));
            return;
        }

        let last = expressions
            .pop()
            .expect("at least one expression must be prepended");
        let last = self
            .ast
            .expr_unary_expression(DUMMY_SP, last, UnaryOperator::Void);
        expressions.push(last);
        let value = self.sequence_or_single(expressions);
        property.set_value(self.ast, Some(value));
    }

    fn build_assignment(
        &mut self,
        name: AstInternedUtf8,
        value: Expr,
        span: swc_next_ecma_ast::Span,
    ) -> Expr {
        let left = self
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(span, name.into());
        self.ast
            .expr_assignment_expression(span, left, value, AssignmentOperator::Assign)
    }

    fn build_call(&mut self, name: AstInternedUtf8) -> Expr {
        let callee = self.ast.expr_identifier_reference(DUMMY_SP, name.into());
        self.ast
            .expr_call_expression(DUMMY_SP, callee, None, TypedSubRange::empty(), false)
    }

    fn build_fallback_property(
        &mut self,
        private_name: AstInternedUtf8,
        initializer: Expr,
    ) -> PropertyDefinition {
        let key = self
            .ast
            .property_key_private_identifier(DUMMY_SP, private_name.into());
        self.ast.property_definition(
            DUMMY_SP,
            TypedSubRange::empty(),
            key,
            None,
            Some(initializer),
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            Accessibility::None,
        )
    }

    fn build_set_function_name(&mut self, name: AstWtf8) -> Expr {
        let helpers_name = self.ast.intern_utf8("babelHelpers");
        let helpers = self
            .ast
            .expr_identifier_reference(DUMMY_SP, helpers_name.into());
        let function_name = self.ast.intern_utf8("setFunctionName");
        let property = self
            .ast
            .property_key_identifier_name(DUMMY_SP, function_name.into());
        let callee = self
            .ast
            .expr_member_expression(DUMMY_SP, helpers, property, false, false);
        let this = Argument::Expr(self.ast.expr_this_expression(DUMMY_SP));
        let name = Argument::Expr(self.ast.expr_string_literal(
            DUMMY_SP,
            name,
            self.ast.empty_utf8(),
        ));
        let arguments = self.ast.add_typed_sub_range([this, name]);
        self.ast
            .expr_call_expression(DUMMY_SP, callee, None, arguments, false)
    }
}

impl StaticBlocks {
    pub(super) fn new() -> Self {
        Self {
            expression_calls: HashMap::new(),
            closure_names: None,
        }
    }

    pub(super) fn enter_class_body<'a>(
        &mut self,
        body: ClassBody,
        semantic: &mut Semantic<'a>,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let class = match ctx.ast.kind_data(
            ctx.parent()
                .expect("a class body must have its class as the parent"),
        ) {
            NodeKindData::Class(class) => class,
            _ => unreachable!("a class body must have its class as the parent"),
        };
        debug_assert_eq!(class.body(ctx.ast).node_id(), body.node_id());
        let is_expression = class.class_type(ctx.ast) == ClassType::ClassExpression;
        let inferred_name = is_expression
            .then(|| inferred_name(class, ctx.ancestor(2), ctx.ast))
            .flatten();
        let call = ClassTransform {
            ast: ctx.ast,
            semantic,
            closure_names: &mut self.closure_names,
        }
        .transform_class(class, is_expression, inferred_name);

        let Some((call, closure_name)) = call else {
            return;
        };
        ctx.state
            .statement_injector
            .prepend_var_current(closure_name);
        if is_expression {
            assert!(
                self.expression_calls
                    .insert(class.node_id(), call)
                    .is_none(),
                "a class expression can only be transformed once"
            );
        } else {
            insert_call_after_class(call, ctx);
        }
    }

    pub(super) fn exit_class<'a>(&mut self, class: Class, ctx: &mut TransformerCtx<'_, 'a>) {
        if self.expression_calls.is_empty() {
            return;
        }
        let Some(call) = self.expression_calls.remove(&class.node_id()) else {
            return;
        };
        let expressions = ctx.ast.add_typed_sub_range([Expr::Class(class), call]);
        let replacement = ctx.ast.expr_sequence_expression(DUMMY_SP, expressions);
        ctx.replace_current_expr(replacement);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.expression_calls.is_empty()
    }
}

fn insert_call_after_class<'a>(call: Expr, ctx: &mut TransformerCtx<'_, 'a>) {
    let (container, anchor) = ctx
        .state
        .statement_injector
        .current_insertion_target(ctx.path());
    let call = ctx.ast.stmt_expression_statement(DUMMY_SP, call);
    ctx.state
        .statement_injector
        .insert_after(container, anchor, call);
}

fn inferred_name(class: Class, parent: Option<NodeId>, ast: &mut Ast<'_>) -> Option<AstWtf8> {
    if class.id(ast).is_some() {
        return None;
    }
    match ast.kind_data(parent?) {
        NodeKindData::VariableDeclarator(node)
            if node
                .init(ast)
                .is_some_and(|value| value.node_id() == class.node_id()) =>
        {
            node.id(ast)
                .as_binding_identifier(ast)
                .map(|identifier| identifier.name(ast).into())
        }
        NodeKindData::ObjectProperty(node) if node.value(ast).node_id() == class.node_id() => {
            property_key_name(node.key(ast), ast)
        }
        NodeKindData::AssignmentExpression(node)
            if node.operator(ast) == AssignmentOperator::Assign
                && node.right(ast).node_id() == class.node_id() =>
        {
            assignment_target_name(node.left(ast), ast)
        }
        NodeKindData::AssignmentPattern(node) if node.right(ast).node_id() == class.node_id() => {
            node.left(ast)
                .as_binding_identifier(ast)
                .map(|identifier| identifier.name(ast).into())
        }
        NodeKindData::PropertyDefinition(node)
            if node
                .value(ast)
                .is_some_and(|value| value.node_id() == class.node_id()) =>
        {
            property_key_name(node.key(ast), ast)
        }
        _ => None,
    }
}

fn property_key_name(key: PropertyKey, ast: &mut Ast<'_>) -> Option<AstWtf8> {
    if let Some(literal) = key.as_string_literal(ast) {
        return Some(literal.value(ast));
    }
    if let Some(identifier) = key.as_identifier_name(ast) {
        return Some(identifier.name(ast).into());
    }
    if let Some(private) = key.as_private_identifier(ast) {
        let name = format!("#{}", ast.get_utf8(private.name(ast)));
        return Some(ast.add_wtf8(name.as_str().into()));
    }
    None
}

fn assignment_target_name(
    target: swc_next_ecma_ast::AssignmentTarget,
    ast: &Ast<'_>,
) -> Option<AstWtf8> {
    let target = target.as_simple_assignment_target(ast)?;
    target
        .as_identifier_reference(ast)
        .map(|identifier| identifier.name(ast).into())
}

#[derive(Clone, Copy)]
struct BlockExpression {
    expression: Expr,
    block_body: Option<TypedSubRange<Stmt>>,
    span: swc_next_ecma_ast::Span,
}

fn static_property(element: ClassElement, ast: &Ast<'_>) -> Option<PropertyDefinition> {
    element
        .as_property_definition(ast)
        .filter(|property| property.r#static(ast))
}
