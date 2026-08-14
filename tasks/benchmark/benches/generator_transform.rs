use std::time::{Duration, Instant};

use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Program as NextProgram, SourceType};
use swc_next_ecma_codegen::{Options as NextCodegenOptions, print as next_print};
use swc_next_ecma_parser::{
    NoTokenParserConfig, Options as NextParserOptions, Parser as NextParser,
};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{Transformer, TransformerOptions, es2015::ES2015Options};
use swc_old_common::{
    FileName, GLOBALS, Globals, Mark, SourceMap, comments::NoopComments, sync::Lrc,
};
use swc_old_ecma_ast::{EsVersion, Pass, Program as OldProgram};
use swc_old_ecma_codegen::{Config as OldCodegenConfig, Emitter, text_writer::JsWriter};
use swc_old_ecma_compat_es2015::generator::generator;
use swc_old_ecma_parser::{Parser as OldParser, StringInput, Syntax, lexer::Lexer};
use swc_old_ecma_transforms_base::{helpers::HELPERS, helpers::Helpers, resolver};

const DEFAULT_GENERATOR_COUNT: usize = 48;

fn source() -> (String, usize) {
    let mut source = String::from("const sentinel = value => ({ value });\n");
    let generator_count = std::env::var("GENERATOR_BENCH_COUNT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_GENERATOR_COUNT);
    for index in 0..generator_count {
        source.push_str(&format!(
            r#"
function* generator_{index}(input) {{
    var total = yield {index};
    var current = 1 + (yield 2);
    for (var i = 0; i < 12; i++) {{
        if (i & 1) {{
            total += yield 3;
        }} else {{
            total += yield* [4, 5, 6];
        }}
        if (total > 100) break;
    }}
    for (var key in {{ first: 1, second: 2 }}) {{
        current += yield 7;
    }}
    current = consume(current, yield 8, total);
    try {{
        switch (yield 9) {{
            case 0: current = yield 10; break;
            case 1: current = choose(yield 11); break;
            default: current = yield 12;
        }}
    }} catch (error) {{
        current = yield 13;
    }} finally {{
        yield 14;
    }}
    return current + total;
}}
"#
        ));
    }
    (source, generator_count)
}

fn nongenerator_source(source: &str) -> String {
    source
        .replace("function* generator_", "function generator_")
        .replace("yield* ", "void ")
        .replace("yield ", "void ")
}

fn old_parse(file: &swc_old_common::SourceFile) -> OldProgram {
    let lexer = Lexer::new(
        Syntax::default(),
        EsVersion::EsNext,
        StringInput::from(file),
        None,
    );
    OldParser::new_from(lexer)
        .parse_program()
        .expect("SWC should parse the benchmark corpus")
}

fn old_resolve(program: &mut OldProgram) -> Mark {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();
    resolver(unresolved_mark, top_level_mark, false).process(program);
    unresolved_mark
}

fn old_transform(program: &mut OldProgram, unresolved_mark: Mark) {
    generator(unresolved_mark, NoopComments).process(program);
}

fn old_context() -> (Globals, Helpers) {
    let globals = Globals::new();
    let helpers = GLOBALS.set(&globals, || Helpers::new(false));
    (globals, helpers)
}

fn old_codegen(cm: Lrc<SourceMap>, program: &OldProgram) -> Vec<u8> {
    let mut output = Vec::new();
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut output, None);
        let mut emitter = Emitter {
            cfg: OldCodegenConfig::default().with_minify(true),
            cm,
            comments: None,
            wr: writer,
        };
        emitter
            .emit_program(program)
            .expect("SWC should print the transformed corpus");
    }
    output
}

fn next_codegen_options() -> NextCodegenOptions {
    NextCodegenOptions {
        minify: true,
        ..NextCodegenOptions::default()
    }
}

fn next_parse<'a>(ast: &mut Ast<'a>) -> NextProgram {
    let parsed = NextParser::init(
        ast,
        NextParserOptions {
            source_type: SourceType::Script,
            ..NextParserOptions::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "SWC Next should parse the benchmark corpus: {:#?}",
        parsed.diagnostics
    );
    parsed.program
}

fn next_transform(program: NextProgram, ast: &mut Ast<'_>) {
    let semantic = analyze(program, ast, AnalyzeOptions::default()).semantic;
    Transformer::transform(
        program,
        ast,
        &semantic,
        TransformerOptions {
            es2015: ES2015Options {
                generator: true,
                ..ES2015Options::default()
            },
            ..TransformerOptions::default()
        },
    );
}

fn verify_only_generator_transform(source: &str) -> (usize, usize, usize, usize) {
    let old_output = GLOBALS.set(&Globals::new(), || {
        HELPERS.set(&Helpers::new(false), || {
            let cm = Lrc::new(SourceMap::default());
            let file = cm.new_source_file(
                FileName::Custom("generator-transform-benchmark.js".into()).into(),
                source.to_owned(),
            );
            let mut program = old_parse(&file);
            let unresolved_mark = old_resolve(&mut program);
            old_transform(&mut program, unresolved_mark);
            String::from_utf8(old_codegen(cm, &program)).expect("SWC output should be UTF-8")
        })
    });

    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = next_parse(&mut ast);
    let next_input_nodes = ast.node_count();
    next_transform(program, &mut ast);
    let next_output_nodes = ast.node_count();
    let next_output = next_print(&allocator, &ast, program, next_codegen_options())
        .code
        .to_owned();

    for (name, output) in [
        ("swc", old_output.as_str()),
        ("swc-next", next_output.as_str()),
    ] {
        assert!(
            !output.contains("function*"),
            "{name} did not lower generators"
        );
        assert!(
            output.contains("_ts_generator"),
            "{name} did not emit the generator helper call"
        );
        assert!(
            output.contains("=>"),
            "{name} unexpectedly ran the arrow transform"
        );
    }
    if std::env::var_os("GENERATOR_BENCH_DUMP").is_some() {
        eprintln!("--- swc ---\n{old_output}\n--- swc-next ---\n{next_output}");
    }

    (
        old_output.len(),
        next_output.len(),
        next_input_nodes,
        next_output_nodes,
    )
}

fn bench_prerequisites(c: &mut Criterion, source: &str, generator_count: usize) {
    let mut group = c.benchmark_group(format!("generator/{generator_count}/prerequisite"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    let cm = Lrc::new(SourceMap::default());
    let file = cm.new_source_file(
        FileName::Custom("generator-transform-benchmark.js".into()).into(),
        source.to_owned(),
    );
    group.bench_function("swc/resolver", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let globals = Globals::new();
                GLOBALS.set(&globals, || {
                    let mut program = old_parse(&file);
                    let start = Instant::now();
                    black_box(old_resolve(&mut program));
                    elapsed += start.elapsed();
                    black_box(program);
                });
            }
            elapsed
        });
    });

    group.bench_function("swc-next/semantic", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                let start = Instant::now();
                black_box(analyze(program, &ast, AnalyzeOptions::default()));
                elapsed += start.elapsed();
            }
            elapsed
        });
    });
    group.finish();
}

fn bench_parse(c: &mut Criterion, source: &str, generator_count: usize) {
    let mut group = c.benchmark_group(format!("generator/{generator_count}/parse"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    let cm = Lrc::new(SourceMap::default());
    let file = cm.new_source_file(
        FileName::Custom("generator-transform-benchmark.js".into()).into(),
        source.to_owned(),
    );
    group.bench_function("swc", |b| b.iter(|| black_box(old_parse(&file))));

    group.bench_function("swc-next", |b| {
        b.iter_batched(
            Allocator::new,
            |allocator| {
                let mut ast = Ast::new(source, &allocator);
                black_box(next_parse(&mut ast));
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_transform(c: &mut Criterion, source: &str, generator_count: usize) {
    let mut group = c.benchmark_group(format!("generator/{generator_count}/transform"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    let cm = Lrc::new(SourceMap::default());
    let file = cm.new_source_file(
        FileName::Custom("generator-transform-benchmark.js".into()).into(),
        source.to_owned(),
    );
    group.bench_function("swc", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let (globals, helpers) = old_context();
                GLOBALS.set(&globals, || {
                    HELPERS.set(&helpers, || {
                        let mut program = old_parse(&file);
                        let unresolved_mark = old_resolve(&mut program);
                        let start = Instant::now();
                        old_transform(&mut program, unresolved_mark);
                        elapsed += start.elapsed();
                        black_box(program);
                    });
                });
            }
            elapsed
        });
    });

    group.bench_function("swc-next", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                let semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
                let start = Instant::now();
                Transformer::transform(
                    program,
                    &mut ast,
                    &semantic,
                    TransformerOptions {
                        es2015: ES2015Options {
                            generator: true,
                            ..ES2015Options::default()
                        },
                        ..TransformerOptions::default()
                    },
                );
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });
    group.finish();
}

fn bench_next_breakdown(c: &mut Criterion, source: &str, generator_count: usize) {
    let nongenerator_source = nongenerator_source(source);
    let mut group = c.benchmark_group(format!("generator/{generator_count}/swc-next-breakdown"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("framework-noop/input", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                let semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
                let start = Instant::now();
                Transformer::transform(program, &mut ast, &semantic, TransformerOptions::default());
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });

    group.bench_function("generator/no-match", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(&nongenerator_source, &allocator);
                let program = next_parse(&mut ast);
                let semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
                let start = Instant::now();
                Transformer::transform(
                    program,
                    &mut ast,
                    &semantic,
                    TransformerOptions {
                        es2015: ES2015Options {
                            generator: true,
                            ..ES2015Options::default()
                        },
                        ..TransformerOptions::default()
                    },
                );
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });

    group.bench_function("framework-noop/lowered", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                let semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
                Transformer::transform(
                    program,
                    &mut ast,
                    &semantic,
                    TransformerOptions {
                        es2015: ES2015Options {
                            generator: true,
                            ..ES2015Options::default()
                        },
                        ..TransformerOptions::default()
                    },
                );
                let start = Instant::now();
                Transformer::transform(program, &mut ast, &semantic, TransformerOptions::default());
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });
    group.finish();
}

fn bench_codegen(c: &mut Criterion, source: &str, generator_count: usize) {
    let mut group = c.benchmark_group(format!("generator/{generator_count}/codegen"));

    let cm = Lrc::new(SourceMap::default());
    let file = cm.new_source_file(
        FileName::Custom("generator-transform-benchmark.js".into()).into(),
        source.to_owned(),
    );
    let output_len = {
        let (globals, helpers) = old_context();
        GLOBALS.set(&globals, || {
            HELPERS.set(&helpers, || {
                let mut program = old_parse(&file);
                let unresolved_mark = old_resolve(&mut program);
                old_transform(&mut program, unresolved_mark);
                old_codegen(cm.clone(), &program).len()
            })
        })
    };
    group.throughput(Throughput::Bytes(output_len as u64));
    group.bench_function("swc", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let (globals, helpers) = old_context();
                GLOBALS.set(&globals, || {
                    HELPERS.set(&helpers, || {
                        let mut program = old_parse(&file);
                        let unresolved_mark = old_resolve(&mut program);
                        old_transform(&mut program, unresolved_mark);
                        let start = Instant::now();
                        black_box(old_codegen(cm.clone(), &program));
                        elapsed += start.elapsed();
                    });
                });
            }
            elapsed
        });
    });

    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = next_parse(&mut ast);
    next_transform(program, &mut ast);
    let output_len = next_print(&allocator, &ast, program, next_codegen_options())
        .code
        .len();
    group.throughput(Throughput::Bytes(output_len as u64));
    group.bench_function("swc-next", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                next_transform(program, &mut ast);
                let start = Instant::now();
                black_box(next_print(
                    &allocator,
                    &ast,
                    program,
                    next_codegen_options(),
                ));
                elapsed += start.elapsed();
            }
            elapsed
        });
    });
    group.finish();
}

fn bench_end_to_end(c: &mut Criterion, source: &str, generator_count: usize) {
    let mut group = c.benchmark_group(format!("generator/{generator_count}/end-to-end"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    let cm = Lrc::new(SourceMap::default());
    let file = cm.new_source_file(
        FileName::Custom("generator-transform-benchmark.js".into()).into(),
        source.to_owned(),
    );
    group.bench_function("swc", |b| {
        b.iter_batched(
            old_context,
            |(globals, helpers)| {
                GLOBALS.set(&globals, || {
                    HELPERS.set(&helpers, || {
                        let mut program = old_parse(&file);
                        let unresolved_mark = old_resolve(&mut program);
                        old_transform(&mut program, unresolved_mark);
                        black_box(old_codegen(cm.clone(), &program));
                    });
                });
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("swc-next", |b| {
        b.iter_batched(
            Allocator::new,
            |allocator| {
                let mut ast = Ast::new(source, &allocator);
                let program = next_parse(&mut ast);
                next_transform(program, &mut ast);
                black_box(next_print(
                    &allocator,
                    &ast,
                    program,
                    next_codegen_options(),
                ));
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_generator_transform(c: &mut Criterion) {
    let (source, generator_count) = source();
    let (old_output_bytes, next_output_bytes, next_input_nodes, next_output_nodes) =
        verify_only_generator_transform(&source);
    eprintln!(
        "fixture: generators={}, source={}B, old_output={}B, next_output={}B, next_nodes={}=>{}",
        generator_count,
        source.len(),
        old_output_bytes,
        next_output_bytes,
        next_input_nodes,
        next_output_nodes,
    );
    bench_parse(c, &source, generator_count);
    bench_prerequisites(c, &source, generator_count);
    bench_transform(c, &source, generator_count);
    bench_next_breakdown(c, &source, generator_count);
    bench_codegen(c, &source, generator_count);
    bench_end_to_end(c, &source, generator_count);
}

criterion_group!(benches, bench_generator_transform);
criterion_main!(benches);
