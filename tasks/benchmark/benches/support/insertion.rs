use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, DUMMY_SP, Lang, Stmt, StorageMetrics, StoreTag, TypedSubRange};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

pub const SOURCE: &str = concat!(
    "function outer(value) { return value; }\n",
    "outer(0);outer(1);outer(2);outer(3);",
    "outer(4);outer(5);outer(6);outer(7);",
    "outer(8);outer(9);outer(10);outer(11);",
    "outer(12);outer(13);outer(14);outer(15);"
);

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

/// Builds exactly one 16-node statement chunk without parser capacity
/// headroom, so the next four transform insertions isolate hole reuse.
pub fn setup_hole_boundary_ast() -> Ast<'static> {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let mut ast = Ast::new("", allocator);
    let name = ast.add_utf8("seed");
    for _ in 0..16 {
        let identifier = ast.expr_identifier_reference(DUMMY_SP, name);
        let call =
            ast.expr_call_expression(DUMMY_SP, identifier, None, TypedSubRange::empty(), false);
        ast.stmt_expression_statement(DUMMY_SP, call);
    }
    ast
}

/// Builds the expression and statement nodes used by transform insertion.
///
/// The nodes are intentionally left detached so this isolates node-store
/// allocation from child-list rebuilding in `extra_data`.
pub fn build_detached_statements(ast: &mut Ast<'_>, count: usize) -> Stmt {
    ast.begin_transform();
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
    ast.begin_transform();
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

pub fn insertion_metrics(ast: &Ast<'_>) -> [(StoreTag, StorageMetrics); 5] {
    [
        (StoreTag::Common, ast.storage_metrics(StoreTag::Common)),
        (
            StoreTag::Expression,
            ast.storage_metrics(StoreTag::Expression),
        ),
        (
            StoreTag::Statement,
            ast.storage_metrics(StoreTag::Statement),
        ),
        (
            StoreTag::NamePattern,
            ast.storage_metrics(StoreTag::NamePattern),
        ),
        (
            StoreTag::ClassElement,
            ast.storage_metrics(StoreTag::ClassElement),
        ),
    ]
}

#[allow(dead_code)]
pub fn policy_label(ast: &Ast<'_>) -> String {
    let expression = ast.storage_metrics(StoreTag::Expression);
    let statement = ast.storage_metrics(StoreTag::Statement);
    let name_pattern = ast.storage_metrics(StoreTag::NamePattern);
    let class_element = ast.storage_metrics(StoreTag::ClassElement);
    format!(
        "expr-{}+{}_stmt-{}+{}_name-{}+{}_class-element-{}+{}",
        expression.live_per_chunk,
        expression.holes_per_chunk,
        statement.live_per_chunk,
        statement.holes_per_chunk,
        name_pattern.live_per_chunk,
        name_pattern.holes_per_chunk,
        class_element.live_per_chunk,
        class_element.holes_per_chunk,
    )
}
