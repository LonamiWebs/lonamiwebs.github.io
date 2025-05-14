use super::{ParseResult, Token};

pub fn preprocess(parsed: ParseResult) -> Vec<Token> {
    parsed.tokens
}
