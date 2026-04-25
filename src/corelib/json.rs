//! @core/json — JSON parse and stringify built on tinyjson.
//! json.parse(s) converts a JSON string into a Risc value tree (arrays→tables, objects→tables).
//! json.stringify(v) and json.stringify(v, indent) serialise back to compact or pretty JSON.
//! Tables with dense 0-based integer keys serialise as JSON arrays; all others as objects.
//! Custom serialisers avoid the serde_json macro dependency; output is deterministic (sorted keys).

use std::{collections::HashMap, rc::Rc};

use tinyjson::JsonValue;

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "json.parse", json_parse);
    define_in(&t, "json.stringify", json_stringify);
    Value::Table(t)
}

fn json_parse(ctx: CallContext) -> Result<Value, Signal> {
    let s = get_string(&ctx, 0, "s", "json.parse")?;
    let jv: JsonValue = s
        .parse()
        .map_err(|e| ctx.error(NativeError::new("json error", format!("json.parse: {e}"))))?;
    Ok(json_to_risc(jv))
}

fn json_to_risc(jv: JsonValue) -> Value {
    match jv {
        JsonValue::Null => Value::Nil,
        JsonValue::Boolean(b) => Value::Boolean(b),
        JsonValue::Number(n) => Value::Number(n),
        JsonValue::String(s) => Value::String(Rc::from(s.as_str())),
        JsonValue::Array(arr) => {
            Value::Table(Table::from_vec(arr.into_iter().map(json_to_risc).collect()))
        }
        JsonValue::Object(obj) => {
            let mut t = Table::new();
            for (k, v) in obj {
                t.set(k.as_str(), json_to_risc(v));
            }
            Value::Table(t)
        }
    }
}

fn json_stringify(ctx: CallContext) -> Result<Value, Signal> {
    let value = ctx.get(0, "value");
    let indent: Option<usize> = match ctx.get(1, "indent") {
        Value::Number(n) => Some(*n as usize),
        Value::Nil => None,
        other => {
            return Err(ctx.error(NativeError::new(
                "type error",
                format!(
                    "json.stringify: 'indent' must be a number, got {}",
                    other.type_name()
                ),
            )));
        }
    };
    let jv = risc_to_json(value).map_err(|msg| {
        ctx.error(NativeError::new(
            "json error",
            format!("json.stringify: {msg}"),
        ))
    })?;
    let out = match indent {
        None => stringify_compact(&jv),
        Some(n) => stringify_pretty(&jv, n, 0),
    };
    Ok(Value::String(Rc::from(out.as_str())))
}

fn stringify_compact(jv: &JsonValue) -> String {
    match jv {
        JsonValue::Null => "null".to_owned(),
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Number(n) => format_number(*n),
        JsonValue::String(s) => json_escape_string(s),
        JsonValue::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(stringify_compact).collect();
            format!("[{}]", parts.join(","))
        }
        JsonValue::Object(obj) => {
            let mut pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}:{}", json_escape_string(k), stringify_compact(v)))
                .collect();
            pairs.sort(); // deterministic output regardless of HashMap order
            format!("{{{}}}", pairs.join(","))
        }
    }
}

fn stringify_pretty(jv: &JsonValue, indent: usize, depth: usize) -> String {
    let pad = " ".repeat(indent * depth);
    let inner = " ".repeat(indent * (depth + 1));
    match jv {
        JsonValue::Null => "null".to_owned(),
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Number(n) => format_number(*n),
        JsonValue::String(s) => json_escape_string(s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_owned();
            }
            let parts: Vec<String> = arr
                .iter()
                .map(|v| format!("{inner}{}", stringify_pretty(v, indent, depth + 1)))
                .collect();
            format!("[\n{}\n{pad}]", parts.join(",\n"))
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_owned();
            }
            let mut pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{inner}{}: {}",
                        json_escape_string(k),
                        stringify_pretty(v, indent, depth + 1)
                    )
                })
                .collect();
            pairs.sort();
            format!("{{\n{}\n{pad}}}", pairs.join(",\n"))
        }
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.is_finite() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n:?}")
    }
}

fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn risc_to_json(value: &Value) -> Result<JsonValue, String> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Boolean(*b)),
        Value::Number(n) => {
            if n.is_finite() {
                Ok(JsonValue::Number(*n))
            } else {
                Err(format!("cannot serialise {n} as a JSON number"))
            }
        }
        Value::String(s) => Ok(JsonValue::String(s.to_string())),
        Value::Table(t) => table_to_json(t),
        other => Err(format!("cannot serialise {} as JSON", other.type_name())),
    }
}

fn table_to_json(table: &Table) -> Result<JsonValue, String> {
    let entries = table.entries();
    let len = entries.len();

    let mut int_keys: Vec<i64> = entries
        .iter()
        .filter_map(|(k, _)| {
            if let TableKey::Integer(i) = k {
                Some(*i)
            } else {
                None
            }
        })
        .collect();
    int_keys.sort_unstable();

    let is_array = int_keys.len() == len
        && int_keys.first().copied() == Some(0)
        && int_keys.last().copied() == Some((len as i64) - 1);

    if is_array {
        let mut pairs: Vec<(i64, &Value)> = entries
            .iter()
            .filter_map(|(k, v)| {
                if let TableKey::Integer(i) = k {
                    Some((*i, v))
                } else {
                    None
                }
            })
            .collect();
        pairs.sort_by_key(|(i, _)| *i);
        let arr: Result<Vec<JsonValue>, _> = pairs.iter().map(|(_, v)| risc_to_json(v)).collect();
        Ok(JsonValue::Array(arr?))
    } else {
        let mut map = HashMap::new();
        for (k, v) in &entries {
            let key = match k {
                TableKey::String(s) => s.to_string(),
                TableKey::Integer(i) => i.to_string(),
                TableKey::Boolean(b) => b.to_string(),
            };
            map.insert(key, risc_to_json(v)?);
        }
        Ok(JsonValue::Object(map))
    }
}
