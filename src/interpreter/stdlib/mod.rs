use crate::interpreter::{Interpreter};
use crate::interpreter::control_flow::ControlFlow;
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
        ("print",  builtin_print),
        ("type",   builtin_type),
        ("string", builtin_string),
        ("number", builtin_number),
        ("len",    builtin_len),
        ("keys",   builtin_keys),
        ("assert", builtin_assert),
        ("error",  builtin_error),
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

fn builtin_print(args: &[Value], _span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    let text = args
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\t");
    println!("{text}");
    Ok(Value::Nil)
}

fn builtin_type(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    let value = require_one_arg(args, "type", span)?;
    Ok(Value::String(value.type_name().to_string()))
}

fn builtin_string(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    let value = require_one_arg(args, "string", span)?;
    Ok(Value::String(value.to_string()))
}

fn builtin_number(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    match require_one_arg(args, "number", span)? {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(s) => s
            .trim()
            .parse::<f64>()
            .map(Number::from)
            .map(Value::Number)
            .map_err(|_| ControlFlow::error(format!("cannot convert {s:?} to number"), span)),
        v => Err(ControlFlow::error(format!("cannot convert {} to number", v.type_name()), span)),
    }
}

fn builtin_len(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    match require_one_arg(args, "len", span)? {
        Value::String(s) => Ok(Value::Number((s.chars().count() as f64).into())),
        Value::Table(t) => Ok(Value::Number((t.len() as f64).into())),
        v => Err(ControlFlow::error(format!("len() is not defined for {}", v.type_name()), span)),
    }
}

fn builtin_keys(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    match require_one_arg(args, "keys", span)? {
        Value::Table(t) => Ok(Value::Table(t.keys().into())),
        v => Err(ControlFlow::error(format!("keys() is not defined for {}", v.type_name()), span)),
    }
}

fn builtin_assert(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    let condition = require_one_arg(args, "assert", span)?;
    if condition.is_truthy() {
        return Ok(Value::Nil);
    }
    let message = args
        .get(1)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "assertion failed".to_string());
    Err(ControlFlow::error(message, span))
}

fn require_one_arg<'a>(args: &'a [Value], name: &str, span: Span) -> Result<&'a Value, ControlFlow> {
    args.first()
        .ok_or_else(|| ControlFlow::error(format!("{name}() requires at least 1 argument"), span))
}

fn builtin_error(args: &[Value], span: Span, _interpreter: &Interpreter) -> Result<Value, ControlFlow> {
    use crate::interpreter::value::table::Table;

    let message = require_one_arg(args, "error", span)?;
    let kind = args.get(1).unwrap_or_else(|| &Value::Nil);

    let table = Table::new();
    table.set("error".into(), kind.clone());
    table.set("msg".into(), message.clone());
    Err(ControlFlow::error_from_value(Value::Table(table), span))
}
