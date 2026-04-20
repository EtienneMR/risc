use std::rc::Rc;

use crate::{
    error::NativeError,
    value::{
        CallContext, EnvRef, Function, NativeFunction, Signal, SignalKind, Table, TableKey, Value,
    },
};

pub fn register_builtins(env: &EnvRef) {
    define(env, "print", builtin_print);
    define(env, "assert", builtin_assert);
    define(env, "number", builtin_number);
    define(env, "bool", builtin_bool);
    define(env, "type", builtin_type);
    define(env, "len", builtin_len);
    define(env, "error", builtin_error);
}

pub fn get_corelib(key: &str) -> Option<Value> {
    match key {
        "string" => Some(create_string()),
        "os" => Some(create_os()),
        _ => None,
    }
}

fn define(env: &EnvRef, name: &'static str, func: fn(CallContext) -> Result<Value, Signal>) {
    env.define(
        Rc::from(name),
        Value::Function(Function::Native(NativeFunction { name, func })),
    )
    .ok();
}

fn define_in(table: &Table, name: &'static str, func: fn(CallContext) -> Result<Value, Signal>) {
    let inner_name = name.split_once('.').expect("name should be scoped").1;
    let mut t = table.clone();
    t.set(
        TableKey::String(Rc::from(inner_name)),
        Value::Function(Function::Native(NativeFunction { name, func })),
    );
}

pub fn create_string() -> Value {
    let table = Table::new();

    define_in(&table, "string.from", string_from);
    define_in(&table, "string.upper", string_upper);
    define_in(&table, "string.lower", string_lower);
    define_in(&table, "string.trim", string_trim);
    define_in(&table, "string.trim_start", string_trim_start);
    define_in(&table, "string.trim_end", string_trim_end);
    define_in(&table, "string.split", string_split);
    define_in(&table, "string.contains", string_contains);
    define_in(&table, "string.starts_with", string_starts_with);
    define_in(&table, "string.ends_with", string_ends_with);
    define_in(&table, "string.replace", string_replace);
    define_in(&table, "string.slice", string_slice);
    define_in(&table, "string.find", string_find);
    define_in(&table, "string.repeat", string_repeat);
    define_in(&table, "string.reverse", string_reverse);
    define_in(&table, "string.len", string_len);

    Value::Table(table)
}

pub fn create_os() -> Value {
    let table = Table::new();

    define_in(&table, "os.read", os_read);
    define_in(&table, "os.write", os_write);
    define_in(&table, "os.append", os_append);

    Value::Table(table)
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
    if !ctx.arg(0).to_boolean() {
        let msg = match ctx.arg(1) {
            Value::Nil => "assertion failed".to_string(),
            v => v.to_string_ref().to_string(),
        };
        return Err(ctx.error(NativeError::new("assertion error", msg)));
    }
    Ok(Value::Nil)
}

fn builtin_number(ctx: CallContext) -> Result<Value, Signal> {
    ctx.arg(0)
        .to_number()
        .map(Value::Number)
        .map_err(|e| ctx.error(e))
}

fn builtin_bool(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::Boolean(ctx.arg(0).to_boolean()))
}

fn builtin_type(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(Rc::from(ctx.arg(0).type_name())))
}

fn builtin_len(ctx: CallContext) -> Result<Value, Signal> {
    let obj = ctx.arg(0);
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
            kind: ctx.arg(1).to_string_ref(),
            message: ctx.arg(0).to_string_ref(),
        },
        span: ctx.span,
    })
}

fn os_read(ctx: CallContext) -> Result<Value, Signal> {
    let path = require_string(&ctx, 0, "os.read")?;
    std::fs::read_to_string(path.as_ref())
        .map(|s| Value::String(Rc::from(s.as_str())))
        .map_err(|e| ctx.error(NativeError::new("os error", format!("os.read: {}", e))))
}

fn os_write(ctx: CallContext) -> Result<Value, Signal> {
    let path = require_string(&ctx, 0, "os.write")?;
    let content = ctx.arg(1).to_string_ref();
    std::fs::write(path.as_ref(), content.as_bytes())
        .map(|_| Value::Nil)
        .map_err(|e| ctx.error(NativeError::new("os error", format!("os.write: {}", e))))
}

fn os_append(ctx: CallContext) -> Result<Value, Signal> {
    use std::io::Write;
    let path = require_string(&ctx, 0, "os.append")?;
    let content = ctx.arg(1).to_string_ref();
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map(|_| Value::Nil)
        .map_err(|e| ctx.error(NativeError::new("os error", format!("os.append: {}", e))))
}

fn string_from(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(ctx.arg(0).to_string_ref()))
}

fn string_upper(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.upper")?;
    Ok(Value::String(Rc::from(s.to_uppercase().as_str())))
}

fn string_lower(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.lower")?;
    Ok(Value::String(Rc::from(s.to_lowercase().as_str())))
}

fn string_trim(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.trim")?;
    Ok(Value::String(Rc::from(s.trim())))
}

fn string_trim_start(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.trim_start")?;
    Ok(Value::String(Rc::from(s.trim_start())))
}

fn string_trim_end(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.trim_end")?;
    Ok(Value::String(Rc::from(s.trim_end())))
}

fn string_split(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.split")?;
    let sep = require_string(&ctx, 1, "string.split")?;
    let mut table = Table::new();
    for (i, part) in s.split(sep.as_ref()).enumerate() {
        table.set(TableKey::Integer(i as i64), Value::String(Rc::from(part)));
    }
    Ok(Value::Table(table))
}

fn string_contains(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.contains")?;
    let sub = require_string(&ctx, 1, "string.contains")?;
    Ok(Value::Boolean(s.contains(sub.as_ref())))
}

fn string_starts_with(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.starts_with")?;
    let prefix = require_string(&ctx, 1, "string.starts_with")?;
    Ok(Value::Boolean(s.starts_with(prefix.as_ref())))
}

fn string_ends_with(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.ends_with")?;
    let suffix = require_string(&ctx, 1, "string.ends_with")?;
    Ok(Value::Boolean(s.ends_with(suffix.as_ref())))
}

fn string_replace(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.replace")?;
    let from = require_string(&ctx, 1, "string.replace")?;
    let to = require_string(&ctx, 2, "string.replace")?;
    Ok(Value::String(Rc::from(
        s.replace(from.as_ref(), to.as_ref()).as_str(),
    )))
}

fn string_slice(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.slice")?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    let resolve = |raw: i64| -> usize {
        let i = if raw < 0 { len + raw } else { raw };
        i.clamp(0, len) as usize
    };

    let start = match ctx.arg(1) {
        Value::Number(n) => resolve(*n as i64),
        Value::Nil => 0,
        _ => {
            return Err(ctx.error(NativeError::new(
                "type error",
                "string.slice: start must be a number".to_string(),
            )));
        }
    };
    let end = match ctx.arg(2) {
        Value::Number(n) => resolve(*n as i64),
        Value::Nil => chars.len(),
        _ => {
            return Err(ctx.error(NativeError::new(
                "type error",
                "string.slice: end must be a number".to_string(),
            )));
        }
    };

    let lo = start.min(end);
    let hi = start.max(end);
    let sliced: String = chars[lo..hi].iter().collect();
    Ok(Value::String(Rc::from(sliced.as_str())))
}

fn string_find(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.find")?;
    let sub = require_string(&ctx, 1, "string.find")?;
    match s.find(sub.as_ref()) {
        Some(byte_idx) => {
            let char_idx = s[..byte_idx].chars().count();
            Ok(Value::Number(char_idx as f64))
        }
        None => Ok(Value::Nil),
    }
}

fn string_repeat(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.repeat")?;
    let n = match ctx.arg(1) {
        Value::Number(n) => *n as usize,
        _ => {
            return Err(ctx.error(NativeError::new(
                "type error",
                "string.repeat: n must be a number".to_string(),
            )));
        }
    };
    Ok(Value::String(Rc::from(s.repeat(n).as_str())))
}

fn string_reverse(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.reverse")?;
    Ok(Value::String(Rc::from(
        s.chars().rev().collect::<String>().as_str(),
    )))
}

fn string_len(ctx: CallContext) -> Result<Value, Signal> {
    let s = require_string(&ctx, 0, "string.len")?;
    Ok(Value::Number(s.chars().count() as f64))
}

fn require_string(
    ctx: &CallContext,
    index: usize,
    fn_name: &'static str,
) -> Result<Rc<str>, Signal> {
    match ctx.arg(index) {
        Value::String(s) => Ok(s.clone()),
        other => Err(ctx.error(NativeError::new(
            "type error",
            format!(
                "{}: argument {} must be a string, got {}",
                fn_name,
                index,
                other.type_name()
            ),
        ))),
    }
}
