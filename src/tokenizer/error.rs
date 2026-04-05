use crate::source::Span;
use std::fmt;

#[derive(Debug)]
pub struct TokenizationError {
    pub message: String,
    pub span: Span,
}

impl TokenizationError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (bytes {}..{})",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for TokenizationError {}
