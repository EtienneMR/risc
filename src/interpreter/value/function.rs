use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::{interpreter::env::Env, parser::ast::Block};

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Block,
    pub env: Env,
}

impl Function {
    pub fn new(params: Vec<String>, body: Block, env: crate::interpreter::env::Env) -> Self {
        Self { params, body, env }
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl Eq for Function {}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0.hash(state);
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function>")
    }
}
