mod generator;
mod parser;
mod preprocessor;

pub use parser::{ParseResult, Token, parse};
pub use preprocessor::preprocess;
