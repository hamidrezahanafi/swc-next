//! ES2015: spread — Transform spread elements in arrays, calls, and new expressions.
//!
//! ```js
//! // Array spread:
//! [...arr] → [].concat(arr)
//! [a, ...arr] → [].concat([a], arr)
//! [a, ...arr, b] → [].concat([a], arr, [b])
//!
//! // Call spread:
//! fn(...args) → fn.apply(void 0, args)
//! obj.method(...args) → obj.method.apply(obj, args)
//!
//! // New spread:
//! new F(...args) → _construct(F, args)
//! ```

use swc_next_ecma_ast::{
    Argument, ArrayExpression, CallExpression, Expr, GetSpan, NewExpression, NumericLiteralKind,
    TypedSubRange, UnaryOperator,
};

use crate::state::TransformerCtx;

pub(super) struct Spread;

impl Spread {
    pub fn new() -> Self {
        Self
    }

    /// Transform array expressions with spread elements.
    /// [...arr] → [].concat(arr)
    pub fn exit_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let span = node.span(ctx.ast);
        let elements = node.elements(ctx.ast);

        if !has_array_spread(elements, ctx) {
            return;
        }

        let replacement = self.build_array_spread(span, elements, ctx);
        unsafe { ctx.ast.replace_data(node, replacement) };
    }

    /// Transform call expressions with spread arguments.
    /// fn(...args) → fn.apply(void 0, args)
    pub fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let span = node.span(ctx.ast);
        let callee = node.callee(ctx.ast);
        let args = node.arguments(ctx.ast);

        if !has_call_spread(args, ctx) {
            return;
        }

        let replacement = self.build_call_spread(span, callee, args, ctx);
        unsafe { ctx.ast.replace_data(node, replacement) };
    }

    /// Transform new expressions with spread arguments.
    /// new F(...args) → _construct(F, args)
    pub fn exit_new_expression(
        &mut self,
        node: NewExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) {
        let span = node.span(ctx.ast);
        let callee = node.callee(ctx.ast);
        let args = node.arguments(ctx.ast);

        if !has_call_spread(args, ctx) {
            return;
        }

        let replacement = self.build_new_spread(span, callee, args, ctx);
        unsafe { ctx.ast.replace_data(node, replacement) };
    }

    fn build_array_spread(
        &self,
        span: swc_next_ecma_ast::Span,
        elements: TypedSubRange<Option<Argument>>,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let len = elements.len();
        let mut arg_list: Vec<Argument> = Vec::new();
        let mut current_elems: Vec<Option<Argument>> = Vec::new();

        for i in 0..len {
            let elem = elements.get_node(ctx.ast, i).unwrap();
            match elem {
                Some(arg) if arg.is_spread_element(ctx.ast) => {
                    // Flush non-spread elements as an array literal
                    if !current_elems.is_empty() {
                        let flushed = ctx.ast.add_typed_sub_range(
                            current_elems.drain(..).collect::<Vec<_>>(),
                        );
                        let arr = ctx.ast.expr_array_expression(
                            swc_next_ecma_ast::DUMMY_SP,
                            flushed,
                        );
                        arg_list.push(Argument::Expr(arr));
                    }

                    let spread = arg.as_spread_element(ctx.ast).unwrap();
                    let spread_expr = spread.argument(ctx.ast);
                    arg_list.push(Argument::Expr(spread_expr));
                }
                other => {
                    current_elems.push(other);
                }
            }
        }

        // Flush remaining non-spread elements
        if !current_elems.is_empty() {
            let flushed = ctx.ast.add_typed_sub_range(current_elems);
            let arr = ctx.ast.expr_array_expression(
                swc_next_ecma_ast::DUMMY_SP,
                flushed,
            );
            arg_list.push(Argument::Expr(arr));
        }

        // Build [].concat(...)
        let empty_elems = ctx.ast.add_typed_sub_range(Vec::<Option<Argument>>::new());
        let empty_arr = ctx.ast.expr_array_expression(
            swc_next_ecma_ast::DUMMY_SP,
            empty_elems,
        );
        let concat_key = ctx.ast.add_utf8("concat");
        let concat_key = ctx.ast.property_key_identifier_name(
            swc_next_ecma_ast::DUMMY_SP,
            concat_key,
        );
        let concat = ctx.ast.expr_member_expression(
            swc_next_ecma_ast::DUMMY_SP,
            empty_arr,
            concat_key,
            false,
            false,
        );
        let concat_args = ctx.ast.add_typed_sub_range(arg_list);
        ctx.ast.expr_call_expression(span, concat, concat_args, false, None)
    }

    fn build_call_spread(
        &self,
        span: swc_next_ecma_ast::Span,
        callee: Expr,
        args: TypedSubRange<Argument>,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        // Determine `this` value
        let this_arg = if callee.is_identifier_reference(ctx.ast) {
            make_undefined(span, ctx)
        } else if callee.is_super(ctx.ast) {
            ctx.ast.expr_this_expression(span)
        } else if let Some(member) = callee.as_member_expression(ctx.ast) {
            member.object(ctx.ast)
        } else {
            make_undefined(span, ctx)
        };

        // Build the arguments array
        let args_array = self.build_args_array(span, args, ctx);

        // Build callee.apply(this, args)
        let apply_name = ctx.ast.add_utf8("apply");
        let apply_key = ctx.ast.property_key_identifier_name(
            swc_next_ecma_ast::DUMMY_SP,
            apply_name,
        );
        let apply_callee = ctx.ast.expr_member_expression(
            swc_next_ecma_ast::DUMMY_SP,
            callee,
            apply_key,
            false,
            false,
        );

        let apply_args = ctx.ast.add_typed_sub_range([
            Argument::Expr(this_arg),
            Argument::Expr(args_array),
        ]);
        ctx.ast.expr_call_expression(span, apply_callee, apply_args, false, None)
    }

    fn build_new_spread(
        &self,
        span: swc_next_ecma_ast::Span,
        callee: Expr,
        args: TypedSubRange<Argument>,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let args_array = self.build_args_array(span, args, ctx);

        let helper_name = ctx.ast.add_utf8("_construct");
        let construct_callee =
            ctx.ast
                .expr_identifier_reference(swc_next_ecma_ast::DUMMY_SP, helper_name);

        let construct_args = ctx.ast
            .add_typed_sub_range([Argument::Expr(callee), Argument::Expr(args_array)]);
        ctx.ast.expr_call_expression(span, construct_callee, construct_args, false, None)
    }

    fn build_args_array(
        &self,
        span: swc_next_ecma_ast::Span,
        args: TypedSubRange<Argument>,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let len = args.len();
        let mut elems: Vec<Option<Argument>> = Vec::new();

        for i in 0..len {
            let arg = args.get_node(ctx.ast, i).unwrap();
            if arg.is_spread_element(ctx.ast) {
                let spread = arg.as_spread_element(ctx.ast).unwrap();
                let spread_expr = spread.argument(ctx.ast);
                let wrapped = make_to_consumable_array(spread_expr, ctx);
                elems.push(Some(Argument::Expr(wrapped)));
            } else {
                let expr = arg.as_expr(ctx.ast).unwrap();
                elems.push(Some(Argument::Expr(expr)));
            }
        }

        let elems_range = ctx.ast.add_typed_sub_range(elems);
        ctx.ast.expr_array_expression(span, elems_range)
    }
}

fn has_array_spread(
    elements: TypedSubRange<Option<Argument>>,
    ctx: &mut TransformerCtx<'_, '_>,
) -> bool {
    for i in 0..elements.len() {
        if let Some(arg) = elements.get_node(ctx.ast, i).unwrap() {
            if arg.is_spread_element(ctx.ast) {
                return true;
            }
        }
    }
    false
}

fn has_call_spread(
    args: TypedSubRange<Argument>,
    ctx: &mut TransformerCtx<'_, '_>,
) -> bool {
    for i in 0..args.len() {
        if args.get_node(ctx.ast, i).unwrap().is_spread_element(ctx.ast) {
            return true;
        }
    }
    false
}

fn make_undefined(
    span: swc_next_ecma_ast::Span,
    ctx: &mut TransformerCtx<'_, '_>,
) -> Expr {
    let zero_str = ctx.ast.add_utf8("0");
    let zero = ctx.ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero_str);
    ctx.ast.expr_unary_expression(span, zero, UnaryOperator::Void)
}

fn make_to_consumable_array(
    expr: Expr,
    ctx: &mut TransformerCtx<'_, '_>,
) -> Expr {
    let helper_name = ctx.ast.add_utf8("_to_consumable_array");
    let callee =
        ctx.ast
            .expr_identifier_reference(swc_next_ecma_ast::DUMMY_SP, helper_name);
    let args = ctx.ast.add_typed_sub_range([Argument::Expr(expr)]);
    ctx.ast.expr_call_expression(
        swc_next_ecma_ast::DUMMY_SP,
        callee,
        args,
        false,
        None,
    )
}