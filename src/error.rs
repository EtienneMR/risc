use std::fmt::Debug;

use crate::{lexer::TokenKind, source::Span};

#[derive(Debug, Clone)]
pub struct LangError {
    kind: &'static str,
    message: String,
    pub span: Span,
}

impl LangError {
    pub fn new(kind: &'static str, message: String, span: Span) -> Self {
        Self {
            kind,
            message,
            span: span,
        }
    }

    pub fn expected(expected: impl Debug, got: &TokenKind, span: Span) -> Self {
        if matches!(got, TokenKind::EndOfFile) {
            Self::new(
                "missing token",
                format!("expected {expected:?}, got {got:?}"),
                span,
            )
        } else {
            Self::new(
                "unexpected token",
                format!("expected {expected:?}, got {got:?}"),
                span,
            )
        }
    }
}

impl std::fmt::Display for LangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lang error: {}: {}", self.kind, self.message)
    }
}

#[derive(Debug, Clone)]
pub struct NativeError {
    pub kind: &'static str,
    pub message: String,
}

impl NativeError {
    pub fn new(kind: &'static str, message: String) -> Self {
        Self { kind, message }
    }

    pub fn operation_type_error(op: &str, a: &str, b: &str) -> Self {
        Self::new(
            "type error",
            format!("invalid operands for '{}': {} and {}", op, a, b),
        )
    }
}
