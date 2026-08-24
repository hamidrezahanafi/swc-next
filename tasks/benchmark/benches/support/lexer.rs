use std::hint::black_box;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, GetSpan, JsxAttributeValue, Lang, Visit, VisitWith};
use swc_next_ecma_parser::{
    Lexer, LexerMode, NoTokenParserConfig, Options, Parser, Token, TokenParserConfig, TokenTag,
};

pub struct LexerBenchmark {
    source: String,
    mode: LexerMode,
    token_count: usize,
}

pub fn prepare_normal_strings(source: &str, lang: Lang) -> LexerBenchmark {
    let allocator = Allocator::new();
    let result = Parser::init(
        &allocator,
        source,
        Options {
            lang,
            ..Options::default()
        },
        TokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);

    let mut benchmark = LexerBenchmark {
        source: String::new(),
        mode: LexerMode::Normal,
        token_count: 0,
    };
    for token in &result.tokens {
        if token.tag == TokenTag::StringLiteral {
            benchmark.push_span(source, token.span);
        }
    }
    assert!(benchmark.token_count > 0);
    benchmark
}

pub fn prepare_jsx_attribute_strings(source: &str) -> LexerBenchmark {
    let allocator = Allocator::new();
    let result = Parser::init(
        &allocator,
        source,
        Options {
            lang: Lang::Tsx,
            ..Options::default()
        },
        NoTokenParserConfig,
    )
    .parse();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);

    let mut collector = JsxStringCollector {
        ast: &result.ast,
        source,
        benchmark: LexerBenchmark {
            source: String::new(),
            mode: LexerMode::JsxTag,
            token_count: 0,
        },
    };
    result.ast.root_program().visit_with(&mut collector);
    assert!(collector.benchmark.token_count > 0);
    collector.benchmark
}

pub fn run_lexer_benchmark(input: &LexerBenchmark) -> usize {
    let allocator = Allocator::new();
    let mut lexer = Lexer::init(
        &allocator,
        black_box(input.source.as_str()),
        false,
        false,
        NoTokenParserConfig,
    );
    lexer.set_mode(input.mode);

    let mut token = Token::eof(0);
    let mut token_count = 0;
    loop {
        lexer.next_token(&mut token).unwrap();
        black_box(token);
        if token.tag == TokenTag::Eof {
            break;
        }
        token_count += 1;
    }
    assert_eq!(token_count, input.token_count);
    token_count
}

impl LexerBenchmark {
    fn push_span(&mut self, source: &str, span: swc_next_ecma_ast::Span) {
        self.source
            .push_str(&source[span.start as usize..span.end as usize]);
        self.source.push(' ');
        self.token_count += 1;
    }
}

struct JsxStringCollector<'visit, 'ast> {
    ast: &'visit Ast<'ast>,
    source: &'ast str,
    benchmark: LexerBenchmark,
}

impl<'ast> Visit<'ast> for JsxStringCollector<'_, 'ast> {
    fn ast(&self) -> &Ast<'ast> {
        self.ast
    }

    fn visit_jsx_attribute_value(&mut self, node: JsxAttributeValue) {
        if let Some(string) = node.as_string_literal(self.ast) {
            self.benchmark.push_span(self.source, string.span(self.ast));
        } else {
            node.visit_children_with(self);
        }
    }
}
