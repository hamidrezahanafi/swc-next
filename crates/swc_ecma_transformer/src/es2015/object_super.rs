//! ES2015: object-super — Transform `super` in object literal methods.
//!
//! When an object literal method uses `super`, the object must be wrapped in an
//! assignment expression with a hoisted variable, and `super.foo` references
//! are replaced with `_get/_set` helper calls.
//!
//! Input:
//! ```js
//! let obj = {
//!     foo() {
//!         super.bar();
//!     }
//! }
//! ```
//!
//! Output:
//! ```js
//! var _obj;
//! let obj = _obj = {
//!     foo() {
//!         _get(_get_prototype_of(_obj), "bar", this).call(this);
//!     }
//! };
//! ```
//!
//! See docs/design/object-super.md for full design.

use swc_next_ecma_ast::{
    Ast, AstUtf8, DUMMY_SP, Expr, ExprData, VisitMut, VisitMutWith,
};

use crate::state::TransformerCtx;

pub(super) struct ObjectSuper {
    /// The `_obj` name for the current wrapping context, set during enter_object_expression.
    obj_name: Option<AstUtf8>,
    /// Stack of object expression node IDs that need wrapping.
    wrap_queue: Vec<swc_next_ecma_ast::NodeId>,
    /// Unique counter for `_obj` variable names.
    obj_counter: usize,
    /// Variable names that need to be hoisted to the enclosing scope.
    hoist_vars: Vec<AstUtf8>,
}

impl ObjectSuper {
    pub fn new() -> Self {
        Self {
            obj_name: None,
            wrap_queue: Vec::new(),
            obj_counter: 0,
            hoist_vars: Vec::new(),
        }
    }

    fn fresh_obj_name(&mut self, ast: &mut Ast<'_>) -> AstUtf8 {
        self.obj_counter += 1;
        if self.obj_counter == 1 {
            ast.add_utf8("_obj")
        } else {
            let name = format!("_obj{}", self.obj_counter - 1);
            ast.add_utf8(&name)
        }
    }

    pub fn enter_object_expression(
        &mut self,
        node: swc_next_ecma_ast::ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let properties = node.properties(ctx.ast);
        let len = properties.len();
        let mut has_super = false;

        for i in 0..len {
            let prop = properties.get_node(ctx.ast, i).unwrap();
            if let Some(object_prop) = prop.as_object_property(ctx.ast) {
                if object_prop.method(ctx.ast) {
                    let value = object_prop.value(ctx.ast);
                    if value.is_function(ctx.ast) {
                        if has_super_ref(ctx.ast, value) {
                            has_super = true;
                            break;
                        }
                    }
                }
            }
        }

        if has_super {
            let name = self.fresh_obj_name(ctx.ast);
            self.obj_name = Some(name);
            self.hoist_vars.push(name);

            for i in 0..len {
                let prop = properties.get_node(ctx.ast, i).unwrap();
                if let Some(object_prop) = prop.as_object_property(ctx.ast) {
                    if object_prop.method(ctx.ast) {
                        let value = object_prop.value(ctx.ast);
                        if value.is_function(ctx.ast) {
                            rewrite_function_body(ctx.ast, value, name);
                        }
                    }
                }
            }
        }
    }

    pub fn exit_object_expression(
        &mut self,
        node: swc_next_ecma_ast::ObjectExpression,
        _ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.obj_name.is_some() {
            self.wrap_queue.push(node.node_id());
            self.obj_name = None;
        }
    }

    pub fn exit_expression(
        &mut self,
        node: Expr,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> swc_next_ecma_ast::TraverseAction<Expr> {
        if let Some(obj) = node.as_object_expression(ctx.ast) {
            let idx = self.wrap_queue.iter().position(|id| *id == obj.node_id());
            if let Some(idx) = idx {
                self.wrap_queue.swap_remove(idx);
                return swc_next_ecma_ast::TraverseAction::Replace(self.wrap_object(obj, ctx));
            }
        }
        swc_next_ecma_ast::TraverseAction::Proceed
    }

    pub fn exit_program(
        &mut self,
        node: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        if self.hoist_vars.is_empty() {
            return;
        }
        let mut declarators = Vec::with_capacity(self.hoist_vars.len());
        let empty_decorators = ctx.ast.add_typed_sub_range(Vec::new());
        for &name in &self.hoist_vars {
            let id = ctx.ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                name,
                empty_decorators,
                None,
                false,
            );
            let declarator = ctx.ast.variable_declarator(DUMMY_SP, id, None, false);
            declarators.push(declarator);
        }
        let decl_range = ctx.ast.add_typed_sub_range(declarators);
        let var_decl = ctx.ast.variable_declaration(
            DUMMY_SP,
            swc_next_ecma_ast::VariableKind::Var,
            decl_range,
            false,
        );
        let var_stmt = swc_next_ecma_ast::Stmt::Declaration(
            swc_next_ecma_ast::Decl::VariableDeclaration(var_decl),
        );
        let body = node.body(ctx.ast);
        let mut stmts = Vec::with_capacity(body.len() + 1);
        stmts.push(var_stmt);
        for j in 0..body.len() {
            stmts.push(body.get_node(ctx.ast, j).unwrap());
        }
        let new_body = ctx.ast.add_typed_sub_range(stmts);
        node.set_body(ctx.ast, new_body);
    }

    fn wrap_object(
        &self,
        node: swc_next_ecma_ast::ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let obj_name = *self
            .hoist_vars
            .last()
            .expect("hoist_vars should have at least one entry");

        let left = ctx
            .ast
            .assignment_target_simple_assignment_target_identifier_reference(
                DUMMY_SP,
                obj_name,
            );
        let obj_expr = ctx.ast.expr_object_expression(DUMMY_SP, node.properties(ctx.ast));
        ctx.ast.expr_assignment_expression(
            DUMMY_SP,
            left,
            obj_expr,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        )
    }
}

/// Check if an expression tree contains any `super` reference.
fn has_super_ref(ast: &Ast<'_>, expr: Expr) -> bool {
    /// Internal visitor-like walker
    fn walk(ast: &Ast<'_>, expr: Expr) -> bool {
        match ast.expr_data(expr) {
            ExprData::Super(_) => return true,
            ExprData::Function(_) | ExprData::Class(_) | ExprData::ObjectExpression(_) => return false,
            _ => {}
        }
        let mut found = false;
        let mut check = |e: Expr| {
            if !found && walk(ast, e) { found = true; }
        };
        macro_rules! check_opt {
            ($e:expr) => {
                if let Some(e) = $e { check(e); }
            };
        }
        match ast.expr_data(expr) {
            ExprData::MemberExpression(m) => check(m.object(ast)),
            ExprData::CallExpression(c) => {
                check(c.callee(ast));
                for i in 0..c.arguments(ast).len() {
                    if let Some(a) = c.arguments(ast).get_node(ast, i) {
                        if let Some(e) = a.as_expr(ast) { check(e); }
                    }
                }
            }
            ExprData::AssignmentExpression(a) => {
                if let Some(s) = a.left(ast).as_simple_assignment_target(ast) {
                    if let Some(m) = s.as_member_expression(ast) { check(m.object(ast)); }
                }
                check(a.right(ast));
            }
            ExprData::UnaryExpression(u) => check(u.argument(ast)),
            ExprData::BinaryExpression(b) => { check(b.left(ast)); check(b.right(ast)); }
            ExprData::LogicalExpression(l) => { check(l.left(ast)); check(l.right(ast)); }
            ExprData::ConditionalExpression(c) => {
                check(c.test(ast)); check(c.consequent(ast)); check(c.alternate(ast));
            }
            ExprData::SequenceExpression(s) => {
                for i in 0..s.expressions(ast).len() {
                    check(s.expressions(ast).get_node(ast, i).unwrap());
                }
            }
            ExprData::NewExpression(n) => check(n.callee(ast)),
            ExprData::TaggedTemplateExpression(t) => check(t.tag(ast)),
            ExprData::YieldExpression(y) => check_opt!(y.argument(ast)),
            ExprData::AwaitExpression(a) => check(a.argument(ast)),
            ExprData::UpdateExpression(u) => {
                if let swc_next_ecma_ast::SimpleAssignmentTargetData::MemberExpression(m) = ast.simple_assignment_target_data(u.argument(ast)) {
                    check(m.object(ast));
                }
            }
            ExprData::ArrayExpression(a) => {
                for i in 0..a.elements(ast).len() {
                    let elem = a.elements(ast).get_node(ast, i);
                    if let Some(Some(elem)) = elem {
                        if let Some(e) = elem.as_expr(ast) { check(e); }
                    }
                }
            }
            ExprData::ParenthesizedExpression(p) => check(p.expression(ast)),
            _ => {}
        }
        found
    }
    walk(ast, expr)
}

/// Rewrite a function body, replacing `super.foo` with `_get/_set` calls.
fn rewrite_function_body(ast: &mut Ast<'_>, function: Expr, obj_name: AstUtf8) {
    /// Replaces `super.foo` patterns with `_get/_set` helper calls.
    struct SuperReplacer<'visit, 'ast> {
        ast: &'visit mut Ast<'ast>,
        obj_name: AstUtf8,
    }

    impl SuperReplacer<'_, '_> {
        fn property_key_to_expr(ast: &mut Ast<'_>, key: swc_next_ecma_ast::PropertyKey) -> Expr {
            if key.is_identifier_name(ast) {
                let id = key.as_identifier_name(ast).unwrap();
                let name = ast.get_utf8(id.name(ast)).to_owned();
                let value = ast.add_wtf8((&*name).into());
                let raw = ast.add_utf8(&format!("\"{}\"", name));
                ast.expr_string_literal(DUMMY_SP, value, raw)
            } else if key.is_string_literal(ast) {
                let sl = key.as_string_literal(ast).unwrap();
                Expr::StringLiteral(sl)
            } else if key.is_private_identifier(ast) {
                let pi = key.as_private_identifier(ast).unwrap();
                let name = ast.get_utf8(pi.name(ast)).to_owned();
                let value = ast.add_wtf8((&*name).into());
                let raw = ast.add_utf8(&format!("\"{}\"", name));
                ast.expr_string_literal(DUMMY_SP, value, raw)
            } else {
                key.as_expr(ast).expect("computed key should be an expression")
            }
        }

        fn get_proto(ast: &mut Ast<'_>, obj_name: AstUtf8) -> Expr {
            let proto_name = ast.add_utf8("_get_prototype_of");
            let obj_ref = ast.expr_identifier_reference(DUMMY_SP, obj_name);
            let args = ast.add_typed_sub_range([swc_next_ecma_ast::Argument::Expr(obj_ref)]);
            let callee = ast.expr_identifier_reference(DUMMY_SP, proto_name);
            ast.expr_call_expression(DUMMY_SP, callee, args, false, None)
        }

        fn build_get_call(ast: &mut Ast<'_>, obj_name: AstUtf8, property: Expr) -> Expr {
            let get_name = ast.add_utf8("_get");
            let proto = Self::get_proto(ast, obj_name);
            let this_expr = ast.expr_this_expression(DUMMY_SP);
            let args = ast.add_typed_sub_range([
                swc_next_ecma_ast::Argument::Expr(proto),
                swc_next_ecma_ast::Argument::Expr(property),
                swc_next_ecma_ast::Argument::Expr(this_expr),
            ]);
            let callee = ast.expr_identifier_reference(DUMMY_SP, get_name);
            ast.expr_call_expression(DUMMY_SP, callee, args, false, None)
        }

        fn build_set_call(ast: &mut Ast<'_>, obj_name: AstUtf8, property: Expr, value: Expr) -> Expr {
            let set_name = ast.add_utf8("_set");
            let proto = Self::get_proto(ast, obj_name);
            let this_expr = ast.expr_this_expression(DUMMY_SP);
            let true_lit = ast.expr_boolean_literal(DUMMY_SP, true);
            let args = ast.add_typed_sub_range([
                swc_next_ecma_ast::Argument::Expr(proto),
                swc_next_ecma_ast::Argument::Expr(property),
                swc_next_ecma_ast::Argument::Expr(value),
                swc_next_ecma_ast::Argument::Expr(this_expr),
                swc_next_ecma_ast::Argument::Expr(true_lit),
            ]);
            let callee = ast.expr_identifier_reference(DUMMY_SP, set_name);
            ast.expr_call_expression(DUMMY_SP, callee, args, false, None)
        }
    }

    impl<'ast> VisitMut<'ast> for SuperReplacer<'_, 'ast> {
        fn ast(&mut self) -> &mut Ast<'ast> {
            self.ast
        }

        fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
            if node.is_object_expression(self.ast)
                || node.is_function(self.ast)
                || node.is_class(self.ast)
            {
                return None;
            }

            let obj_name = self.obj_name;

            // Check patterns in a scope that borrows self.ast, then drops it
            let result = {
                let ast = &mut *self.ast;

                // Pattern 1: super.foo(args) — CallExpression
                if let Some(call) = node.as_call_expression(ast) {
                    let callee = call.callee(ast);
                    if let Some(member) = callee.as_member_expression(ast) {
                        if member.object(ast).is_super(ast) {
                            let property = SuperReplacer::property_key_to_expr(ast, member.property(ast));
                            let get_call = SuperReplacer::build_get_call(ast, obj_name, property);
                            let call_name = ast.add_utf8("call");
                            let call_key = ast.property_key_identifier_name(DUMMY_SP, call_name);
                            let this_expr = ast.expr_this_expression(DUMMY_SP);
                            let args = call.arguments(ast);
                            let mut new_args = Vec::with_capacity(args.len() + 1);
                            new_args.push(swc_next_ecma_ast::Argument::Expr(this_expr));
                            for j in 0..args.len() {
                                new_args.push(args.get_node(ast, j).unwrap());
                            }
                            let new_args_range = ast.add_typed_sub_range(new_args);
                            let get_call_member = ast.expr_member_expression(
                                DUMMY_SP,
                                get_call,
                                call_key,
                                false,
                                false,
                            );
                            return Some(ast.expr_call_expression(
                                DUMMY_SP,
                                get_call_member,
                                new_args_range,
                                false,
                                None,
                            ));
                        }
                    }
                }

                // Pattern 2: super.foo = val — AssignmentExpression
                if let Some(assign) = node.as_assignment_expression(ast) {
                    let left = assign.left(ast);
                    if let Some(simple) = left.as_simple_assignment_target(ast) {
                        if let swc_next_ecma_ast::SimpleAssignmentTargetData::MemberExpression(member) = ast.simple_assignment_target_data(simple) {
                            if member.object(ast).is_super(ast) {
                                let property = SuperReplacer::property_key_to_expr(ast, member.property(ast));
                                let value = assign.right(ast);
                                return Some(SuperReplacer::build_set_call(ast, obj_name, property, value));
                            }
                        }
                    }
                }

                // Pattern 3: super.foo — standalone MemberExpression
                if let Some(member) = node.as_member_expression(ast) {
                    if member.object(ast).is_super(ast) {
                        let property = SuperReplacer::property_key_to_expr(ast, member.property(ast));
                        return Some(SuperReplacer::build_get_call(ast, obj_name, property));
                    }
                }

                None::<Expr>
            };

            if let Some(replacement) = result {
                return Some(replacement);
            }

            // No match, continue recursion
            node.visit_mut_children_with(self)
        }
    }

    let mut replacer = SuperReplacer { ast, obj_name };
    function.visit_mut_with(&mut replacer);
}