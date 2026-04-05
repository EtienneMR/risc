use crate::interpreter::control_flow::ControlFlow;
use crate::interpreter::value::Value;
use crate::source::Span;

pub fn arithmetic(
    left: Value,
    right: Value,
    span: Span,
    op: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Value, ControlFlow> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a.value, b.value).into())),
        (l, r) => Err(ControlFlow::error(
            format!(
                "'{op}' requires two numbers, got {} and {}",
                l.type_name(),
                r.type_name()
            ),
            span,
        )),
    }
}

pub fn compare(
    left: Value,
    right: Value,
    span: Span,
    op: &str,
    f: impl Fn(f64, f64) -> bool,
) -> Result<Value, ControlFlow> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a.value, b.value))),
        (l, r) => Err(ControlFlow::error(
            format!(
                "'{op}' requires two numbers, got {} and {}",
                l.type_name(),
                r.type_name()
            ),
            span,
        )),
    }
}

pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Table(a), Value::Table(b)) => a == b,
        _ => false,
    }
}
