use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, SourceType};
use swc_next_ecma_codegen::{Options as PrintOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParseOptions, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

use crate::{Transformer, TransformerOptions, es2015::ES2015Options};

fn transform(source: &str) -> String {
    transform_with_arrow(source, false)
}

fn transform_with_arrow(source: &str, arrow: bool) -> String {
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let parsed = Parser::init(
        &mut ast,
        ParseOptions {
            source_type: SourceType::Script,
            ..ParseOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:#?}", parsed.diagnostics);
    let semantic = analyze(parsed.program, &ast, AnalyzeOptions::default()).semantic;
    Transformer::transform(
        parsed.program,
        &mut ast,
        &semantic,
        TransformerOptions {
            es2015: ES2015Options {
                arrow_functions: arrow.then(Default::default),
                generator: true,
                ..ES2015Options::default()
            },
            ..TransformerOptions::default()
        },
    );
    print(&allocator, &ast, parsed.program, PrintOptions::default())
        .code
        .to_owned()
}

#[test]
fn lowers_simple_yield_and_return() {
    let output = transform("function* gen(a) { var x = a + (yield 1); return x; }");
    assert!(!output.contains("function*"), "{output}");
    assert!(!output.contains("yield"), "{output}");
    assert!(output.contains("_ts_generator(this"), "{output}");
    assert!(output.contains("return [4, 1]"), "{output}");
    assert!(output.contains("_state.sent()"), "{output}");
}

#[test]
fn ports_swc_empty_generator_fixture() {
    let output = transform(include_str!(
        "../../../tests/fixtures/es2015_generator/empty_fn_decl/input.js"
    ));
    assert!(!output.contains("function*"), "{output}");
    assert!(output.contains("return _ts_generator(this"), "{output}");
    assert!(output.contains("return [2]"), "{output}");
}

#[test]
fn nested_generator_is_lowered_without_tainting_outer_expression() {
    let output = transform(
        "function* outer() { function plain() { return function* inner() { yield 2; }; } yield 1; }",
    );
    assert_eq!(output.matches("_ts_generator(this").count(), 2, "{output}");
    assert!(!output.contains("yield"), "{output}");
}

#[test]
fn captures_generator_arguments_and_new_target() {
    let output = transform_with_arrow(
        "function* gen() { yield (() => arguments[0])(); return new.target; }",
        true,
    );
    assert!(output.contains("var _arguments = arguments"), "{output}");
    assert!(output.contains("_newtarget = new.target"), "{output}");
    assert!(output.contains("_arguments[0]"), "{output}");
    assert!(output.contains("return [2, _newtarget]"), "{output}");
    assert!(!output.contains("=>"), "{output}");
}

#[test]
fn lowers_branch_and_loop_control_flow() {
    let output = transform(
        "function* gen(n) { for (var i = 0; i < n; i++) { if (i) yield i; else yield 0; if (i > 2) break; } return i; }",
    );
    assert!(!output.contains("yield"), "{output}");
    assert!(!output.contains("for ("), "{output}");
    assert!(output.contains("switch (_state.label)"), "{output}");
    assert!(output.matches("return [4,").count() >= 2, "{output}");
    assert!(output.contains("return [3,"), "{output}");
}

#[test]
fn preserves_expression_evaluation_order_across_yield() {
    let output = transform(
        "function* gen(obj) { obj[key()] = call(obj.method, first(), yield second(), third()); return cond() ? yield 1 : yield 2; }",
    );
    assert!(!output.contains("yield"), "{output}");
    let assignment = output
        .find("= obj;")
        .expect("object is cached before suspension");
    let first_yield = output
        .find("return [4, second()]")
        .expect("yield is emitted");
    assert!(assignment < first_yield, "{output}");
    assert!(output.contains("first()"), "{output}");
    assert!(output.contains("third()"), "{output}");
    assert!(output.matches("return [4,").count() >= 3, "{output}");
}

#[test]
fn lowers_switch_and_try_finally() {
    let output = transform(
        "function* gen(x) { try { switch (x) { case yield 1: yield 2; break; default: yield 3; } } catch (e) { yield e; } finally { yield 4; } }",
    );
    assert!(!output.contains("yield"), "{output}");
    assert!(output.contains("_state.trys.push"), "{output}");
    assert!(output.contains("return [7]"), "{output}");
    assert!(output.matches("return [4,").count() >= 5, "{output}");
}

#[test]
fn ports_swc_try_finally_fixture() {
    let output = transform(include_str!(
        "../../../tests/fixtures/es2015_generator/try_finally/input.js"
    ));
    assert!(!output.contains("yield"), "{output}");
    assert!(output.contains("_state.trys.push"), "{output}");
    assert!(output.contains("console.log(2)"), "{output}");
    assert!(output.contains("return [7]"), "{output}");
}

#[test]
fn lowers_yield_delegate_and_labeled_control_flow() {
    let output = transform(
        "function* gen(xs) { outer: for (var i = 0; i < 3; i++) { yield* xs; if (i) continue outer; break outer; } }",
    );
    assert!(!output.contains("yield"), "{output}");
    assert!(!output.contains("outer:"), "{output}");
    assert!(output.contains("return [5, xs]"), "{output}");
    assert!(output.matches("return [3,").count() >= 3, "{output}");
}

#[test]
fn lowers_for_in_with_a_suspending_body() {
    let output = transform("function* gen(object) { for (var key in object) { yield key; } }");
    assert!(!output.contains("yield"), "{output}");
    assert!(output.contains("for (_tmp1 in object)"), "{output}");
    assert!(output.contains("return [4, key]"), "{output}");
}

#[test]
fn preserves_with_environment_across_suspension() {
    let output = transform("function* gen(scope) { with (scope) { yield value; value++; } }");
    assert!(!output.contains("yield"), "{output}");
    assert!(output.matches("with (_tmp)").count() >= 2, "{output}");
    assert!(output.contains("return [4, value]"), "{output}");
}
