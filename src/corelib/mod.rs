//! Registry of all @core/ built-in modules.
//! get_corelib(key) constructs the module Table on demand; no caching here —
//! caching is handled by Session::module_cache.
//! register_builtins() injects global functions (print, error, len, type, …)
//! into the top-level environment before any script runs.

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
        "utf8" => Some(self::utf8::create()),
        _ => None,
    }
}
