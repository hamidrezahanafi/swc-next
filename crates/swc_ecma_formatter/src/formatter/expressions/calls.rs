//! Callees and source-owned argument delimiters.

use swc_next_ecma_ast::{CallExpression, Expr, ExprData, GetSpan, PropertyKeyData};

use super::arguments::{ArgumentGroup, SourceArguments};
use super::{AccessKind, ExpressionContext, ExpressionPosition, Formatter};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::suppression::ContentMode;
use crate::{FormatError, document::Writer};

enum TestCallee {
    It,
    Describe,
    Test,
    TestDescribe,
    ParallelOrSerial,
    Terminal,
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let arguments = self.source_call(call, context)?;
        let group = arguments.curried_callee.map(|_| arguments.group);
        let write = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(
                arguments.callee,
                arguments.callee_context(context),
                writer,
            )?;
            self.format_source_arguments(arguments, tail, writer)
        };
        match group {
            Some(group) => group.write(false, writer, write),
            None => write(writer),
        }
    }

    pub(super) fn source_call(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<SourceArguments<'ast>, FormatError> {
        let ast = self.ast;
        if call.type_arguments(ast).is_some() {
            return Err(self.unsupported("call suffix", call));
        }
        let callee = call.callee(ast);
        let call_arguments = call.arguments(ast);
        let access = if call.optional(ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let mut arguments = self
            .source_arguments(call_arguments, callee, call.span(ast).end, access)
            .ok_or_else(|| self.unsupported("call opening boundary", call))?;
        arguments.test_callback = self.test_call_callback(call);
        arguments.group = if matches!(context.position,
            ExpressionPosition::CallCallee { curried_call: Some(curried), .. } if curried == call)
        {
            ArgumentGroup::ParentCall
        } else {
            ArgumentGroup::Local
        };
        if !call_arguments.is_empty() && arguments.callee_mode == ContentMode::Formatted {
            arguments.curried_callee = match self.expression_content(callee).data {
                ExprData::CallExpression(child)
                    if child.arguments(ast).len() > call_arguments.len() =>
                {
                    Some(child)
                }
                _ => None,
            };
        }
        Ok(arguments)
    }

    pub(super) fn is_test_callee(&self, expression: Expr) -> bool {
        self.test_callee(expression).is_some()
    }

    fn test_callee(&self, expression: Expr) -> Option<TestCallee> {
        let ast = self.ast;
        match ast.expr_data(expression) {
            ExprData::IdentifierReference(identifier) => match ast.get_utf8(identifier.name(ast)) {
                "it" => Some(TestCallee::It),
                "describe" => Some(TestCallee::Describe),
                "test" => Some(TestCallee::Test),
                "skip" | "xit" | "xdescribe" | "xtest" | "fit" | "fdescribe" | "ftest" => {
                    Some(TestCallee::Terminal)
                }
                _ => None,
            },
            ExprData::MemberExpression(member)
                if !member.computed(ast) && !member.optional(ast) =>
            {
                let PropertyKeyData::IdentifierName(property) =
                    ast.property_key_data(member.property(ast))
                else {
                    return None;
                };
                let receiver = self.test_callee(member.object(ast))?;
                match (receiver, ast.get_utf8(property.name(ast))) {
                    (
                        TestCallee::It
                        | TestCallee::Describe
                        | TestCallee::Test
                        | TestCallee::TestDescribe,
                        "only" | "skip",
                    )
                    | (TestCallee::Test, "fixme" | "step")
                    | (TestCallee::TestDescribe, "fixme")
                    | (TestCallee::ParallelOrSerial, "only") => Some(TestCallee::Terminal),
                    (TestCallee::Test, "describe") => Some(TestCallee::TestDescribe),
                    (TestCallee::TestDescribe, "parallel" | "serial") => {
                        Some(TestCallee::ParallelOrSerial)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
