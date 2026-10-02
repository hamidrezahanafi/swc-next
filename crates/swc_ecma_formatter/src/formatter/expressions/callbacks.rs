//! Typed callback selection and immutable layouts over source-owned argument slots.

use swc_next_ecma_ast::{
    Argument, ArgumentData, ArrowFunctionBodyData, ArrowFunctionExpression, CallExpression, Expr,
    ExprData, Function, GetSpan, Span, SubRange,
};

use super::{ExpressionContext, ExpressionLayout, ExpressionPosition, Formatter};
use crate::formatter::comments::ExpressionTail;
use crate::formatter::diagnostics::FlatSeparator;
use crate::formatter::suppression::ContentMode;
use crate::{
    FormatError, TrailingCommas,
    document::{FlatFragment, Fragment, Writer},
};

#[derive(Clone, Copy)]
enum CallbackKind {
    Arrow(ArrowFunctionExpression),
    Function(Function),
}

#[derive(Clone, Copy)]
struct Callback {
    expression: Expr,
    kind: CallbackKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CallbackRole {
    First,
    Last,
    Test,
}

pub(super) struct SelectedCallback {
    argument: Argument,
    callback: Callback,
    role: CallbackRole,
}

enum RecordedValue<'a> {
    Ordinary(Fragment<'a>),
    Callback {
        signature: Fragment<'a>,
        body: Fragment<'a>,
        normal: Fragment<'a>,
        layout: ExpressionLayout,
    },
}

struct RecordedArgument<'a> {
    prefix: Fragment<'a>,
    value: RecordedValue<'a>,
    suffix: Fragment<'a>,
}

enum GroupedValue<'a> {
    Ordinary(FlatFragment<'a>),
    Callback {
        signature: FlatFragment<'a>,
        body: Fragment<'a>,
    },
}

struct GroupedArgument<'a> {
    prefix: FlatFragment<'a>,
    value: GroupedValue<'a>,
    suffix: FlatFragment<'a>,
}

impl<'a> RecordedArgument<'a> {
    fn try_group(&self) -> Option<GroupedArgument<'a>> {
        let prefix = self.prefix.try_flat()?;
        let suffix = self.suffix.try_flat()?;
        let value = match self.value {
            RecordedValue::Ordinary(value) => GroupedValue::Ordinary(value.try_flat()?),
            RecordedValue::Callback {
                signature,
                body,
                normal,
                layout,
            } => {
                if layout == ExpressionLayout::Flat
                    && let Some(normal) = normal.try_flat()
                {
                    GroupedValue::Ordinary(normal)
                } else {
                    GroupedValue::Callback {
                        signature: signature.try_flat()?,
                        body,
                    }
                }
            }
        };
        Some(GroupedArgument {
            prefix,
            value,
            suffix,
        })
    }
}

impl<'ast, 'source> Formatter<'ast, 'source> {
    pub(super) fn test_call_callback(&self, call: CallExpression) -> Option<SelectedCallback> {
        let ast = self.ast;
        let arguments = call.arguments(ast);
        if !matches!(arguments.len(), 2 | 3) || !self.is_test_callee(call.callee(ast)) {
            return None;
        }
        let content = Span::new(call.callee(ast).span(ast).end, call.span(ast).end);
        if !self.comments.in_range(content).is_empty() {
            return None;
        }
        let mut arguments = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
        let first = arguments.next()?;
        if !matches!(ast.argument_data(first), ArgumentData::Expr(expression) if matches!(ast.expr_data(expression), ExprData::StringLiteral(_)))
        {
            return None;
        }
        let argument = arguments.next()?;
        let callback = self.callback(argument)?;
        if let Some(timeout) = arguments.next() {
            if !matches!(ast.argument_data(timeout), ArgumentData::Expr(expression) if matches!(ast.expr_data(expression), ExprData::NumericLiteral(_)))
                || !self.callback_has_block(callback)
            {
                return None;
            }
            let parameters = match callback.kind {
                CallbackKind::Arrow(arrow) => arrow.params(ast),
                CallbackKind::Function(function) => function.params(ast),
            };
            if parameters.items(ast).len() + usize::from(parameters.rest(ast).is_some()) > 1 {
                return None;
            }
        }
        Some(SelectedCallback {
            argument,
            callback,
            role: CallbackRole::Test,
        })
    }

    fn callback(&self, argument: Argument) -> Option<Callback> {
        let ArgumentData::Expr(expression) = self.ast.argument_data(argument) else {
            return None;
        };
        let kind = match self.ast.expr_data(expression) {
            ExprData::ArrowFunctionExpression(arrow) => CallbackKind::Arrow(arrow),
            ExprData::Function(function) => CallbackKind::Function(function),
            _ => return None,
        };
        Some(Callback { expression, kind })
    }

    pub(super) fn select_callback(
        &self,
        arguments: SubRange<Argument>,
    ) -> Option<SelectedCallback> {
        let ast = self.ast;
        let mut items = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
        let first = items.next()?;
        if arguments.len() == 2
            && let Some(callback) = self.callback(first)
            && self.callback_has_block(callback)
            && let Some(second) = items.next()
            && let ArgumentData::Expr(expression) = ast.argument_data(second)
            && self.short_callback_neighbor(expression)
        {
            return Some(SelectedCallback {
                argument: first,
                callback,
                role: CallbackRole::First,
            });
        }
        let mut items = arguments.iter().map(|id| ast.get_node_in_sub_range(id));
        let argument = items.next_back()?;
        let callback = self.callback(argument)?;
        if let Some(previous) = items.next_back()
            && let Some(previous) = self.callback(previous)
            && std::mem::discriminant(&previous.kind) == std::mem::discriminant(&callback.kind)
        {
            return None;
        }
        self.callback_can_expand(callback)
            .then_some(SelectedCallback {
                argument,
                callback,
                role: CallbackRole::Last,
            })
    }

    fn callback_can_expand(&self, callback: Callback) -> bool {
        let CallbackKind::Arrow(mut arrow) = callback.kind else {
            return true;
        };
        let mut nested = false;
        loop {
            let expression = match self.ast.arrow_function_body_data(arrow.body(self.ast)) {
                ArrowFunctionBodyData::FunctionBody(_) => return true,
                ArrowFunctionBodyData::Expr(expression) => expression,
            };
            match self.expression_content(expression).data {
                ExprData::ObjectExpression(_) | ExprData::ArrayExpression(_) => return true,
                ExprData::ArrowFunctionExpression(child) => {
                    arrow = child;
                    nested = true;
                }
                ExprData::ConditionalExpression(_) | ExprData::CallExpression(_) => return !nested,
                _ => return false,
            }
        }
    }

    fn callback_has_block(&self, callback: Callback) -> bool {
        match callback.kind {
            CallbackKind::Function(_) => true,
            CallbackKind::Arrow(arrow) => matches!(
                self.ast.arrow_function_body_data(arrow.body(self.ast)),
                ArrowFunctionBodyData::FunctionBody(_)
            ),
        }
    }

    fn short_callback_neighbor(&self, expression: Expr) -> bool {
        match self.ast.expr_data(expression) {
            ExprData::IdentifierReference(_)
            | ExprData::ThisExpression(_)
            | ExprData::StringLiteral(_)
            | ExprData::NumericLiteral(_)
            | ExprData::BooleanLiteral(_)
            | ExprData::NullLiteral(_)
            | ExprData::BigIntLiteral(_)
            | ExprData::RegExpLiteral(_) => true,
            ExprData::CallExpression(call) => call.arguments(self.ast).is_empty(),
            ExprData::MemberExpression(member) => {
                !member.computed(self.ast)
                    && !member.optional(self.ast)
                    && matches!(
                        self.ast.expr_data(member.object(self.ast)),
                        ExprData::IdentifierReference(_) | ExprData::ThisExpression(_)
                    )
            }
            _ => false,
        }
    }

    fn record_callback(
        &mut self,
        callback: Callback,
        writer: &mut Writer<'ast>,
    ) -> Result<(RecordedValue<'ast>, Option<ExpressionTail<'ast>>), FormatError> {
        let (signature, body, normal, tail) = match callback.kind {
            CallbackKind::Arrow(arrow) => {
                let source = self.source_arrow(arrow)?;
                let context = ExpressionContext::at(ExpressionPosition::Nested)
                    .with_arrow_layout(super::arrows::ArrowLayout::Argument);
                let recorded = self.record_arrow(source, context, writer)?;
                (
                    recorded.signature,
                    recorded.body,
                    recorded.normal,
                    recorded.tail,
                )
            }
            CallbackKind::Function(function) => {
                let (signature, body) = writer
                    .record_with_value(|writer| self.format_function_head(function, writer))?;
                let (body, ()) = writer
                    .record_with_value(|writer| self.format_function_body_boundary(body, writer))?;
                let (normal, ()) = writer.record_with_value(|writer| {
                    writer.write_fragment(signature);
                    writer.write_fragment(body);
                    Ok::<_, FormatError>(())
                })?;
                (signature, body, normal, None)
            }
        };
        let layout = self.expression_layout(callback.expression);
        let (normal, ()) = writer.record_with_value(|writer| {
            layout.write(writer, |writer| writer.write_fragment(normal));
            Ok::<_, FormatError>(())
        })?;
        Ok((
            RecordedValue::Callback {
                signature,
                body,
                normal,
                layout,
            },
            tail,
        ))
    }

    pub(super) fn format_callback_arguments(
        &mut self,
        arguments: SubRange<Argument>,
        content: Span,
        first_mode: ContentMode,
        expand: bool,
        selected: SelectedCallback,
        writer: &mut Writer<'ast>,
    ) -> Result<Option<ExpressionTail<'ast>>, FormatError> {
        let ast = self.ast;
        let (opening, ()) = writer.record_with_value(|writer| {
            self.opening_line_constraint(content).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            Ok::<_, FormatError>(())
        })?;
        let mut recorded = Vec::with_capacity(arguments.len());
        let mut arguments = arguments
            .iter()
            .map(|id| ast.get_node_in_sub_range(id))
            .peekable();
        let mut previous = None;
        while let Some(argument) = arguments.next() {
            let (prefix, mut item) = writer.record_with_value(|writer| {
                self.prepare_argument_slot(
                    argument,
                    arguments.peek().copied(),
                    previous,
                    content,
                    self.options.trailing_commas == TrailingCommas::All,
                    writer,
                )
            })?;
            if previous.is_none() && first_mode == ContentMode::Verbatim {
                item.mode = first_mode;
            }
            let (value, tail) =
                if argument == selected.argument && item.mode == ContentMode::Formatted {
                    self.record_callback(selected.callback, writer)?
                } else {
                    let (value, tail) = writer.record_with_value(|writer| {
                        self.format_argument(argument, item.mode, writer)
                    })?;
                    (RecordedValue::Ordinary(value), tail)
                };
            let (suffix, boundary) = writer
                .record_with_value(|writer| self.finish_delimited_item(item, tail, writer))?;
            previous = Some(boundary);
            recorded.push(RecordedArgument {
                prefix,
                value,
                suffix,
            });
        }
        let (closing, ()) = writer.record_with_value(|writer| {
            let end = previous.map_or(content.start, |boundary| boundary.end);
            let end = self.format_argument_dangling(Span::new(end, content.end), writer)?;
            self.line_constraint(end, content.end).write(
                writer,
                FlatSeparator::Empty,
                Writer::soft_line,
            );
            Ok::<_, FormatError>(())
        })?;
        let (fallback, ()) = writer.record_with_value(|writer| {
            writer.group(expand, |writer| {
                writer.text("(");
                writer.indent(|writer| {
                    writer.write_fragment(opening);
                    for item in &recorded {
                        writer.write_fragment(item.prefix);
                        match item.value {
                            RecordedValue::Ordinary(value) => writer.write_fragment(value),
                            RecordedValue::Callback { normal, .. } => writer.write_fragment(normal),
                        }
                        writer.write_fragment(item.suffix);
                    }
                });
                writer.write_fragment(closing);
                writer.text(")");
            });
            Ok::<_, FormatError>(())
        })?;
        if !expand
            && let Some(grouped_items) = recorded
                .iter()
                .map(RecordedArgument::try_group)
                .collect::<Option<Vec<_>>>()
            && let Some(opening) = opening.try_flat()
            && let Some(closing) = closing.try_flat()
        {
            let (grouped, ()) = writer.record_with_value(|writer| {
                write_grouped_arguments(opening, &grouped_items, closing, false, writer);
                Ok::<_, FormatError>(())
            })?;
            if selected.role == CallbackRole::Test {
                writer.write_fragment(grouped);
            } else if grouped_items
                .iter()
                .any(|item| matches!(item.value, GroupedValue::Callback { .. }))
            {
                let (expanded, ()) = writer.record_with_value(|writer| {
                    write_grouped_arguments(opening, &grouped_items, closing, true, writer);
                    Ok::<_, FormatError>(())
                })?;
                writer.write_alternatives(fallback, &[grouped, expanded], fallback);
            } else {
                writer.write_alternatives(fallback, &[grouped], fallback);
            }
        } else {
            writer.write_fragment(fallback);
        }
        Ok(None)
    }
}

fn write_grouped_arguments<'a>(
    opening: FlatFragment<'a>,
    items: &[GroupedArgument<'a>],
    closing: FlatFragment<'a>,
    expand_body: bool,
    writer: &mut Writer<'a>,
) {
    writer.text("(");
    writer.write_flat_fragment(opening);
    for item in items {
        writer.write_flat_fragment(item.prefix);
        match item.value {
            GroupedValue::Ordinary(value) => writer.write_flat_fragment(value),
            GroupedValue::Callback { signature, body } => {
                writer.write_flat_fragment(signature);
                writer.group(expand_body, |writer| writer.write_fragment(body));
            }
        }
        writer.write_flat_fragment(item.suffix);
    }
    writer.write_flat_fragment(closing);
    writer.text(")");
}
