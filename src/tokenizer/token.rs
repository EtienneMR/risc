use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Number(f64),
    String(String),
    Identifier(String),
    EndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    // grouping
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // separators
    Comma,
    Dot,
    Colon,
    Semicolon,

    // arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // comparison
    Eq,
    EqEq,
    Not,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,

    // logical
    AndAnd,
    OrOr,

    // pipe
    Pipe,
    PipeGt,

    // misc
    Arrow,    // ->
    Question, // ?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // control flow
    If,
    Do,
    Else,
    End,
    For,
    In,
    While,
    Return,
    Break,
    Continue,
    Try,
    Catch,
    As,

    // declarations
    Let,
    Fn,

    // literals
    Nil,
    True,
    False,
}

impl From<Keyword> for TokenKind {
    fn from(value: Keyword) -> Self {
        Self::Keyword(value)
    }
}

impl From<Symbol> for TokenKind {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}
