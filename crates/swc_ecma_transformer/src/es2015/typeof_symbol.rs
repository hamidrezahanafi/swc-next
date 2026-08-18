//! ES2015: typeof-symbol — Convert `typeof` to handle Symbol type.
//!
//! ```js
//! typeof sym → typeof sym === "undefined" ? "undefined" : _typeof(sym)
//! typeof Symbol() → _typeof(Symbol())
//! ```

use swc_next_ecma_ast::{
    Argument, BinaryExpression, BinaryOperator, Expr, GetSpan, IdentifierReference,
    UnaryExpression, UnaryOperator,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct TypeofSymbol {
    /// When non-zero, skip transforming typeof expressions inside binary
    /// comparisons like `typeof x === "string"`.
    skip_depth: u32,
}

impl TypeofSymbol {
    pub fn new() -> Self {
        Self { skip_depth: 0 }
    }

    pub fn enter_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if !matches!(
            op,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictEqual
                | BinaryOperator::StrictNotEqual
        ) {
            return;
        }

        let left = node.left(ctx.ast);
        if left.is_unary_expression(ctx.ast) {
            let unary = left.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let right = node.right(ctx.ast);
                if is_non_symbol_literal(right, ctx.ast) {
                    self.skip_depth += 1;
                    return;
                }
            }
        }

        let right = node.right(ctx.ast);
        if right.is_unary_expression(ctx.ast) {
            let unary = right.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let left = node.left(ctx.ast);
                if is_non_symbol_literal(left, ctx.ast) {
                    self.skip_depth += 1;
                }
            }
        }
    }

    pub fn exit_binary_expression(
        &mut self,
        node: BinaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let op = node.operator(ctx.ast);
        if !matches!(
            op,
            BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictEqual
                | BinaryOperator::StrictNotEqual
        ) {
            return;
        }

        let left = node.left(ctx.ast);
        if left.is_unary_expression(ctx.ast) {
            let unary = left.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let right = node.right(ctx.ast);
                if is_non_symbol_literal(right, ctx.ast) {
                    self.skip_depth = self.skip_depth.saturating_sub(1);
                    return;
                }
            }
        }

        let right = node.right(ctx.ast);
        if right.is_unary_expression(ctx.ast) {
            let unary = right.as_unary_expression(ctx.ast).unwrap();
            if unary.operator(ctx.ast) == UnaryOperator::Typeof {
                let left = node.left(ctx.ast);
                if is_non_symbol_literal(left, ctx.ast) {
                    self.skip_depth = self.skip_depth.saturating_sub(1);
                }
            }
        }
    }

    pub fn exit_unary_expression(
        &mut self,
        node: UnaryExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.skip_depth > 0 {
            return;
        }
        if node.operator(ctx.ast) != UnaryOperator::Typeof {
            return;
        }

        let span = node.span(ctx.ast);
        let old = Expr::UnaryExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let unary = clone.root.as_unary_expression(tx.ast()).unwrap();
            let argument = unary.argument(tx.ast());
            let identifier = argument.as_identifier_reference(tx.ast());
            let (replacement, helper, value_reference) = tx.build_detached(|ast| {
                let helper_name = ast.add_utf8("_typeof");
                let helper = ast.identifier_reference(span, helper_name);
                let value_reference = identifier.map(|identifier| {
                    let reference = ast.identifier_reference(span, identifier.name(ast));
                    ast.set_syntax_context(
                        reference.node_id(),
                        ast.syntax_context(identifier.node_id()),
                    );
                    reference
                });
                let call_argument = value_reference
                    .map(Expr::IdentifierReference)
                    .unwrap_or(argument);
                let call_args = ast.add_typed_sub_range([Argument::Expr(call_argument)]);
                let call = ast.expr_call_expression(
                    span,
                    Expr::IdentifierReference(helper),
                    call_args,
                    false,
                    None,
                );
                let replacement = if identifier.is_some() {
                    let undefined_value = ast.add_wtf8("undefined".into());
                    let undefined_raw = ast.add_utf8("\"undefined\"");
                    let undefined = ast.expr_string_literal(span, undefined_value, undefined_raw);
                    let typeof_argument =
                        ast.expr_unary_expression(span, argument, UnaryOperator::Typeof);
                    let test = ast.expr_binary_expression(
                        span,
                        typeof_argument,
                        undefined,
                        BinaryOperator::StrictEqual,
                    );
                    let consequent_value = ast.add_wtf8("undefined".into());
                    let consequent_raw = ast.add_utf8("\"undefined\"");
                    let consequent =
                        ast.expr_string_literal(span, consequent_value, consequent_raw);
                    ast.expr_conditional_expression(span, test, consequent, call)
                } else {
                    call
                };
                (replacement, helper, value_reference)
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            register_value_reference(tx, scope, helper);
            if let Some(reference) = value_reference {
                register_value_reference(tx, scope, reference);
            }
        })
        .expect("typeof-symbol mutation must pass preflight");
    }
}

fn register_value_reference(
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
    scope: swc_next_ecma_ast::ScopeId,
    reference: IdentifierReference,
) {
    let name = tx.ast().canonicalize_utf8(reference.name(tx.ast()));
    let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
    tx.reference(
        scope,
        reference.node_id(),
        name,
        ReferenceFlags::default(),
        symbol,
    );
}

fn is_non_symbol_literal(expr: Expr, ast: &swc_next_ecma_ast::Ast) -> bool {
    let Some(s) = expr.as_string_literal(ast) else {
        return false;
    };
    let value = ast.get_wtf8(s.value(ast));
    matches!(
        value.as_str(),
        Some("undefined" | "boolean" | "number" | "string" | "function")
    )
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, UnaryExpression, UnaryOperator, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        unary: Option<UnaryExpression>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_unary_expression(&mut self, node: UnaryExpression) {
            node.visit_children_with(self);
            if node.operator(self.ast) == UnaryOperator::Typeof {
                self.unary = Some(node);
            }
        }
    }

    #[test]
    fn identifier_has_distinct_typeof_and_value_references() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("function f(x) { return typeof x; }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let unary = {
            let mut collector = Collector {
                ast: &ast,
                unary: None,
            };
            parse.program.visit_children_with(&mut collector);
            collector.unary.unwrap()
        };
        let mut options = TransformerOptions::default();
        options.es2015.typeof_symbol = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let original = semantic
            .reference_of(unary.argument(&ast).node_id())
            .unwrap();
        let original_flags = semantic.reference(original).flags;

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(semantic.reference(original).flags, original_flags);
        assert_eq!(
            semantic
                .iter_references()
                .filter(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"x")
                .count(),
            2
        );
        assert!(
            semantic
                .iter_references()
                .any(|entry| { ast.get_ast_str_bytes(entry.reference.name) == b"_typeof" })
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
