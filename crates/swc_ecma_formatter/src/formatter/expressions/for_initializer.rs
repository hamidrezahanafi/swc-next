//! The for initializer carries its NoIn grammar decision into expression layout.

use swc_next_ecma_ast::{ArrowFunctionBodyData, BinaryOperator, Expr, ExprData};

use super::{ExpressionContext, ExpressionPosition, Formatter};

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(in crate::formatter) fn for_initializer_context(
        &self,
        expression: Expr,
    ) -> ExpressionContext {
        ExpressionContext::at(ExpressionPosition::ForInitializer {
            parentheses: self.for_initializer_contains_in(expression),
        })
    }

    /// Delimited operands and function blocks use the ordinary In grammar.
    fn for_initializer_contains_in(&self, expression: Expr) -> bool {
        let ast = self.ast;
        match ast.expr_data(expression) {
            ExprData::BinaryExpression(node) => {
                node.operator(ast) == BinaryOperator::In
                    || self.for_initializer_contains_in(node.left(ast))
                    || self.for_initializer_contains_in(node.right(ast))
            }
            ExprData::LogicalExpression(node) => {
                self.for_initializer_contains_in(node.left(ast))
                    || self.for_initializer_contains_in(node.right(ast))
            }
            ExprData::ParenthesizedExpression(node) => {
                self.for_initializer_contains_in(node.expression(ast))
            }
            ExprData::AssignmentExpression(node) => {
                self.for_initializer_contains_in(node.right(ast))
            }
            ExprData::SequenceExpression(node) => node
                .expressions(ast)
                .iter()
                .any(|id| self.for_initializer_contains_in(ast.get_node_in_sub_range(id))),
            ExprData::ConditionalExpression(node) => {
                self.for_initializer_contains_in(node.test(ast))
                    || self.for_initializer_contains_in(node.alternate(ast))
            }
            ExprData::UnaryExpression(node) => self.for_initializer_contains_in(node.argument(ast)),
            ExprData::AwaitExpression(node) => self.for_initializer_contains_in(node.argument(ast)),
            ExprData::YieldExpression(node) => node
                .argument(ast)
                .is_some_and(|argument| self.for_initializer_contains_in(argument)),
            ExprData::ArrowFunctionExpression(node) => {
                match ast.arrow_function_body_data(node.body(ast)) {
                    ArrowFunctionBodyData::Expr(body) => self.for_initializer_contains_in(body),
                    ArrowFunctionBodyData::FunctionBody(_) => false,
                }
            }
            _ => false,
        }
    }
}
