use rustc_hash::FxHashMap;
use swc_next_ecma_ast::{
    ArrowFunctionBody, Ast, AstInternedUtf8, BindingPattern, DUMMY_SP, Decl, NodeId, NodeKindData,
    Stmt, TypedSubRange, VariableKind,
};

/// Collects statements to prepend to scopes or insert after existing statements.
///
/// Transforms enqueue edits while visiting descendants. All after-insertions
/// for a statement container are applied in one rebuild, and all generated
/// variables for a scope are prepended when that scope exits.
#[derive(Default)]
struct PendingStatements {
    /// Appended in parent-body traversal order so `flush` can merge linearly.
    insertions: Vec<Insertion>,
    prepend_vars: Vec<AstInternedUtf8>,
}

struct Insertion {
    anchor: NodeId,
    statement: Stmt,
}

#[derive(Default)]
pub(crate) struct StatementInjector {
    pending: FxHashMap<NodeId, PendingStatements>,
}

impl StatementInjector {
    pub(crate) fn insert_after(&mut self, container: NodeId, anchor: NodeId, statement: Stmt) {
        self.pending
            .entry(container)
            .or_default()
            .insertions
            .push(Insertion { anchor, statement });
    }

    pub(crate) fn prepend_var(&mut self, scope: NodeId, name: AstInternedUtf8, ast: &Ast<'_>) {
        let container = match ast.kind_data(scope) {
            NodeKindData::Program(_) | NodeKindData::StaticBlock(_) => scope,
            NodeKindData::Function(function) => function.body(ast).node_id(),
            NodeKindData::ArrowFunctionExpression(arrow) => arrow
                .body(ast)
                .as_function_body(ast)
                .map_or(scope, |body| body.node_id()),
            _ => unreachable!("generated var must belong to a var scope"),
        };
        self.pending
            .entry(container)
            .or_default()
            .prepend_vars
            .push(name);
    }

    pub(crate) fn flush(&mut self, container: NodeId, ast: &mut Ast<'_>) {
        if self.pending.is_empty() {
            return;
        }
        let Some(pending) = self.pending.remove(&container) else {
            return;
        };
        let body = statement_list(container, ast);
        let insertion_count = pending.insertions.len();
        let prepend_count = usize::from(!pending.prepend_vars.is_empty());
        let mut statements = Vec::with_capacity(body.len() + insertion_count + prepend_count);
        if prepend_count != 0 {
            statements.push(var_statement(pending.prepend_vars, ast));
        }

        let mut insertions = pending.insertions.into_iter().peekable();
        for id in body.iter() {
            let statement = ast.get_node_in_sub_range(id);
            statements.push(statement);
            while insertions
                .peek()
                .is_some_and(|insertion| insertion.anchor == statement.node_id())
            {
                statements.push(
                    insertions
                        .next()
                        .expect("peeked insertion must exist")
                        .statement,
                );
            }
        }

        assert!(
            insertions.next().is_none(),
            "statement insertions must follow parent body traversal order"
        );
        let body = ast.add_typed_sub_range(statements);
        set_statement_list(container, body, ast);
    }

    pub(crate) fn flush_arrow_expression(
        &mut self,
        arrow: swc_next_ecma_ast::ArrowFunctionExpression,
        ast: &mut Ast<'_>,
    ) {
        if self.pending.is_empty() {
            return;
        }
        let Some(pending) = self.pending.remove(&arrow.node_id()) else {
            return;
        };
        assert!(
            pending.insertions.is_empty(),
            "an expression-bodied arrow cannot contain statement insertions"
        );
        let statement = var_statement(pending.prepend_vars, ast);
        let body = arrow.body(ast);
        let expression = body
            .as_expr(ast)
            .expect("queued arrow scope must have an expression body");
        let return_statement = ast.stmt_return_statement(DUMMY_SP, Some(expression));
        let statements = ast.add_typed_sub_range([statement, return_statement]);
        let function_body = ast.function_body(DUMMY_SP, TypedSubRange::empty(), statements);
        arrow.set_expression(ast, false);
        arrow.set_body(ast, ArrowFunctionBody::FunctionBody(function_body));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

fn var_statement<'a>(names: Vec<AstInternedUtf8>, ast: &mut Ast<'a>) -> Stmt {
    let declarators: Vec<_> = names
        .into_iter()
        .map(|name| {
            let identifier =
                ast.binding_identifier(DUMMY_SP, name.into(), TypedSubRange::empty(), None, false);
            let pattern = BindingPattern::BindingIdentifier(identifier);
            ast.variable_declarator(DUMMY_SP, pattern, None, false)
        })
        .collect();
    let declarators = ast.add_typed_sub_range(declarators);
    let declaration = ast.variable_declaration(DUMMY_SP, VariableKind::Var, declarators, false);
    Stmt::Declaration(Decl::VariableDeclaration(declaration))
}

fn statement_list(node: NodeId, ast: &Ast<'_>) -> TypedSubRange<Stmt> {
    match ast.kind_data(node) {
        NodeKindData::Program(node) => node.body(ast),
        NodeKindData::FunctionBody(node) => node.body(ast),
        NodeKindData::BlockStatement(node) => node.body(ast),
        NodeKindData::StaticBlock(node) => node.body(ast),
        NodeKindData::SwitchCase(node) => node.consequent(ast),
        NodeKindData::TsModuleBlock(node) => node.body(ast),
        _ => unreachable!("node must be a statement container"),
    }
}

fn set_statement_list<'a>(node: NodeId, body: TypedSubRange<Stmt>, ast: &mut Ast<'a>) {
    match ast.kind_data(node) {
        NodeKindData::Program(node) => node.set_body(ast, body),
        NodeKindData::FunctionBody(node) => node.set_body(ast, body),
        NodeKindData::BlockStatement(node) => node.set_body(ast, body),
        NodeKindData::StaticBlock(node) => node.set_body(ast, body),
        NodeKindData::SwitchCase(node) => node.set_consequent(ast, body),
        NodeKindData::TsModuleBlock(node) => node.set_body(ast, body),
        _ => unreachable!("node must be a statement container"),
    }
}
