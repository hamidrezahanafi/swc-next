use std::{hint::black_box, time::Instant};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Lang, ffi::ast::buffer_size};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticFeatureFlags, analyze};

const SOURCE: &str = include_str!("../files/class-heavy.js");
const ITERATIONS: usize = 200;

fn parse<'a>(allocator: &'a Allocator) -> swc_next_ecma_ast::Ast<'a> {
    let result = Parser::init(
        allocator,
        SOURCE,
        Options {
            lang: Lang::Js,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    result.ast
}

fn main() {
    let parse_start = Instant::now();
    for _ in 0..ITERATIONS {
        let allocator = Allocator::new();
        let ast = parse(&allocator);
        black_box(ast.node_count());
    }
    let parse_ns = parse_start.elapsed().as_nanos() / ITERATIONS as u128;

    let mut semantic_ns = 0u128;
    for _ in 0..ITERATIONS {
        let allocator = Allocator::new();
        let ast = parse(&allocator);
        let start = Instant::now();
        black_box(analyze(
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::CLASS,
                ..AnalyzeOptions::default()
            },
        ));
        semantic_ns += start.elapsed().as_nanos();
    }
    semantic_ns /= ITERATIONS as u128;

    let allocator = Allocator::new();
    let ast = parse(&allocator);
    let parse_allocator_bytes = allocator.allocated_bytes();
    let semantic = analyze(
        &ast,
        AnalyzeOptions {
            features: SemanticFeatureFlags::CLASS,
            ..AnalyzeOptions::default()
        },
    );
    black_box(&semantic);

    println!("source_bytes={}", SOURCE.len());
    println!("common_nodes={}", ast.node_count());
    println!("classes={}", ast.class_count());
    println!("node_edges={}", ast.node_edge_count());
    println!("residual_extra={}", ast.extra_data_count());
    println!("parse_allocator_bytes={parse_allocator_bytes}");
    println!("semantic_allocator_bytes={}", allocator.allocated_bytes());
    println!("ffi_buffer_bytes={}", buffer_size(&ast));
    println!("parse_ns_per_iter={parse_ns}");
    println!("semantic_ns_per_iter={semantic_ns}");
}
