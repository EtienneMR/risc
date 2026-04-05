use std::fmt;

use crate::parser::error::ParseError;
use crate::source::Span;
use crate::tokenizer::{Token, TokenKind, TokenizationError, Tokenizer};

pub struct Cursor<'a> {
    tokenizer: Tokenizer<'a>,
    lookahead: Option<Token>,
}

impl<'a> Cursor<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer,
            lookahead: None,
        }
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        self.tokenizer
            .read_token()
            .map_err(tokenization_to_parse_error)
    }

    pub fn peek(&mut self) -> Result<&Token, ParseError> {
        if self.lookahead.is_none() {
            self.lookahead = Some(self.advance()?);
        }
        Ok(self
            .lookahead
            .as_ref()
            .expect("lookahead should be defined"))
    }

    pub fn bump(&mut self) -> Result<Token, ParseError> {
        match self.lookahead.take() {
            Some(tok) => Ok(tok),
            None => self.advance(),
        }
    }

    pub fn peek_kind(&mut self, kind: impl Into<TokenKind>) -> Result<bool, ParseError> {
        Ok(self.peek()?.kind == kind.into())
    }

    pub fn expect_kind(
        &mut self,
        kind: impl Into<TokenKind> + fmt::Debug,
    ) -> Result<Span, ParseError> {
        let tok = self.bump()?;
        let sym_label = format!("{kind:?}");

        if tok.kind == kind.into() {
            return Ok(tok.span);
        }

        Err(ParseError::expected(&sym_label, &tok))
    }
}

fn tokenization_to_parse_error(e: TokenizationError) -> ParseError {
    ParseError::new(e.message, e.span)
}
