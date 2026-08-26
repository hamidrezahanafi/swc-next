#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};
use swc_next_ecma_ast::Ast;

#[path = "support/static_blocks.rs"]
mod static_blocks_support;

use static_blocks_support::{CLASS_HEAVY, SINGLE_CLASS, TransformCase};

#[library_benchmark]
#[bench::single_class(static_blocks_support::setup_case(SINGLE_CLASS))]
#[bench::class_heavy(static_blocks_support::setup_case(CLASS_HEAVY))]
fn bench_transform(mut case: TransformCase) {
    static_blocks_support::transform_case(&mut case);
    black_box(case);
}

#[library_benchmark]
#[bench::single_class(static_blocks_support::setup_transformed(SINGLE_CLASS))]
#[bench::class_heavy(static_blocks_support::setup_transformed(CLASS_HEAVY))]
fn bench_post_transform_visit(ast: Ast<'static>) {
    black_box(static_blocks_support::count_identifier_references(&ast));
}

#[library_benchmark]
#[bench::single_class(static_blocks_support::setup_transformed(SINGLE_CLASS))]
#[bench::class_heavy(static_blocks_support::setup_transformed(CLASS_HEAVY))]
fn bench_post_transform_codegen(ast: Ast<'static>) {
    black_box(static_blocks_support::codegen_len(&ast));
}

#[library_benchmark]
#[bench::single_class(SINGLE_CLASS)]
#[bench::class_heavy(CLASS_HEAVY)]
fn bench_end_to_end(source: &'static str) {
    black_box(static_blocks_support::end_to_end(source));
}

library_benchmark_group!(name = transform, benchmarks = bench_transform);

library_benchmark_group!(
    name = locality,
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::with_args(["--branch-sim=yes", "--cache-sim=yes"]).format([
            EventKind::Ir,
            EventKind::D1mr,
            EventKind::DLmr,
            EventKind::Bc,
            EventKind::Bcm,
        ]),
    ),
    benchmarks = [bench_post_transform_visit, bench_post_transform_codegen]
);

library_benchmark_group!(name = end_to_end, benchmarks = bench_end_to_end);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::with_args(["--branch-sim=no", "--cache-sim=no"]).format([EventKind::Ir]),),
    library_benchmark_groups = [transform, locality, end_to_end]
);
