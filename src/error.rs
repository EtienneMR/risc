//! Error types for parse-time (LangError) and native-call-time (NativeError) failures.
//! LangError carries a kind tag, a message, and a traceback of source Spans for display.
//! is_incomplete() — via LangErrorKind::UnexpectedEOF — signals the REPL to ask for more input.
//! NativeError is a lightweight struct used by all built-in functions to report typed errors.
//! LangError::display() renders a multi-line human-readable message with source context lines.

use std::fmt::Debug;

use crate::source::{SourceMap, Span};

#[derive(Debug, Clone)]
pub struct LangError {
    pub kind: LangErrorKind,
    pub message: String,
    pub traceback: Vec<Span>,
}

#[derive(Debug, Clone)]
pub enum LangErrorKind {
    InvalidInput { subkind: &'static str },
    InvalidSyntax { subkind: &'static str },
    UnexpectedEOF { subkind: &'static str },
    RuntimeError { subkind: String },
}

impl LangError {
    pub fn new(kind: LangErrorKind, message: String, traceback: Vec<Span>) -> Self {
        Self {
            kind,
            message,
            traceback,
        }
    }

    pub fn invalid_input(subkind: &'static str, message: String, span: Span) -> Self {
        Self::new(LangErrorKind::InvalidInput { subkind }, message, vec![span])
    }

    pub fn invalid_syntax(subkind: &'static str, message: String, span: Span) -> Self {
        Self::new(
            LangErrorKind::InvalidSyntax { subkind },
            message,
            vec![span],
        )
    }

    pub fn runtime_error(subkind: String, message: String, traceback: Vec<Span>) -> Self {
        Self::new(LangErrorKind::RuntimeError { subkind }, message, traceback)
    }

    pub fn expected_token(expected: impl Debug, got: &crate::lexer::TokenKind, span: Span) -> Self {
        if matches!(got, crate::lexer::TokenKind::EndOfFile) {
            Self::new(
                LangErrorKind::UnexpectedEOF {
                    subkind: "expected token",
                },
                format!("missing {expected:?}"),
                vec![span],
            )
        } else {
            Self::invalid_syntax(
                "unexpected token",
                format!("exepcted {expected:?}; got {got:?}"),
                span,
            )
        }
    }

    pub fn add_context(mut self, span: Span) -> Self {
        self.traceback.push(span);
        self
    }

    pub fn extract(&self) -> String {
        format!(
            "{} ({})",
            match &self.kind {
                LangErrorKind::InvalidInput { subkind } => format!("invalid input: {subkind}"),
                LangErrorKind::InvalidSyntax { subkind } => format!("invalid syntax: {subkind}"),
                LangErrorKind::UnexpectedEOF { subkind } =>
                    format!("unexpected end of file: {subkind}"),
                LangErrorKind::RuntimeError { subkind } => format!("runtime error: {subkind}"),
            },
            self.message
        )
    }

    pub fn display(&self, source_map: &SourceMap) -> String {
        let mut traceback = self.traceback.iter().peekable();

        let mut out = self.extract();
        out.push_str("\n\n");

        if let Some(source) = traceback.next() {
            out.push_str("at ");
            out.push_str(&source_map.format_location(*source));
            out.push_str("\n");
            out.push_str(&source_map.render_span_context(*source, 3, 1));
        }

        if traceback.peek().is_some() {
            out.push_str("\n\nstack trace (most recent call last):");

            for span in traceback.rev() {
                out.push_str("\n  at ");
                out.push_str(&source_map.format_location(*span));
                if let Some(line) = source_map.extract_line(*span) {
                    out.push_str("  ");
                    out.push_str(line);
                }
            }
        }

        out
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
