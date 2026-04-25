//! @core/path — OS-aware path operations that require native filesystem APIs.
//! path.join(base, …) builds a path with the OS separator using PathBuf::push semantics.
//! path.absolute(p) canonicalises to an absolute path, falling back to cwd/p on error.
//! path.is_absolute(p) returns true for paths that are absolute on the current platform.
//! Pure string decomposition (basename, dirname, ext, stem, normalize) lives in @std/path.

use std::{path::Path, rc::Rc};

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "path.join", path_join);
    define_in(&t, "path.absolute", path_absolute);
    define_in(&t, "path.is_absolute", path_is_absolute);
    Value::Table(t)
}

// Accepts any number of positional string segments and builds a path using the
// OS separator.  Mirrors `PathBuf::push` semantics: if a later segment is
// absolute it replaces the accumulated path (rather than being appended).
//
//   path.join("/usr", "local", "bin")  →  "/usr/local/bin"
//   path.join("/usr", "/etc")          →  "/etc"   (absolute override, Unix)

fn path_join(ctx: CallContext) -> Result<Value, Signal> {
    if ctx.args.is_empty() {
        return Ok(Value::String(Rc::from("")));
    }
    let base = get_string(&ctx, 0, "base", "path.join")?;
    let mut p = std::path::PathBuf::from(base.as_ref());
    for i in 1..ctx.args.len() {
        match &ctx.args[i] {
            Value::String(s) => p.push(s.as_ref()),
            other => {
                return Err(ctx.error(NativeError::new(
                    "type error",
                    format!(
                        "path.join: all segments must be strings, got {}",
                        other.type_name()
                    ),
                )));
            }
        }
    }
    Ok(Value::String(Rc::from(p.to_string_lossy().as_ref())))
}

fn path_absolute(ctx: CallContext) -> Result<Value, Signal> {
    let p = get_string(&ctx, 0, "path", "path.absolute")?;
    let abs = std::fs::canonicalize(p.as_ref())
        .map(|pb| pb.to_string_lossy().into_owned())
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|cwd| cwd.join(p.as_ref()).to_string_lossy().into_owned())
                .unwrap_or_else(|_| p.to_string())
        });
    Ok(Value::String(Rc::from(abs.as_str())))
}

fn path_is_absolute(ctx: CallContext) -> Result<Value, Signal> {
    let p = get_string(&ctx, 0, "path", "path.is_absolute")?;
    Ok(Value::Boolean(Path::new(p.as_ref()).is_absolute()))
}
