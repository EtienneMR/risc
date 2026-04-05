pub mod control_flow;
pub mod env;
pub mod stdlib;
pub mod value;

mod eval;
mod ops;

use env::Env;
use stdlib::register_builtins;
use value::Value;

use crate::{interpreter::control_flow::ControlFlowKind, parser::ast::Block, source::Span};

#[derive(Debug)]
pub struct Interpreter {
    env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Env::new();
        register_builtins(&mut env);
        Self { env }
    }

    pub fn inner_scope(&self) -> Self {
        Self {
            env: self.env.inner_scope(),
        }
    }

    pub fn run(&mut self, body: &Block) -> Result<Value, (String, Span)> {
        match self.exec(body) {
            Ok(value) => Ok(value),
            Err(other) => {
                let other = other.reject_loop_control();
                match other.kind {
                    ControlFlowKind::Return(value) => Ok(value),
                    ControlFlowKind::Error(e) => Err((e.to_string(), other.span)),
                    _ => unreachable!(),
                }
            }
        }
    }
}
