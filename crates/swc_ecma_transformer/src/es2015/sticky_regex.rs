use swc_next_ecma_ast::{Expr, GetSpan, RegExpLiteral};
use swc_next_ecma_semantic::{ReferenceFlags, ReferenceSpace};

use crate::state::{TransformerCtx, TransformerCtxExt};

pub(super) struct StickyRegex;

impl StickyRegex {
    pub fn exit_reg_exp_literal(
        &mut self,
        literal: RegExpLiteral,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        let (span, pattern, flags, is_sticky) = {
            let ast = &*ctx.ast;
            let flags = literal.flags(ast);
            (
                literal.span(ast),
                literal.pattern(ast),
                flags,
                ast.get_utf8(flags).contains('y'),
            )
        };
        if !is_sticky {
            return;
        }

        let old = Expr::RegExpLiteral(literal);
        let scope = ctx.semantic().scope_of(literal.node_id());
        ctx.transaction(literal.node_id(), |tx| {
            let mut clone = tx.clone_detached(old);
            let (replacement, callee) = tx.build_detached(|ast| {
                let pattern =
                    ast.argument_expr_string_literal(span, pattern.into(), ast.empty_utf8());
                let flags = ast.argument_expr_string_literal(span, flags.into(), ast.empty_utf8());
                let arguments = ast.add_typed_sub_range([pattern, flags]);
                let reg_exp_name = ast.add_utf8("RegExp");
                let callee = ast.identifier_reference(span, reg_exp_name);
                let replacement =
                    ast.new_expression(span, Expr::IdentifierReference(callee), arguments, None);
                (Expr::NewExpression(replacement), callee)
            });
            clone.replace_root(replacement);
            tx.replace_expression_from_transformed_clone(old, clone, scope, &[]);
            let name = tx.ast().canonicalize_utf8(callee.name(tx.ast()));
            let symbol = tx.semantic().lookup(scope, name, ReferenceSpace::Value);
            tx.reference(
                scope,
                callee.node_id(),
                name,
                ReferenceFlags::default(),
                symbol,
            );
        })
        .expect("sticky-regex mutation must pass preflight");
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, NodeKind};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    #[test]
    fn sticky_literal_becomes_registered_regexp_construction() {
        let allocator = Allocator::new();
        let mut ast = Ast::new("const value = /a/y;", &allocator);
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let declaration = parse
            .program
            .body(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .as_declaration(&ast)
            .and_then(|declaration| declaration.as_variable_declaration(&ast))
            .unwrap();
        let literal = declaration
            .declarators(&ast)
            .get_node(&ast, 0)
            .unwrap()
            .init(&ast)
            .unwrap()
            .as_reg_exp_literal(&ast)
            .unwrap();
        let mut options = TransformerOptions::default();
        options.es2015.sticky_regex = true;
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

        assert_eq!(ast.node_kind(literal.node_id()), NodeKind::NewExpression);
        assert!(semantic.iter_references().any(|entry| {
            ast.get_ast_str_bytes(entry.reference.name) == b"RegExp"
                && semantic.parent_of(entry.reference.node) == Some(literal.node_id())
        }));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
