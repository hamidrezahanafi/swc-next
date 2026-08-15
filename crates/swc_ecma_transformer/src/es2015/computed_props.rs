//! ES2015: computed-props — Transform computed property keys in object literals.
//! ```js
//! // Single computed property:
//! ({ [key]: value }) → _define_property({}, key, value)
//!
//! // Multiple computed properties (not yet implemented):
//! // Needs var _obj; hoisting which requires statement-level injection
//! ```

use swc_next_ecma_ast::{
    Argument, Expr, GetSpan, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKey,
    PropertyKind,
};

use crate::state::TransformerCtx;

pub(super) struct ComputedProps;

impl ComputedProps {
    pub fn exit_object_expression(
        &mut self,
        node: ObjectExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let span = node.span(ctx.ast);
        let props = node.properties(ctx.ast);

        // Collect properties into a Vec since we need multiple passes
        let mut all_items: Vec<ObjectPropertyKind> = Vec::new();
        let mut has_computed = false;
        for i in 0..props.len() {
            let item = props.get_node(ctx.ast, i).unwrap();
            if let Some(prop) = item.as_object_property(ctx.ast) {
                if prop.computed(ctx.ast) {
                    has_computed = true;
                }
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
            self.transform_single_computed(node, &all_items, span, ctx);
        }
        // Multi-property/spread case: not yet supported — requires variable injection
        // into the enclosing scope
    }

    fn transform_single_computed(
        &self,
        node: ObjectExpression,
        all_items: &[ObjectPropertyKind],
        span: swc_next_ecma_ast::Span,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let mut static_props: Vec<ObjectProperty> = Vec::new();
        let mut computed_prop: Option<ObjectProperty> = None;

        for item in all_items {
            if let Some(prop) = item.as_object_property(ctx.ast) {
                if prop.computed(ctx.ast) {
                    computed_prop = Some(prop);
                } else {
                    static_props.push(prop);
                }
            }
        }

        let Some(comp_prop) = computed_prop else { return };
        let key = comp_prop.key(ctx.ast);
        let value = comp_prop.value(ctx.ast);

        // Build the initial object literal with static properties
        let init_obj = if static_props.is_empty() {
            let empty_range = ctx.ast.add_typed_sub_range([]);
            ctx.ast.expr_object_expression(span, empty_range)
        } else {
            let static_items: Vec<ObjectPropertyKind> = static_props
                .into_iter()
                .map(|p| ObjectPropertyKind::ObjectProperty(p))
                .collect();
            let items_range = ctx.ast.add_typed_sub_range(static_items);
            ctx.ast.expr_object_expression(span, items_range)
        };

        // Build _define_property(init_obj, key, value)
        let helper_name = ctx.ast.add_utf8("_define_property");
        let callee = ctx.ast.expr_identifier_reference(span, helper_name);
        let key_expr = key_to_expr(key, span, ctx);
        let args = ctx.ast.add_typed_sub_range([
            Argument::Expr(init_obj),
            Argument::Expr(key_expr),
            Argument::Expr(value),
        ]);
        let call = ctx.ast.expr_call_expression(span, callee, args, false, None);

        unsafe { ctx.ast.replace_data(node, call) };
    }
}

fn key_to_expr(key: PropertyKey, span: swc_next_ecma_ast::Span, ctx: &mut TransformerCtx<'_, '_>) -> Expr {
    if key.is_identifier_name(ctx.ast) {
        let id = key.as_identifier_name(ctx.ast).unwrap();
        let name = ctx.ast.get_utf8(id.name(ctx.ast)).to_owned();
        let value = ctx.ast.add_wtf8((&*name).into());
        let raw = ctx.ast.add_utf8(&format!("\"{}\"", name));
        ctx.ast.expr_string_literal(span, value, raw)
    } else if key.is_string_literal(ctx.ast) {
        Expr::StringLiteral(key.as_string_literal(ctx.ast).unwrap())
    } else if key.is_numeric_literal(ctx.ast) {
        Expr::NumericLiteral(key.as_numeric_literal(ctx.ast).unwrap())
    } else if key.is_big_int_literal(ctx.ast) {
        Expr::BigIntLiteral(key.as_big_int_literal(ctx.ast).unwrap())
    } else {
        // Computed expression — use directly
        let expr = key.as_expr(ctx.ast).unwrap();
        expr
    }
}