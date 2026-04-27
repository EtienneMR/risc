//! @core/string — string manipulation primitives, all operating on Unicode codepoints.
//! Splitting: split(s, sep); searching: contains, starts_with, ends_with, find, char_at.
//! Transformation: upper, lower, trim, trim_start, trim_end, replace, replace_first, slice.
//! Padding: pad_start(s, width, fill) and pad_end share a single internal helper.
//! Higher-level utilities (lines, words, join, indent, count) live in @std/string.

use std::rc::Rc;

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, Value},
};

use super::helpers::{define_in, get_number, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "string.from", string_from);
    define_in(&t, "string.upper", string_upper);
    define_in(&t, "string.lower", string_lower);
    define_in(&t, "string.trim", string_trim);
    define_in(&t, "string.trim_start", string_trim_start);
    define_in(&t, "string.trim_end", string_trim_end);
    define_in(&t, "string.split", string_split);
    define_in(&t, "string.contains", string_contains);
    define_in(&t, "string.starts_with", string_starts_with);
    define_in(&t, "string.ends_with", string_ends_with);
    define_in(&t, "string.replace", string_replace);
    define_in(&t, "string.replace_first", string_replace_first);
    define_in(&t, "string.slice", string_slice);
    define_in(&t, "string.find", string_find);
    define_in(&t, "string.repeat", string_repeat);
    define_in(&t, "string.reverse", string_reverse);
    define_in(&t, "string.len", string_len);
    define_in(&t, "string.bytes", string_bytes);
    define_in(&t, "string.char_at", string_char_at);
    define_in(&t, "string.pad_start", string_pad_start);
    define_in(&t, "string.pad_end", string_pad_end);
    Value::Table(t)
}

fn string_from(ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(ctx.get(0, "source").to_string_ref()))
}

fn string_upper(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.upper")?;
    Ok(Value::String(Rc::from(s.to_uppercase().as_str())))
}

fn string_lower(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.lower")?;
    Ok(Value::String(Rc::from(s.to_lowercase().as_str())))
}

fn string_trim(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.trim")?;
    Ok(Value::String(Rc::from(s.trim())))
}

fn string_trim_start(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.trim_start")?;
    Ok(Value::String(Rc::from(s.trim_start())))
}

fn string_trim_end(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.trim_end")?;
    Ok(Value::String(Rc::from(s.trim_end())))
}

fn string_split(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.split")?;
    let sep = get_string(&ctx, 1, "sep", "string.split")?;
    let parts: Vec<Value> = s
        .split(sep.as_ref())
        .map(|p| Value::String(Rc::from(p)))
        .collect();
    Ok(Value::Table(Table::from_vec(parts)))
}

fn string_contains(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.contains")?;
    let sub = get_string(&ctx, 1, "needle", "string.contains")?;
    Ok(Value::Boolean(s.contains(sub.as_ref())))
}

fn string_starts_with(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.starts_with")?;
    let prefix = get_string(&ctx, 1, "prefix", "string.starts_with")?;
    Ok(Value::Boolean(s.starts_with(prefix.as_ref())))
}

fn string_ends_with(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.ends_with")?;
    let suffix = get_string(&ctx, 1, "suffix", "string.ends_with")?;
    Ok(Value::Boolean(s.ends_with(suffix.as_ref())))
}

fn string_replace(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.replace")?;
    let from = get_string(&ctx, 1, "from", "string.replace")?;
    let to = get_string(&ctx, 2, "to", "string.replace")?;
    Ok(Value::String(Rc::from(
        s.replace(from.as_ref(), to.as_ref()).as_str(),
    )))
}

fn string_replace_first(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.replace_first")?;
    let from = get_string(&ctx, 1, "from", "string.replace_first")?;
    let to = get_string(&ctx, 2, "to", "string.replace_first")?;
    Ok(Value::String(Rc::from(
        s.replacen(from.as_ref(), to.as_ref(), 1).as_str(),
    )))
}

fn string_slice(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.slice")?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    let resolve =
        |raw: i64| -> usize { (if raw < 0 { len + raw } else { raw }).clamp(0, len) as usize };

    let start = match ctx.get(1, "start") {
        Value::Number(n) => resolve(*n as i64),
        Value::Nil => 0,
        _ => {
            return Err(ctx.error(NativeError::new(
                "type error",
                "string.slice: 'start' must be a number".into(),
            )));
        }
    };
    let end = match ctx.get(2, "end") {
        Value::Number(n) => resolve(*n as i64),
        Value::Nil => chars.len(),
        _ => {
            return Err(ctx.error(NativeError::new(
                "type error",
                "string.slice: 'end' must be a number".into(),
            )));
        }
    };
    let lo = start.min(end);
    let hi = start.max(end);
    Ok(Value::String(Rc::from(
        chars[lo..hi].iter().collect::<String>().as_str(),
    )))
}

fn string_find(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.find")?;
    let sub = get_string(&ctx, 1, "needle", "string.find")?;
    match s.find(sub.as_ref()) {
        Some(byte_idx) => {
            let char_idx = s[..byte_idx].chars().count();
            Ok(Value::Number(char_idx as f64))
        }
        None => Ok(Value::Nil),
    }
}

fn string_repeat(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.repeat")?;
    let n = get_number(&ctx, 1, "n", "string.repeat")? as usize;
    Ok(Value::String(Rc::from(s.repeat(n).as_str())))
}

fn string_reverse(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.reverse")?;
    Ok(Value::String(Rc::from(
        s.chars().rev().collect::<String>().as_str(),
    )))
}

fn string_len(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.len")?;
    Ok(Value::Number(s.chars().count() as f64))
}

fn string_bytes(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.bytes")?;
    Ok(Value::Number(s.len() as f64))
}

fn string_char_at(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.char_at")?;
    let chars: Vec<char> = s.chars().collect();
    let idx = get_number(&ctx, 1, "index", "string.char_at")?;
    match chars.get(idx as usize) {
        Some(c) => Ok(Value::String(Rc::from(c.to_string().as_str()))),
        None => Ok(Value::Nil),
    }
}

fn string_pad_start(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.pad_start")?;
    let width = get_number(&ctx, 1, "width", "string.pad_start")? as usize;
    let fill = get_fill_char(&ctx, "string.pad_start")?;
    Ok(Value::String(Rc::from(
        apply_pad(&s, width, fill, true).as_str(),
    )))
}

fn string_pad_end(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "string.pad_end")?;
    let width = get_number(&ctx, 1, "width", "string.pad_end")? as usize;
    let fill = get_fill_char(&ctx, "string.pad_end")?;
    Ok(Value::String(Rc::from(
        apply_pad(&s, width, fill, false).as_str(),
    )))
}

fn get_fill_char(ctx: &CallContext, fn_name: &str) -> Result<char, Signal> {
    match ctx.get(2, "fill") {
        Value::String(c) => Ok(c.chars().next().unwrap_or(' ')),
        Value::Nil => Ok(' '),
        _ => Err(ctx.error(NativeError::new(
            "type error",
            format!("{fn_name}: 'fill' must be a string"),
        ))),
    }
}

fn apply_pad(s: &str, width: usize, fill: char, at_start: bool) -> String {
    let cur_len = s.chars().count();
    if cur_len >= width {
        return s.to_string();
    }
    let padding: String = std::iter::repeat(fill).take(width - cur_len).collect();
    if at_start {
        format!("{padding}{s}")
    } else {
        format!("{s}{padding}")
    }
}
