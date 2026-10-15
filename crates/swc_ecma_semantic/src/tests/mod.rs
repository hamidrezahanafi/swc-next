mod aliases;
mod bindings;
mod exports;
mod members;
mod properties;
mod references;
mod support;
mod visibility;

use support::*;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Lang, ScopeId, SourceType, SymbolId};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

use crate::{
    AnalyzeOptions, ExportOrigin, ExportTarget, NameResolution, ReferenceSpace, Semantic,
    SemanticReturn, SymbolFlags, analyze,
};

#[track_caller]
fn analyze_source<'id, 'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
    guard: swc_next_ecma_ast::AstGuard<'id>,
) -> (Ast<'a, 'id>, Semantic<'a, 'id>) {
    analyze_with_options(
        allocator,
        source,
        parser_options,
        AnalyzeOptions::default(),
        guard,
    )
}

#[track_caller]
fn analyze_with_options<'id, 'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
    analyze_options: AnalyzeOptions,
    guard: swc_next_ecma_ast::AstGuard<'id>,
) -> (Ast<'a, 'id>, Semantic<'a, 'id>) {
    let parse = Parser::init(
        allocator,
        source,
        parser_options,
        NoTokenParserConfig,
        guard,
    )
    .parse();
    let ast = parse.ast;
    assert!(parse.diagnostics.is_empty());

    let result = analyze(&ast, analyze_options);
    assert!(result.diagnostics.is_empty());
    (ast, result.semantic)
}

#[track_caller]
fn analyze_syntax_source<'id, 'a>(
    allocator: &'a Allocator,
    source: &'a str,
    parser_options: Options,
    guard: swc_next_ecma_ast::AstGuard<'id>,
) -> (Ast<'a, 'id>, SemanticReturn<'a, 'id>) {
    let parse = Parser::init(
        allocator,
        source,
        parser_options,
        NoTokenParserConfig,
        guard,
    )
    .parse();
    assert!(parse.diagnostics.is_empty());
    let result = analyze(
        &parse.ast,
        AnalyzeOptions {
            check_syntax: true,
            ..AnalyzeOptions::default()
        },
    );
    (parse.ast, result)
}

#[track_caller]
fn has_syntax_diagnostics(allocator: &Allocator, source: &str, parser_options: Options) -> bool {
    swc_next_ecma_ast::make_ast_guard!(ast_guard);
    let (_, result) = analyze_syntax_source(allocator, source, parser_options, ast_guard);
    !result.diagnostics.is_empty()
}

#[track_caller]
fn reference_symbol<'id>(
    ast: &Ast<'_, 'id>,
    semantic: &Semantic<'_, 'id>,
    name: &[u8],
) -> Option<SymbolId> {
    let mut references = semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name);
    let symbol = references.next().unwrap().reference.symbol;
    assert!(references.next().is_none());
    symbol
}

fn reference_symbols<'id>(
    ast: &Ast<'_, 'id>,
    semantic: &Semantic<'_, 'id>,
    name: &[u8],
) -> Vec<Option<SymbolId>> {
    semantic
        .iter_references()
        .filter(|entry| ast.get_utf8(entry.reference.name).as_bytes() == name)
        .map(|entry| entry.reference.symbol)
        .collect()
}

#[track_caller]
fn symbol_named<'id>(ast: &Ast<'_, 'id>, semantic: &Semantic<'_, 'id>, name: &[u8]) -> SymbolId {
    let mut symbols = semantic
        .iter_symbols()
        .filter(|entry| ast.get_wtf8(entry.symbol.name).as_bytes() == name);
    let symbol = symbols.next().unwrap().id;
    assert!(symbols.next().is_none());
    symbol
}

fn ts_module_options() -> Options {
    Options {
        lang: Lang::Ts,
        ..Options::default()
    }
}

fn dts_module_options() -> Options {
    Options {
        lang: Lang::Dts,
        ..Options::default()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TsModuleState {
    NonInstantiated,
    Instantiated,
    ConstEnumOnly,
}

#[track_caller]
fn assert_ts_module_state<'id>(
    semantic: &Semantic<'_, 'id>,
    symbol: SymbolId,
    expected: TsModuleState,
) {
    let flags = semantic.symbol(symbol).flags;
    let actual = match (
        flags.contains(SymbolFlags::VALUE_MODULE),
        flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE),
    ) {
        (false, false) => TsModuleState::NonInstantiated,
        (true, false) => TsModuleState::Instantiated,
        (true, true) => TsModuleState::ConstEnumOnly,
        (false, true) => panic!("const-enum-only modules must exist in value space"),
    };
    assert_eq!(actual, expected);
}
