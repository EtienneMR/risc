//! @core/table — minimal table introspection primitives for the runtime.
//! table.raw_keys(t) returns a 0-indexed array of all keys present in t (any key type).
//! Key ordering is unspecified; sort the result if a stable order is required.
//! Higher-level helpers (values, items, from, clone) live in @std/table.
//! This module is intentionally small — most table logic belongs in Risc, not Rust.

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, Value},
};

use super::helpers::define_in;

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "table.raw_keys", table_raw_keys);
    Value::Table(t)
}

/// table.keys(t) → 0-indexed array of all keys in t.
fn table_raw_keys(ctx: CallContext) -> Result<Value, Signal> {
    let tbl = match ctx.get(0, "t") {
        Value::Table(t) => t.clone(),
        other => {
            return Err(ctx.error(NativeError::new(
                "type error",
                format!("table.keys: expected table, got {}", other.type_name()),
            )));
        }
    };

    let keys: Vec<Value> = tbl
        .entries()
        .into_iter()
        .map(|(k, _)| Value::from(k))
        .collect();
    Ok(Value::Table(Table::from_vec(keys)))
}
