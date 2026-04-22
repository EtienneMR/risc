//! @core/exec — run external processes and collect or inherit their output.
//! exec.run / exec.shell capture stdout and stderr into {code, stdout, stderr}.
//! exec.spawn / exec.shell_spawn inherit the parent stdio for interactive use.
//! All variants accept optional named args: env={key=val}, cwd="path".
//! spawn variants return only {code}; no output is captured.

use std::{process::Stdio, rc::Rc};

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "exec.run", exec_run);
    define_in(&t, "exec.shell", exec_shell);
    define_in(&t, "exec.spawn", exec_spawn);
    define_in(&t, "exec.shell_spawn", exec_shell_spawn);
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

fn build_command(
    ctx: &CallContext,
    program: &str,
    env_arg: usize,
    cwd_arg: usize,
) -> std::process::Command {
    let args: Vec<String> = ctx
        .args
        .iter()
        .skip(1)
        .map(|v| v.to_string_ref().to_string())
        .collect();

    let env_vars: Vec<(String, String)> = match ctx.get(env_arg, "env") {
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

    let cwd: Option<String> = match ctx.get(cwd_arg, "cwd") {
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
    build_command(&ctx, program.as_ref(), 2, 3)
        .output()
        .map(collect_output)
        .map_err(|e| ctx.error(NativeError::new("exec error", format!("exec.run: {e}"))))
}

fn exec_shell(ctx: CallContext) -> Result<Value, Signal> {
    let cmd = get_string(&ctx, 0, "cmd", "exec.shell")?;
    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };
    std::process::Command::new(shell)
        .arg(flag)
        .arg(cmd.as_ref())
        .output()
        .map(collect_output)
        .map_err(|e| ctx.error(NativeError::new("exec error", format!("exec.shell: {e}"))))
}

fn exec_spawn(ctx: CallContext) -> Result<Value, Signal> {
    let program = get_string(&ctx, 0, "cmd", "exec.spawn")?;
    let status = build_command(&ctx, program.as_ref(), 2, 3)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| ctx.error(NativeError::new("exec error", format!("exec.spawn: {e}"))))?;
    Ok(exit_code(status))
}

fn exec_shell_spawn(ctx: CallContext) -> Result<Value, Signal> {
    let cmd = get_string(&ctx, 0, "cmd", "exec.shell_spawn")?;
    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };
    let status = std::process::Command::new(shell)
        .arg(flag)
        .arg(cmd.as_ref())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            ctx.error(NativeError::new(
                "exec error",
                format!("exec.shell_spawn: {e}"),
            ))
        })?;
    Ok(exit_code(status))
}
