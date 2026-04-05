use crate::interpreter::value::table::Table;
use crate::interpreter::value::Value;
use crate::source::Span;

#[derive(Debug)]
pub enum ControlFlowKind {
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
        let value = Table::new();
        value.set(Value::String("error".into()), Value::String(message.into()));

        Self {
            kind: ControlFlowKind::Error(Value::Table(value)),
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
