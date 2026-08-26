use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, DUMMY_SP, Lang, Stmt, TypedSubRange};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

pub const SOURCE: &str = "function outer(value) { return value; }\nouter(0);";

pub fn parse_program(allocator: &Allocator) -> Ast<'_> {
    let result = Parser::init(
        allocator,
        SOURCE,
        Options {
            lang: Lang::Js,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    result.ast
}

pub fn setup_program() -> Ast<'static> {
    let allocator = Box::leak(Box::new(Allocator::new()));
    parse_program(allocator)
}

/// Builds the expression and statement nodes used by transform insertion.
///
/// The nodes are intentionally left detached so this isolates node-store
/// allocation from child-list rebuilding in `extra_data`.
pub fn build_detached_statements(ast: &mut Ast<'_>, count: usize) -> Stmt {
    let name = ast.add_utf8("inserted");
    let mut last = None;
    for _ in 0..count {
        let callee = ast.expr_identifier_reference(DUMMY_SP, name);
        let call = ast.expr_call_expression(DUMMY_SP, callee, None, TypedSubRange::empty(), false);
        last = Some(ast.stmt_expression_statement(DUMMY_SP, call));
    }
    last.expect("insertion benchmark count must be non-zero")
}

/// Builds transform-like statements and attaches them to the Program body.
pub fn insert_program_statements(ast: &mut Ast<'_>, count: usize) -> usize {
    let program = ast.root_program();
    let old_body = program.body(ast);
    let mut body = Vec::with_capacity(old_body.len() + count);
    body.extend(old_body.iter().map(|id| ast.get_node_in_sub_range(id)));

    let name = ast.add_utf8("inserted");
    for _ in 0..count {
        let callee = ast.expr_identifier_reference(DUMMY_SP, name);
        let call = ast.expr_call_expression(DUMMY_SP, callee, None, TypedSubRange::empty(), false);
        body.push(ast.stmt_expression_statement(DUMMY_SP, call));
    }

    let len = body.len();
    let body = ast.add_typed_sub_range(body);
    program.set_body(ast, body);
    len
}

pub fn setup_inserted_program(count: usize) -> Ast<'static> {
    let mut ast = setup_program();
    insert_program_statements(&mut ast, count);
    ast
}
