use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options as ParseOptions, Parser};

#[track_caller]
pub fn expect_printed(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_minify(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ts(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Ts, SourceType::Module, false, false);
}

#[track_caller]
pub fn expect_printed_jsx(source: &str, expected: &str) {
    expect_printed_common(
        source,
        expected,
        Lang::Jsx,
        SourceType::Module,
        false,
        false,
    );
}

#[track_caller]
pub fn expect_printed_jsx_minify(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, true, false);
}

#[track_caller]
pub fn expect_printed_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_minify_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Module, true, true);
}

#[track_caller]
pub fn expect_printed_jsx_ascii(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Jsx, SourceType::Module, false, true);
}

#[track_caller]
pub fn expect_printed_script(source: &str, expected: &str) {
    expect_printed_common(source, expected, Lang::Js, SourceType::Script, false, false);
}

#[track_caller]
pub fn expect_printed_stable(source: &str, expected: &str) {
    let allocator = Allocator::new();
    let codegen1 = print_source(
        &allocator,
        source,
        Lang::Js,
        SourceType::Module,
        PrintOptions::default(),
    )
    .expect("source should parse without errors");

    assert_eq!(
        codegen1, expected,
        "failed for {source}\nactual:\n{codegen1}\nexpected:\n{expected}",
    );

    let codegen2 = print_source(
        &allocator,
        &codegen1,
        Lang::Js,
        SourceType::Module,
        PrintOptions::default(),
    )
    .expect("printed source should reparse without errors");

    assert_eq!(
        codegen1, codegen2,
        "codegen output changed after reparse for {source}\nfirst:\n{codegen1}\nsecond:\n{codegen2}",
    );
}

#[track_caller]
fn expect_printed_common(
    source: &str,
    expected: &str,
    lang: Lang,
    source_type: SourceType,
    minify: bool,
    ascii_only: bool,
) {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let parse_options = ParseOptions {
        source_type,
        lang,
        preserve_parens: false,
        comments: CommentMode::Attached,
        ..Default::default()
    };
    let result = Parser::init(&mut ast, parse_options, NoTokenParserConfig).parse();
    let has_errors = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error);
    if has_errors {
        return;
    }
    let program = result.program;

    let print_options = PrintOptions {
        minify,
        ascii_only,
        ..PrintOptions::default()
    };
    let result = print(&allocator, &ast, program, print_options);

    assert_eq!(
        result.code, expected,
        "failed for {}\nactual:\n{}\nexpected:\n{}",
        source, result.code, expected,
    );
}

fn print_source<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    lang: Lang,
    source_type: SourceType,
    options: PrintOptions,
) -> Option<String> {
    let mut ast = Ast::new(source, allocator);
    let parse_options = ParseOptions {
        source_type,
        lang,
        preserve_parens: false,
        comments: CommentMode::Attached,
        ..Default::default()
    };
    let result = Parser::init(&mut ast, parse_options, NoTokenParserConfig).parse();
    let has_errors = result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == swc_next_ecma_ast::Severity::Error);
    if has_errors {
        return None;
    }

    let program = result.program;
    Some(print(allocator, &ast, program, options).code.to_string())
}
