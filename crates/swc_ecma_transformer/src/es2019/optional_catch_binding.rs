//! ES2019: optional-catch-binding — Add a binding to catch clauses without one.
//!
//! ```js
//! try {} catch {} → try {} catch (_unused) {}
//! ```

use swc_next_ecma_ast::{BindingPattern, CatchClause};
use swc_next_ecma_semantic::SymbolFlags;

use crate::{
    mutation::IdentityPolicy,
    state::{TransformerCtx, TransformerCtxExt},
};

pub(super) struct OptionalCatchBinding;

impl OptionalCatchBinding {
    pub fn enter_catch_clause(&mut self, node: CatchClause, ctx: &mut TransformerCtx<'_, '_, '_>) {
        if node.param(ctx.ast).is_some() {
            return;
        }

        let scope = ctx.semantic().scope_of(node.node_id());
        ctx.transaction(node.node_id(), |tx| {
            let binding = tx.declare_private(
                scope,
                "_unused",
                SymbolFlags::FUNCTION_SCOPED_VAR | SymbolFlags::CATCH_VAR,
            );
            let span = tx.ast().span(node.node_id());
            let body = node.body(tx.ast());
            let replacement = tx.build_detached(|ast| {
                ast.catch_clause(
                    span,
                    Some(BindingPattern::BindingIdentifier(binding.declaration)),
                    body,
                )
            });
            tx.replace(
                node.node_id(),
                replacement.node_id(),
                IdentityPolicy::Preserve,
            );
            tx.attach_private_declaration(binding, node.node_id(), scope);
        })
        .expect("optional catch binding mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::Ast;
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, SymbolFlags, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn declares_generated_catch_binding_transactionally() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("try { work(); } catch { recover(); }", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let catch = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_try_statement(&ast)
            .and_then(|statement| statement.handler(&ast))
            .unwrap();
        let mut options = TransformerOptions::default();
        options.es2019.optional_catch_binding = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let catch_scope = semantic.scope_of(catch.node_id());

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        let parameter = catch.param(&ast).unwrap();
        let binding = parameter.as_binding_identifier(&ast).unwrap();
        let symbol = semantic.declaration_of(binding.node_id()).unwrap();
        let symbol = semantic.declaration(symbol).symbol;
        assert_eq!(semantic.symbol(symbol).scope, catch_scope);
        assert!(
            semantic
                .symbol(symbol)
                .flags
                .contains(SymbolFlags::CATCH_VAR)
        );
        assert_eq!(ast.mutation_epoch(), 1);
        assert_eq!(semantic.mutation_epoch(), 1);
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
