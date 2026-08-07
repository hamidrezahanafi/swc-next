mod parser;

pub use parser::{
    CommentMode, Lang, ParserOptions, SourceType, lang_from_path, parse_to_buffer,
    source_type_from_path,
};
