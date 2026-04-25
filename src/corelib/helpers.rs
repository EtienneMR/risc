//! Shared helpers used by all @core/ module implementations.
//! get_string / get_number / get_bool extract typed positional or named args from a CallContext,
//! wrapping type errors with the function name and argument name for clear diagnostic messages.
//! define_in registers a native function into a Table under the last dotted segment of its name,
//! so "string.upper" registers as key "upper" inside the string module table.

use crate::{
    error::NativeError,
    value::{CallContext, Function, NativeFunction, Signal, StrRef, Table, Value},
};

pub fn get_string(
    ctx: &CallContext,
    index: usize,
    name: &str,
    fn_name: &str,
) -> Result<StrRef, Signal> {
    ctx.get(index, name).as_string_ref().map_err(|e| {
        ctx.error(NativeError::new(
            e.kind,
            format!("{fn_name}: argument '{name}' {}", e.message),
        ))
    })
}

pub fn get_number(
    ctx: &CallContext,
    index: usize,
    name: &str,
    fn_name: &str,
) -> Result<f64, Signal> {
    ctx.get(index, name).as_number().map_err(|e| {
        ctx.error(NativeError::new(
            e.kind,
            format!("{fn_name}: argument '{name}' {}", e.message),
        ))
    })
}

pub fn get_bool(
    ctx: &CallContext,
    index: usize,
    name: &str,
    fn_name: &str,
) -> Result<bool, Signal> {
    ctx.get(index, name).as_boolean().map_err(|e| {
        ctx.error(NativeError::new(
            e.kind,
            format!("{fn_name}: argument '{name}' {}", e.message),
        ))
    })
}

pub fn define_in(
    table: &Table,
    name: &'static str,
    func: fn(CallContext) -> Result<Value, Signal>,
) {
    let key = name.rsplit_once('.').map(|(_, r)| r).unwrap_or(name);
    let mut t = table.clone();
    t.set(
        key,
        Value::Function(Function::Native(NativeFunction { name, func })),
    );
}
