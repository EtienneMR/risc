use crate::{
    error::LangError,
    source::{Source, Span},
};

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: impl Into<TokenKind>, span: Span) -> Self {
        Self {
            kind: kind.into(),
            span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Number(f64),
    String(String),
    Identifier(String),
    EndOfFile,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Comma,
    Dot,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Eq,
    EqEq,
    Not,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,

    AndAnd,
    OrOr,
    Pipe,
    PipeGt,
}

impl Into<TokenKind> for Symbol {
    fn into(self) -> TokenKind {
        TokenKind::Symbol(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    If,
    Then,
    Else,
    End,
    For,
    In,
    Do,
    While,

    Return,
    Break,
    Continue,

    Let,
    Fn,

    Try,
    Catch,
    As,

    Nil,
    True,
    False,
}

impl Into<TokenKind> for Keyword {
    fn into(self) -> TokenKind {
        TokenKind::Keyword(self)
    }
}

pub struct Lexer<'a> {
    source: &'a Source,
    iter: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            iter: source.content.char_indices().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LangError> {
        self.skip_ignored();

        let Some((start, ch)) = self.iter.next() else {
            let end = self.source.content.len();
            return Ok(Token {
                kind: TokenKind::EndOfFile,
                span: self.make_span(end, end),
            });
        };

        let token = match ch {
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(start),
            '0'..='9' => self.lex_number(start, ch),
            '"' => self.lex_string(start)?,

            '(' => self.make_symbol(start, ch, Symbol::LParen),
            ')' => self.make_symbol(start, ch, Symbol::RParen),
            '{' => self.make_symbol(start, ch, Symbol::LBrace),
            '}' => self.make_symbol(start, ch, Symbol::RBrace),
            '[' => self.make_symbol(start, ch, Symbol::LBracket),
            ']' => self.make_symbol(start, ch, Symbol::RBracket),
            ',' => self.make_symbol(start, ch, Symbol::Comma),
            '.' => self.make_symbol(start, ch, Symbol::Dot),
            '+' => self.make_symbol(start, ch, Symbol::Plus),
            '-' => self.make_symbol(start, ch, Symbol::Minus),
            '*' => self.make_symbol(start, ch, Symbol::Star),
            '/' => self.make_symbol(start, ch, Symbol::Slash),
            '%' => self.make_symbol(start, ch, Symbol::Percent),

            '=' => self.make_symbol_or_double(start, ch, '=', Symbol::Eq, Symbol::EqEq),
            '!' => self.make_symbol_or_double(start, ch, '=', Symbol::Not, Symbol::NotEq),
            '<' => self.make_symbol_or_double(start, ch, '=', Symbol::Lt, Symbol::Lte),
            '>' => self.make_symbol_or_double(start, ch, '=', Symbol::Gt, Symbol::Gte),

            '&' if self.iter.next_if(|c| c.1 == '&').is_some() => {
                self.make_token(start, start + 2, Symbol::AndAnd)
            }

            '|' => {
                if self.take_if('|') {
                    self.make_token(start, start + 2, Symbol::OrOr)
                } else if self.take_if('>') {
                    self.make_token(start, start + 2, Symbol::PipeGt)
                } else {
                    self.make_symbol(start, ch, Symbol::Pipe)
                }
            }

            _ => {
                return Err(LangError::new(
                    "unexpected character",
                    format!("got {ch}"),
                    self.make_span(start, start + ch.len_utf8()),
                ));
            }
        };

        Ok(token)
    }

    fn skip_ignored(&mut self) {
        while let Some((_, ch)) = self.iter.peek() {
            if ch.is_whitespace() {
                self.iter.next();
            } else if *ch == '#' {
                while !matches!(self.iter.next(), None | Some((_, '\n'))) {}
            } else {
                break;
            }
        }
    }

    fn take_if(&mut self, expected: char) -> bool {
        if let Some(&(_, ch)) = self.iter.peek() {
            if ch == expected {
                self.iter.next();
                return true;
            }
        }
        false
    }

    fn make_span(&self, start: usize, end: usize) -> Span {
        self.source.create_span(start, end)
    }

    fn make_token(&self, start: usize, end: usize, kind: impl Into<TokenKind>) -> Token {
        Token::new(kind, self.make_span(start, end))
    }

    fn make_symbol(&self, start: usize, ch: char, symbol: Symbol) -> Token {
        self.make_token(start, start + ch.len_utf8(), symbol)
    }

    fn make_symbol_or_double(
        &mut self,
        start: usize,
        ch: char,
        expected: char,
        single: Symbol,
        double: Symbol,
    ) -> Token {
        if self.take_if(expected) {
            self.make_token(start, start + ch.len_utf8() + expected.len_utf8(), double)
        } else {
            self.make_symbol(start, ch, single)
        }
    }

    fn keyword_kind(text: &str) -> Option<Keyword> {
        Some(match text {
            "if" => Keyword::If,
            "then" => Keyword::Then,
            "else" => Keyword::Else,
            "end" => Keyword::End,
            "for" => Keyword::For,
            "in" => Keyword::In,
            "do" => Keyword::Do,
            "while" => Keyword::While,
            "return" => Keyword::Return,
            "break" => Keyword::Break,
            "continue" => Keyword::Continue,
            "let" => Keyword::Let,
            "fn" => Keyword::Fn,
            "try" => Keyword::Try,
            "catch" => Keyword::Catch,
            "as" => Keyword::As,
            "nil" => Keyword::Nil,
            "true" => Keyword::True,
            "false" => Keyword::False,
            _ => return None,
        })
    }

    fn lex_identifier(&mut self, start: usize) -> Token {
        let mut end = start + 1;

        while let Some(&(idx, ch)) = self.iter.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.iter.next();
                end = idx + ch.len_utf8();
            } else {
                break;
            }
        }

        let text = &self.source.content[start..end];
        let kind = Self::keyword_kind(text)
            .map(TokenKind::Keyword)
            .unwrap_or_else(|| TokenKind::Identifier(text.to_owned()));

        self.make_token(start, end, kind)
    }

    fn lex_number(&mut self, start: usize, first: char) -> Token {
        let mut end = start + first.len_utf8();
        let mut found_dot = false;

        while let Some(&(idx, ch)) = self.iter.peek() {
            if ch.is_ascii_digit() {
                self.iter.next();
            } else if ch == '.' && !found_dot {
                found_dot = true;
                self.iter.next();
            } else {
                break;
            }
            end = idx + ch.len_utf8();
        }

        let text = &self.source.content[start..end];
        let value = text.parse().expect("number slice should be valid");

        self.make_token(start, end, TokenKind::Number(value))
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, LangError> {
        let mut value = String::new();
        let mut end = start + 1;

        while let Some((idx, ch)) = self.iter.next() {
            end = idx + ch.len_utf8();

            value.push(match ch {
                '"' => {
                    return Ok(self.make_token(start, end, TokenKind::String(value)));
                }
                '\\' => {
                    let Some((esc_idx, esc)) = self.iter.next() else {
                        break;
                    };

                    end = esc_idx + esc.len_utf8();

                    match esc {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        '0' => '\0',
                        _ => {
                            return Err(LangError::new(
                                "invalid escape sequence",
                                format!("got {esc}"),
                                self.make_span(idx, end),
                            ));
                        }
                    }
                }
                other => other,
            });
        }

        Err(LangError::new(
            "unterminated string literal",
            String::new(),
            self.make_span(start, end),
        ))
    }
}
