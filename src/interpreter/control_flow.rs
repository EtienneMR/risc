use crate::interpreter::value::table::Table;
use crate::interpreter::value::Value;
use crate::source::Span;

#[derive(Debug)]
pub enum ControlFlowKind {
    /// A runtime error carrying an error-table `{ error = kind, msg = message }`.
    Error(Value),
    Break(Value),
    Continue,
    Return(Value),
}

#[derive(Debug)]
pub struct ControlFlow {
    pub kind: ControlFlowKind,
    pub span: Span,
}

impl ControlFlow {
    pub fn new(kind: ControlFlowKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::error_with_kind(message, "error", span)
    }

    pub fn error_with_kind(
        message: impl Into<String>,
        kind: impl Into<String>,
        span: Span,
    ) -> Self {
        let table = Table::new();
        table.set("error".into(), Value::String(kind.into()));
        table.set("msg".into(), Value::String(message.into()));
        Self::error_from_value(Value::Table(table), span)
    }

    pub fn error_from_value(value: Value, span: Span) -> Self {
        Self {
            kind: ControlFlowKind::Error(value),
            span,
        }
    }

    pub fn reject_loop_control(self) -> Self {
        match self.kind {
            ControlFlowKind::Break(..) => Self::error("break used outside of a loop", self.span),
            ControlFlowKind::Continue => Self::error("continue used outside of a loop", self.span),
            _ => self,
        }
    }
}
