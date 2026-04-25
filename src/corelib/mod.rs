//! Registry of all @core/ built-in modules available via require("@core/<name>").
//! get_corelib(key) constructs the module Table on demand; caching is handled by Runtime.
//! register_builtins() injects global functions (print, error, len, type, …) before any script.
//! Adding a new core module: implement create() in a submodule, add an arm in get_corelib.
//! Standard-library Risc modules live under stdlib/ and are embedded at build time by build.rs.

use crate::value::Value;

mod builtin;
mod exec;
mod helpers;
mod http;
mod json;
mod os;
mod path;
mod regex;
mod string;
mod table;
mod utf8;

pub use builtin::register_builtins;

pub fn get_corelib(key: &str) -> Option<Value> {
    match key {
        "exec" => Some(self::exec::create()),
        "http" => Some(self::http::create()),
        "json" => Some(self::json::create()),
        "os" => Some(self::os::create()),
        "path" => Some(self::path::create()),
        "regex" => Some(self::regex::create()),
        "string" => Some(self::string::create()),
        "table" => Some(self::table::create()),
        "utf8" => Some(self::utf8::create()),
        _ => None,
    }
}
