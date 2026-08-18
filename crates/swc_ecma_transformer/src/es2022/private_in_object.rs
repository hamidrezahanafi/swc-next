//! ES2022: private-in-object — Transform `#x in obj` to `_checkInRHS(obj)`.
//!
//! ```js
//! #x in obj → _checkInRHS(obj)
//! ```

use swc_next_ecma_ast::{Argument, BinaryExpression, BinaryOperator, Expr, GetSpan};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct PrivateInObject;

impl PrivateInObject {
    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if op != BinaryOperator::In {
            return;
        }

        let left = node.left(ctx.ast);
        if !left.is_private_identifier(ctx.ast) {
            return;
        }

        let old = Expr::BinaryExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let cloned = clone.root.as_binary_expression(tx.ast()).unwrap();
            let right = cloned.right(tx.ast());
            let (replacement, helper) = tx.build_detached(|ast| {
                let helper_name = ast.add_utf8("_checkInRHS");
                let helper = ast.identifier_reference(node.span(ast), helper_name);
                let args = ast.add_typed_sub_range([Argument::Expr(right)]);
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
            let helper_name = tx.ast().canonicalize_utf8(helper.name(tx.ast()));
            let symbol = tx
                .semantic()
                .lookup(scope, helper_name, ReferenceSpace::Value);
            tx.reference(
                scope,
                helper.node_id(),
                helper_name,
                ReferenceFlags::default(),
                symbol,
            );
        })
        .expect("private-in-object mutation must pass preflight");
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

    struct PrivateInCollector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        expressions: Vec<BinaryExpression>,
    }

    impl<'ast> Visit<'ast> for PrivateInCollector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_binary_expression(&mut self, node: BinaryExpression) {
            node.visit_children_with(self);
            if node.operator(self.ast) == BinaryOperator::In
                && node.left(self.ast).is_private_identifier(self.ast)
            {
                self.expressions.push(node);
            }
        }
    }

    #[test]
    fn private_in_replacement_retires_key_and_registers_helper_reference() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("class C { #x; has(o) { return #x in o; } }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut collector = PrivateInCollector {
            ast: &ast,
            expressions: Vec::new(),
        };
        parse.program.visit_children_with(&mut collector);
        let old = collector.expressions[0];
        let old_key = old.left(&ast).node_id();

        let mut options = TransformerOptions::default();
        options.es2022.private_in_object = true;
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
        assert!(!semantic.is_node_active(old_key));
        let helper = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"_checkInRHS")
            .unwrap();
        assert_eq!(
            semantic.parent_of(helper.reference.node),
            Some(old.node_id())
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
