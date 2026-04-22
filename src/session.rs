//! Session manages module loading, source tracking, and the stdlib embed.
//! ModuleKind distinguishes @core/ built-ins, @std/ embedded sources, and user paths.
//! Parsed modules are cached by ModuleKind to avoid re-parsing on repeated require().
//! The stdlib directory is compiled into the binary at build time via build.rs.
//! SourceOutcome::Cached short-circuits re-evaluation for already-loaded modules.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    ast::Program,
    corelib::get_corelib,
    error::NativeError,
    lexer::Lexer,
    parser::Parser,
    source::{SourceMap, Span},
    value::{Signal, Value},
};

mod stdlib {
    include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));
}

pub struct Session {
    pub source_map: SourceMap,

    module_cache: HashMap<ModuleKind, Value>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            source_map: SourceMap::new(),
            module_cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, kind: &ModuleKind, call_span: Span) -> Result<SourceOutcome, Signal> {
        if let Some(cached) = self.module_cache.get(&kind) {
            return Ok(SourceOutcome::Cached(cached.clone()));
        }
        match kind {
            ModuleKind::Core(name) => {
                let lib = get_corelib(name).ok_or_else(|| {
                    Signal::from_error(
                        NativeError::new(
                            "module error",
                            format!("core library module not found: {name}"),
                        ),
                        call_span,
                    )
                })?;
                self.cache_module(kind.clone(), lib.clone());
                Ok(SourceOutcome::Cached(lib))
            }

            ModuleKind::Std(path) => {
                let source_text = stdlib::get(path).ok_or_else(|| {
                    Signal::from_error(
                        NativeError::new(
                            "module error",
                            format!("standard library module not found: {path}"),
                        ),
                        call_span,
                    )
                })?;
                self.parse_source(format!("@std/{path}"), source_text.to_string())
                    .map(SourceOutcome::ParsedProgram)
            }

            ModuleKind::UserPath(path) => {
                let display = path.display().to_string();
                let source_text = std::fs::read_to_string(path).map_err(|e| {
                    Signal::from_error(
                        NativeError::new(
                            "module error",
                            format!("cannot read module '{display}': {e}"),
                        ),
                        call_span,
                    )
                })?;
                self.parse_source(display, source_text)
                    .map(SourceOutcome::ParsedProgram)
            }
        }
    }

    fn parse_source(&mut self, name: String, source_text: String) -> Result<Program, Signal> {
        let source = self.source_map.add(name, source_text);
        Parser::new(Lexer::new(source)).parse().map_err(|e| {
            Signal::from_error(NativeError::new("parse error", format!("{e}")), e.span)
        })
    }

    pub fn cache_module(&mut self, key: ModuleKind, value: Value) {
        self.module_cache.insert(key, value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    Core(String),
    Std(String),
    UserPath(PathBuf),
}

impl ModuleKind {
    pub fn from(raw: &str) -> Self {
        if let Some(name) = raw.strip_prefix("@core/") {
            return Self::Core(name.to_string());
        }
        if let Some(name) = raw.strip_prefix("@std/") {
            return Self::Std(name.to_string());
        }
        Self::UserPath(Self::resolve_user_path(raw))
    }

    fn resolve_user_path(raw: &str) -> PathBuf {
        let p = Path::new(raw);
        let base = if p.is_absolute() {
            p.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(p)
        };

        let extended = if base.extension().is_none() {
            base.with_extension("ri")
        } else {
            base
        };

        extended.canonicalize().unwrap_or(extended)
    }
}

pub enum SourceOutcome {
    Cached(Value),
    ParsedProgram(Program),
}
