#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, IdentifierName, IdentifierReference, LabelIdentifier, Lang, NodeId,
    PrivateIdentifier, Program, Traverse, TraverseAction, TraverseCtx, TraverseWith, Visit,
    VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, Semantic, analyze};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions,
    mutation::{IdentityPolicy, MutationTransaction},
};

const ANGULAR: &str = include_str!("../files/angular-1.2.5.js");
const BACKBONE: &str = include_str!("../files/backbone-1.1.0.js");
const JQUERY: &str = include_str!("../files/jquery-1.9.1.js");
const JQUERY_MOBILE: &str = include_str!("../files/jquery.mobile-1.4.2.js");
const MOOTOOLS: &str = include_str!("../files/mootools-1.4.5.js");
const UNDERSCORE: &str = include_str!("../files/underscore-1.5.2.js");
const THREE: &str = include_str!("../files/three-0.138.3.js");
const YUI: &str = include_str!("../files/yui-3.12.0.js");
const TYPESCRIPT: &str = include_str!("../files/typescript.js");
const CAL_COM_TSX: &str = include_str!("../files/cal.com.tsx");

#[library_benchmark]
#[bench::angular(ANGULAR, Lang::Js)]
#[bench::backbone(BACKBONE, Lang::Js)]
#[bench::jquery(JQUERY, Lang::Js)]
#[bench::jquery_mobile(JQUERY_MOBILE, Lang::Js)]
#[bench::mootools(MOOTOOLS, Lang::Js)]
#[bench::underscore(UNDERSCORE, Lang::Js)]
#[bench::three(THREE, Lang::Js)]
#[bench::yui(YUI, Lang::Js)]
#[bench::typescript(TYPESCRIPT, Lang::Js)]
#[bench::cal_com_tsx(CAL_COM_TSX, Lang::Tsx)]
fn bench_parser(source: &str, lang: Lang) {
    let source = black_box(source);
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    black_box(
        Parser::init(
            &mut ast,
            Options {
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
        )
        .parse(),
    );
}

struct ParsedProgram {
    ast: Ast<'static>,
    program: Program,
}

fn setup_program(source: &'static str, lang: Lang) -> ParsedProgram {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let mut ast = Ast::new(source, allocator);
    let result = Parser::init(
        &mut ast,
        Options {
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    let program = result.program;
    ParsedProgram { ast, program }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions::default(),
    ));
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_semantic_with_syntax_check(parsed: ParsedProgram) {
    black_box(analyze(
        parsed.program,
        &parsed.ast,
        AnalyzeOptions {
            check_syntax: true,
            ..AnalyzeOptions::default()
        },
    ));
}

struct ClassTransformProgram {
    ast: Ast<'static>,
    program: Program,
    semantic: Semantic<'static>,
    options: TransformerOptions,
}

fn class_source(class_count: usize, derived: bool, nested_depth: usize) -> &'static str {
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
    Box::leak(source.into_boxed_str())
}

fn setup_class_transform(
    class_count: usize,
    derived: bool,
    nested_depth: usize,
) -> ClassTransformProgram {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let source = class_source(class_count, derived, nested_depth);
    let mut ast = Ast::new(source, allocator);
    let result = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let program = result.program;
    let mut options = TransformerOptions::default();
    options.es2015.classes = true;
    let semantic = analyze(
        program,
        &ast,
        AnalyzeOptions {
            features: options.semantic_features(),
            ..AnalyzeOptions::default()
        },
    )
    .semantic;
    ClassTransformProgram {
        ast,
        program,
        semantic,
        options,
    }
}

#[library_benchmark]
#[bench::base_1(setup_class_transform(1, false, 0))]
#[bench::base_32(setup_class_transform(32, false, 0))]
#[bench::derived_32(setup_class_transform(32, true, 0))]
#[bench::nested_8(setup_class_transform(0, false, 8))]
fn bench_class_transform(mut parsed: ClassTransformProgram) {
    Transformer::transform(
        parsed.program,
        &mut parsed.ast,
        &mut parsed.semantic,
        parsed.options,
    );
    black_box(parsed.ast.mutation_epoch());
    black_box(parsed.semantic.mutation_epoch());
}

struct LeafMutationProgram {
    ast: Ast<'static>,
    program: Program,
    semantic: Semantic<'static>,
    target: NodeId,
}

fn setup_leaf_mutation(sibling_count: usize) -> LeafMutationProgram {
    let allocator = Box::leak(Box::new(Allocator::new()));
    let source = Box::leak(";".repeat(sibling_count).into_boxed_str());
    let mut ast = Ast::new(source, allocator);
    let result = Parser::init(&mut ast, Options::default(), NoTokenParserConfig).parse();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let program = result.program;
    let target = program.body(&ast).get_node(&ast, 0).unwrap().node_id();
    let semantic = analyze(program, &ast, AnalyzeOptions::default()).semantic;
    LeafMutationProgram {
        ast,
        program,
        semantic,
        target,
    }
}

// Replacing one leaf should cost the same order of work regardless of how
// many unrelated sibling statements are present in the program.
#[library_benchmark]
#[bench::one_sibling(setup_leaf_mutation(1))]
#[bench::four_thousand_siblings(setup_leaf_mutation(4096))]
fn bench_leaf_mutation(mut parsed: LeafMutationProgram) {
    let mut transaction = MutationTransaction::begin(
        &mut parsed.ast,
        &mut parsed.semantic,
        parsed.program.node_id(),
    )
    .unwrap();
    let replacement = transaction.build_detached(|ast| ast.empty_statement(Default::default()));
    transaction.replace(
        parsed.target,
        replacement.node_id(),
        IdentityPolicy::Preserve,
    );
    black_box(transaction.commit().unwrap());
}

struct VisitCounter<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    count: usize,
}

impl<'ast> Visit<'ast> for VisitCounter<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, _node: IdentifierReference) {
        self.count += 1;
    }

    fn visit_private_identifier(&mut self, _node: PrivateIdentifier) {
        self.count += 1;
    }

    fn visit_binding_identifier(&mut self, _node: BindingIdentifier) {
        self.count += 1;
    }

    fn visit_identifier_name(&mut self, _node: IdentifierName) {
        self.count += 1;
    }

    fn visit_label_identifier(&mut self, _node: LabelIdentifier) {
        self.count += 1;
    }
}

struct VisitMutCounter<'visit, 'ast> {
    ast: &'visit mut Ast<'ast>,
    count: usize,
}

impl<'ast> VisitMut<'ast> for VisitMutCounter<'_, 'ast> {
    fn ast(&mut self) -> &mut Ast<'ast> {
        self.ast
    }

    fn visit_mut_identifier_reference(
        &mut self,
        _node: IdentifierReference,
    ) -> Option<IdentifierReference> {
        self.count += 1;
        None
    }

    fn visit_mut_private_identifier(
        &mut self,
        _node: PrivateIdentifier,
    ) -> Option<PrivateIdentifier> {
        self.count += 1;
        None
    }

    fn visit_mut_binding_identifier(
        &mut self,
        _node: BindingIdentifier,
    ) -> Option<BindingIdentifier> {
        self.count += 1;
        None
    }

    fn visit_mut_identifier_name(&mut self, _node: IdentifierName) -> Option<IdentifierName> {
        self.count += 1;
        None
    }

    fn visit_mut_label_identifier(&mut self, _node: LabelIdentifier) -> Option<LabelIdentifier> {
        self.count += 1;
        None
    }
}

struct TraverseCounter {
    count: usize,
}

impl<'ast> Traverse<'ast> for TraverseCounter {
    fn enter_identifier_reference(
        &mut self,
        _node: IdentifierReference,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<IdentifierReference> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_private_identifier(
        &mut self,
        _node: PrivateIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<PrivateIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_binding_identifier(
        &mut self,
        _node: BindingIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<BindingIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_identifier_name(
        &mut self,
        _node: IdentifierName,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<IdentifierName> {
        self.count += 1;
        TraverseAction::Proceed
    }

    fn enter_label_identifier(
        &mut self,
        _node: LabelIdentifier,
        _ctx: &mut TraverseCtx<'_, 'ast>,
    ) -> TraverseAction<LabelIdentifier> {
        self.count += 1;
        TraverseAction::Proceed
    }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit(parsed: ParsedProgram) {
    let mut counter = VisitCounter {
        ast: &parsed.ast,
        count: 0,
    };
    parsed.program.visit_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit_mut(mut parsed: ParsedProgram) {
    let mut counter = VisitMutCounter {
        ast: &mut parsed.ast,
        count: 0,
    };
    parsed.program.visit_mut_with(&mut counter);
    black_box(counter.count);
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_traverse(mut parsed: ParsedProgram) {
    let mut counter = TraverseCounter { count: 0 };
    let _ = parsed
        .program
        .traverse_with(&mut counter, &mut TraverseCtx::new(&mut parsed.ast, ()));
    black_box(counter.count);
}

library_benchmark_group!(name = parser, benchmarks = bench_parser);

library_benchmark_group!(
    name = semantic,
    benchmarks = [bench_semantic, bench_semantic_with_syntax_check]
);

library_benchmark_group!(
    name = visit,
    benchmarks = [bench_visit, bench_visit_mut, bench_traverse]
);

library_benchmark_group!(
    name = transform,
    benchmarks = [bench_class_transform, bench_leaf_mutation]
);

main!(
    config =
        LibraryBenchmarkConfig::default().tool(
            Callgrind::with_args(["--branch-sim=no", "--cache-sim=no",]).format([EventKind::Ir]),
        ),
    library_benchmark_groups = [parser, semantic, visit, transform]
);
