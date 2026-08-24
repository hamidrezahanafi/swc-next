use std::{
    env,
    fmt::Write,
    hint::black_box,
    mem::{align_of, size_of},
};

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, AstId, Class, ExtraData, Lang, NodeKind, NodePayload, Span, ffi::ast::buffer_size,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticFeatureFlags, analyze};

const DEFAULT_OUTER_CLASS_COUNTS: &[usize] = &[1, 10, 25, 50, 100, 200, 400, 800];

fn build_class_fixture(outer_class_count: usize) -> String {
    let mut source = String::with_capacity(outer_class_count * 192);
    for index in 0..outer_class_count {
        writeln!(
            source,
            "class C{index} extends base({index}) {{\n  #x{index};\n  a = {index};\n  b;\n  c() {{ return this.#x{index}; }}\n  static {{ register({index}); }}\n  method() {{ return class {{ #y{index}; value() {{ return this.#y{index}; }} }}; }}\n}}\n"
        )
        .unwrap();
    }
    source.pop();
    source
}

fn estimated_node_capacity(source_len: usize) -> usize {
    let baseline_nodes = if source_len < 4_000 {
        256.max(source_len / 4)
    } else if source_len < 10_000 {
        source_len / 3
    } else if source_len < 100_000 {
        source_len / 6
    } else {
        source_len / 8
    };
    baseline_nodes + baseline_nodes / 2
}

fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
    let result = Parser::init(
        allocator,
        source,
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
    let requested_counts = env::args()
        .skip(1)
        .map(|value| {
            value
                .parse::<usize>()
                .expect("fixture size must be a non-negative integer")
        })
        .collect::<Vec<_>>();
    let outer_class_counts = if requested_counts.is_empty() {
        DEFAULT_OUTER_CLASS_COUNTS.to_vec()
    } else {
        requested_counts
    };

    println!(
        "layout_bytes: span={} kind={} payload={} ast_id={} extra={} class_hot={} class_hot_align={} class_optional={}",
        size_of::<Span>(),
        size_of::<NodeKind>(),
        size_of::<NodePayload>(),
        size_of::<AstId>(),
        size_of::<ExtraData>(),
        size_of::<Class>(),
        align_of::<Class>(),
        Ast::class_optional_record_size(),
    );
    println!(
        "outer_classes\tsource_bytes\testimated_nodes\tclasses\tcommon_nodes\tnode_edges\tresidual_extra\tlogical_ast_bytes\tparse_allocator_bytes\tsemantic_allocator_bytes\tffi_buffer_bytes"
    );

    for outer_class_count in outer_class_counts {
        let source = build_class_fixture(outer_class_count);
        let allocator = Allocator::new();
        let ast = parse(&allocator, &source);
        let parse_allocator_bytes = allocator.allocated_bytes();
        let logical_ast_bytes = ast.node_count()
            * (size_of::<Span>() + size_of::<NodeKind>() + size_of::<NodePayload>())
            + ast.class_storage_bytes()
            + ast.node_edge_count() * size_of::<AstId>()
            + ast.extra_data_count() * size_of::<ExtraData>();
        let ffi_buffer_bytes = buffer_size(&ast);
        let semantic = analyze(
            &ast,
            AnalyzeOptions {
                features: SemanticFeatureFlags::CLASS,
                ..AnalyzeOptions::default()
            },
        );
        black_box(&semantic);

        println!(
            "{outer_class_count}\t{}\t{}\t{}\t{}\t{}\t{}\t{logical_ast_bytes}\t{parse_allocator_bytes}\t{}\t{ffi_buffer_bytes}",
            source.len(),
            estimated_node_capacity(source.len()),
            ast.class_count(),
            ast.node_count(),
            ast.node_edge_count(),
            ast.extra_data_count(),
            allocator.allocated_bytes(),
        );
    }
}
