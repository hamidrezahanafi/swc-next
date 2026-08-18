//! ES2021: logical-assignment-operators — Convert `a ||= b`, `a &&= b` to `a || (a = b)`, `a && (a = b)`.

use swc_next_ecma_ast::{AssignmentExpression, AssignmentOperator, Expr, GetSpan, LogicalOperator};
use swc_next_ecma_semantic::ReferenceFlags;

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct LogicalAssignment;

impl LogicalAssignment {
    pub fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        let logical_op = match op {
            AssignmentOperator::LogicalOrAssign => LogicalOperator::Or,
            AssignmentOperator::LogicalAndAssign => LogicalOperator::And,
            _ => return,
        };

        let span = node.span(ctx.ast);
        let left = node.left(ctx.ast);

        // Only handle simple identifier: x ||= y → x || (x = y)
        if !left.is_simple_assignment_target(ctx.ast) {
            return;
        }
        let sat = left.as_simple_assignment_target(ctx.ast).unwrap();
        if !sat.is_identifier_reference(ctx.ast) {
            return;
        }
        let identifier = sat.as_identifier_reference(ctx.ast).unwrap();
        let reference = ctx.semantic().reference_of(identifier.node_id()).unwrap();
        let reference_data = ctx.semantic().reference(reference);
        let scope = ctx.semantic().scope_of(node.node_id());
        let old = Expr::AssignmentExpression(node);
        let mut read_flags = ReferenceFlags::default().with_space(reference_data.flags.space);
        let mut write_flags = ReferenceFlags::default()
            .with_space(reference_data.flags.space)
            .write_only();
        if reference_data.flags.is_dynamic() {
            read_flags = read_flags.dynamic();
            write_flags = write_flags.dynamic();
        }

        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let assignment = clone.root.as_assignment_expression(tx.ast()).unwrap();
            let right = assignment.right(tx.ast());
            let read = assignment
                .left(tx.ast())
                .as_simple_assignment_target(tx.ast())
                .and_then(|target| target.as_identifier_reference(tx.ast()))
                .unwrap();
            let (replacement, write) = tx.build_detached(|ast| {
                let target = ast.assignment_target_simple_assignment_target_identifier_reference(
                    span,
                    read.name(ast),
                );
                let target_identifier = target
                    .as_simple_assignment_target(ast)
                    .and_then(|target| target.as_identifier_reference(ast))
                    .unwrap();
                ast.set_syntax_context(
                    target_identifier.node_id(),
                    ast.syntax_context(read.node_id()),
                );
                let assignment =
                    ast.expr_assignment_expression(span, target, right, AssignmentOperator::Assign);
                (
                    ast.expr_logical_expression(
                        span,
                        Expr::IdentifierReference(read),
                        assignment,
                        logical_op,
                    ),
                    target_identifier,
                )
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            tx.update_reference_flags(reference, read_flags);
            let name = tx.ast().canonicalize_utf8(write.name(tx.ast()));
            tx.reference(
                scope,
                write.node_id(),
                name,
                write_flags,
                reference_data.symbol,
            );
        })
        .expect("logical-assignment mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{AssignmentExpression, AssignmentOperator, Ast, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        assignments: Vec<AssignmentExpression>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_assignment_expression(&mut self, node: AssignmentExpression) {
            node.visit_children_with(self);
            if matches!(
                node.operator(self.ast),
                AssignmentOperator::LogicalOrAssign | AssignmentOperator::LogicalAndAssign
            ) {
                self.assignments.push(node);
            }
        }
    }

    #[test]
    fn compound_references_split_into_read_and_write_occurrences() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("function f(x, y) { x ||= y; x &&= y; }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let assignments = {
            let mut collector = Collector {
                ast: &ast,
                assignments: Vec::new(),
            };
            parse.program.visit_children_with(&mut collector);
            collector.assignments
        };
        let mut options = TransformerOptions::default();
        options.es2021.logical_assignment = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let old_references: Vec<_> = assignments
            .iter()
            .map(|assignment| {
                let identifier = assignment
                    .left(&ast)
                    .as_simple_assignment_target(&ast)
                    .and_then(|target| target.as_identifier_reference(&ast))
                    .unwrap();
                semantic.reference_of(identifier.node_id()).unwrap()
            })
            .collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(
            old_references
                .into_iter()
                .all(|reference| semantic.reference(reference).flags.is_read_only())
        );
        assert_eq!(
            semantic
                .iter_references()
                .filter(|entry| {
                    ast.get_ast_str_bytes(entry.reference.name) == b"x"
                        && entry.reference.flags.is_write_only()
                })
                .count(),
            2
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
