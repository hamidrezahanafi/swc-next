#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use gungraun::prelude::*;
use gungraun::{Callgrind, EventKind};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, Decorator, IdentifierName, IdentifierReference, LabelIdentifier, Lang,
    PrivateIdentifier, SourceType, Traverse, TraverseCtx, TraverseWith, TsTypeAnnotation, Visit,
    VisitMut, VisitMutWith, VisitWith,
};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{CommentMode, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{
    AnalyzeOptions, Semantic, analyze, name_resolver::resolver as resolve_names,
};
use swc_next_ecma_transformer::{
    Transformer, TransformerOptions, es2022::ES2022Options, normalize::Normalizer,
    proposal::ProposalOptions, typescript::TypeScriptOptions,
};
use swc_next_type_strip::strip_types;

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
const ASTRO_CONFIG: &str = include_str!("../files/astro-config.ts");
const EFFECT_SCHEMA_AST: &str = include_str!("../files/effect-schema-ast.ts");
const HONO_TYPES: &str = include_str!("../files/hono-types.ts");
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
#[bench::hono_types(HONO_TYPES, Lang::Ts)]
fn bench_parser(source: &str, lang: Lang) {
    let source = black_box(source);
    let allocator = Allocator::new();
    swc_next_ecma_ast::make_ast_guard!(ast_guard);
    black_box(
        Parser::init(
            &allocator,
            source,
            Options {
                lang,
                ..Options::default()
            },
            NoTokenParserConfig,
            ast_guard,
        )
        .parse(),
    );
}

// Benchmark setup must return a concrete type. Keep the erased brand private
// and introduce a fresh brand whenever the fixture is accessed. Handles cannot
// escape the higher-ranked callback, so fixtures cannot exchange strings.
struct ProgramFixture(Ast<'static, 'static>);

impl ProgramFixture {
    fn new<'id>(ast: Ast<'static, 'id>) -> Self {
        // SAFETY: Only the phantom pool brand changes. The erased AST remains
        // private and is only exposed through a freshly branded callback.
        Self(unsafe { std::mem::transmute::<Ast<'static, 'id>, Ast<'static, 'static>>(ast) })
    }

    fn with<R>(self, f: impl for<'id> FnOnce(Ast<'id, 'id>) -> R) -> R {
        fn bind<'id, R>(
            fixture: ProgramFixture,
            _guard: swc_next_ecma_ast::AstGuard<'id>,
            f: impl FnOnce(Ast<'id, 'id>) -> R,
        ) -> R {
            // SAFETY: The callback receives a fresh invariant brand for this
            // pool, and its result cannot contain any handles with that brand.
            f(unsafe { std::mem::transmute::<Ast<'static, 'static>, Ast<'id, 'id>>(fixture.0) })
        }
        swc_next_ecma_ast::make_ast_guard!(guard);
        bind(self, guard, f)
    }
}

fn setup_program(source: &'static str, lang: Lang) -> ProgramFixture {
    let allocator = Box::leak(Box::new(Allocator::new()));
    swc_next_ecma_ast::make_ast_guard!(guard);
    let result = Parser::init(
        allocator,
        source,
        Options {
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
        guard,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    ProgramFixture::new(result.ast)
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_analyze(fixture: ProgramFixture) {
    fixture.with(|ast| {
        black_box(analyze(&ast, AnalyzeOptions::default()));
    });
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_syntax_check(fixture: ProgramFixture) {
    fixture.with(|ast| {
        black_box(analyze(
            &ast,
            AnalyzeOptions {
                check_syntax: true,
                ..AnalyzeOptions::default()
            },
        ));
    });
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
fn bench_resolver(fixture: ProgramFixture) {
    fixture.with(|ast| {
        black_box(resolve_names(&ast));
    });
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_codegen(fixture: ProgramFixture) {
    fixture.with(|ast| {
        black_box(print(ast.allocator(), &ast, CodegenOptions::default()));
    });
}

type TransformData<'id> = (Ast<'id, 'id>, Semantic<'id, 'id>, TransformerOptions);
struct TransformInput(TransformData<'static>);
impl TransformInput {
    fn with<R>(self, f: impl for<'id> FnOnce(TransformData<'id>) -> R) -> R {
        fn bind<'id, R>(
            input: TransformInput,
            _guard: swc_next_ecma_ast::AstGuard<'id>,
            f: impl FnOnce(TransformData<'id>) -> R,
        ) -> R {
            // SAFETY: Setup packages an AST and its own semantic model. Only
            // their phantom brand changes, and the callback cannot export it.
            f(
                unsafe {
                    std::mem::transmute::<TransformData<'static>, TransformData<'id>>(input.0)
                },
            )
        }
        swc_next_ecma_ast::make_ast_guard!(guard);
        bind(self, guard, f)
    }
}
fn setup_transform(source: &'static str, lang: Lang) -> TransformInput {
    setup_program(source, lang).with(|mut ast| {
        Normalizer::normalize_runtime(ast.root_program(), &mut ast);
        let options = TransformerOptions {
            typescript: Some(TypeScriptOptions::default()),
            es2022: ES2022Options {
                static_blocks: true,
            },
            proposal: ProposalOptions {
                prevent_extensions: true,
            },
            ..TransformerOptions::default()
        };
        let semantic = analyze(&ast, AnalyzeOptions::default()).semantic;
        // SAFETY: Both models refer to this pool. The erased pair remains private
        // and is only accessed by TransformInput::with with one fresh brand.
        TransformInput(unsafe {
            std::mem::transmute::<TransformData<'_>, TransformData<'static>>((
                ast, semantic, options,
            ))
        })
    })
}

#[library_benchmark]
#[bench::angular(setup_transform(ANGULAR, Lang::Js))]
#[bench::backbone(setup_transform(BACKBONE, Lang::Js))]
#[bench::jquery(setup_transform(JQUERY, Lang::Js))]
#[bench::jquery_mobile(setup_transform(JQUERY_MOBILE, Lang::Js))]
#[bench::mootools(setup_transform(MOOTOOLS, Lang::Js))]
#[bench::underscore(setup_transform(UNDERSCORE, Lang::Js))]
#[bench::three(setup_transform(THREE, Lang::Js))]
#[bench::yui(setup_transform(YUI, Lang::Js))]
#[bench::typescript(setup_transform(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_transform(CAL_COM_TSX, Lang::Tsx))]
#[bench::astro_config(setup_transform(ASTRO_CONFIG, Lang::Ts))]
#[bench::effect_schema_ast(setup_transform(EFFECT_SCHEMA_AST, Lang::Ts))]
#[bench::hono_types(setup_transform(HONO_TYPES, Lang::Ts))]
fn bench_transform(input: TransformInput) {
    black_box(input).with(|(mut ast, semantic, options)| {
        let result = Transformer::transform(&mut ast, semantic, options);
        black_box((ast, result));
    });
}

fn type_strip_options(lang: Lang) -> Options {
    Options {
        source_type: SourceType::Module,
        lang,
        preserve_parens: true,
        comments: CommentMode::None,
    }
}

#[library_benchmark(
    config = LibraryBenchmarkConfig::default().tool(
        Callgrind::with_args(["--dump-before=*benchmark_parse_boundary"]),
    )
)]
#[bench::astro_config(ASTRO_CONFIG, type_strip_options(Lang::Ts))]
#[bench::effect_schema_ast(EFFECT_SCHEMA_AST, type_strip_options(Lang::Ts))]
#[bench::hono_types(HONO_TYPES, type_strip_options(Lang::Ts))]
fn bench_type_strip(source: &str, options: Options) {
    black_box(strip_types(black_box(source), black_box(options)));
}

struct VisitCounter<'visit, 'ast, 'id> {
    ast: &'visit Ast<'ast, 'id>,
    count: usize,
}

impl<'id, 'ast> Visit<'ast, 'id> for VisitCounter<'_, 'ast, 'id> {
    fn ast(&self) -> &Ast<'ast, 'id> {
        self.ast
    }

    fn visit_identifier_reference(&mut self, _node: IdentifierReference<'id>) {
        self.count += 1;
    }

    fn visit_private_identifier(&mut self, _node: PrivateIdentifier<'id>) {
        self.count += 1;
    }

    fn visit_binding_identifier(&mut self, _node: BindingIdentifier<'id>) {
        self.count += 1;
    }

    fn visit_identifier_name(&mut self, _node: IdentifierName<'id>) {
        self.count += 1;
    }

    fn visit_label_identifier(&mut self, _node: LabelIdentifier<'id>) {
        self.count += 1;
    }
}

struct VisitMutCounter<'visit, 'ast, 'id> {
    ast: &'visit mut Ast<'ast, 'id>,
    count: usize,
}

impl<'id, 'ast> VisitMut<'ast, 'id> for VisitMutCounter<'_, 'ast, 'id> {
    fn ast(&mut self) -> &mut Ast<'ast, 'id> {
        self.ast
    }

    fn visit_mut_identifier_reference(
        &mut self,
        _node: IdentifierReference<'id>,
    ) -> Option<IdentifierReference<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_private_identifier(
        &mut self,
        _node: PrivateIdentifier<'id>,
    ) -> Option<PrivateIdentifier<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_binding_identifier(
        &mut self,
        _node: BindingIdentifier<'id>,
    ) -> Option<BindingIdentifier<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_identifier_name(
        &mut self,
        _node: IdentifierName<'id>,
    ) -> Option<IdentifierName<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_label_identifier(
        &mut self,
        _node: LabelIdentifier<'id>,
    ) -> Option<LabelIdentifier<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_ts_type_annotation(
        &mut self,
        _node: TsTypeAnnotation<'id>,
    ) -> Option<TsTypeAnnotation<'id>> {
        self.count += 1;
        None
    }

    fn visit_mut_decorator(&mut self, _node: Decorator<'id>) -> Option<Decorator<'id>> {
        self.count += 1;
        None
    }
}

struct TraverseCounter {
    count: usize,
}

impl<'id, 'ast> Traverse<'ast, 'id> for TraverseCounter {
    fn enter_identifier_reference(
        &mut self,
        _node: IdentifierReference<'id>,
        _ctx: &mut TraverseCtx<'_, 'ast, 'id>,
    ) {
        self.count += 1;
    }

    fn enter_private_identifier(
        &mut self,
        _node: PrivateIdentifier<'id>,
        _ctx: &mut TraverseCtx<'_, 'ast, 'id>,
    ) {
        self.count += 1;
    }

    fn enter_binding_identifier(
        &mut self,
        _node: BindingIdentifier<'id>,
        _ctx: &mut TraverseCtx<'_, 'ast, 'id>,
    ) {
        self.count += 1;
    }

    fn enter_identifier_name(
        &mut self,
        _node: IdentifierName<'id>,
        _ctx: &mut TraverseCtx<'_, 'ast, 'id>,
    ) {
        self.count += 1;
    }

    fn enter_label_identifier(
        &mut self,
        _node: LabelIdentifier<'id>,
        _ctx: &mut TraverseCtx<'_, 'ast, 'id>,
    ) {
        self.count += 1;
    }
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit(fixture: ProgramFixture) {
    fixture.with(|ast| {
        let program = ast.root_program();
        let mut counter = VisitCounter {
            ast: &ast,
            count: 0,
        };
        program.visit_with(&mut counter);
        black_box(counter.count);
    });
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_visit_mut(fixture: ProgramFixture) {
    fixture.with(|mut ast| {
        let program = ast.root_program();
        let mut counter = VisitMutCounter {
            ast: &mut ast,
            count: 0,
        };
        program.visit_mut_with(&mut counter);
        black_box(counter.count);
    });
}

#[library_benchmark]
#[bench::typescript(setup_program(TYPESCRIPT, Lang::Js))]
#[bench::cal_com_tsx(setup_program(CAL_COM_TSX, Lang::Tsx))]
fn bench_traverse(fixture: ProgramFixture) {
    fixture.with(|ast| {
        let program = ast.root_program();
        let mut counter = TraverseCounter { count: 0 };
        program.traverse_with(&mut counter, &mut TraverseCtx::new(&ast, ()));
        black_box(counter.count);
    });
}

library_benchmark_group!(name = parser, benchmarks = bench_parser);

library_benchmark_group!(
    name = semantic,
    benchmarks = [bench_analyze, bench_syntax_check]
);

library_benchmark_group!(name = resolver, benchmarks = bench_resolver);

library_benchmark_group!(name = codegen, benchmarks = bench_codegen);

library_benchmark_group!(name = transform, benchmarks = bench_transform);

library_benchmark_group!(name = type_strip, benchmarks = bench_type_strip);

library_benchmark_group!(
    name = visit,
    benchmarks = [bench_visit, bench_visit_mut, bench_traverse]
);

main!(
    config =
        LibraryBenchmarkConfig::default().tool(
            Callgrind::with_args(["--branch-sim=no", "--cache-sim=no",]).format([EventKind::Ir]),
        ),
    library_benchmark_groups = [
        parser, semantic, resolver, codegen, transform, type_strip, visit
    ]
);
