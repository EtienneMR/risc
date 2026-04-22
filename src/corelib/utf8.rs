//! @core/utf8 — UTF-8 encoding and Unicode utilities.
//! Byte sequences are 0-indexed Tables of Numbers (0–255).
//! Codepoint sequences are Tables of Unicode scalar values.
//! utf8.encode(s) → bytes table; utf8.decode(bytes) → string.
//! Higher-level iteration over codepoints lives in @std/utf8.

use std::rc::Rc;

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();

    define_in(&t, "utf8.encode", utf8_encode);
    define_in(&t, "utf8.decode", utf8_decode);
    define_in(&t, "utf8.is_valid_bytes", utf8_is_valid_bytes);

    define_in(&t, "utf8.codepoints", utf8_codepoints);
    define_in(&t, "utf8.from_codepoints", utf8_from_codepoints);
    define_in(&t, "utf8.char", utf8_char);
    define_in(&t, "utf8.codepoint", utf8_codepoint);

    define_in(&t, "utf8.len", utf8_len);
    define_in(&t, "utf8.byte_len", utf8_byte_len);

    Value::Table(t)
}

fn utf8_encode(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "utf8.encode")?;
    let bytes: Vec<Value> = s
        .as_bytes()
        .iter()
        .map(|&b| Value::Number(b as f64))
        .collect();
    Ok(Value::Table(Table::from_vec(bytes)))
}

fn utf8_decode(ctx: CallContext) -> Result<Value, Signal> {
    let table = require_table(&ctx, 0, "bytes", "utf8.decode")?;
    let bytes = sorted_int_values(&table);
    let raw: Result<Vec<u8>, _> = bytes
        .iter()
        .map(|v| match v {
            Value::Number(n) => {
                let b = *n as i64;
                if b < 0 || b > 255 {
                    Err("byte out of range 0–255")
                } else {
                    Ok(b as u8)
                }
            }
            _ => Err("non-number in byte table"),
        })
        .collect();

    let raw = raw.map_err(|msg| {
        ctx.error(NativeError::new(
            "utf8 error",
            format!("utf8.decode: {msg}"),
        ))
    })?;

    String::from_utf8(raw)
        .map(|s| Value::String(Rc::from(s.as_str())))
        .map_err(|e| ctx.error(NativeError::new("utf8 error", format!("utf8.decode: {e}"))))
}

fn utf8_is_valid_bytes(ctx: CallContext) -> Result<Value, Signal> {
    let table = require_table(&ctx, 0, "bytes", "utf8.is_valid_bytes")?;
    let bytes = sorted_int_values(&table);
    let raw: Option<Vec<u8>> = bytes
        .iter()
        .map(|v| match v {
            Value::Number(n) => {
                let b = *n as i64;
                if b >= 0 && b <= 255 {
                    Some(b as u8)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    Ok(Value::Boolean(
        raw.map(|r| std::str::from_utf8(&r).is_ok())
            .unwrap_or(false),
    ))
}

fn utf8_codepoints(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "utf8.codepoints")?;
    let points: Vec<Value> = s.chars().map(|c| Value::Number(c as u32 as f64)).collect();
    Ok(Value::Table(Table::from_vec(points)))
}

fn utf8_from_codepoints(ctx: CallContext) -> Result<Value, Signal> {
    let table = require_table(&ctx, 0, "points", "utf8.from_codepoints")?;
    let values = sorted_int_values(&table);
    let mut s = String::new();
    for v in values {
        match v {
            Value::Number(n) => {
                let cp = n as u32;
                let c = char::from_u32(cp).ok_or_else(|| {
                    ctx.error(NativeError::new(
                        "utf8 error",
                        format!("utf8.from_codepoints: {cp} is not a valid Unicode scalar value"),
                    ))
                })?;
                s.push(c);
            }
            _ => {
                return Err(ctx.error(NativeError::new(
                    "type error",
                    "utf8.from_codepoints: table must contain numbers".into(),
                )));
            }
        }
    }
    Ok(Value::String(Rc::from(s.as_str())))
}

fn utf8_char(ctx: CallContext) -> Result<Value, Signal> {
    let n = match ctx.get(0, "codepoint") {
        Value::Number(n) => *n as u32,
        other => {
            return Err(ctx.error(NativeError::new(
                "type error",
                format!("utf8.char: expected number, got {}", other.type_name()),
            )));
        }
    };
    let c = char::from_u32(n).ok_or_else(|| {
        ctx.error(NativeError::new(
            "utf8 error",
            format!("utf8.char: {n} is not a valid Unicode scalar value"),
        ))
    })?;
    Ok(Value::String(Rc::from(c.to_string().as_str())))
}

fn utf8_codepoint(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "utf8.codepoint")?;
    match s.chars().next() {
        Some(c) => Ok(Value::Number(c as u32 as f64)),
        None => Err(ctx.error(NativeError::new(
            "utf8 error",
            "utf8.codepoint: string is empty".into(),
        ))),
    }
}

fn utf8_len(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "utf8.len")?;
    Ok(Value::Number(s.chars().count() as f64))
}

fn utf8_byte_len(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "utf8.byte_len")?;
    Ok(Value::Number(s.len() as f64))
}

fn require_table<'a>(
    ctx: &'a CallContext,
    index: usize,
    name: &str,
    fn_name: &str,
) -> Result<&'a Table, Signal> {
    match ctx.get(index, name) {
        Value::Table(t) => Ok(t),
        other => Err(ctx.error(NativeError::new(
            "type error",
            format!(
                "{fn_name}: argument '{name}' must be a table, got {}",
                other.type_name()
            ),
        ))),
    }
}

fn sorted_int_values(table: &Table) -> Vec<Value> {
    let mut pairs: Vec<(i64, Value)> = table
        .entries()
        .into_iter()
        .filter_map(|(k, v)| {
            if let TableKey::Integer(i) = k {
                Some((i, v))
            } else {
                None
            }
        })
        .collect();
    pairs.sort_by_key(|(i, _)| *i);
    pairs.into_iter().map(|(_, v)| v).collect()
}
