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
    AstNode, BinaryOperator, GetSpan, MetaProperty, MethodDefinition, MethodDefinitionKind,
    NodeKind, NumericLiteralKind, UnaryOperator,
};

use crate::state::TransformerCtx;

pub(super) struct NewTarget;

impl NewTarget {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_meta_property(
        &mut self,
        node: MetaProperty,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
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
                    let f = unsafe { swc_next_ecma_ast::Function::from_node_id_unchecked(ancestor) };
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

        if is_constructor {
            // new.target → this.constructor
            let this = ctx.ast.expr_this_expression(span);
            let constructor_name = ctx.ast.add_utf8("constructor");
            let prop_key = ctx.ast.property_key_identifier_name(span, constructor_name);
            let member = ctx.ast.expr_member_expression(span, this, prop_key, false, false);
            unsafe { ctx.ast.replace_data(node, member) };
        } else if is_method {
            // new.target → void 0
            let zero_str = ctx.ast.add_utf8("0");
            let zero = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero_str);
            let void = ctx.ast.expr_unary_expression(span, zero, UnaryOperator::Void);
            unsafe { ctx.ast.replace_data(node, void) };
        } else if let Some(func) = func_node {
            // new.target → this instanceof Foo ? this.constructor : void 0
            let this = ctx.ast.expr_this_expression(span);

            let func_name = func.id(ctx.ast).map(|b| b.name(ctx.ast)).unwrap_or_else(|| {
                ctx.ast.add_utf8("_target")
            });
            let func_ref = ctx.ast.expr_identifier_reference(span, func_name);
            let instanceof = ctx.ast.expr_binary_expression(
                span,
                this.clone(),
                func_ref,
                BinaryOperator::Instanceof,
            );

            let constructor_name = ctx.ast.add_utf8("constructor");
            let prop_key = ctx.ast.property_key_identifier_name(span, constructor_name);
            let member = ctx.ast.expr_member_expression(span, this, prop_key, false, false);

            let zero_str = ctx.ast.add_utf8("0");
            let zero = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero_str);
            let void = ctx.ast.expr_unary_expression(span, zero, UnaryOperator::Void);

            let ternary = ctx.ast.expr_conditional_expression(span, instanceof, member, void);
            unsafe { ctx.ast.replace_data(node, ternary) };
        }
    }
}