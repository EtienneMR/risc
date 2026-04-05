pub mod ast;
mod cursor;
mod error;
mod expr;

pub use error::ParseError;

use crate::{
    parser::{ast::Block, expr::parse_block},
    tokenizer::{TokenKind, Tokenizer},
};
use cursor::Cursor;

pub fn parse(tokenizer: Tokenizer) -> Result<Block, ParseError> {
    let mut cursor = Cursor::new(tokenizer);
    parse_block(&mut cursor, &[TokenKind::EndOfFile])
}
