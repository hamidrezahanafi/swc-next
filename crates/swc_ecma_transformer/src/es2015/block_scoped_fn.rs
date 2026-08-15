//! ES2015: block-scoped-functions — Convert block-scoped function declarations
//! to `let`-bound function expressions.
//!
//! ```js
//! {
//!   function foo() {}
//!   bar();
//! }
//! // →
//! {
//!   let foo = function foo() {};
//!   bar();
//! }
//! ```

use swc_next_ecma_ast::{
    BlockStatement, FunctionBody, FunctionType, GetSpan, Stmt, VariableKind,
};

use crate::state::TransformerCtx;

pub(super) struct BlockScopedFns;

impl BlockScopedFns {
    pub fn exit_block_statement(
        &mut self,
        node: BlockStatement,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let body = node.body(ctx.ast);
        if body.is_empty() {
            return;
        }

        let stmts: Vec<Stmt> = body.iter().map(|id| ctx.ast.get_node_in_sub_range(id)).collect();
        let has_fn_decl = stmts.iter().any(|s| {
            s.as_declaration(ctx.ast)
                .is_some_and(|d| d.is_function(ctx.ast))
        });
        if !has_fn_decl {
            return;
        }

        let mut fns: Vec<(Stmt, FunctionBody)> = Vec::new();
        let mut other_stmts: Vec<Stmt> = Vec::new();

        for stmt in stmts {
            if let Some(decl) = stmt.as_declaration(ctx.ast) {
                if let Some(func) = decl.as_function(ctx.ast) {
                    let fn_body = func.body(ctx.ast);
                    fns.push((stmt, fn_body));
                    continue;
                }
            }
            other_stmts.push(stmt);
        }

        if fns.is_empty() {
            return;
        }

        let mut new_stmts: Vec<Stmt> = Vec::new();

        for (fn_stmt, _fn_body) in fns {
            let decl = fn_stmt.as_declaration(ctx.ast).unwrap();
            let func = decl.as_function(ctx.ast).unwrap();
            let span = func.span(ctx.ast);
            let id = func.id(ctx.ast);
            let params = func.params(ctx.ast);
            let generator = func.generator(ctx.ast);
            let async_flag = func.r#async(ctx.ast);
            let type_params = func.type_parameters(ctx.ast);
            let return_type = func.return_type(ctx.ast);
            let fn_body = func.body(ctx.ast);

            if let Some(binding_id) = id {
                let name = ctx.ast.get_utf8(binding_id.name(ctx.ast)).to_owned();
                let name_utf8 = ctx.ast.add_utf8(&name);

                // Check if the function body references its own name (self-referencing)
                let is_self_ref = {
                    let body_stmts = fn_body.body(ctx.ast);
                    let mut found = false;
                    for stmt_id in body_stmts.iter() {
                        let stmt: Stmt = ctx.ast.get_node_in_sub_range(stmt_id);
                        if contains_ident_ref(stmt, &name, ctx) {
                            found = true;
                            break;
                        }
                    }
                    found
                };

                // Create function expression
                let fn_expr = ctx.ast.expr_function(
                    span,
                    FunctionType::FunctionExpression,
                    Some(binding_id),
                    generator,
                    async_flag,
                    params,
                    fn_body,
                    type_params,
                    return_type,
                );

                let binding_pat = ctx.ast.binding_pattern_binding_identifier(
                    span,
                    name_utf8,
                    swc_next_ecma_ast::TypedSubRange::empty(),
                    None,
                    false,
                );

                let declarator = ctx.ast.variable_declarator(span, binding_pat, Some(fn_expr), false);
                let declarators = ctx.ast.add_typed_sub_range([declarator]);
                let decl = ctx.ast.decl_variable_declaration(
                    span,
                    VariableKind::Let,
                    declarators,
                    false,
                );
                let new_stmt = Stmt::Declaration(decl);

                if is_self_ref {
                    other_stmts.push(new_stmt);
                } else {
                    new_stmts.push(new_stmt);
                }
            }
        }

        new_stmts.extend(other_stmts);

        let new_body = ctx.ast.add_typed_sub_range(new_stmts);
        node.set_body(ctx.ast, new_body);
    }
}

fn contains_ident_ref(stmt: Stmt, name: &str, ctx: &TransformerCtx<'_, '_>) -> bool {
    if let Some(expr_stmt) = stmt.as_expression_statement(ctx.ast) {
        return expr_contains_ident(expr_stmt.expression(ctx.ast), name, ctx);
    }
    if let Some(ret) = stmt.as_return_statement(ctx.ast) {
        if let Some(expr) = ret.argument(ctx.ast) {
            return expr_contains_ident(expr, name, ctx);
        }
    }
    if let Some(decl) = stmt.as_declaration(ctx.ast) {
        if let Some(func) = decl.as_function(ctx.ast) {
            let body = func.body(ctx.ast);
            for stmt_id in body.body(ctx.ast).iter() {
                let s: Stmt = ctx.ast.get_node_in_sub_range(stmt_id);
                if contains_ident_ref(s, name, ctx) {
                    return true;
                }
            }
        }
    }
    false
}

fn expr_contains_ident(expr: swc_next_ecma_ast::Expr, name: &str, ctx: &TransformerCtx<'_, '_>) -> bool {
    if let Some(ident) = expr.as_identifier_reference(ctx.ast) {
        return ctx.ast.get_utf8(ident.name(ctx.ast)) == name;
    }
    if let Some(call) = expr.as_call_expression(ctx.ast) {
        if expr_contains_ident(call.callee(ctx.ast), name, ctx) {
            return true;
        }
        for arg_id in call.arguments(ctx.ast).iter() {
            let arg: swc_next_ecma_ast::Argument = ctx.ast.get_node_in_sub_range(arg_id);
            if let Some(e) = arg.as_expr(ctx.ast) {
                if expr_contains_ident(e, name, ctx) {
                    return true;
                }
            }
        }
    }
    if let Some(bin) = expr.as_binary_expression(ctx.ast) {
        return expr_contains_ident(bin.left(ctx.ast), name, ctx)
            || expr_contains_ident(bin.right(ctx.ast), name, ctx);
    }
    if let Some(unary) = expr.as_unary_expression(ctx.ast) {
        return expr_contains_ident(unary.argument(ctx.ast), name, ctx);
    }
    if let Some(member) = expr.as_member_expression(ctx.ast) {
        return expr_contains_ident(member.object(ctx.ast), name, ctx);
    }
    if let Some(cond) = expr.as_conditional_expression(ctx.ast) {
        return expr_contains_ident(cond.test(ctx.ast), name, ctx)
            || expr_contains_ident(cond.consequent(ctx.ast), name, ctx)
            || expr_contains_ident(cond.alternate(ctx.ast), name, ctx);
    }
    if let Some(seq) = expr.as_sequence_expression(ctx.ast) {
        for e_id in seq.expressions(ctx.ast).iter() {
            let e: swc_next_ecma_ast::Expr = ctx.ast.get_node_in_sub_range(e_id);
            if expr_contains_ident(e, name, ctx) {
                return true;
            }
        }
    }
    false
}