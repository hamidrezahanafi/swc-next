//! ES2020: nullish-coalescing — Convert `a ?? b` to `a !== null && a !== void 0 ? a : b`.

use swc_next_ecma_ast::{
    AssignmentExpression, AssignmentOperator, BinaryOperator, Expr, GetSpan, LogicalExpression,
    LogicalOperator, NumericLiteralKind, UnaryOperator,
};
use swc_next_ecma_semantic::ReferenceFlags;

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct NullishCoalescing;

impl NullishCoalescing {
    pub fn exit_logical_expression(
        &mut self,
        node: LogicalExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.operator(ctx.ast) != LogicalOperator::NullishCoalescing {
            return;
        }

        let span = node.span(ctx.ast);
        let old = Expr::LogicalExpression(node);
        let source_left = node.left(ctx.ast);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let logical = clone.root.as_logical_expression(tx.ast()).unwrap();
            let first = logical.left(tx.ast());
            let right = logical.right(tx.ast());
            let second = tx.clone_detached(source_left);
            let third = tx.clone_detached(source_left);
            let replacement = tx.build_detached(|ast| {
                build_nullish_conditional(span, first, second.root, third.root, right, ast)
            });
            clone.replace_root(replacement);
            let layout = tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            tx.duplicate_semantics_from_detached_clone(&second, &layout);
            tx.duplicate_semantics_from_detached_clone(&third, &layout);
        })
        .expect("nullish-coalescing mutation must pass preflight");
    }

    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.operator(ctx.ast) != AssignmentOperator::NullishAssign {
            return;
        }

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);

        if !left.is_simple_assignment_target(ctx.ast) {
            return;
        }
        let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
        if !sat.is_identifier_reference(ctx.ast) {
            return;
        }
        let ident = sat.as_identifier_reference(ctx.ast).unwrap();

        let old = Expr::AssignmentExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        let reference = ctx.semantic().reference_of(ident.node_id()).unwrap();
        let reference_data = ctx.semantic().reference(reference);
        let mut write_flags = ReferenceFlags::default()
            .with_space(reference_data.flags.space)
            .write_only();
        let mut read_flags = ReferenceFlags::default().with_space(reference_data.flags.space);
        if reference_data.flags.is_dynamic() {
            write_flags = write_flags.dynamic();
            read_flags = read_flags.dynamic();
        }

        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let assignment = clone.root.as_assignment_expression(tx.ast()).unwrap();
            let left = assignment.left(tx.ast());
            let right = assignment.right(tx.ast());
            let cloned_ident = left
                .as_simple_assignment_target(tx.ast())
                .and_then(|target| target.as_identifier_reference(tx.ast()))
                .unwrap();
            let context = tx.ast().syntax_context(cloned_ident.node_id());
            let name = cloned_ident.name(tx.ast());
            let (replacement, reads) = tx.build_detached(|ast| {
                let reads: [swc_next_ecma_ast::IdentifierReference; 3] =
                    std::array::from_fn(|_| {
                        let read = ast.identifier_reference(span, name);
                        ast.set_syntax_context(read.node_id(), context);
                        read
                    });
                let ternary = build_nullish_conditional(
                    span,
                    Expr::IdentifierReference(reads[0]),
                    Expr::IdentifierReference(reads[1]),
                    Expr::IdentifierReference(reads[2]),
                    right,
                    ast,
                );
                let assignment =
                    ast.expr_assignment_expression(span, left, ternary, AssignmentOperator::Assign);
                (assignment, reads)
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            tx.update_reference_flags(reference, write_flags);
            let name = tx.ast().canonicalize_utf8(name);
            for read in reads {
                tx.reference(
                    scope,
                    read.node_id(),
                    name,
                    read_flags,
                    reference_data.symbol,
                );
            }
        })
        .expect("nullish assignment mutation must pass preflight");
    }
}

fn build_nullish_conditional(
    span: swc_next_ecma_ast::Span,
    first: Expr,
    second: Expr,
    third: Expr,
    right: Expr,
    ast: &mut swc_next_ecma_ast::Ast<'_>,
) -> Expr {
    let zero = ast.add_utf8("0");
    let zero_lit = ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero);
    let void0 = ast.expr_unary_expression(span, zero_lit, UnaryOperator::Void);
    let null_val = ast.expr_null_literal(span);
    let not_null =
        ast.expr_binary_expression(span, first, null_val, BinaryOperator::StrictNotEqual);
    let not_void = ast.expr_binary_expression(span, second, void0, BinaryOperator::StrictNotEqual);
    let test = ast.expr_logical_expression(span, not_null, not_void, LogicalOperator::And);
    ast.expr_conditional_expression(span, test, third, right)
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

    #[test]
    fn duplicated_occurrences_receive_independent_semantic_identities() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function f(value, fallback) { value ??= fallback; return (() => value) ?? fallback; }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2020.nullish_coalescing = true;
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
}
