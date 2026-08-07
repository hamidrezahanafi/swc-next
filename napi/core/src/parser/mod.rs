use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use swc_next_napi::{
    Lang, ParserOptions, SourceType, lang_from_path as shared_lang_from_path, parse_to_buffer,
    source_type_from_path as shared_source_type_from_path,
};

#[napi]
pub fn parse_sync(source: String, options: Option<ParserOptions>) -> Buffer {
    Buffer::from(parse_to_buffer(&source, options))
}

#[napi]
pub fn source_type_from_path(path: String) -> SourceType {
    shared_source_type_from_path(&path)
}

#[napi]
pub fn lang_from_path(path: String) -> Lang {
    shared_lang_from_path(&path)
}
