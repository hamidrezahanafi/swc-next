//! ES2015 `for..of` lowering for the array fast path.

use std::collections::HashSet;

use swc_next_ecma_ast::{
    BinaryOperator, BindingPattern, Expr, ForOfStatement, ForStatementLeft, GetSpan,
    NumericLiteralKind, PropertyKey, SimpleAssignmentTarget, Stmt, UpdateOperator, VariableKind,
};
use swc_next_ecma_semantic::{ReferenceFlags, SymbolFlags, scope};

use crate::{
    mutation::{DetachedScopeRegion, ScopeHandle},
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct ForOf;

impl ForOf {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_for_of_statement(
        &mut self,
        node: ForOfStatement,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.r#await(ctx.ast) {
            return;
        }

        let old = Stmt::ForOfStatement(node);
        let source_right = node.right(ctx.ast);
        let existing_body_scope = node
            .body(ctx.ast)
            .as_block_statement(ctx.ast)
            .map(|body| ctx.semantic().scope_of(body.node_id()));
        let for_scope = ctx.semantic().scope_of(node.node_id());
        let hoist_scope = ctx.semantic().scope(for_scope).hoist_target;
        let lexical_symbols = lexical_left_symbols(node.left(ctx.ast), ctx);
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let for_of = clone.root.as_for_of_statement(tx.ast()).unwrap();
            let span = for_of.span(tx.ast());
            let left = for_of.left(tx.ast());
            let right = for_of.right(tx.ast());
            let source_body = for_of.body(tx.ast());
            let right_copy = tx.clone_detached(source_right);
            let iterator = tx.declare_private(hoist_scope, "_i", SymbolFlags::FUNCTION_SCOPED_VAR);

            let built = tx.build_detached(|ast| {
                build_for_statement(
                    span,
                    left,
                    right,
                    right_copy.root,
                    source_body,
                    iterator,
                    ast,
                )
            });
            clone.replace_root(built.statement);

            let mut regions = Vec::new();
            let body_scope = if built.generated_body {
                let flags = scope::ScopeFlags {
                    strict: tx.semantic().scope(for_scope).flags.strict,
                    ..scope::ScopeFlags::default()
                };
                let pending = tx.create_scope(
                    for_scope,
                    built.body.node_id(),
                    scope::ScopeKind::Block,
                    flags,
                );
                regions.push(DetachedScopeRegion {
                    root: built.body.node_id(),
                    scope: ScopeHandle::Pending(pending),
                });
                ScopeHandle::Pending(pending)
            } else {
                ScopeHandle::Existing(
                    existing_body_scope.expect("an existing block body has a scope"),
                )
            };

            let layout =
                tx.replace_statement_from_transformed_clone(old, clone, for_scope, &regions);
            tx.duplicate_semantics_from_detached_clone(&right_copy, &layout);
            for symbol in lexical_symbols {
                tx.move_symbol(symbol, body_scope, std::iter::empty());
            }
            for &(reference, flags, in_body) in &built.references {
                tx.reference(
                    if in_body {
                        body_scope
                    } else {
                        ScopeHandle::Existing(for_scope)
                    },
                    reference.node_id(),
                    tx.ast().canonicalize_utf8(reference.name(tx.ast())),
                    flags,
                    Some(iterator.symbol),
                );
            }
        })
        .expect("for-of mutation must pass preflight");
    }
}

struct BuiltForOf {
    statement: Stmt,
    body: swc_next_ecma_ast::BlockStatement,
    generated_body: bool,
    references: Vec<(swc_next_ecma_ast::IdentifierReference, ReferenceFlags, bool)>,
}

fn build_for_statement(
    span: swc_next_ecma_ast::Span,
    left: ForStatementLeft,
    right: Expr,
    right_copy: Expr,
    source_body: Stmt,
    iterator: crate::mutation::PrivateBinding,
    ast: &mut swc_next_ecma_ast::Ast<'_>,
) -> BuiltForOf {
    let zero = ast.add_utf8("0");
    let zero = ast.expr_numeric_literal(
        swc_next_ecma_ast::DUMMY_SP,
        NumericLiteralKind::Decimal,
        zero,
    );
    let iterator_declarator = ast.variable_declarator(
        swc_next_ecma_ast::DUMMY_SP,
        BindingPattern::BindingIdentifier(iterator.declaration),
        Some(zero),
        false,
    );

    let test_reference = generated_reference(ast, iterator);
    let length = ast.add_utf8("length");
    let length = ast.property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, length);
    let length =
        ast.expr_member_expression(swc_next_ecma_ast::DUMMY_SP, right, length, false, false);
    let test = ast.expr_binary_expression(
        swc_next_ecma_ast::DUMMY_SP,
        Expr::IdentifierReference(test_reference),
        length,
        BinaryOperator::LessThan,
    );

    let update_reference = generated_reference(ast, iterator);
    let update_target = SimpleAssignmentTarget::IdentifierReference(update_reference);
    let update = ast.expr_update_expression(
        swc_next_ecma_ast::DUMMY_SP,
        update_target,
        UpdateOperator::Increment,
        false,
    );

    let element_reference = generated_reference(ast, iterator);
    let element = ast.expr_member_expression(
        swc_next_ecma_ast::DUMMY_SP,
        right_copy,
        PropertyKey::Expr(Expr::IdentifierReference(element_reference)),
        true,
        false,
    );
    let assignment = make_assignment(left, element, ast);

    let (body, generated_body) = if let Some(body) = source_body.as_block_statement(ast) {
        let statements = std::iter::once(assignment)
            .chain(
                body.body(ast)
                    .iter()
                    .map(|id| ast.get_node_in_sub_range(id)),
            )
            .collect::<Vec<_>>();
        let statements = ast.add_typed_sub_range(statements);
        body.set_body(ast, statements);
        (body, false)
    } else {
        let statements = ast.add_typed_sub_range([assignment, source_body]);
        (ast.block_statement(span, statements), true)
    };

    let declarators = ast.add_typed_sub_range([iterator_declarator]);
    let init = ast.for_statement_init_variable_declaration(
        swc_next_ecma_ast::DUMMY_SP,
        VariableKind::Var,
        declarators,
        false,
    );
    let statement = ast.stmt_for_statement(
        span,
        Some(init),
        Some(test),
        Some(update),
        Stmt::BlockStatement(body),
    );
    BuiltForOf {
        statement,
        body,
        generated_body,
        references: vec![
            (test_reference, ReferenceFlags::default(), false),
            (
                update_reference,
                ReferenceFlags::default().read_write(),
                false,
            ),
            (element_reference, ReferenceFlags::default(), true),
        ],
    }
}

fn generated_reference(
    ast: &mut swc_next_ecma_ast::Ast<'_>,
    binding: crate::mutation::PrivateBinding,
) -> swc_next_ecma_ast::IdentifierReference {
    let reference = ast.identifier_reference(swc_next_ecma_ast::DUMMY_SP, binding.text);
    ast.set_syntax_context(reference.node_id(), binding.context);
    reference
}

fn make_assignment(
    left: ForStatementLeft,
    element: Expr,
    ast: &mut swc_next_ecma_ast::Ast<'_>,
) -> Stmt {
    if let Some(declaration) = left.as_variable_declaration(ast) {
        let declarator = declaration.declarators(ast).get_node(ast, 0).unwrap();
        let declarator = ast.variable_declarator(
            swc_next_ecma_ast::DUMMY_SP,
            declarator.id(ast),
            Some(element),
            false,
        );
        let declarators = ast.add_typed_sub_range([declarator]);
        ast.stmt_decl_variable_declaration(
            swc_next_ecma_ast::DUMMY_SP,
            declaration.kind(ast),
            declarators,
            false,
        )
    } else {
        let assignment = ast.expr_assignment_expression(
            swc_next_ecma_ast::DUMMY_SP,
            left.as_assignment_target(ast).unwrap(),
            element,
            swc_next_ecma_ast::AssignmentOperator::Assign,
        );
        ast.stmt_expression_statement(swc_next_ecma_ast::DUMMY_SP, assignment)
    }
}

fn lexical_left_symbols(
    left: ForStatementLeft,
    ctx: &TransformerCtx<'_, '_, '_>,
) -> Vec<swc_next_ecma_ast::SymbolId> {
    let Some(declaration) = left.as_variable_declaration(ctx.ast) else {
        return Vec::new();
    };
    if declaration.kind(ctx.ast) == VariableKind::Var {
        return Vec::new();
    }
    let mut symbols = HashSet::new();
    let mut pending = vec![declaration.node_id()];
    while let Some(node) = pending.pop() {
        if let Some(declaration) = ctx.semantic().declaration_of(node) {
            symbols.insert(ctx.semantic().declaration(declaration).symbol);
        }
        pending.extend(ctx.semantic().node_children(node));
    }
    symbols.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    fn lower_and_assert(source: &str) {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.for_of = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        Transformer::transform(parse.program, &mut ast, &mut semantic, options);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_lexical_and_assignment_heads_with_consistent_semantics() {
        lower_and_assert(
            "function f(values, target) { for (let value of values) use(value); for (target of values) { use(target); } }",
        );
    }
}
