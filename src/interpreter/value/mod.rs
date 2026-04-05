use crate::interpreter::value::{
    builtin::Builtin, function::Function, number::Number, table::Table,
};
use std::{fmt, hash::Hash};

pub mod builtin;
pub mod function;
pub mod number;
pub mod table;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(Number),
    String(String),
    Table(Table),
    Builtin(Builtin),
    Function(Function),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Table(t) => write!(f, "{t}"),
            Value::Builtin(b) => write!(f, "{b}"),
            Value::Function(r#fn) => write!(f, "{fn}"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Builtin(_) => "function",
            Value::Function(_) => "function",
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}
