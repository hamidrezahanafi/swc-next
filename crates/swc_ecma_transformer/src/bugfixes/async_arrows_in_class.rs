//! Safari 10.3 bugfix: async arrow functions in class methods throw.
//!
//! Converts async arrow functions inside class methods to regular function
//! expressions. See https://bugs.webkit.org/show_bug.cgi?id=166879

use swc_next_ecma_ast::{
    ArrowFunctionExpression, Expr, ExprData, FunctionType, GetSpan, Stmt, TraverseAction,
    TypedSubRange,
};

use crate::state::TransformerCtx;

pub(super) struct AsyncArrowsInClass {
    in_class_method: bool,
}

impl AsyncArrowsInClass {
    pub fn new() -> Self {
        Self {
            in_class_method: false,
        }
    }

    pub fn enter_method_definition(&mut self) {
        self.in_class_method = true;
    }

    pub fn exit_method_definition(&mut self) {
        self.in_class_method = false;
    }

    pub fn exit_expression(
        &mut self,
        node: Expr,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> TraverseAction<Expr> {
        if !self.in_class_method {
            return TraverseAction::Proceed;
        }
        match ctx.ast.expr_data(node) {
            ExprData::ArrowFunctionExpression(arrow) => {
                if arrow.r#async(ctx.ast) {
                    let lowered = self.lower_arrow(arrow, ctx);
                    TraverseAction::Replace(lowered)
                } else {
                    TraverseAction::Proceed
                }
            }
            _ => TraverseAction::Proceed,
        }
    }

    fn lower_arrow(
        &self,
        arrow: ArrowFunctionExpression,
        ctx: &mut TransformerCtx<'_, '_>,
    ) -> Expr {
        let span = arrow.span(ctx.ast);
        let params = arrow.params(ctx.ast);
        let body = arrow.body(ctx.ast);
        let is_async = arrow.r#async(ctx.ast);

        let body = match ctx.ast.arrow_function_body_data(body) {
            swc_next_ecma_ast::ArrowFunctionBodyData::FunctionBody(body) => body,
            swc_next_ecma_ast::ArrowFunctionBodyData::Expr(expr) => {
                let return_stmt = ctx.ast.return_statement(span, Some(expr));
                let items = ctx.ast.add_typed_sub_range([Stmt::ReturnStatement(return_stmt)]);
                ctx.ast.function_body(span, TypedSubRange::empty(), items)
            }
        };

        let func = ctx.ast.function(
            span,
            FunctionType::FunctionExpression,
            None,
            false,
            is_async,
            params,
            body,
            None,
            None,
        );
        Expr::Function(func)
    }
}