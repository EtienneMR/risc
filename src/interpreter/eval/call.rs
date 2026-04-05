use std::iter::repeat;

use crate::interpreter::{
    Interpreter,
    control_flow::{ControlFlow, ControlFlowKind},
    value::{Value, function::Function},
};
use crate::parser::ast::Expr;
use crate::source::Span;

impl Interpreter {
    pub fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: Span,
    ) -> Result<Value, ControlFlow> {
        let callee_val = self.eval(callee)?;
        let mut arg_vals = Vec::with_capacity(args.len());
        for arg in args {
            arg_vals.push(self.eval(arg)?);
        }
        dispatch_call(&callee_val, arg_vals, span)
    }
}

pub fn dispatch_call(callee: &Value, args: Vec<Value>, span: Span) -> Result<Value, ControlFlow> {
    match callee {
        Value::Builtin(b) => (b.function)(&args).map_err(|m| ControlFlow::error(m, span)),
        Value::Function(f) => dispatch_function_call(f, args, span),
        other => Err(ControlFlow::error(
            format!("'{}' is not callable", other.type_name()),
            span,
        )),
    }
}

fn dispatch_function_call(
    func: &Function,
    args: Vec<Value>,
    span: Span,
) -> Result<Value, ControlFlow> {
    if func.params.len() != args.len() {
        return Err(ControlFlow::error(
            format!(
                "expected {} arguments, got {}",
                func.params.len(),
                args.len()
            ),
            span,
        ));
    }

    let mut interpreter = Interpreter {
        env: func.env.inner_scope(),
    };

    for (param, arg) in func
        .params
        .iter()
        .zip(args.into_iter().chain(repeat(Value::Nil)))
    {
        interpreter
            .env
            .define(Value::String(param.clone()), arg, span)?;
    }

    match interpreter.exec(&func.body) {
        Ok(v) => Ok(v),
        Err(ControlFlow {
            kind: ControlFlowKind::Return(v),
            ..
        }) => Ok(v),
        Err(e) => Err(e.reject_loop_control()),
    }
}
