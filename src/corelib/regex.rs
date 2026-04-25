//! @core/regex — regular expression matching via regex-lite (RE2-compatible, no backtracking).
//! Match tables carry {text, start, end} with codepoint (not byte) offsets.
//! Captures tables use integer keys for groups (0=full match) and strings for named groups.
//! replace / replace_all support $1 / $name back-references in the replacement string.
//! Higher-level scan_all and scan_group helpers live in @std/regex.

use std::rc::Rc;

use regex_lite::{Captures, Regex};

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();

    define_in(&t, "regex.test", regex_test);
    define_in(&t, "regex.find", regex_find);
    define_in(&t, "regex.find_all", regex_find_all);
    define_in(&t, "regex.captures", regex_captures);
    define_in(&t, "regex.replace", regex_replace);
    define_in(&t, "regex.replace_all", regex_replace_all);
    define_in(&t, "regex.split", regex_split);

    Value::Table(t)
}

fn compile(ctx: &CallContext, pattern: &str) -> Result<Regex, Signal> {
    Regex::new(pattern).map_err(|e| {
        ctx.error(NativeError::new(
            "regex error",
            format!("invalid pattern: {e}"),
        ))
    })
}

fn byte_to_char(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset].chars().count()
}

fn make_match_table(text: &str, full: &str, byte_start: usize, byte_end: usize) -> Value {
    let mut t = Table::new();
    t.set("text", Value::String(Rc::from(full)));
    t.set(
        "start",
        Value::Number(byte_to_char(text, byte_start) as f64),
    );
    t.set("end", Value::Number(byte_to_char(text, byte_end) as f64));
    Value::Table(t)
}

fn make_captures_table(re: &Regex, caps: &Captures<'_>) -> Value {
    let mut t = Table::new();

    for (i, m) in caps.iter().enumerate() {
        let val = match m {
            Some(m) => Value::String(Rc::from(m.as_str())),
            None => Value::Nil,
        };
        t.set(TableKey::Integer(i as i64), val);
    }

    for name in re.capture_names().flatten() {
        if let Some(m) = caps.name(name) {
            t.set(name, Value::String(Rc::from(m.as_str())));
        }
    }

    Value::Table(t)
}

fn regex_test(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.test")?;
    let s = get_string(&ctx, 1, "s", "regex.test")?;
    let re = compile(&ctx, &pat)?;
    Ok(Value::Boolean(re.is_match(&s)))
}

fn regex_find(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.find")?;
    let s = get_string(&ctx, 1, "s", "regex.find")?;
    let re = compile(&ctx, &pat)?;
    Ok(match re.find(&s) {
        Some(m) => make_match_table(&s, m.as_str(), m.start(), m.end()),
        None => Value::Nil,
    })
}

fn regex_find_all(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.find_all")?;
    let s = get_string(&ctx, 1, "s", "regex.find_all")?;
    let re = compile(&ctx, &pat)?;
    let matches: Vec<Value> = re
        .find_iter(&s)
        .map(|m| make_match_table(&s, m.as_str(), m.start(), m.end()))
        .collect();
    Ok(Value::Table(Table::from_vec(matches)))
}

fn regex_captures(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.captures")?;
    let s = get_string(&ctx, 1, "s", "regex.captures")?;
    let re = compile(&ctx, &pat)?;
    Ok(match re.captures(&s) {
        Some(caps) => make_captures_table(&re, &caps),
        None => Value::Nil,
    })
}

fn regex_replace(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.replace")?;
    let s = get_string(&ctx, 1, "s", "regex.replace")?;
    let repl = get_string(&ctx, 2, "replacement", "regex.replace")?;
    let re = compile(&ctx, &pat)?;
    Ok(Value::String(Rc::from(
        re.replacen(&s, 1, repl.as_ref()).as_ref(),
    )))
}

fn regex_replace_all(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.replace_all")?;
    let s = get_string(&ctx, 1, "s", "regex.replace_all")?;
    let repl = get_string(&ctx, 2, "replacement", "regex.replace_all")?;
    let re = compile(&ctx, &pat)?;
    Ok(Value::String(Rc::from(
        re.replace_all(&s, repl.as_ref()).as_ref(),
    )))
}

fn regex_split(ctx: CallContext) -> Result<Value, Signal> {
    let pat = get_string(&ctx, 0, "pattern", "regex.split")?;
    let s = get_string(&ctx, 1, "s", "regex.split")?;
    let re = compile(&ctx, &pat)?;
    let parts: Vec<Value> = re.split(&s).map(|p| Value::String(Rc::from(p))).collect();
    Ok(Value::Table(Table::from_vec(parts)))
}
