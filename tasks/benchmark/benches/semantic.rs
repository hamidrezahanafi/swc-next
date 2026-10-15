#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, AstGuard, Lang};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, analyze};

// Criterion's setup callback must return one concrete type. The erased pool
// brand stays private; analysis creates a fresh brand before accessing strings.
struct SemanticInput<'a>(Ast<'a, 'static>);
impl<'a> SemanticInput<'a> {
    fn new<'id>(ast: Ast<'a, 'id>) -> Self {
        // SAFETY: Only the phantom pool brand changes. No safe method exposes
        // the erased AST or accepts a handle from another pool.
        Self(unsafe { std::mem::transmute::<Ast<'a, 'id>, Ast<'a, 'static>>(ast) })
    }
    fn analyze(self) {
        fn bind<'a, 'id>(input: SemanticInput<'a>, _guard: AstGuard<'id>) {
            // SAFETY: Analysis receives one fresh invariant brand for this pool;
            // its handles and semantic model are dropped before returning.
            let ast = unsafe { std::mem::transmute::<Ast<'a, 'static>, Ast<'a, 'id>>(input.0) };
            black_box(analyze(&ast, AnalyzeOptions::default()));
        }
        swc_next_ecma_ast::make_ast_guard!(guard);
        bind(self, guard);
    }
}

fn bench_semantic(c: &mut Criterion) {
    let bench_cases = [
        (
            "typescript",
            include_str!("../files/typescript.js"),
            Lang::Js,
        ),
        (
            "cal.com.tsx",
            include_str!("../files/cal.com.tsx"),
            Lang::Tsx,
        ),
    ];
    let mut group = c.benchmark_group("semantic");
    for (name, source, lang) in bench_cases {
        group.bench_function(format!("{name}/semantic/legacy"), |b| {
            let allocator = Allocator::new();
            b.iter_batched(
                || {
                    swc_next_ecma_ast::make_ast_guard!(ast_guard);
                    let result = Parser::init(
                        &allocator,
                        source,
                        Options {
                            lang,
                            ..Options::default()
                        },
                        NoTokenParserConfig,
                        ast_guard,
                    )
                    .parse();
                    assert!(result.diagnostics.is_empty());
                    SemanticInput::new(result.ast)
                },
                SemanticInput::analyze,
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
