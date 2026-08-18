//! ES2015 block-scoped functions — lower declarations to `let`-bound named
//! function expressions while preserving the outer binding identity.

use swc_next_ecma_ast::{
    BindingPattern, BlockStatement, Expr, FunctionType, GetSpan, NodeId, Stmt, VariableKind,
};
use swc_next_ecma_semantic::{DeclarationId, ReferenceId, SymbolFlags, scope};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct BlockScopedFns;

impl BlockScopedFns {
    pub fn exit_block_statement(
        &mut self,
        block: BlockStatement,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let plans = collect_plans(block, ctx);
        if plans.is_empty() {
            return;
        }
        let block_scope = ctx.semantic().scope_of(block.node_id());
        ctx.transaction(block.node_id(), |tx| {
            let clone = tx.clone_detached(block);
            let mut statements = Vec::with_capacity(block.body(tx.ast()).len());
            let mut transitions = Vec::with_capacity(plans.len());
            for statement in clone.root.body(tx.ast()).iter() {
                let statement: Stmt = tx.ast().get_node_in_sub_range(statement);
                let Some(function) = statement
                    .as_declaration(tx.ast())
                    .and_then(|declaration| declaration.as_function(tx.ast()))
                else {
                    statements.push(statement);
                    continue;
                };
                let origin = clone
                    .origin_of(function.node_id())
                    .expect("cloned function has an attached origin");
                let plan = plans
                    .iter()
                    .find(|plan| plan.function == origin)
                    .expect("every direct function declaration has a lowering plan");
                let binding = function.id(tx.ast()).unwrap();
                let span = function.span(tx.ast());
                let name = binding.name(tx.ast());
                let context = tx.ast().syntax_context(binding.node_id());
                let inner = tx.build_detached(|ast| {
                    let inner = ast.binding_identifier(
                        span,
                        name,
                        swc_next_ecma_ast::TypedSubRange::empty(),
                        None,
                        false,
                    );
                    let mark = ast.fresh_mark(swc_next_ecma_ast::Mark::root());
                    let inner_context = ast.apply_mark(context, mark);
                    ast.set_syntax_context(inner.node_id(), inner_context);
                    function.set_function_type(ast, FunctionType::FunctionExpression);
                    function.set_id(ast, Some(inner));
                    inner
                });
                let replacement = tx.build_detached(|ast| {
                    let declarator = ast.variable_declarator(
                        span,
                        BindingPattern::BindingIdentifier(binding),
                        Some(Expr::Function(function)),
                        false,
                    );
                    let declarators = ast.add_typed_sub_range([declarator]);
                    let declaration =
                        ast.variable_declaration(span, VariableKind::Let, declarators, false);
                    Stmt::Declaration(swc_next_ecma_ast::Decl::VariableDeclaration(declaration))
                });
                statements.push(replacement);

                let expression_name_scope = tx.create_scope(
                    block_scope,
                    function.node_id(),
                    scope::ScopeKind::ExpressionName,
                    tx.semantic().scope(plan.function_scope).flags,
                );
                tx.reparent_scope(plan.function_scope, expression_name_scope);
                let inner_symbol = tx.declare(
                    expression_name_scope,
                    inner.node_id(),
                    tx.ast().canonicalize_utf8(name),
                    SymbolFlags::FUNCTION,
                );
                transitions.push((plan, inner_symbol));
            }
            let statements = tx.build_detached(|ast| ast.add_typed_sub_range(statements));
            tx.build_detached(|ast| clone.root.set_body(ast, statements));
            tx.replace_from_transformed_clone(block, clone, block_scope, &[]);

            for (plan, inner_symbol) in transitions {
                tx.update_declaration_flags(plan.declaration, SymbolFlags::BLOCK_SCOPED_VAR);
                tx.update_symbol_flags(plan.symbol, SymbolFlags::BLOCK_SCOPED_VAR);
                for &reference in &plan.self_references {
                    tx.rebind_reference(reference, Some(inner_symbol));
                }
            }
        })
        .expect("block-scoped function mutation must pass preflight");
    }
}

struct FunctionPlan {
    function: NodeId,
    declaration: DeclarationId,
    symbol: swc_next_ecma_ast::SymbolId,
    function_scope: swc_next_ecma_ast::ScopeId,
    self_references: Vec<ReferenceId>,
}

fn collect_plans(block: BlockStatement, ctx: &TransformerCtx<'_, '_, '_>) -> Vec<FunctionPlan> {
    block
        .body(ctx.ast)
        .iter()
        .filter_map(|statement| {
            let statement: Stmt = ctx.ast.get_node_in_sub_range(statement);
            let function = statement
                .as_declaration(ctx.ast)
                .and_then(|declaration| declaration.as_function(ctx.ast))?;
            let binding = function.id(ctx.ast)?;
            let declaration = ctx.semantic().declaration_of(binding.node_id())?;
            let symbol = ctx.semantic().declaration(declaration).symbol;
            let self_references = ctx
                .semantic()
                .uses(symbol)
                .iter()
                .filter(|reference| {
                    let node = ctx.semantic().reference(**reference).node;
                    ctx.semantic()
                        .ancestors(Some(node))
                        .any(|ancestor| ancestor == function.node_id())
                })
                .copied()
                .collect();
            Some(FunctionPlan {
                function: function.node_id(),
                declaration,
                symbol,
                function_scope: ctx.semantic().scope_of(function.node_id()),
                self_references,
            })
        })
        .collect()
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
    fn outer_binding_and_recursive_expression_name_are_updated_atomically() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "'use strict'; { function recurse(n) { return n ? recurse(n - 1) : 0; } use(recurse); }",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.block_scoped_fns = true;
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
