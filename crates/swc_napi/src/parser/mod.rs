mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::ffi::ast::{buffer_size, serialize_into};
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
    let result = Parser::init(
        &allocator,
        source,
        options.unwrap_or_default().into(),
        EstreeParserConfig,
    )
    .parse();

    let buffer_size = buffer_size(&result.ast);
    let mut buffer = vec![0; buffer_size];
    serialize_into(&result.ast, &mut buffer);
    buffer
}
