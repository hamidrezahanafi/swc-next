#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{hint::black_box, time::Instant};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[path = "support/static_blocks.rs"]
mod static_blocks_support;

use static_blocks_support::{CLASS_HEAVY, SINGLE_CLASS};

fn bench_static_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_blocks");
    for (name, source) in [("single_class", SINGLE_CLASS), ("class_heavy", CLASS_HEAVY)] {
        group.bench_with_input(
            BenchmarkId::new("transform", name),
            &source,
            |b, &source| {
                b.iter_custom(|iterations| {
                    let mut elapsed = std::time::Duration::ZERO;
                    for _ in 0..iterations {
                        let mut case = static_blocks_support::setup_case(source);
                        let start = Instant::now();
                        static_blocks_support::transform_case(&mut case);
                        elapsed += start.elapsed();
                        black_box(case);
                    }
                    elapsed
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("post_transform_visit", name),
            &source,
            |b, &source| {
                b.iter_custom(|iterations| {
                    let mut elapsed = std::time::Duration::ZERO;
                    for _ in 0..iterations {
                        let ast = static_blocks_support::setup_transformed(source);
                        let start = Instant::now();
                        black_box(static_blocks_support::count_identifier_references(&ast));
                        elapsed += start.elapsed();
                    }
                    elapsed
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("post_transform_codegen", name),
            &source,
            |b, &source| {
                b.iter_custom(|iterations| {
                    let mut elapsed = std::time::Duration::ZERO;
                    for _ in 0..iterations {
                        let ast = static_blocks_support::setup_transformed(source);
                        let start = Instant::now();
                        black_box(static_blocks_support::codegen_len(&ast));
                        elapsed += start.elapsed();
                    }
                    elapsed
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("end_to_end", name),
            &source,
            |b, &source| {
                b.iter(|| black_box(static_blocks_support::end_to_end(source)));
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_static_blocks);
criterion_main!(benches);
