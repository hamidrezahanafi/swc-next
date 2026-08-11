mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::{Parser, ParserConfig};

pub use self::options::{CommentMode, Lang, ParserOptions, SourceType};

#[derive(Clone, Copy)]
struct EstreeParserConfig;

impl ParserConfig for EstreeParserConfig {
    fn tokens(&self) -> bool {
        false
    }

    fn preserve_estree_assignment_targets(&self) -> bool {
        true
    }
}

pub fn parse_to_buffer(source: &str, options: Option<ParserOptions>) -> Vec<u8> {
    let allocator = Allocator::default();
    let mut ast = Ast::new(source, &allocator);
    let program = Parser::init(
        &mut ast,
        options.unwrap_or_default().into(),
        EstreeParserConfig,
    )
    .parse()
    .program;

    let buffer_size = buffer_size(&ast);
    let mut buffer = vec![0; buffer_size];
    serialize_into(program, &ast, &mut buffer);
    buffer
}
