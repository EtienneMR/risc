//! @core/os — filesystem, environment, and process utilities.
//! File I/O: read, write, append, copy, rename, remove.
//! Directory ops: mkdir (recursive flag), rmdir (recursive flag), list.
//! Environment: cwd, chdir, env, setenv, unsetenv, args, platform, sep.
//! os.stat returns {is_file, is_dir, size, modified} for a given path.

use std::{rc::Rc, time::UNIX_EPOCH};

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, Value},
};

use super::helpers::{define_in, get_bool, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "os.read", os_read);
    define_in(&t, "os.write", os_write);
    define_in(&t, "os.append", os_append);
    define_in(&t, "os.copy", os_copy);
    define_in(&t, "os.rename", os_rename);
    define_in(&t, "os.remove", os_remove);
    define_in(&t, "os.mkdir", os_mkdir);
    define_in(&t, "os.rmdir", os_rmdir);
    define_in(&t, "os.list", os_list);
    define_in(&t, "os.exists", os_exists);
    define_in(&t, "os.stat", os_stat);
    define_in(&t, "os.cwd", os_cwd);
    define_in(&t, "os.chdir", os_chdir);
    define_in(&t, "os.env", os_env);
    define_in(&t, "os.setenv", os_setenv);
    define_in(&t, "os.unsetenv", os_unsetenv);
    define_in(&t, "os.args", os_args);
    define_in(&t, "os.platform", os_platform);
    define_in(&t, "os.sep", os_sep);
    Value::Table(t)
}

fn os_read(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.read")?;
    std::fs::read_to_string(path.as_ref())
        .map(|s| Value::String(Rc::from(s.as_str())))
        .map_err(|e| os_err(&ctx, format!("os.read: {e}")))
}

fn os_write(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.write")?;
    let content = ctx.get(1, "content").to_string_ref();
    std::fs::write(path.as_ref(), content.as_bytes())
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.write: {e}")))
}

fn os_append(ctx: CallContext) -> Result<Value, Signal> {
    use std::io::Write;
    let path = get_string(&ctx, 0, "path", "os.append")?;
    let content = ctx.get(1, "content").to_string_ref();
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.append: {e}")))
}

fn os_copy(ctx: CallContext) -> Result<Value, Signal> {
    let from = get_string(&ctx, 0, "from", "os.copy")?;
    let to = get_string(&ctx, 1, "to", "os.copy")?;
    std::fs::copy(from.as_ref(), to.as_ref())
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.copy: {e}")))
}

fn os_rename(ctx: CallContext) -> Result<Value, Signal> {
    let from = get_string(&ctx, 0, "from", "os.rename")?;
    let to = get_string(&ctx, 1, "to", "os.rename")?;
    std::fs::rename(from.as_ref(), to.as_ref())
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.rename: {e}")))
}

fn os_remove(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.remove")?;
    let p = std::path::Path::new(path.as_ref());
    let result = if p.is_dir() {
        std::fs::remove_dir(p)
    } else {
        std::fs::remove_file(p)
    };
    result
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.remove: {e}")))
}

fn os_mkdir(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.mkdir")?;
    let recursive = get_bool(&ctx, 1, "recursive", "os.mkdir")?;
    let result = if recursive {
        std::fs::create_dir_all(path.as_ref())
    } else {
        std::fs::create_dir(path.as_ref())
    };
    result
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.mkdir: {e}")))
}

fn os_rmdir(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.rmdir")?;
    let recursive = get_bool(&ctx, 1, "recursive", "os.rmdir")?;
    let result = if recursive {
        std::fs::remove_dir_all(path.as_ref())
    } else {
        std::fs::remove_dir(path.as_ref())
    };
    result
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.rmdir: {e}")))
}

fn os_list(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.list")?;
    let full_path = get_bool(&ctx, 1, "full_path", "os.list")?;

    let entries =
        std::fs::read_dir(path.as_ref()).map_err(|e| os_err(&ctx, format!("os.list: {e}")))?;

    let mut items: Vec<Value> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| os_err(&ctx, format!("os.list: {e}")))?;
        let name = if full_path {
            entry.path().to_string_lossy().into_owned()
        } else {
            entry.file_name().to_string_lossy().into_owned()
        };
        items.push(Value::String(Rc::from(name.as_str())));
    }
    items.sort_unstable_by(|a, b| {
        let sa = if let Value::String(s) = a {
            s.as_ref()
        } else {
            ""
        };
        let sb = if let Value::String(s) = b {
            s.as_ref()
        } else {
            ""
        };
        sa.cmp(sb)
    });
    Ok(Value::Table(Table::from_vec(items)))
}

fn os_exists(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.exists")?;
    Ok(Value::Boolean(std::path::Path::new(path.as_ref()).exists()))
}

fn os_stat(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.stat")?;
    let meta =
        std::fs::metadata(path.as_ref()).map_err(|e| os_err(&ctx, format!("os.stat: {e}")))?;

    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| Value::Number(d.as_secs_f64()))
        .unwrap_or(Value::Nil);

    let mut t = Table::new();
    t.set("is_file", Value::Boolean(meta.is_file()));
    t.set("is_dir", Value::Boolean(meta.is_dir()));
    t.set("size", Value::Number(meta.len() as f64));
    t.set("modified", modified);
    Ok(Value::Table(t))
}

fn os_cwd(_ctx: CallContext) -> Result<Value, Signal> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Value::String(Rc::from(cwd.as_str())))
}

fn os_chdir(ctx: CallContext) -> Result<Value, Signal> {
    let path = get_string(&ctx, 0, "path", "os.chdir")?;
    std::env::set_current_dir(path.as_ref())
        .map(|_| Value::Nil)
        .map_err(|e| os_err(&ctx, format!("os.chdir: {e}")))
}

fn os_env(ctx: CallContext) -> Result<Value, Signal> {
    let name = get_string(&ctx, 0, "name", "os.env")?;
    Ok(match std::env::var(name.as_ref()) {
        Ok(val) => Value::String(Rc::from(val.as_str())),
        Err(_) => Value::Nil,
    })
}

fn os_setenv(ctx: CallContext) -> Result<Value, Signal> {
    let name = get_string(&ctx, 0, "name", "os.setenv")?;
    let value = get_string(&ctx, 1, "value", "os.setenv")?;
    unsafe { std::env::set_var(name.as_ref(), value.as_ref()) }
    Ok(Value::Nil)
}

fn os_unsetenv(ctx: CallContext) -> Result<Value, Signal> {
    let name = get_string(&ctx, 0, "name", "os.unsetenv")?;
    unsafe { std::env::remove_var(name.as_ref()) }
    Ok(Value::Nil)
}

fn os_args(_ctx: CallContext) -> Result<Value, Signal> {
    let args: Vec<Value> = std::env::args()
        .map(|a| Value::String(Rc::from(a.as_str())))
        .collect();
    Ok(Value::Table(Table::from_vec(args)))
}

fn os_platform(_ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(Rc::from(std::env::consts::OS)))
}

fn os_sep(_ctx: CallContext) -> Result<Value, Signal> {
    Ok(Value::String(Rc::from(
        std::path::MAIN_SEPARATOR.to_string().as_str(),
    )))
}

pub fn os_err(ctx: &CallContext, msg: impl Into<String>) -> Signal {
    ctx.error(NativeError::new("os error", msg.into()))
}
