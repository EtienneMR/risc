use crate::interpreter::{
    Interpreter,
    control_flow::{ControlFlow, ControlFlowKind},
    eval::call::dispatch_call,
    value::Value,
};
use crate::parser::ast::{Block, Expr};

impl Interpreter {
    pub fn eval_if(
        &mut self,
        condition: &Expr,
        then_branch: &Block,
        else_branch: Option<&Block>,
    ) -> Result<Value, ControlFlow> {
        if self.eval(condition)?.is_truthy() {
            self.exec(then_branch)
        } else if let Some(branch) = else_branch {
            self.exec(branch)
        } else {
            Ok(Value::Nil)
        }
    }

    pub fn eval_for(
        &mut self,
        identifier: &str,
        iterator: &Expr,
        body: &Block,
    ) -> Result<Value, ControlFlow> {
        let iterator_val = self.eval(iterator)?;
        let mut last = Value::Nil;

        loop {
            let item = dispatch_call(&iterator_val, Vec::new(), iterator.span)?;
            if item == Value::Nil {
                break;
            }

            let mut inner = self.inner_scope();
            inner.env.define(identifier.into(), item, iterator.span)?;

            match inner.exec(body) {
                Ok(v) => last = v,
                Err(ControlFlow {
                    kind: ControlFlowKind::Break(value),
                    ..
                }) => return Ok(value),
                Err(ControlFlow {
                    kind: ControlFlowKind::Continue,
                    ..
                }) => continue,
                Err(other) => return Err(other),
            }
        }

        Ok(last)
    }

    pub fn eval_while(&mut self, condition: &Expr, body: &Block) -> Result<Value, ControlFlow> {
        let mut last = Value::Nil;

        loop {
            let mut inner = self.inner_scope();
            if !inner.eval(condition)?.is_truthy() {
                break;
            }
            match inner.exec(body) {
                Ok(v) => last = v,
                Err(ControlFlow {
                    kind: ControlFlowKind::Break(value),
                    ..
                }) => return Ok(value),
                Err(ControlFlow {
                    kind: ControlFlowKind::Continue,
                    ..
                }) => continue,
                Err(other) => return Err(other),
            }
        }

        Ok(last)
    }
}
