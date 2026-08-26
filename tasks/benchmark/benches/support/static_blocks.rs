use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, IdentifierReference, Lang, Visit, VisitWith};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};
use swc_next_ecma_transformer::{Transformer, TransformerOptions, es2022::ES2022Options};

pub const SINGLE_CLASS: &str = r#"
let sink;
class Example {
  static before = 0;
  static { sink = this.before; }
  static { sink += 1; }
  static { sink += 2; }
  static { sink += 3; }
  static after = sink;
}
"#;

pub const CLASS_HEAVY: &str = include_str!("../../files/static-block-heavy.js");

pub struct TransformCase {
    pub ast: Ast<'static>,
    pub semantic: Semantic<'static>,
}

fn transformer_options() -> TransformerOptions {
    TransformerOptions {
        es2022: ES2022Options {
            static_blocks: true,
        },
        ..TransformerOptions::default()
    }
}

pub fn setup_case(source: &'static str) -> TransformCase {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let parsed = Parser::init(
        allocator,
        source,
        ParserOptions {
            lang: Lang::Js,
            ..ParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);

    let ast = parsed.ast;
    let options = transformer_options();
    let analyzed = analyze(
        &ast,
        AnalyzeOptions {
            features: options.semantic_features(),
            ..AnalyzeOptions::default()
        },
    );
    assert!(
        analyzed.diagnostics.is_empty(),
        "{:#?}",
        analyzed.diagnostics
    );
    TransformCase {
        ast,
        semantic: analyzed.semantic,
    }
}

pub fn transform_case(case: &mut TransformCase) {
    Transformer::transform(&mut case.ast, &mut case.semantic, transformer_options());
}

pub fn setup_transformed(source: &'static str) -> Ast<'static> {
    let mut case = setup_case(source);
    transform_case(&mut case);
    case.ast
}

pub fn count_identifier_references(ast: &Ast<'_>) -> usize {
    struct Counter<'visit, 'ast> {
        ast: &'visit Ast<'ast>,
        count: usize,
    }

    impl<'ast> Visit<'ast> for Counter<'_, 'ast> {
        fn ast(&self) -> &Ast<'ast> {
            self.ast
        }

        fn visit_identifier_reference(&mut self, _node: IdentifierReference) {
            self.count += 1;
        }
    }

    let mut counter = Counter { ast, count: 0 };
    ast.root_program().visit_with(&mut counter);
    counter.count
}

pub fn codegen_len(ast: &Ast<'_>) -> usize {
    print(ast.allocator(), ast, CodegenOptions::default())
        .code
        .len()
}

pub fn end_to_end(source: &'static str) -> usize {
    let mut case = setup_case(source);
    transform_case(&mut case);
    codegen_len(&case.ast)
}
