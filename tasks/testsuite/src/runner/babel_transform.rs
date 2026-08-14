use std::panic::{AssertUnwindSafe, catch_unwind};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Program, Severity};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{Transformer, TransformerOptions};

use crate::{
    cases::{Case, babel::BabelTransformCase},
    runner::Runner,
    suite::TestResult,
};

pub struct BabelTransformRunner;

const MAX_SNAPSHOT_OUTPUT_CHARS: usize = 4_000;

impl Runner<BabelTransformCase> for BabelTransformRunner {
    const NAME: &'static str = "babel_transform";

    fn should_run(case: &BabelTransformCase) -> bool {
        case.options.plugins.errors.is_empty()
            && case.options.plugins.unsupported.is_empty()
            && case.options.presets.errors.is_empty()
            && case.options.presets.unsupported.is_empty()
    }

    fn run(case: &BabelTransformCase) -> TestResult {
        let path = case.relative_path().to_path_buf();
        let result = catch_unwind(AssertUnwindSafe(|| run_case(case)));
        match result {
            Ok(Ok(())) => TestResult::Passed {
                path,
                negative: false,
            },
            Ok(Err(error)) => TestResult::Failed {
                path,
                error,
                negative: false,
            },
            Err(_) => TestResult::Panic {
                path,
                negative: false,
            },
        }
    }
}

fn run_case(case: &BabelTransformCase) -> Result<(), String> {
    let actual = transform_input(case)?;
    let expected = normalize_expected_output(case)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{}: transformed output differs from output.js\n\
             --- expected (Babel output.js)\n{}\n\
             --- actual (SWC Next)\n{}",
            case.name,
            snapshot_output(&expected),
            snapshot_output(&actual),
        ))
    }
}

fn snapshot_output(output: &str) -> String {
    let char_count = output.chars().count();
    if char_count <= MAX_SNAPSHOT_OUTPUT_CHARS {
        return output.to_owned();
    }

    let head_count = MAX_SNAPSHOT_OUTPUT_CHARS / 2;
    let tail_count = MAX_SNAPSHOT_OUTPUT_CHARS - head_count;
    let head = output.chars().take(head_count).collect::<String>();
    let tail = output
        .chars()
        .rev()
        .take(tail_count)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let omitted = char_count - MAX_SNAPSHOT_OUTPUT_CHARS;
    format!("{head}\n... {omitted} characters omitted ...\n{tail}")
}

fn transform_input(case: &BabelTransformCase) -> Result<String, String> {
    let source = case.input.as_str();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    let options: TransformerOptions = case.options.clone().into();
    let mut semantic_ret = analyze(
        program,
        &ast,
        AnalyzeOptions {
            check_syntax: true,
            features: options.semantic_features(),
        },
    );
    if let Some(diagnostic) = semantic_ret
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "semantic: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    Transformer::transform(program, &mut ast, &mut semantic_ret.semantic, options);
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_owned())
}

fn normalize_expected_output(case: &BabelTransformCase) -> Result<String, String> {
    let source = case.output.as_str();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_owned())
}

fn parse_program(ast: &mut Ast<'_>, case: &BabelTransformCase) -> Result<Program, String> {
    let options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        ..ParserOptions::default()
    };
    let (program, error) = {
        let result = Parser::init(ast, options, NoTokenParserConfig).parse();
        let error = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| {
                format!(
                    "parse: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            });
        (result.program, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    Ok(program)
}
