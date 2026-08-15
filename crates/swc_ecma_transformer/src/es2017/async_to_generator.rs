//! ES2017: async-to-generator — Transform async functions into generator-based wrappers.
//!
//! ```js
//! // Input:
//! async function foo() {
//!     return await bar();
//! }
//!
//! // Output:
//! function foo() {
//!     return _asyncToGenerator(function*() {
//!         return yield bar();
//!     })();
//! }
//! ```
//!
//! See docs/design/async-to-generator.md for full design.

use swc_next_ecma_ast::{
    ArrowFunctionBody, ArrowFunctionExpression, Ast, AstUtf8, DUMMY_SP, Expr, ExprData, Function,
    FunctionBody, FunctionType, GetSpan, NodeKind, VisitMut, VisitMutWith,
};

use crate::state::TransformerCtx;

#[derive(Debug, Default, Clone)]
struct FnState {
    should_transform: bool,
    use_this: bool,
    use_arguments: bool,
    use_super: bool,
    /// True if this function is in a constructor body (not nested in another function)
    in_constructor: bool,
}

pub(super) struct AsyncToGenerator {
    helper_name: Option<AstUtf8>,
}

impl AsyncToGenerator {
    pub fn new() -> Self {
        Self { helper_name: None }
    }

    /// Entry point: transform all async functions in the program.
    pub fn transform<'a>(
        &mut self,
        program: swc_next_ecma_ast::Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let helper_name = ctx.ast.add_utf8("_asyncToGenerator");
        self.helper_name = Some(helper_name);

        let mut visitor = AsyncToGeneratorVisitor {
            ast: ctx.ast,
            fn_state: None,
            fn_state_stack: Vec::new(),
            helper_name,
        };
        program.visit_mut_children_with(&mut visitor);
    }
}

struct AsyncToGeneratorVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    fn_state: Option<FnState>,
    fn_state_stack: Vec<FnState>,
    helper_name: AstUtf8,
}

impl<'ast> VisitMut<'ast> for AsyncToGeneratorVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_function(&mut self, node: Function) -> Option<Function> {
        let is_async = node.r#async(self.ast);
        let is_generator = node.generator(self.ast);

        // Push current fn_state to stack
        if let Some(prev) = self.fn_state.take() {
            self.fn_state_stack.push(prev);
        }

        // Set new fn_state for this function
        self.fn_state = Some(FnState {
            should_transform: is_async && !is_generator,
            ..Default::default()
        });

        // Visit children (params, body) — modifies in-place
        node.visit_mut_children_with(self);

        let fn_state = self.fn_state.take().unwrap();

        // Restore parent fn_state
        self.fn_state = self.fn_state_stack.pop();

        if !fn_state.should_transform {
            return None;
        }

        // Build the _asyncToGenerator(function*() { ... }) wrapper
        let body = node.body(self.ast);
        let inner_stmts = body.body(self.ast);

        let result = self.make_fn_ref(&fn_state, inner_stmts);

        let return_stmt = self.ast.stmt_return_statement(DUMMY_SP, Some(result));
        let empty_directives = self.ast.add_typed_sub_range(Vec::new());
        let body_stmts = self.ast.add_typed_sub_range(vec![return_stmt]);
        let new_body = self.ast.function_body(DUMMY_SP, empty_directives, body_stmts);

        let id = node.id(self.ast);
        let type_params = node.type_parameters(self.ast);
        let return_type = node.return_type(self.ast);
        let params = node.params(self.ast);

        Some(self.ast.function(
            DUMMY_SP,
            FunctionType::FunctionDeclaration,
            id,
            false, // not a generator
            false, // not async
            params,
            new_body,
            type_params,
            return_type,
        ))
    }

    fn visit_mut_arrow_function_expression(
        &mut self,
        node: ArrowFunctionExpression,
    ) -> Option<ArrowFunctionExpression> {
        let is_async = node.r#async(self.ast);

        // Push current fn_state to stack
        let parent_fn_state = self.fn_state.take();
        if let Some(prev) = parent_fn_state {
            self.fn_state_stack.push(prev);
        }

        let in_constructor = self
            .fn_state_stack
            .last()
            .map(|s| s.in_constructor)
            .unwrap_or(false);

        self.fn_state = Some(FnState {
            should_transform: is_async,
            in_constructor,
            ..Default::default()
        });

        // Visit children — this modifies in-place and tracks this/arguments/super
        let node = node.visit_mut_children_with(self).unwrap_or(node);

        let arrow_fn_state = self.fn_state.take().unwrap();

        // Propagate usage to parent fn_state
        if let Some(parent_fs) = self.fn_state_stack.last_mut() {
            parent_fs.use_this |= arrow_fn_state.use_this;
            parent_fs.use_arguments |= arrow_fn_state.use_arguments;
            parent_fs.use_super |= arrow_fn_state.use_super;
        }

        // Restore parent fn_state
        self.fn_state = self.fn_state_stack.pop();

        if !arrow_fn_state.should_transform {
            return Some(node);
        }

        // Convert async arrow to generator wrapper
        let params = node.params(self.ast);
        let body = node.body(self.ast);
        let expression = node.expression(self.ast);
        let span = node.span(self.ast);
        let type_params = node.type_parameters(self.ast);
        let return_type = node.return_type(self.ast);

        // Convert body to statements
        let inner_stmts = if expression {
            if let Some(body_expr) = body.as_expr(self.ast) {
                let return_stmt = self.ast.stmt_return_statement(DUMMY_SP, Some(body_expr));
                self.ast.add_typed_sub_range(vec![return_stmt])
            } else {
                body.as_function_body(self.ast)
                    .map(|fb| fb.body(self.ast))
                    .unwrap_or_else(|| self.ast.add_typed_sub_range(Vec::new()))
            }
        } else {
            body.as_function_body(self.ast)
                .map(|fb| fb.body(self.ast))
                .unwrap_or_else(|| self.ast.add_typed_sub_range(Vec::new()))
        };

        let result = self.make_fn_ref(&arrow_fn_state, inner_stmts);

        let new_body = if arrow_fn_state.use_super {
            let return_stmt = self.ast.stmt_return_statement(DUMMY_SP, Some(result));
            let block_body = self.ast.add_typed_sub_range(vec![return_stmt]);
            let empty_directives = self.ast.add_typed_sub_range(Vec::new());
            let fb = self.ast.function_body(DUMMY_SP, empty_directives, block_body);
            ArrowFunctionBody::FunctionBody(fb)
        } else {
            ArrowFunctionBody::Expr(result)
        };

        Some(self.ast.arrow_function_expression(
            span,
            false, // No longer concise (now a block or expr)
            false, // Not async anymore
            params,
            new_body,
            type_params,
            return_type,
        ))
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        // First, visit children
        let node = node.visit_mut_children_with(self).unwrap_or(node);

        let should_transform = self
            .fn_state
            .as_ref()
            .map(|s| s.should_transform)
            .unwrap_or(false);

        if !should_transform {
            return Some(node);
        }

        // Check node kind
        match self.ast.node_kind(node.node_id()) {
            NodeKind::AwaitExpression => {
                self.fn_state.as_mut().unwrap().use_this = true;

                let arg = node
                    .as_await_expression(self.ast)
                    .unwrap()
                    .argument(self.ast);

                Some(self.ast.expr_yield_expression(DUMMY_SP, Some(arg), false))
            }
            NodeKind::ThisExpression => {
                if let Some(fn_state) = &mut self.fn_state {
                    fn_state.use_this = true;
                }
                Some(node)
            }
            _ => {
                if node.is_identifier_reference(self.ast) {
                    let id = node.as_identifier_reference(self.ast).unwrap();
                    let name = self.ast.get_utf8(id.name(self.ast)).to_string();
                    if name == "arguments" {
                        self.fn_state.as_mut().unwrap().use_arguments = true;
                    }
                }
                Some(node)
            }
        }
    }
}

impl AsyncToGeneratorVisitor<'_, '_> {
    /// Build `_asyncToGenerator(function*() { ... })` or `.call(this)` variant.
    fn make_fn_ref(
        &mut self,
        fn_state: &FnState,
        body_stmts: swc_next_ecma_ast::TypedSubRange<swc_next_ecma_ast::Stmt>,
    ) -> Expr {
        let helper = self.ast.expr_identifier_reference(DUMMY_SP, self.helper_name);

        // Pre-compute temporaries to avoid borrow conflicts
        let empty_directives = self.ast.add_typed_sub_range(Vec::new());
        let empty_params = self.ast.add_typed_sub_range(Vec::new());

        // Create the inner generator function
        let inner_fn_body = self.ast.function_body(DUMMY_SP, empty_directives, body_stmts);
        let inner_fn_params = self.ast.formal_parameters(
            DUMMY_SP,
            empty_params,
            None,
            swc_next_ecma_ast::FormalParameterKind::FormalParameters,
        );
        let inner_fn = self.ast.expr_function(
            DUMMY_SP,
            FunctionType::FunctionExpression,
            None,
            true,  // generator
            false, // not async
            inner_fn_params,
            inner_fn_body,
            None,
            None,
        );

        // _asyncToGenerator(function*() { ... })
        let call_args =
            self.ast
                .add_typed_sub_range(vec![swc_next_ecma_ast::Argument::Expr(inner_fn)]);
        let call_expr = self.ast.expr_call_expression(DUMMY_SP, helper, call_args, false, None);

        if fn_state.use_arguments {
            // .apply(this, arguments)
            let apply_name = self.ast.add_utf8("apply");
            let apply_key = self.ast.property_key_identifier_name(DUMMY_SP, apply_name);
            let member =
                self.ast
                    .expr_member_expression(DUMMY_SP, call_expr, apply_key, false, false);
            let this_expr = self.ast.expr_this_expression(DUMMY_SP);
            let arguments_name = self.ast.add_utf8("arguments");
            let arguments_ref = self.ast.expr_identifier_reference(DUMMY_SP, arguments_name);
            let apply_args = self.ast.add_typed_sub_range(vec![
                swc_next_ecma_ast::Argument::Expr(this_expr),
                swc_next_ecma_ast::Argument::Expr(arguments_ref),
            ]);
            self.ast.expr_call_expression(DUMMY_SP, member, apply_args, false, None)
        } else if fn_state.use_this && !fn_state.in_constructor {
            // .call(this)
            let call_name = self.ast.add_utf8("call");
            let call_key = self.ast.property_key_identifier_name(DUMMY_SP, call_name);
            let member =
                self.ast
                    .expr_member_expression(DUMMY_SP, call_expr, call_key, false, false);
            let this_expr = self.ast.expr_this_expression(DUMMY_SP);
            let call_args = self.ast.add_typed_sub_range(vec![
                swc_next_ecma_ast::Argument::Expr(this_expr),
            ]);
            self.ast.expr_call_expression(DUMMY_SP, member, call_args, false, None)
        } else {
            // () — just call
            call_expr
        }
    }
}