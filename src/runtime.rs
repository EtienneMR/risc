//! Runtime links the parser, interpreter, source map, and module cache together.
//! ModuleKind distinguishes @core/ built-ins, @std/ embedded sources, and user file paths.
//! Parsed modules are cached by ModuleKind so repeated require() calls pay no re-parse cost.
//! The stdlib directory is compiled into the binary at build time via build.rs / OUT_DIR.
//! Runtime::new() runs the prelude; run() and run_repl() are the two public entry points.

use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    ast::Program,
    corelib::{get_corelib, register_builtins},
    error::{LangError, NativeError},
    interpreter::Interpreter,
    lexer::Lexer,
    parser::Parser,
    project::Project,
    source::{SourceMap, Span},
    value::{Env, EnvRef, Signal, Value},
};

mod stdlib {
    include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));
}

pub struct Runtime {
    source_map: RefCell<SourceMap>,
    module_cache: RefCell<HashMap<ModuleKind, Value>>,
    global_env: EnvRef,
    project: Project,
    args: Vec<String>,
}

impl Runtime {
    pub fn new(project: Project, args: Vec<String>) -> Rc<Self> {
        let global_env = Env::new();
        register_builtins(&global_env);

        let this = Rc::new(Self {
            source_map: RefCell::new(SourceMap::new()),
            module_cache: RefCell::new(HashMap::new()),
            global_env,
            project,
            args,
        });

        if let Some(src) = stdlib::get("prelude") {
            if let Err(e) = this.run_global("@std/prelude".to_string(), src.to_string()) {
                panic!(
                    "risc: prelude error\n{}",
                    e.display(&this.source_map.borrow())
                );
            }
        }

        this
    }

    pub fn create_module_env(&self) -> EnvRef {
        self.global_env.inner()
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn source_map(&self) -> std::cell::Ref<'_, SourceMap> {
        self.source_map.borrow()
    }

    pub fn run(self: &Rc<Self>, name: String, source_text: String) -> Result<Value, LangError> {
        let program = self.parse(name, source_text)?;
        self.eval(program, false, self.create_module_env())
    }

    pub fn run_repl(self: &Rc<Self>, source_text: String, env: EnvRef) -> Result<Value, LangError> {
        let program = self.parse("<repl>".to_owned(), source_text)?;
        self.eval(program, true, env)
    }

    pub fn run_global(
        self: &Rc<Self>,
        name: String,
        source_text: String,
    ) -> Result<Value, LangError> {
        let program = self.parse(name, source_text)?;
        self.eval(program, false, self.global_env.clone())
    }

    pub fn load_module(self: &Rc<Self>, path: &str, call_span: Span) -> Result<Value, Signal> {
        let kind: ModuleKind = ModuleKind::resolve(path, &self.project);

        if let Some(cached) = self.module_cache.borrow().get(&kind) {
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

        self.module_cache.borrow_mut().insert(kind, module.clone());
        Ok(module)
    }

    fn parse(&self, name: String, source_text: String) -> Result<Program, LangError> {
        let mut sm = self.source_map.borrow_mut();
        let source = sm.add(name, source_text);
        Parser::new(Lexer::new(source)).parse()
    }

    fn eval(
        self: &Rc<Self>,
        program: Program,
        repl_mode: bool,
        env: EnvRef,
    ) -> Result<Value, LangError> {
        Interpreter::new(self.clone(), repl_mode, env)
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
    fn resolve(raw: &str, project: &Project) -> Self {
        if let Some(name) = raw.strip_prefix("@core/") {
            return Self::Core(name.to_string());
        }
        if let Some(path) = raw.strip_prefix("@std/") {
            return Self::Std(path.to_string());
        }
        Self::UserPath(Self::resolve_user_path(raw, project))
    }

    fn resolve_user_path(raw: &str, project: &Project) -> PathBuf {
        let mut path = Cow::Borrowed(Path::new(&raw));

        for (prefix, base_path) in project.includes() {
            if let Some(relative_path) = raw.strip_prefix(prefix) {
                path = Cow::Owned(base_path.join(relative_path));
                break;
            }
        }

        let base = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        };

        let extended = if base.extension().is_none() {
            base.with_extension("ri")
        } else {
            base
        };

        extended.canonicalize().unwrap_or(extended)
    }
}
