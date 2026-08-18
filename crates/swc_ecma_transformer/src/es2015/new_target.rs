//! ES2015: new-target — Convert `new.target` meta property.
//!
//! ```js
//! // Inside constructor:
//! new.target → this.constructor
//!
//! // Inside method:
//! new.target → void 0
//!
//! // Inside function Foo:
//! new.target → this instanceof Foo ? this.constructor : void 0
//! ```

use swc_next_ecma_ast::{
    AstNode, BinaryOperator, Expr, GetSpan, IdentifierReference, MetaProperty, MethodDefinition,
    MethodDefinitionKind, NodeKind, NumericLiteralKind, UnaryOperator,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct NewTarget;

impl NewTarget {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_meta_property(&mut self, node: MetaProperty, ctx: &mut TransformerCtx<'_, '_, '_>) {
        let meta = node.meta(ctx.ast);
        let property = node.property(ctx.ast);
        let meta_name = ctx.ast.get_utf8(meta.name(ctx.ast));
        let prop_name = ctx.ast.get_utf8(property.name(ctx.ast));
        if meta_name != "new" || prop_name != "target" {
            return;
        }

        let span = node.span(ctx.ast);

        // Walk ancestors to find the enclosing function/method
        let mut is_constructor = false;
        let mut is_method = false;
        let mut func_node: Option<swc_next_ecma_ast::Function> = None;

        for ancestor in ctx.ancestors() {
            match ctx.ast.node_kind(ancestor) {
                NodeKind::Function => {
                    let f =
                        unsafe { swc_next_ecma_ast::Function::from_node_id_unchecked(ancestor) };
                    func_node = Some(f);
                }
                NodeKind::MethodDefinition => {
                    let method = unsafe { MethodDefinition::from_node_id_unchecked(ancestor) };
                    match method.kind(ctx.ast) {
                        MethodDefinitionKind::Constructor => is_constructor = true,
                        _ => is_method = true,
                    }
                }
                NodeKind::StaticBlock => {
                    is_method = true;
                }
                _ => {}
            }
        }

        let mode = if is_constructor {
            NewTargetMode::Constructor
        } else if is_method {
            NewTargetMode::Method
        } else if let Some(function) = func_node {
            NewTargetMode::Function(
                function
                    .id(ctx.ast)
                    .map(|binding| binding.name(ctx.ast))
                    .unwrap_or_else(|| ctx.ast.add_utf8("_target")),
            )
        } else {
            return;
        };
        let old = Expr::MetaProperty(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let (replacement, reference) = tx.build_detached(|ast| match mode {
                NewTargetMode::Constructor => (constructor_member(ast, span), None),
                NewTargetMode::Method => (void_zero(ast, span), None),
                NewTargetMode::Function(name) => {
                    let reference = ast.identifier_reference(span, name);
                    let receiver = ast.expr_this_expression(span);
                    let instanceof = ast.expr_binary_expression(
                        span,
                        receiver,
                        Expr::IdentifierReference(reference),
                        BinaryOperator::Instanceof,
                    );
                    let member = constructor_member(ast, span);
                    let alternate = void_zero(ast, span);
                    (
                        ast.expr_conditional_expression(span, instanceof, member, alternate),
                        Some(reference),
                    )
                }
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            if let Some(reference) = reference {
                register_reference(tx, scope, reference);
            }
        })
        .expect("new-target mutation must pass preflight");
    }
}

#[derive(Clone, Copy)]
enum NewTargetMode {
    Constructor,
    Method,
    Function(swc_next_ecma_ast::AstUtf8),
}

fn constructor_member(ast: &mut swc_next_ecma_ast::Ast<'_>, span: swc_next_ecma_ast::Span) -> Expr {
    let receiver = ast.expr_this_expression(span);
    let constructor = ast.add_utf8("constructor");
    let key = ast.property_key_identifier_name(span, constructor);
    ast.expr_member_expression(span, receiver, key, false, false)
}

fn void_zero(ast: &mut swc_next_ecma_ast::Ast<'_>, span: swc_next_ecma_ast::Span) -> Expr {
    let zero = ast.add_utf8("0");
    let zero = ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero);
    ast.expr_unary_expression(span, zero, UnaryOperator::Void)
}

fn register_reference(
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

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, MetaProperty, NodeKind, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        nodes: Vec<MetaProperty>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_meta_property(&mut self, node: MetaProperty) {
            self.nodes.push(node);
        }
    }

    #[test]
    fn new_target_variants_publish_complete_semantics() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function Foo() { return new.target; } class C { m() { return new.target; } constructor() { new.target; } }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let nodes = {
            let mut collector = Collector {
                ast: &ast,
                nodes: Vec::new(),
            };
            parse.program.visit_children_with(&mut collector);
            collector.nodes
        };
        assert_eq!(nodes.len(), 3);
        let mut options = TransformerOptions::default();
        options.es2015.new_target = true;
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

        let kinds: Vec<_> = nodes
            .into_iter()
            .map(|node| ast.node_kind(node.node_id()))
            .collect();
        assert!(kinds.contains(&NodeKind::ConditionalExpression));
        assert!(kinds.contains(&NodeKind::UnaryExpression));
        assert!(kinds.contains(&NodeKind::MemberExpression));
        let function_reference = semantic
            .iter_references()
            .find(|entry| ast.get_ast_str_bytes(entry.reference.name) == b"Foo")
            .unwrap();
        assert!(function_reference.reference.symbol.is_some());
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
