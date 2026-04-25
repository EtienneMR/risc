//! @core/exec — run external processes and capture or inherit their output.
//! run(cmd, …) and shell(cmd) capture {code, stdout, stderr}; shell routes via /bin/sh -c.
//! spawn(cmd, …) and shell_spawn(cmd) inherit parent stdio for interactive use, returning {code}.
//! All variants accept optional named args: env={key=val}, cwd="path".
//! Prefer @std/exec when writing Risc scripts; shell/shell_spawn move there in the stdlib.

use std::{process::Stdio, rc::Rc};

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "exec.run", exec_run);
    define_in(&t, "exec.spawn", exec_spawn);
    Value::Table(t)
}

fn collect_output(output: std::process::Output) -> Value {
    let mut t = Table::new();
    t.set(
        "code",
        Value::Number(output.status.code().unwrap_or(-1) as f64),
    );
    t.set(
        "stdout",
        Value::String(Rc::from(String::from_utf8_lossy(&output.stdout).as_ref())),
    );
    t.set(
        "stderr",
        Value::String(Rc::from(String::from_utf8_lossy(&output.stderr).as_ref())),
    );
    Value::Table(t)
}

fn exit_code(status: std::process::ExitStatus) -> Value {
    let mut t = Table::new();
    t.set("code", Value::Number(status.code().unwrap_or(-1) as f64));
    Value::Table(t)
}

fn build_command(ctx: &CallContext, program: &str) -> std::process::Command {
    let args: Vec<String> = ctx
        .args
        .iter()
        .skip(1)
        .map(|v| v.to_string_ref().to_string())
        .collect();

    let env_vars: Vec<(String, String)> = match ctx.named("env") {
        Value::Table(t) => t
            .entries()
            .into_iter()
            .filter_map(|(k, v)| match k {
                TableKey::String(s) => Some((s.to_string(), v.to_string_ref().to_string())),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };

    let cwd: Option<String> = match ctx.named("cwd") {
        Value::String(s) => Some(s.to_string()),
        _ => None,
    };

    let mut cmd = std::process::Command::new(program);
    cmd.args(&args);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    cmd
}

fn exec_run(ctx: CallContext) -> Result<Value, Signal> {
    let program = get_string(&ctx, 0, "cmd", "exec.run")?;
    build_command(&ctx, program.as_ref())
        .output()
        .map(collect_output)
        .map_err(|e| ctx.error(NativeError::new("exec error", format!("exec.run: {e}"))))
}

fn exec_spawn(ctx: CallContext) -> Result<Value, Signal> {
    let program = get_string(&ctx, 0, "cmd", "exec.spawn")?;
    let status = build_command(&ctx, program.as_ref())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| ctx.error(NativeError::new("exec error", format!("exec.spawn: {e}"))))?;
    Ok(exit_code(status))
}
