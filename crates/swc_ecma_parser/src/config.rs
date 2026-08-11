pub trait ParserConfig {
    fn tokens(&self) -> bool;

    /// Preserve ESTree assignment targets whose concrete expression kind is
    /// not represented by the public assignment-target AST enums.
    ///
    /// This mode is only safe when the AST is immediately serialized by its
    /// concrete node kinds without typed assignment-target traversal.
    fn preserve_estree_assignment_targets(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Default)]
pub struct TokenParserConfig;

impl ParserConfig for TokenParserConfig {
    fn tokens(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Default)]
pub struct NoTokenParserConfig;

impl ParserConfig for NoTokenParserConfig {
    fn tokens(&self) -> bool {
        false
    }
}
