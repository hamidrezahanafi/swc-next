//! Safari 10/11 bugfix: cached tagged template literals.
//!
//! Safari 10/11 uses the same cached Strings value when the string parts are
//! the same, which violates the spec. This transform wraps each tagged template
//! with an inline cache.
//!
//! Input:
//! ```js
//! tag`hello ${name}`
//! ```
//! Output:
//! ```js
//! var _t, _ = function(t) { return t; };
//! tag(_t || (_t = _`hello ${0}`), name)
//! ```

use swc_next_ecma_ast::{
    Ast, AstUtf8, DUMMY_SP, Expr, NodeKind, Program, Stmt, VisitMut, VisitMutWith,
};

use crate::state::TransformerCtx;

pub(super) struct TemplateLiteralCaching;

impl TemplateLiteralCaching {
    pub fn new() -> Self {
        Self
    }

    pub fn transform<'a>(
        &mut self,
        program: Program,
        ctx: &mut TransformerCtx<'_, 'a>,
    ) {
        let helper_name = ctx.ast.add_utf8("_");

        let mut visitor = TemplateLiteralCachingVisitor {
            ast: ctx.ast,
            helper_name,
            cache_counter: 0,
            cache_idents: Vec::new(),
        };
        program.visit_mut_children_with(&mut visitor);

        // Extract results before the visitor is dropped to release the ast borrow
        let cache_idents = std::mem::take(&mut visitor.cache_idents);
        drop(visitor);

        // Prepend declarations to the program
        if !cache_idents.is_empty() {
            let mut decls = Vec::new();

            // Create the identity function helper: `_ = function(t) { return t; }`
            let t_name = ctx.ast.add_utf8("t");
            let t_decorators = ctx.ast.add_typed_sub_range(Vec::new());
            let t_param = ctx.ast.formal_parameter_pattern_binding_pattern_binding_identifier(
                DUMMY_SP, t_name, t_decorators, None, false,
            );
            let t_ref = ctx.ast.expr_identifier_reference(DUMMY_SP, t_name);
            let return_stmt = ctx.ast.stmt_return_statement(DUMMY_SP, Some(t_ref));
            let fn_body_stmts = ctx.ast.add_typed_sub_range(vec![return_stmt]);
            let fn_body_empty = ctx.ast.add_typed_sub_range(Vec::new());
            let fn_body = ctx.ast.function_body(DUMMY_SP, fn_body_empty, fn_body_stmts);
            let param_item = ctx.ast.formal_parameter_item_formal_parameter(
                DUMMY_SP,
                t_param,
            );
            let params_items = ctx.ast.add_typed_sub_range(vec![param_item]);
            let params = ctx.ast.formal_parameters(
                DUMMY_SP,
                params_items,
                None,
                swc_next_ecma_ast::FormalParameterKind::FormalParameters,
            );
            let helper_fn = ctx.ast.expr_function(
                DUMMY_SP,
                swc_next_ecma_ast::FunctionType::FunctionExpression,
                None,
                false,
                false,
                params,
                fn_body,
                None,
                None,
            );

            let helper_decorators = ctx.ast.add_typed_sub_range(Vec::new());
            let helper_id = ctx.ast.binding_pattern_binding_identifier(
                DUMMY_SP,
                helper_name,
                helper_decorators,
                None,
                false,
            );
            let helper_decl = ctx.ast.variable_declarator(DUMMY_SP, helper_id, Some(helper_fn), false);
            decls.push(helper_decl);

            // Create cache variable declarations: `var _t = undefined`
            let empty_decorators = ctx.ast.add_typed_sub_range(Vec::new());
            for &cache_name in &cache_idents {
                let cache_id = ctx.ast.binding_pattern_binding_identifier(
                    DUMMY_SP,
                    cache_name,
                    empty_decorators,
                    None,
                    false,
                );
                let cache_decl = ctx.ast.variable_declarator(DUMMY_SP, cache_id, None, false);
                decls.push(cache_decl);
            }

            let decl_range = ctx.ast.add_typed_sub_range(decls);
            let var_decl = ctx.ast.variable_declaration(
                DUMMY_SP,
                swc_next_ecma_ast::VariableKind::Var,
                decl_range,
                false,
            );
            let var_stmt = Stmt::Declaration(swc_next_ecma_ast::Decl::VariableDeclaration(var_decl));

            let body = program.body(ctx.ast);
            let mut stmts = Vec::with_capacity(body.len() + 1);
            stmts.push(var_stmt);
            for i in 0..body.len() {
                stmts.push(body.get_node(ctx.ast, i).unwrap());
            }
            let new_body = ctx.ast.add_typed_sub_range(stmts);
            program.set_body(ctx.ast, new_body);
        }
    }
}

struct TemplateLiteralCachingVisitor<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    helper_name: AstUtf8,
    cache_counter: usize,
    cache_idents: Vec<AstUtf8>,
}

impl<'ast> VisitMut<'ast> for TemplateLiteralCachingVisitor<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_expr(&mut self, node: Expr) -> Option<Expr> {
        let node = node.visit_mut_children_with(self).unwrap_or(node);

        if self.ast.node_kind(node.node_id()) != NodeKind::TaggedTemplateExpression {
            return Some(node);
        }

        let tagged = node.as_tagged_template_expression(self.ast).unwrap();
        let tag = tagged.tag(self.ast);
        let tpl = tagged.quasi(self.ast);
        let quasis = tpl.quasis(self.ast);
        let exprs = tpl.expressions(self.ast);
        let expr_len = exprs.len();

        // Create a cache variable name
        let cache_name = if self.cache_counter == 0 {
            self.ast.add_utf8("_t")
        } else {
            let name = format!("_t{}", self.cache_counter);
            self.ast.add_utf8(&name)
        };
        self.cache_counter += 1;
        self.cache_idents.push(cache_name);

        // Build the identity call: `_`template${0}${0}...``
        // Replace all expressions with `0` (NumericLiteral)
        let raw_zero = self.ast.add_utf8("0");
        let zero_expr = self.ast.expr_numeric_literal(DUMMY_SP, swc_next_ecma_ast::NumericLiteralKind::Decimal, raw_zero);
        let mut new_exprs: Vec<Expr> = Vec::with_capacity(expr_len);
        for _ in 0..expr_len {
            new_exprs.push(zero_expr);
        }
        let new_exprs_range = self.ast.add_typed_sub_range(new_exprs);

        let template_literal = self.ast.template_literal(DUMMY_SP, quasis, new_exprs_range);

        // Build: `_`...` -- call the identity helper with the template
        let helper_ref = self.ast.expr_identifier_reference(DUMMY_SP, self.helper_name);
        let identity_call = self.ast.expr_tagged_template_expression(
            DUMMY_SP,
            helper_ref,
            template_literal,
            None,
        );

        // Build: `_t || (_t = _`...`)`
        let cache_ref = self.ast.expr_identifier_reference(DUMMY_SP, cache_name);
        let assign_target = self.ast.assignment_target_simple_assignment_target_identifier_reference(
            DUMMY_SP,
            cache_name,
        );
        let assign = self.ast.expr_assignment_expression(
            DUMMY_SP,
            assign_target,
            identity_call,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        let inline_cache = self.ast.expr_logical_expression(
            DUMMY_SP,
            cache_ref,
            assign,
            swc_next_ecma_ast::LogicalOperator::Or,
        );

        // Build: `tag(inline_cache, ...exprs)`
        let mut args = Vec::new();
        args.push(swc_next_ecma_ast::Argument::Expr(inline_cache));
        for i in 0..expr_len {
            args.push(swc_next_ecma_ast::Argument::Expr(exprs.get_node(self.ast, i).unwrap()));
        }
        let args_range = self.ast.add_typed_sub_range(args);

        Some(self.ast.expr_call_expression(DUMMY_SP, tag, args_range, false, None))
    }
}