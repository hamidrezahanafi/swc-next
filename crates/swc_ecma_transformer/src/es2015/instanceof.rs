//! ES2015: instanceof — Convert `a instanceof B` to `_instanceof(a, B)`.
//!
//! ```js
//! foo instanceof Bar → _instanceof(foo, Bar)
//! ```

use swc_next_ecma_ast::{Argument, BinaryExpression, BinaryOperator, Expr, GetSpan};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct InstanceOf;

impl InstanceOf {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if node.operator(ctx.ast) != BinaryOperator::Instanceof {
            return;
        }

        let old = Expr::BinaryExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let cloned = clone.root.as_binary_expression(tx.ast()).unwrap();
            let left = cloned.left(tx.ast());
            let right = cloned.right(tx.ast());
            let (replacement, helper) = tx.build_detached(|ast| {
                let helper_name = ast.add_utf8("_instanceof");
                let helper = ast.identifier_reference(node.span(ast), helper_name);
                let args = ast.add_typed_sub_range([Argument::Expr(left), Argument::Expr(right)]);
                let call = ast.call_expression(
                    node.span(ast),
                    Expr::IdentifierReference(helper),
                    args,
                    false,
                    None,
                );
                (Expr::CallExpression(call), helper)
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            let name = tx.ast().canonicalize_utf8(helper.name(tx.ast()));
            let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
            tx.reference(
                scope,
                helper.node_id(),
                name,
                ReferenceFlags::default(),
                symbol,
            );
        })
        .expect("instanceof mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, BinaryExpression, BinaryOperator, NodeKind, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        expression: Option<BinaryExpression>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_binary_expression(&mut self, node: BinaryExpression) {
            node.visit_children_with(self);
            if node.operator(self.ast) == BinaryOperator::Instanceof {
                self.expression = Some(node);
            }
        }
    }

    #[test]
    fn helper_call_preserves_operand_references() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("function f(a, B) { return a instanceof B; }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut collector = Collector {
            ast: &ast,
            expression: None,
        };
        parse.program.visit_children_with(&mut collector);
        let old = collector.expression.unwrap();
        let mut options = TransformerOptions::default();
        options.es2015.instanceof = true;
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

        assert_eq!(ast.node_kind(old.node_id()), NodeKind::CallExpression);
        assert!(semantic.iter_references().any(|entry| {
            ast.get_ast_str_bytes(entry.reference.name) == b"_instanceof"
                && semantic.parent_of(entry.reference.node) == Some(old.node_id())
        }));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
