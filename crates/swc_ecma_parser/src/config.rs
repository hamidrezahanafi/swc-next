pub trait ParserConfig {
    fn tokens(&self) -> bool;

    /// Preserve syntactically invalid ESTree assignment targets so a later
    /// semantic pass can diagnose them without losing the surrounding tree.
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
