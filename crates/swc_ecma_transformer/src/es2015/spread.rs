//! ES2015 spread transform for arrays, calls, and new expressions.

use swc_next_ecma_ast::{
    Argument, ArrayExpression, Ast, CallExpression, Expr, GetSpan, IdentifierReference,
    NewExpression, NumericLiteralKind, TypedSubRange, UnaryOperator,
};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct Spread;

impl Spread {
    pub fn new() -> Self {
        Self
    }

    pub fn exit_array_expression(
        &mut self,
        node: ArrayExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if !has_array_spread(node.elements(ctx.ast), ctx.ast) {
            return;
        }
        let span = node.span(ctx.ast);
        let old = Expr::ArrayExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let array = clone.root.as_array_expression(tx.ast()).unwrap();
            let elements = array.elements(tx.ast());
            let built = tx.build_detached(|ast| build_array_spread(span, elements, ast));
            clone.replace_root(built.expression);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            register_helpers(tx, scope, &built.helpers);
        })
        .expect("array spread mutation must pass preflight");
    }

    pub fn exit_call_expression(
        &mut self,
        node: CallExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if !has_call_spread(node.arguments(ctx.ast), ctx.ast) {
            return;
        }
        let span = node.span(ctx.ast);
        let source_callee = node.callee(ctx.ast);
        let old = Expr::CallExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let call = clone.root.as_call_expression(tx.ast()).unwrap();
            let callee = call.callee(tx.ast());
            let args = call.arguments(tx.ast());
            let receiver = source_callee
                .as_member_expression(tx.ast())
                .map(|member| tx.clone_detached(member.object(tx.ast())));
            let this_arg = if callee.is_identifier_reference(tx.ast()) {
                None
            } else if callee.is_super(tx.ast()) {
                Some(tx.build_detached(|ast| ast.expr_this_expression(span)))
            } else {
                receiver.as_ref().map(|copy| copy.root)
            };
            let built =
                tx.build_detached(|ast| build_call_spread(span, callee, args, this_arg, ast));
            clone.replace_root(built.expression);
            let layout = tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            if let Some(receiver) = &receiver {
                tx.duplicate_semantics_from_detached_clone(receiver, &layout);
            }
            register_helpers(tx, scope, &built.helpers);
        })
        .expect("call spread mutation must pass preflight");
    }

    pub fn exit_new_expression(
        &mut self,
        node: NewExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if !has_call_spread(node.arguments(ctx.ast), ctx.ast) {
            return;
        }
        let span = node.span(ctx.ast);
        let old = Expr::NewExpression(node);
        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let new = clone.root.as_new_expression(tx.ast()).unwrap();
            let callee = new.callee(tx.ast());
            let args = new.arguments(tx.ast());
            let built = tx.build_detached(|ast| build_new_spread(span, callee, args, ast));
            clone.replace_root(built.expression);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            register_helpers(tx, scope, &built.helpers);
        })
        .expect("new spread mutation must pass preflight");
    }
}

struct BuiltSpread {
    expression: Expr,
    helpers: Vec<IdentifierReference>,
}

fn build_array_spread(
    span: swc_next_ecma_ast::Span,
    elements: TypedSubRange<Option<Argument>>,
    ast: &mut Ast<'_>,
) -> BuiltSpread {
    let mut args = Vec::new();
    let mut current = Vec::new();
    for index in 0..elements.len() {
        let element = elements.get_node(ast, index).unwrap();
        match element {
            Some(argument) if argument.is_spread_element(ast) => {
                flush_array_segment(&mut current, &mut args, ast);
                args.push(Argument::Expr(
                    argument.as_spread_element(ast).unwrap().argument(ast),
                ));
            }
            element => current.push(element),
        }
    }
    flush_array_segment(&mut current, &mut args, ast);

    let empty = ast.add_typed_sub_range(Vec::<Option<Argument>>::new());
    let array = ast.expr_array_expression(swc_next_ecma_ast::DUMMY_SP, empty);
    let concat = ast.add_utf8("concat");
    let concat = ast.property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, concat);
    let callee =
        ast.expr_member_expression(swc_next_ecma_ast::DUMMY_SP, array, concat, false, false);
    let args = ast.add_typed_sub_range(args);
    BuiltSpread {
        expression: ast.expr_call_expression(span, callee, args, false, None),
        helpers: Vec::new(),
    }
}

fn flush_array_segment(
    current: &mut Vec<Option<Argument>>,
    args: &mut Vec<Argument>,
    ast: &mut Ast<'_>,
) {
    if current.is_empty() {
        return;
    }
    let elements = ast.add_typed_sub_range(std::mem::take(current));
    args.push(Argument::Expr(
        ast.expr_array_expression(swc_next_ecma_ast::DUMMY_SP, elements),
    ));
}

fn build_call_spread(
    span: swc_next_ecma_ast::Span,
    callee: Expr,
    args: TypedSubRange<Argument>,
    this_arg: Option<Expr>,
    ast: &mut Ast<'_>,
) -> BuiltSpread {
    let mut built_args = build_args_array(span, args, ast);
    let this_arg = this_arg.unwrap_or_else(|| make_undefined(span, ast));
    let apply = ast.add_utf8("apply");
    let apply = ast.property_key_identifier_name(swc_next_ecma_ast::DUMMY_SP, apply);
    let apply =
        ast.expr_member_expression(swc_next_ecma_ast::DUMMY_SP, callee, apply, false, false);
    let args = ast.add_typed_sub_range([
        Argument::Expr(this_arg),
        Argument::Expr(built_args.expression),
    ]);
    built_args.expression = ast.expr_call_expression(span, apply, args, false, None);
    built_args
}

fn build_new_spread(
    span: swc_next_ecma_ast::Span,
    callee: Expr,
    args: TypedSubRange<Argument>,
    ast: &mut Ast<'_>,
) -> BuiltSpread {
    let mut built_args = build_args_array(span, args, ast);
    let name = ast.add_utf8("_construct");
    let helper = ast.identifier_reference(swc_next_ecma_ast::DUMMY_SP, name);
    let args = ast.add_typed_sub_range([
        Argument::Expr(callee),
        Argument::Expr(built_args.expression),
    ]);
    built_args.expression =
        ast.expr_call_expression(span, Expr::IdentifierReference(helper), args, false, None);
    built_args.helpers.push(helper);
    built_args
}

fn build_args_array(
    span: swc_next_ecma_ast::Span,
    args: TypedSubRange<Argument>,
    ast: &mut Ast<'_>,
) -> BuiltSpread {
    let mut elements = Vec::new();
    let mut helpers = Vec::new();
    for index in 0..args.len() {
        let argument = args.get_node(ast, index).unwrap();
        if let Some(spread) = argument.as_spread_element(ast) {
            let name = ast.add_utf8("_to_consumable_array");
            let helper = ast.identifier_reference(swc_next_ecma_ast::DUMMY_SP, name);
            let args = ast.add_typed_sub_range([Argument::Expr(spread.argument(ast))]);
            let wrapped = ast.expr_call_expression(
                swc_next_ecma_ast::DUMMY_SP,
                Expr::IdentifierReference(helper),
                args,
                false,
                None,
            );
            helpers.push(helper);
            elements.push(Some(Argument::Expr(wrapped)));
        } else {
            elements.push(Some(Argument::Expr(argument.as_expr(ast).unwrap())));
        }
    }
    let elements = ast.add_typed_sub_range(elements);
    BuiltSpread {
        expression: ast.expr_array_expression(span, elements),
        helpers,
    }
}

fn register_helpers(
    tx: &mut crate::mutation::MutationTransaction<'_, '_>,
    scope: swc_next_ecma_ast::ScopeId,
    helpers: &[IdentifierReference],
) {
    for &helper in helpers {
        let name = tx.ast().canonicalize_utf8(helper.name(tx.ast()));
        let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
        tx.reference(
            scope,
            helper.node_id(),
            name,
            ReferenceFlags::default(),
            symbol,
        );
    }
}

fn has_array_spread(elements: TypedSubRange<Option<Argument>>, ast: &Ast<'_>) -> bool {
    (0..elements.len()).any(|index| {
        elements
            .get_node(ast, index)
            .unwrap()
            .is_some_and(|argument| argument.is_spread_element(ast))
    })
}

fn has_call_spread(args: TypedSubRange<Argument>, ast: &Ast<'_>) -> bool {
    (0..args.len()).any(|index| args.get_node(ast, index).unwrap().is_spread_element(ast))
}

fn make_undefined(span: swc_next_ecma_ast::Span, ast: &mut Ast<'_>) -> Expr {
    let zero = ast.add_utf8("0");
    let zero = ast.expr_numeric_literal(span, NumericLiteralKind::Decimal, zero);
    ast.expr_unary_expression(span, zero, UnaryOperator::Void)
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn spread_wrappers_helpers_and_receiver_copies_are_transactional() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "function f(obj, xs) { const a = [0, ...xs]; obj.method(...xs); return new obj.C(...xs); }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.spread = true;
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

        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
