use crate::interpreter::control_flow::ControlFlow;
use crate::interpreter::value::table::Table;
use crate::interpreter::value::Value;
use crate::source::Span;

const MAGIC_KEY: &str = "__index";

#[derive(Debug, Clone)]
pub struct Env {
    pub scope: Table,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scope: Table::new(),
        }
    }

    pub fn inner_scope(&self) -> Self {
        let scope = Table::new();
        scope.set(MAGIC_KEY.into(), Value::Table(self.scope.clone()));
        Self { scope }
    }

    pub fn get(&self, key: &Value, span: Span) -> Result<Value, ControlFlow> {
        Ok(resolve_recursive(self.scope.clone(), key, span)?.get(key))
    }

    pub fn set(&mut self, key: Value, value: Value, span: Span) -> Result<(), ControlFlow> {
        resolve_recursive(self.scope.clone(), &key, span)?.set(key, value);
        Ok(())
    }

    pub fn define(&mut self, key: Value, value: Value, span: Span) -> Result<(), ControlFlow> {
        if self.scope.has(&key) {
            Err(ControlFlow::error(
                format!("{} is already defined", key),
                span,
            ))
        } else {
            self.scope.set(key, value);
            Ok(())
        }
    }
}

fn resolve_recursive<'a>(scope: Table, key: &Value, span: Span) -> Result<Table, ControlFlow> {
    if scope.has(key) {
        Ok(scope)
    } else if let Value::Table(parent) = scope.get(&MAGIC_KEY.into()) {
        resolve_recursive(parent, key, span)
    } else {
        Err(ControlFlow::error(format!("{} is not defined", key), span))
    }
}
