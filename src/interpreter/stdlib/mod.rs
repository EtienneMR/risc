use crate::interpreter::value::builtin::BuiltinFn;
use crate::source::Span;
use crate::{
    interpreter::{
        env::Env,
        value::{builtin::Builtin, number::Number, Value},
    },
    source::SourceId,
};

pub fn register_builtins(env: &mut Env) {
    let builtins: &[(&str, BuiltinFn)] = &[
        ("print", builtin_print),
        ("type", builtin_type),
        ("string", builtin_string),
        ("number", builtin_number),
        ("len", builtin_len),
        ("keys", builtin_keys),
        ("assert", builtin_assert),
    ];

    for (name, f) in builtins {
        env.define(
            Value::String(name.to_string()),
            Value::Builtin(Builtin::new(name, *f)),
            Span::new(SourceId(0), 0, 0),
        )
        .expect("builtins should have unique names");
    }
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    let text = args
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\t");
    println!("{text}");
    Ok(Value::Nil)
}

fn builtin_type(args: &[Value]) -> Result<Value, String> {
    let value = require_one_arg(args, "type")?;
    Ok(Value::String(value.type_name().to_string()))
}

fn builtin_string(args: &[Value]) -> Result<Value, String> {
    let value = require_one_arg(args, "string")?;
    Ok(Value::String(value.to_string()))
}

fn builtin_number(args: &[Value]) -> Result<Value, String> {
    match require_one_arg(args, "number")? {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(Number::from)
            .map(Value::Number)
            .map_err(|_| format!("cannot convert {s:?} to number")),
        v => Err(format!("cannot convert {} to number", v.type_name())),
    }
}

fn builtin_len(args: &[Value]) -> Result<Value, String> {
    match require_one_arg(args, "len")? {
        Value::String(s) => Ok(Value::Number((s.chars().count() as f64).into())),
        Value::Table(t) => Ok(Value::Number((t.len() as f64).into())),
        v => Err(format!("len() is not defined for {}", v.type_name())),
    }
}

fn builtin_keys(args: &[Value]) -> Result<Value, String> {
    match require_one_arg(args, "keys")? {
        Value::Table(t) => Ok(Value::Table(t.keys().into())),
        v => Err(format!("keys() is not defined for {}", v.type_name())),
    }
}

fn builtin_assert(args: &[Value]) -> Result<Value, String> {
    let condition = require_one_arg(args, "assert")?;
    if condition.is_truthy() {
        return Ok(Value::Nil);
    }
    let message = args
        .get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "assertion failed".to_string());
    Err(message)
}

fn require_one_arg<'a>(args: &'a [Value], name: &str) -> Result<&'a Value, String> {
    args.first()
        .ok_or_else(|| format!("{name}() requires at least 1 argument"))
}
