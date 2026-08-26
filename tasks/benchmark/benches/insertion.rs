#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{hint::black_box, time::Instant};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, IdentifierReference, Visit, VisitWith};

#[path = "support/insertion.rs"]
#[allow(dead_code)]
mod insertion_support;

fn count_identifier_references(ast: &Ast<'_>) -> usize {
    struct Counter<'ast> {
        ast: &'ast Ast<'ast>,
        count: usize,
    }

    impl<'ast> Visit<'ast> for Counter<'ast> {
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

fn bench_insertion(c: &mut Criterion) {
    let policy_ast = insertion_support::setup_program();
    let mut group = c.benchmark_group(format!(
        "transform_insertion/{}",
        insertion_support::policy_label(&policy_ast)
    ));
    group.bench_function("hole_boundary/4", |b| {
        b.iter_custom(|iterations| {
            let mut elapsed = std::time::Duration::ZERO;
            for _ in 0..iterations {
                let mut ast = insertion_support::setup_hole_boundary_ast();
                let start = Instant::now();
                black_box(insertion_support::build_detached_statements(&mut ast, 4));
                elapsed += start.elapsed();
                black_box(insertion_support::insertion_metrics(&ast));
            }
            elapsed
        });
    });
    for count in [64usize, 512] {
        group.bench_with_input(BenchmarkId::new("detached", count), &count, |b, &count| {
            b.iter_custom(|iterations| {
                let mut elapsed = std::time::Duration::ZERO;
                for _ in 0..iterations {
                    let allocator = Allocator::new();
                    let mut ast = insertion_support::parse_program(&allocator);
                    let start = Instant::now();
                    black_box(insertion_support::build_detached_statements(
                        &mut ast, count,
                    ));
                    elapsed += start.elapsed();
                    black_box(insertion_support::insertion_metrics(&ast));
                }
                elapsed
            });
        });

        group.bench_with_input(
            BenchmarkId::new("program_body", count),
            &count,
            |b, &count| {
                b.iter_custom(|iterations| {
                    let mut elapsed = std::time::Duration::ZERO;
                    for _ in 0..iterations {
                        let allocator = Allocator::new();
                        let mut ast = insertion_support::parse_program(&allocator);
                        let start = Instant::now();
                        black_box(insertion_support::insert_program_statements(
                            &mut ast, count,
                        ));
                        elapsed += start.elapsed();
                        black_box(insertion_support::insertion_metrics(&ast));
                    }
                    elapsed
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("post_insert_visit", count),
            &count,
            |b, &count| {
                b.iter_custom(|iterations| {
                    let mut elapsed = std::time::Duration::ZERO;
                    for _ in 0..iterations {
                        let allocator = Allocator::new();
                        let mut ast = insertion_support::parse_program(&allocator);
                        insertion_support::insert_program_statements(&mut ast, count);
                        let start = Instant::now();
                        black_box(count_identifier_references(&ast));
                        elapsed += start.elapsed();
                    }
                    elapsed
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_insertion);
criterion_main!(benches);
