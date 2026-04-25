//! Runtime links the parser, interpreter, source map, and module cache together.
//! ModuleKind distinguishes @core/ built-ins, @std/ embedded sources, and user file paths.
//! Parsed modules are cached by ModuleKind so repeated require() calls pay no re-parse cost.
//! The stdlib directory is compiled into the binary at build time via build.rs / OUT_DIR.
//! Runtime::new() runs the prelude; run() and run_repl() are the two public entry points.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    ast::Program,
    corelib::{get_corelib, register_builtins},
    error::{LangError, NativeError},
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    source::{SourceMap, Span},
    value::{Env, EnvRef, Signal, Value},
};

mod stdlib {
    include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));
}

pub struct Runtime {
    source_map: SourceMap,
    module_cache: HashMap<ModuleKind, Value>,
    global_env: EnvRef,
}

impl Runtime {
    pub fn new() -> Self {
        let global_env = Env::new();
        register_builtins(&global_env);

        let mut this = Self {
            source_map: SourceMap::new(),
            module_cache: HashMap::new(),
            global_env,
        };

        if let Some(src) = stdlib::get("prelude") {
            let env = this.global_env.clone();
            if let Err(e) = this.run_global("@std/prelude".to_string(), src.to_string(), env) {
                panic!("risc: prelude error: {e:?}");
            }
        }

        this
    }

    pub fn create_module_env(&self) -> EnvRef {
        self.global_env.inner()
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn run(&mut self, name: String, source_text: String) -> Result<Value, LangError> {
        let program = self.parse(name, source_text)?;
        self.eval(program, false, self.create_module_env())
    }

    pub fn run_repl(&mut self, source_text: String, env: EnvRef) -> Result<Value, LangError> {
        let program = self.parse("<repl>".to_owned(), source_text)?;
        self.eval(program, true, env)
    }

    fn run_global(
        &mut self,
        name: String,
        source_text: String,
        env: EnvRef,
    ) -> Result<Value, LangError> {
        let program = self.parse(name, source_text)?;
        self.eval(program, false, env)
    }

    pub fn load_module(&mut self, path: &str, call_span: Span) -> Result<Value, Signal> {
        let kind: ModuleKind = ModuleKind::from(path);

        if let Some(cached) = self.module_cache.get(&kind) {
            return Ok(cached.clone());
        }

        let module = match &kind {
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
                Ok(lib)
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
                self.run(format!("@std/{path}"), source_text.to_string())
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
                self.run(display, source_text)
            }
        }
        .map_err(|e| Signal {
            kind: match e.kind {
                crate::error::LangErrorKind::RuntimeError { subkind } => {
                    crate::value::SignalKind::Error {
                        kind: subkind.into(),
                        message: e.message.into(),
                    }
                }
                _ => crate::value::SignalKind::Error {
                    kind: "parse error".into(),
                    message: e.extract().into(),
                },
            },
            traceback: e.traceback,
        })?;

        self.module_cache.insert(kind, module.clone());
        Ok(module)
    }

    fn parse(&mut self, name: String, source_text: String) -> Result<Program, LangError> {
        let source = self.source_map.add(name, source_text);

        Parser::new(Lexer::new(source)).parse()
    }

    fn eval(&mut self, program: Program, repl_mode: bool, env: EnvRef) -> Result<Value, LangError> {
        Interpreter::new(self, repl_mode, env)
            .eval(program)
            .map_err(|e| e.try_into().expect("signal should be an error"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ModuleKind {
    Core(String),
    Std(String),
    UserPath(PathBuf),
}

impl ModuleKind {
    fn from(raw: &str) -> Self {
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
