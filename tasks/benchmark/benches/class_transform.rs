use std::time::{Duration, Instant};

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::Ast;
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};
use swc_next_ecma_transformer::{Transformer, TransformerOptions, es2015::ES2015Options};

fn class_source(class_count: usize, derived: bool, nested_depth: usize) -> String {
    let mut source = String::new();
    if derived {
        source.push_str(
            "class Base { method(value) { return value; } static make(value) { return value; } }\n",
        );
    }
    for index in 0..class_count {
        if derived {
            source.push_str(&format!(
                "class C{index} extends Base {{ constructor(value) {{ try {{ if (value) super(value); else super(); return {{ value }}; }} finally {{ value += 1; }} }} method(value) {{ return super.method(value); }} static make(value) {{ return super.make(value); }} }}\n"
            ));
        } else {
            source.push_str(&format!(
                "class C{index} {{ constructor(value) {{ this.value = value; }} method(value) {{ return value; }} static make(value) {{ return new C{index}(value); }} }}\n"
            ));
        }
    }
    if nested_depth > 0 {
        source.push_str("function nested() {\n");
        for depth in 0..nested_depth {
            source.push_str(&format!("class Nested{depth} {{ method() {{\n"));
        }
        source.push_str("return 1;\n");
        for _ in 0..nested_depth {
            source.push_str("} }\n");
        }
        source.push_str("return Nested0;\n}\n");
    }
    source
}

fn class_options() -> TransformerOptions {
    TransformerOptions {
        es2015: ES2015Options {
            classes: true,
            ..ES2015Options::default()
        },
        ..TransformerOptions::default()
    }
}

// ── Semantic analysis benchmarks ──────────────────────────────────────────

fn bench_semantic(c: &mut Criterion, source: &str, label: &str) {
    let mut group = c.benchmark_group(format!("class/{label}/semantic"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("features-empty", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = {
                    let result =
                        Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
                    result.program
                };
                let start = Instant::now();
                black_box(analyze(program, &ast, AnalyzeOptions::default()));
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    group.bench_function("classes-only", |b| {
        b.iter_custom(|iterations| {
            let features = class_options().semantic_features();
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = {
                    let result =
                        Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
                    result.program
                };
                let start = Instant::now();
                black_box(analyze(
                    program,
                    &ast,
                    AnalyzeOptions {
                        features,
                        ..AnalyzeOptions::default()
                    },
                ));
                elapsed += start.elapsed();
            }
            elapsed
        });
    });

    group.finish();
}

// ── Full transform benchmark ──────────────────────────────────────────────

fn bench_transform(c: &mut Criterion, source: &str, label: &str) {
    let mut group = c.benchmark_group(format!("class/{label}/transform"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("full", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let options = class_options();
                let features = options.semantic_features();
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = {
                    let result =
                        Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
                    result.program
                };
                let mut semantic = analyze(
                    program,
                    &ast,
                    AnalyzeOptions {
                        features,
                        ..AnalyzeOptions::default()
                    },
                )
                .semantic;
                let start = Instant::now();
                Transformer::transform(program, &mut ast, &mut semantic, options);
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });

    group.finish();
}

// ── Breakdown: framework-noop vs class vs lowered ─────────────────────────

fn bench_breakdown(c: &mut Criterion, source: &str, label: &str) {
    let mut group = c.benchmark_group(format!("class/{label}/breakdown"));
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("framework-noop/input", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = {
                    let result =
                        Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
                    result.program
                };
                let mut semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
                let start = Instant::now();
                Transformer::transform(
                    program,
                    &mut ast,
                    &mut semantic,
                    TransformerOptions::default(),
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
                let options = class_options();
                let features = options.semantic_features();
                let allocator = Allocator::new();
                let mut ast = Ast::new(source, &allocator);
                let program = {
                    let result =
                        Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
                    result.program
                };
                let mut semantic = analyze(
                    program,
                    &ast,
                    AnalyzeOptions {
                        features,
                        ..AnalyzeOptions::default()
                    },
                )
                .semantic;
                Transformer::transform(program, &mut ast, &mut semantic, options);
                let start = Instant::now();
                Transformer::transform(
                    program,
                    &mut ast,
                    &mut semantic,
                    TransformerOptions::default(),
                );
                elapsed += start.elapsed();
                black_box(program);
            }
            elapsed
        });
    });

    group.finish();
}

// ── Main benchmark runner ─────────────────────────────────────────────────

fn bench_class_transform(c: &mut Criterion) {
    // Base classes: 1, 32, 128
    for &count in &[1usize, 32, 128] {
        let source = class_source(count, false, 0);
        let label = format!("base-{count}");
        eprintln!("fixture: {label}, source={}B", source.len());
        bench_semantic(c, &source, &label);
        bench_transform(c, &source, &label);
        bench_breakdown(c, &source, &label);
    }

    // Derived classes: 32
    {
        let source = class_source(32, true, 0);
        eprintln!("fixture: derived-32, source={}B", source.len());
        bench_semantic(c, &source, "derived-32");
        bench_transform(c, &source, "derived-32");
        bench_breakdown(c, &source, "derived-32");
    }
}

criterion_group!(benches, bench_class_transform);
criterion_main!(benches);