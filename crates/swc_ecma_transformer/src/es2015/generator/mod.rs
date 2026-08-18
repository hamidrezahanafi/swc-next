use swc_next_ecma_ast::Ast;
use swc_next_ecma_semantic::Semantic;

mod compiler;

pub(super) struct Generator;

impl Generator {
    pub(super) fn transform<'a>(&mut self, ast: &mut Ast<'a>, semantic: &mut Semantic<'a>) {
        compiler::transform(ast, semantic);
    }
}

#[cfg(test)]
mod tests {
    use swc_next_allocator::Allocator;
    use swc_next_ecma_ast::{Ast, SourceType};
    use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
    use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

    use crate::{
        Transformer, TransformerOptions,
        mutation::{assert_semantic_matches_reanalysis, verify_program},
    };

    fn lower_and_assert(source: &str) {
        lower_and_assert_with_options(source, Options::default());
    }

    fn lower_and_assert_with_options(source: &str, parser_options: Options) {
        let allocator = Allocator::new();
        let mut ast = Ast::new(source, &allocator);
        let parse = Parser::init(&mut ast, parser_options, NoTokenParserConfig).parse();
        assert!(parse.diagnostics.is_empty(), "{:#?}", parse.diagnostics);
        let mut options = TransformerOptions::default();
        options.es2015.generator = true;
        let mut semantic = analyze(
            parse.program,
            &ast,
            AnalyzeOptions {
                features: options.semantic_features(),
                ..AnalyzeOptions::default()
            },
        )
        .semantic;
        let functions = semantic.generator_functions_post_order().unwrap().to_vec();
        assert!(!functions.is_empty(), "test source contains a generator");

        Transformer::transform(parse.program, &mut ast, &mut semantic, options);

        assert!(functions.iter().all(|function| !function.generator(&ast)));
        assert!(
            semantic
                .generator_functions_post_order()
                .unwrap()
                .is_empty()
        );
        verify_program(parse.program, &ast, &semantic).unwrap();
        assert_semantic_matches_reanalysis(parse.program, &ast, &semantic);
    }

    #[test]
    fn lowers_generator_with_transactionally_consistent_semantics() {
        lower_and_assert("function* gen() { yield 1; }");
    }

    #[test]
    fn preserves_generator_bindings_and_control_flow_semantics() {
        lower_and_assert(
            "function* gen(value) { var local = yield value; if (local) { yield local; } return local; }",
        );
    }

    #[test]
    fn preserves_generator_environment_capture_semantics() {
        lower_and_assert("function* gen(value) { yield arguments[0]; return new.target; }");
    }

    #[test]
    fn preserves_nested_function_semantics_during_generator_lowering() {
        lower_and_assert("function* gen() { function inner() { return 1; } yield inner(); }");
    }

    #[test]
    fn preserves_with_scope_semantics_during_generator_lowering() {
        lower_and_assert_with_options(
            "function* gen(object, value) { with (object) { yield value; } }",
            Options {
                source_type: SourceType::Script,
                ..Options::default()
            },
        );
    }
}
