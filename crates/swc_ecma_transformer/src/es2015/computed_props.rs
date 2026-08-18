//! ES2015: computed-props — Transform computed property keys in object literals.
//! ```js
//! // Single computed property:
//! ({ [key]: value }) → _define_property({}, key, value)
//!
//! // Multiple computed properties (not yet implemented):
//! // Needs var _obj; hoisting which requires statement-level injection
//! ```

use swc_next_ecma_ast::{
    Argument, Ast, Expr, GetSpan, ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct ComputedProps;

impl ComputedProps {
    pub fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let span = node.span(ctx.ast);
        let props = node.properties(ctx.ast);

        // Collect properties into a Vec since we need multiple passes
        let mut all_items: Vec<ObjectPropertyKind> = Vec::new();
        let mut has_computed = false;
        for i in 0..props.len() {
            let item = props.get_node(ctx.ast, i).unwrap();
            if let Some(prop) = item.as_object_property(ctx.ast)
                && prop.computed(ctx.ast)
            {
                has_computed = true;
            }
            all_items.push(item);
        }

        if !has_computed {
            return;
        }

        // Count computed properties and check for getters/setters
        let mut computed_count = 0;
        let mut has_accessor = false;
        for item in &all_items {
            if let Some(prop) = item.as_object_property(ctx.ast) {
                if prop.computed(ctx.ast) {
                    computed_count += 1;
                }
                if prop.kind(ctx.ast) != PropertyKind::Init {
                    has_accessor = true;
                }
            }
        }

        // Only handle single computed property without getters/setters or spread for now
        if computed_count == 1 && !has_accessor {
            self.transform_single_computed(node, span, ctx);
        }
        // Multi-property/spread case: not yet supported — requires variable injection
        // into the enclosing scope
    }

    fn transform_single_computed(
        &self,
        node: ObjectExpression,
        span: swc_next_ecma_ast::Span,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let old = Expr::ObjectExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let object = clone.root.as_object_expression(tx.ast()).unwrap();
            let items: Vec<_> = object
                .properties(tx.ast())
                .iter()
                .map(|item| tx.ast().get_node_in_sub_range(item))
                .collect();
            let (replacement, helper) = tx.build_detached(|ast| {
                let mut static_props: Vec<ObjectPropertyKind> = Vec::new();
                let mut computed = None;
                for item in items {
                    if let Some(property) = item.as_object_property(ast)
                        && property.computed(ast)
                    {
                        computed = Some(property);
                    } else {
                        static_props.push(item);
                    }
                }
                let computed = computed.unwrap();
                let key = key_to_expr(computed.key(ast), span, ast);
                let value = computed.value(ast);
                let static_props = ast.add_typed_sub_range(static_props);
                let initial = ast.expr_object_expression(span, static_props);
                let helper_name = ast.add_utf8("_define_property");
                let helper = ast.identifier_reference(span, helper_name);
                let arguments = ast.add_typed_sub_range([
                    Argument::Expr(initial),
                    Argument::Expr(key),
                    Argument::Expr(value),
                ]);
                let call = ast.call_expression(
                    span,
                    Expr::IdentifierReference(helper),
                    arguments,
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
        .expect("computed-properties mutation must pass preflight");
    }
}

fn key_to_expr(key: PropertyKey, span: swc_next_ecma_ast::Span, ast: &mut Ast<'_>) -> Expr {
    if key.is_identifier_name(ast) {
        let id = key.as_identifier_name(ast).unwrap();
        let name = ast.get_utf8(id.name(ast)).to_owned();
        let value = ast.add_wtf8((&*name).into());
        let raw = ast.add_utf8(&format!("\"{}\"", name));
        ast.expr_string_literal(span, value, raw)
    } else if key.is_string_literal(ast) {
        Expr::StringLiteral(key.as_string_literal(ast).unwrap())
    } else if key.is_numeric_literal(ast) {
        Expr::NumericLiteral(key.as_numeric_literal(ast).unwrap())
    } else if key.is_big_int_literal(ast) {
        Expr::BigIntLiteral(key.as_big_int_literal(ast).unwrap())
    } else {
        // Computed expression — use directly
        key.as_expr(ast).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, NodeKind, ObjectExpression, Visit, VisitWith};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    struct Collector<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        object: Option<ObjectExpression>,
    }

    impl<'ast> Visit<'ast> for Collector<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_object_expression(&mut self, node: ObjectExpression) {
            node.visit_children_with(self);
            self.object = Some(node);
        }
    }

    #[test]
    fn computed_property_reuses_existing_reference_identities() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const out = { a: x, [key]: value };", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let object = {
            let mut collector = Collector {
                ast: &ast,
                object: None,
            };
            parse.program.visit_children_with(&mut collector);
            collector.object.unwrap()
        };
        let mut options = TransformerOptions::default();
        options.es2015.computed_props = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let old_references: Vec<_> = semantic.iter_references().map(|entry| entry.id).collect();

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert_eq!(ast.node_kind(object.node_id()), NodeKind::CallExpression);
        assert!(
            old_references
                .into_iter()
                .all(|reference| semantic.is_reference_active(reference))
        );
        assert!(
            semantic.iter_references().any(|entry| {
                ast.get_ast_str_bytes(entry.reference.name) == b"_define_property"
            })
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
