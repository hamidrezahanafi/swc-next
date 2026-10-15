#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, BindingIdentifier, Decorator, IdentifierName, IdentifierReference, LabelIdentifier, Lang,
    PrivateIdentifier, Traverse, TraverseCtx, TraverseWith, TsTypeAnnotation, Visit, VisitMut,
    VisitMutWith, VisitWith,
};
use swc_next_ecma_parser::{NoTokenParserConfig, Options, Parser};

fn parse_program<'id, 'a>(
    src: &'a str,
    allocator: &'a Allocator,
    lang: Lang,
    guard: swc_next_ecma_ast::AstGuard<'id>,
) -> Ast<'a, 'id> {
    let result = Parser::init(
        allocator,
        src,
        Options {
            lang,
            ..Options::default()
        },
        NoTokenParserConfig,
        guard,
    )
    .parse();
    assert!(result.diagnostics.is_empty());
    result.ast
}

fn bench_visit(b: &mut Bencher, src: &'static str, lang: Lang) {
    let allocator = Allocator::new();
    swc_next_ecma_ast::make_ast_guard!(ast_guard);
    let ast = parse_program(src, &allocator, lang, ast_guard);
    let program = ast.root_program();

    struct Counter<'a, 'id> {
        ast: &'a Ast<'a, 'id>,
        count: usize,
    }

    impl<'id, 'a> Visit<'a, 'id> for Counter<'a, 'id> {
        fn ast(&self) -> &Ast<'a, 'id> {
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

    b.iter(|| {
        let mut counter = Counter {
            ast: &ast,
            count: 0,
        };
        program.visit_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_visit_mut(b: &mut Bencher, src: &'static str, lang: Lang) {
    struct Counter<'visit, 'ast, 'id> {
        ast: &'visit mut Ast<'ast, 'id>,
        count: usize,
    }

    impl<'id, 'ast> VisitMut<'ast, 'id> for Counter<'_, 'ast, 'id> {
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

    let allocator = Allocator::new();
    swc_next_ecma_ast::make_ast_guard!(ast_guard);
    let mut ast = parse_program(src, &allocator, lang, ast_guard);
    let program = ast.root_program();

    b.iter(|| {
        let mut counter = Counter {
            ast: &mut ast,
            count: 0,
        };
        program.visit_mut_with(&mut counter);
        std::hint::black_box(counter.count);
    });
}

fn bench_traverse(b: &mut Bencher, src: &'static str, lang: Lang) {
    struct Counter {
        count: usize,
    }

    impl<'id, 'ast> Traverse<'ast, 'id> for Counter {
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

    let allocator = Allocator::new();
    swc_next_ecma_ast::make_ast_guard!(ast_guard);
    let ast = parse_program(src, &allocator, lang, ast_guard);
    let program = ast.root_program();

    b.iter(|| {
        let mut counter = Counter { count: 0 };
        program.traverse_with(&mut counter, &mut TraverseCtx::new(&ast, ()));
        std::hint::black_box(counter.count);
    });
}

fn bench_files(c: &mut Criterion) {
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

    for (name, source, lang) in bench_cases {
        c.bench_function(&format!("{name}/visit/new"), |b| {
            bench_visit(b, source, lang)
        });
        c.bench_function(&format!("{name}/visit_mut/new"), |b| {
            bench_visit_mut(b, source, lang)
        });
        c.bench_function(&format!("{name}/traverse/new"), |b| {
            bench_traverse(b, source, lang)
        });
    }
}

criterion_group!(benches, bench_files);
criterion_main!(benches);
