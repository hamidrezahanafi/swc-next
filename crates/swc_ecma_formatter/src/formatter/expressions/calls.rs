//! Callees and source-owned argument delimiters.

use swc_next_ecma_ast::{
    Argument, CallExpression, Expr, ExprData, GetSpan, PropertyKeyData, Span, SubRange,
};

use super::arguments::{ArgumentGroup, SourceArguments};
use super::{
    AccessKind, ExpressionContext, ExpressionPosition, Formatter, JsxCallbackContext,
    MemberChainLayout, MemberLayout, TailOwner,
};
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

pub(super) struct CallCallee {
    pub(super) expression: Expr,
    pub(super) access: AccessKind,
    pub(super) mode: ContentMode,
    pub(super) curried_call: Option<CallExpression>,
}

impl CallCallee {
    pub(super) fn context(&self, context: ExpressionContext) -> ExpressionContext {
        ExpressionContext::at(ExpressionPosition::CallCallee {
            access: self.access,
            curried_call: self.curried_call,
        })
        .with_expression_start(context.expression_start)
        .with_tail_owner(TailOwner::ParentOperation)
        .with_content_mode(self.mode)
        .with_member_chain_layout(context.member_chain_layout)
        .with_conditional_envelope(context.conditional_envelope.chain_head())
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn call_span_is_source_clear(&self, span: Span) -> bool {
        if self.diagnostic_range_is_sensitive(span) {
            return false;
        }
        let comments = self.ast.comments();
        let first = comments.partition_point(|comment| comment.span.end <= span.start);
        comments
            .get(first)
            .is_none_or(|comment| comment.span.start >= span.end)
    }

    fn call_member_callee_is_compact(&self, mut expression: Expr) -> bool {
        if !self.call_span_is_source_clear(expression.span(self.ast)) {
            return false;
        }
        loop {
            expression = match self.expression_content(expression).data {
                ExprData::MemberExpression(member) => {
                    let Ok(head) = self.prepare_member_head(member) else {
                        return false;
                    };
                    if !matches!(self.source_clear_member_has_chain_barrier(&head), Ok(false)) {
                        return false;
                    }
                    head.object
                }
                ExprData::TsNonNullExpression(node) => node.expression(self.ast),
                ExprData::ChainExpression(node) => node.expression(self.ast),
                data => {
                    return super::chains::atomic_chain_head(data)
                        || matches!(data, ExprData::Super(_));
                }
            };
        }
    }

    pub(super) fn source_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<CallCallee, FormatError> {
        // Only suppression can change the callee's content mode.
        if self.has_suppression {
            return Ok(self.source_call(call, context)?.callee_head());
        }
        let ast = self.ast;
        let expression = call.callee(ast);
        let access = if call.optional(ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let mode = ContentMode::Formatted;
        let curried_call = self.curried_call_callee(expression, call.arguments(ast), mode);
        Ok(CallCallee {
            expression,
            access,
            mode,
            curried_call,
        })
    }

    /// The RHS chain owner has established one immutable clear span covering
    /// this call. Types and suppression retain the complete argument owner.
    pub(super) fn rhs_call_callee(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Option<CallCallee> {
        let ast = self.ast;
        if self.has_suppression || call.type_arguments(ast).is_some() {
            let source = self.source_call(call, context).ok()?;
            if source.callee_mode != ContentMode::Formatted || source.has_outside_prefix() {
                return None;
            }
            return Some(source.callee_head());
        }
        let callee = self.source_call_callee(call, context).ok()?;
        self.source_clear_argument_opening(
            call.arguments(ast),
            callee.expression,
            call.span(ast).end,
            callee.access,
        )
        .then_some(callee)
    }

    pub(super) fn format_call(
        &mut self,
        call: CallExpression,
        context: ExpressionContext,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let arguments = self.source_call(call, context)?;
        let mut callee_context = arguments.callee_context(context);
        let member_callee = arguments.callee_mode == ContentMode::Formatted
            && !arguments.has_fixed_argument_frame()
            && matches!(
                self.expression_content(arguments.callee).data,
                ExprData::MemberExpression(_)
            )
            && self.expression_needs_parentheses(
                self.ast.expr_data(arguments.callee),
                callee_context.position,
                arguments.callee_mode,
            ) != Some(true)
            && !arguments.has_outside_prefix()
            && self.call_member_callee_is_compact(arguments.callee);
        let group = arguments.curried_callee.map(|_| arguments.group);
        if member_callee {
            callee_context = callee_context
                .with_member_layout(MemberLayout::Tight)
                .with_member_chain_layout(MemberChainLayout::Local);
        }
        let write = |writer: &mut Writer<'ast>| {
            let tail = self.format_expression(arguments.callee, callee_context, writer)?;
            self.format_source_arguments(arguments, tail, writer)
        };
        if member_callee {
            writer.group(false, write)
        } else {
            match group {
                Some(group) => group.write(false, writer, write),
                None => write(writer),
            }
        }
    }

    pub(super) fn source_call(
        &self,
        call: CallExpression,
        context: ExpressionContext,
    ) -> Result<SourceArguments<'ast>, FormatError> {
        let ast = self.ast;
        let callee = call.callee(ast);
        let call_arguments = call.arguments(ast);
        let access = if call.optional(ast) {
            AccessKind::Optional
        } else {
            AccessKind::Ordinary
        };
        let source = if let Some(node) = call.type_arguments(ast) {
            let types = self.prepare_expression_type_arguments(callee, node, access)?;
            self.source_arguments_after_types(call_arguments, callee, call.span(ast).end, types)
        } else {
            self.source_arguments(call_arguments, callee, call.span(ast).end, access)
        };
        let mut arguments =
            source.ok_or_else(|| self.unsupported("call opening boundary", call))?;
        arguments.test_callback = self.test_call_callback(call);
        arguments.jsx_call = match context.jsx_callback {
            JsxCallbackContext::ContainerCall(owner) if owner == call => Some(call),
            _ => None,
        };
        arguments.group = if matches!(context.position,
            ExpressionPosition::CallCallee { curried_call: Some(curried), .. } if curried == call)
        {
            ArgumentGroup::ParentCall
        } else {
            ArgumentGroup::Local
        };
        if !call.optional(ast)
            && arguments.callee_mode == ContentMode::Formatted
            && let ExprData::IdentifierReference(identifier) = self.call_callee_content_data(callee)
            && ast.get_utf8(identifier.name(ast)) == "require"
        {
            arguments.frame = self.require_argument_frame(&arguments);
        }
        arguments.curried_callee =
            self.curried_call_callee(callee, call_arguments, arguments.callee_mode);
        Ok(arguments)
    }

    pub(in crate::formatter) fn call_callee_content_data(&self, callee: Expr) -> ExprData {
        match self.ast.expr_data(callee) {
            ExprData::ParenthesizedExpression(_) => self.expression_content(callee).data,
            data => data,
        }
    }

    #[inline]
    fn curried_call_callee(
        &self,
        callee: Expr,
        arguments: SubRange<Argument>,
        mode: ContentMode,
    ) -> Option<CallExpression> {
        if arguments.is_empty() || mode != ContentMode::Formatted {
            return None;
        }
        match self.expression_content(callee).data {
            ExprData::CallExpression(child)
                if child.arguments(self.ast).len() > arguments.len() =>
            {
                Some(child)
            }
            _ => None,
        }
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
