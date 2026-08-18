//! ES3 transforms.

use swc_next_ecma_ast::{AssignmentExpression, MemberExpression, ObjectProperty};

use crate::state::TransformerCtx;

mod member_expr_lits;
mod property_literals;
mod reserved_words;

pub struct ES3 {
    member_expr_lits: bool,
    property_literals: bool,
    reserved_words: bool,
}

#[derive(Default)]
pub struct ES3Options {
    pub member_expr_lits: bool,
    pub property_literals: bool,
    pub reserved_words: bool,
}

impl ES3 {
    pub fn new(options: ES3Options) -> Self {
        Self {
            member_expr_lits: options.member_expr_lits,
            property_literals: options.property_literals,
            reserved_words: options.reserved_words,
        }
    }

    pub(crate) fn needs_node_traversal(&self) -> bool {
        self.member_expr_lits || self.property_literals || self.reserved_words
    }

    pub(crate) fn exit_member_expression(
        &mut self,
        node: MemberExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.member_expr_lits {
            member_expr_lits::MemberExprLits.exit_member_expression(node, ctx);
        }
    }

    pub(crate) fn exit_object_property(
        &mut self,
        node: ObjectProperty,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.property_literals {
            property_literals::PropertyLiterals.exit_object_property(node, ctx);
        }
    }

    pub(crate) fn exit_assignment_expression(
        &mut self,
        node: AssignmentExpression,
        ctx: &mut TransformerCtx<'_, '_, '_>,
    ) {
        if self.reserved_words {
            reserved_words::ReservedWords.exit_assignment_expression(node, ctx);
        }
    }
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
    fn literal_keys_are_replaced_transactionally() {
        let allocator = Allocator::new();
        let mut ast = Ast::new(
            "obj['foo']; obj.abstract; ({ 'bar': 1, abstract: 2 });",
            &allocator,
        );
        let parse = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let first = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let first = first
            .as_expression_statement(&ast)
            .unwrap()
            .expression(&ast)
            .as_member_expression(&ast)
            .unwrap();
        let first_old_property = first.property(&ast).node_id();
        let second = parse.program.body(&ast).get_node(&ast, 1).unwrap();
        let second = second
            .as_expression_statement(&ast)
            .unwrap()
            .expression(&ast)
            .as_member_expression(&ast)
            .unwrap();
        let second_old_property = second.property(&ast).node_id();

        let mut options = TransformerOptions::default();
        options.es3.member_expr_lits = true;
        options.es3.property_literals = true;
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

        let first = parse.program.body(&ast).get_node(&ast, 0).unwrap();
        let first = first
            .as_expression_statement(&ast)
            .unwrap()
            .expression(&ast)
            .as_member_expression(&ast)
            .unwrap();
        assert!(!first.computed(&ast));
        assert_eq!(
            ast.get_utf8(
                first
                    .property(&ast)
                    .as_identifier_name(&ast)
                    .unwrap()
                    .name(&ast)
            ),
            "foo"
        );
        let second = parse.program.body(&ast).get_node(&ast, 1).unwrap();
        let second = second
            .as_expression_statement(&ast)
            .unwrap()
            .expression(&ast)
            .as_member_expression(&ast)
            .unwrap();
        assert!(second.computed(&ast));
        assert_eq!(
            ast.get_wtf8(
                second
                    .property(&ast)
                    .as_string_literal(&ast)
                    .unwrap()
                    .value(&ast)
            )
            .as_bytes(),
            b"abstract"
        );
        assert!(!semantic.is_node_active(first_old_property));
        assert!(!semantic.is_node_active(second_old_property));
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }
}
