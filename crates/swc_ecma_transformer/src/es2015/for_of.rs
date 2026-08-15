//! ES2015: for_of — Transform `for-of` loops into standard `for` loops.
//!
//! ```js
//! // Input:
//! for (var i of arr) { body }
//!
//! // Output (array case):
//! for (var _i = 0; _i < arr.length; _i++) {
//!   var i = arr[_i];
//!   body
//! }
//! ```
//!
//! TODO: Support the full iterator protocol with try/catch/finally.
//! TODO: Support `for await...of`.

use swc_next_ecma_ast::{
    Ast, BinaryOperator, Expr, ForStatementLeft, GetSpan, Program, SimpleAssignmentTarget, Stmt,
    UpdateOperator, VariableKind,
};

use swc_next_ecma_semantic::Semantic;

pub(super) struct ForOf;

impl ForOf {
    pub fn new() -> Self {
        Self
    }

    /// Program-level transform: walk all statements and replace `for-of` loops.
    pub fn transform<'a>(
        &mut self,
        program: Program,
        ast: &mut Ast<'a>,
        _semantic: &mut Semantic<'a>,
    ) {
        walk_stmts_for_of(program, ast);
    }
}

fn walk_stmts_for_of(program: Program, ast: &mut Ast) {
    let body = program.body(ast);
    let len = body.len();
    let mut new_body: Vec<Stmt> = Vec::new();
    for i in 0..len {
        let stmt = body.get_node(ast, i).unwrap();
        process_stmt(stmt, ast, &mut new_body);
    }
    let new_range = ast.add_typed_sub_range(new_body);
    program.set_body(ast, new_range);
}

fn process_stmt(stmt: Stmt, ast: &mut Ast, out: &mut Vec<Stmt>) {
    if stmt.is_for_of_statement(ast) {
        let for_of = stmt.as_for_of_statement(ast).unwrap();
        let span = for_of.span(ast);
        let left = for_of.left(ast);
        let right = for_of.right(ast);
        let body = for_of.body(ast);
        let is_await = for_of.r#await(ast);

        if is_await {
            out.push(stmt);
        } else {
            let replacement = fold_for_of_stmt(span, left, right, body, ast);
            out.push(replacement);
        }
    } else {
        // Recurse into block statements
        if stmt.is_block_statement(ast) {
            let block = stmt.as_block_statement(ast).unwrap();
            let body = block.body(ast);
            let len = body.len();
            let mut new_stmts: Vec<Stmt> = Vec::new();
            for i in 0..len {
                let inner = body.get_node(ast, i).unwrap();
                process_stmt(inner, ast, &mut new_stmts);
            }
            let new_range = ast.add_typed_sub_range(new_stmts);
            block.set_body(ast, new_range);
        }
        // For statement with body
        if stmt.is_for_statement(ast) {
            let for_stmt = stmt.as_for_statement(ast).unwrap();
            let body = for_stmt.body(ast);
            let mut new_body: Vec<Stmt> = Vec::new();
            process_stmt(body, ast, &mut new_body);
            if new_body.len() == 1 {
                for_stmt.set_body(ast, new_body.into_iter().next().unwrap());
            }
        }
        // Labeled statement
        if stmt.is_labeled_statement(ast) {
            let labeled = stmt.as_labeled_statement(ast).unwrap();
            let body = labeled.body(ast);
            let mut new_body: Vec<Stmt> = Vec::new();
            process_stmt(body, ast, &mut new_body);
            if new_body.len() == 1 {
                labeled.set_body(ast, new_body.into_iter().next().unwrap());
            }
        }
        out.push(stmt);
    }
}

fn fold_for_of_stmt(
    span: swc_next_ecma_ast::Span,
    left: ForStatementLeft,
    right: Expr,
    body: Stmt,
    ast: &mut Ast,
) -> Stmt {
    let d = swc_next_ecma_ast::DUMMY_SP;

    // Ensure body is a block statement
    let body = if body.is_block_statement(ast) {
        body.as_block_statement(ast).unwrap()
    } else {
        let stmts = ast.add_typed_sub_range([body]);
        ast.block_statement(span, stmts)
    };

    // Create iterator variable: _i
    let i_name = ast.add_utf8("_i");
    let i_binding = ast.binding_pattern_simple_assignment_target_identifier_reference(d, i_name);
    let zero_str = ast.add_utf8("0");
    let zero = ast.expr_numeric_literal(d, NumericLiteralKind::Decimal, zero_str);

    // var _i = 0
    let i_declarator = ast.variable_declarator(d, i_binding, Some(zero), false);

    // test: _i < arr.length
    let length_name = ast.add_utf8("length");
    let length_key = ast.property_key_identifier_name(d, length_name);
    let right_len = ast.expr_member_expression(d, right.clone(), length_key, false, false);
    let i_ref = ast.expr_identifier_reference(d, i_name);
    let test = ast.expr_binary_expression(d, i_ref, right_len, BinaryOperator::LessThan);

    // update: _i++
    let i_id = ast.identifier_reference(d, i_name);
    let i_target = SimpleAssignmentTarget::IdentifierReference(i_id.into());
    let update = ast.expr_update_expression(d, i_target, UpdateOperator::Increment, false);

    // elem: arr[_i]
    let computed_key = ast.property_key_expr_identifier_reference(d, i_name);
    let elem = ast.expr_member_expression(d, right, computed_key, true, false);

    // Build body with assignment prepended
    let body_stmts = body.body(ast);
    let mut new_stmts: Vec<Stmt> = Vec::new();
    for i in 0..body_stmts.len() {
        new_stmts.push(body_stmts.get_node(ast, i).unwrap());
    }

    let assign_stmt = make_assign_stmt(left, elem, ast);
    new_stmts.insert(0, assign_stmt);

    let new_body_stmts = ast.add_typed_sub_range(new_stmts);
    let new_body = ast.block_statement(span, new_body_stmts);

    // Build the for statement
    let i_declarators = ast.add_typed_sub_range([i_declarator]);
    let init = ast.for_statement_init_variable_declaration(
        d, VariableKind::Var, i_declarators, false,
    );
    ast.stmt_for_statement(
        span,
        Some(init),
        Some(test),
        Some(update),
        Stmt::BlockStatement(new_body.into()),
    )
}

fn make_assign_stmt(
    left: ForStatementLeft,
    elem: Expr,
    ast: &mut Ast,
) -> Stmt {
    let d = swc_next_ecma_ast::DUMMY_SP;
    if left.is_variable_declaration(ast) {
        let var_decl = left.as_variable_declaration(ast).unwrap();
        let kind = var_decl.kind(ast);
        let decls = var_decl.declarators(ast);
        let decl = decls.get_node(ast, 0).unwrap();
        let id = decl.id(ast);

        let new_decl = ast.variable_declarator(d, id, Some(elem), false);
        let new_decls = ast.add_typed_sub_range([new_decl]);
        ast.stmt_decl_variable_declaration(d, kind, new_decls, false)
    } else {
        let target = left.as_assignment_target(ast).unwrap();
        let assign = ast.expr_assignment_expression(
            d, target, elem,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        ast.stmt_expression_statement(d, assign)
    }
}

use swc_next_ecma_ast::NumericLiteralKind;