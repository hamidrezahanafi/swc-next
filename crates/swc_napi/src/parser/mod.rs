mod options;

use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{
    Ast, Lang as AstLang, SourceType as AstSourceType,
    ffi::ast::{buffer_size, serialize_into},
};
use swc_next_ecma_parser::{NoTokenParserConfig, Parser};

pub use self::options::{CommentMode, Lang, ParserOptions, SourceType};

pub fn parse_to_buffer(source: &str, options: Option<ParserOptions>) -> Vec<u8> {
    let allocator = Allocator::default();
    let mut ast = Ast::new(source, &allocator);
    let program = Parser::init(
        &mut ast,
        options.unwrap_or_default().into(),
        NoTokenParserConfig,
    )
    .parse()
    .program;

    let buffer_size = buffer_size(&ast);
    let mut buffer = vec![0; buffer_size];
    serialize_into(program, &ast, &mut buffer);
    buffer
}

pub fn source_type_from_path(path: &str) -> SourceType {
    AstSourceType::from_path(path).into()
}

pub fn lang_from_path(path: &str) -> Lang {
    AstLang::from_path(path).into()
}
