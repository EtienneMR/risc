pub mod token;

mod cursor;
mod error;
mod keywords;
mod readers;
mod symbols;

pub use error::TokenizationError;
pub use token::{Keyword, Symbol, Token, TokenKind};

use crate::source::Source;
use cursor::Cursor;
use keywords::keyword;
use readers::{read_identifier, read_number, read_string};
use symbols::read_symbol;

pub struct Tokenizer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    pub fn read_token(&mut self) -> Result<Token, TokenizationError> {
        self.cursor.skip_whitespace_and_comments();

        let start = self.cursor.pos();

        let Some(byte) = self.cursor.bump() else {
            return Ok(Token::new(
                TokenKind::EndOfFile,
                self.cursor.make_span(start),
            ));
        };

        let kind = match byte {
            b'0'..=b'9' => TokenKind::Number(read_number(&mut self.cursor, start)),

            b'"' => match read_string(&mut self.cursor, start) {
                Ok(s) => TokenKind::String(s),
                Err(e) => return Err(e),
            },

            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let ident = read_identifier(&mut self.cursor, start);
                keyword(&ident).unwrap_or(TokenKind::Identifier(ident))
            }

            other => match read_symbol(&mut self.cursor, other) {
                Some(sym) => TokenKind::Symbol(sym),
                None => {
                    return Err(TokenizationError::new(
                        format!("unexpected character {:?}", other as char),
                        self.cursor.make_span(start),
                    ));
                }
            },
        };

        Ok(Token::new(kind, self.cursor.make_span(start)))
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, TokenizationError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_token() {
            Ok(Token {
                kind: TokenKind::EndOfFile,
                ..
            }) => None,
            t => Some(t),
        }
    }
}
