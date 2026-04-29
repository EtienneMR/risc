//! Global built-in functions injected into every interpreter environment at startup.
//! print(…) joins all arguments with tabs and writes one line to stdout.
//! error(kind, msg) raises a Signal that unwinds to the nearest matching try/catch.
//! len(), type(), number(), bool() are the core introspection and coercion primitives.
//! assert(cond, msg) raises "assertion error" when cond is falsy; require(path) loads modules.

use std::rc::Rc;

use crate::{
    corelib::helpers::get_string,
    error::NativeError,
    value::{CallContext, EnvRef, Function, NativeFunction, Signal, SignalKind, Value},
};

pub fn register_builtins(env: &EnvRef) {
    define(env, "print", builtin_print);
    define(env, "assert", builtin_assert);
    define(env, "number", builtin_number);
    define(env, "bool", builtin_bool);
    define(env, "type", builtin_type);
    define(env, "len", builtin_len);
    define(env, "error", builtin_error);
    define(env, "require", builtin_require);
}

fn builtin_print(ctx: CallContext) -> Result<Value, Signal> {
    let output = ctx
        .args
        .iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>()
        .join("\t");
    println!("{}", output);
    Ok(Value::Nil)
}

fn builtin_assert(ctx: CallContext) -> Result<Value, Signal> {
    if !ctx.get(0, "condition").to_boolean() {
        let msg = match ctx.get(1, "message") {
            Value::Nil => "assertion failed".to_string(),
            v => v.to_string_ref().to_string(),
        };
        return Err(ctx.error(NativeError::new("assertion error", msg)));
    }
    Ok(Value::Nil)
}

fn builtin_number(ctx: CallContext) -> Result<Value, Signal> {
    ctx.get(0, "value")
        .to_number()
        .map(Value::Number)
        .map_err(|e| ctx.error(e))
}

fn builtin_bool(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::Boolean(ctx.get(0, "value").to_boolean()))
}

fn builtin_type(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(Rc::from(ctx.get(0, "value").type_name())))
}

fn builtin_len(ctx: CallContext) -> Result<Value, Signal> {
    let obj = ctx.get(0, "value");
    match obj {
        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
        Value::Table(t) => Ok(Value::Number(t.len() as f64)),
        _ => Err(ctx.error(NativeError::new(
            "type error",
            format!("len requires string or table, got {}", obj.type_name()),
        ))),
    }
}

fn builtin_error(ctx: CallContext) -> Result<Value, Signal> {
    Err(Signal {
        kind: SignalKind::Error {
            kind: ctx.get(0, "kind").to_string_ref(),
            message: ctx.get(1, "message").to_string_ref(),
        },
        traceback: Vec::new(),
    })
}

fn builtin_require(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "require")?;
    ctx.runtime.load_module(&path, ctx.span)
}

fn define(env: &EnvRef, name: &'static str, func: fn(CallContext) -> Result<Value, Signal>) {
    env.define(
        Rc::from(name),
        Value::Function(Function::Native(NativeFunction { name, func })),
    )
    .ok();
}
