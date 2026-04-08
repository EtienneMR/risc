use crate::interpreter::{
    Interpreter,
    control_flow::{ControlFlow, ControlFlowKind},
    value::Value,
};
use crate::parser::ast::{Block, CatchArm, Expr, ExprKind, UnaryOp};
use crate::source::Span;

mod binary;
mod call;
mod control;
mod declaration;

impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
        match &expr.kind {
            ExprKind::Number(n) => Ok(Value::Number((*n).into())),
            ExprKind::String(s) => Ok(Value::String(s.clone())),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Nil => Ok(Value::Nil),
            ExprKind::Identifier(name) => {
                Ok(self.env.get(&Value::String(name.clone()), expr.span)?)
            }

            ExprKind::Unary { op, rhs } => self.eval_unary(op, rhs, expr.span),
            ExprKind::Index { object, key } => self.eval_index(object, key, expr.span),
            ExprKind::Binary { op, lhs, rhs } => self.eval_binary(op, lhs, rhs, expr.span),
            ExprKind::Call { callee, args } => self.eval_call(callee, args, expr.span),

            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if(condition, then_branch, else_branch.as_ref()),
            ExprKind::For {
                identifier,
                iterator,
                body,
            } => self.eval_for(identifier, iterator, body),
            ExprKind::While { condition, body } => self.eval_while(condition, body),
            ExprKind::TryCatch {
                body,
                catches,
                else_branch,
            } => self.eval_try_catch(body, catches, else_branch.as_ref(), expr.span),

            ExprKind::Table(rows) => self.eval_table(rows, expr.span),
            ExprKind::Bind { identifier, value } => self.eval_bind(identifier, value, expr.span),
            ExprKind::Function { params, body } => self.eval_function(params, body.clone()),

            ExprKind::Break => Err(ControlFlow::new(
                ControlFlowKind::Break(Value::Nil),
                expr.span,
            )),
            ExprKind::Continue => Err(ControlFlow::new(ControlFlowKind::Continue, expr.span)),
            ExprKind::Return(value_expr) => Err(ControlFlow::new(
                ControlFlowKind::Return(self.eval(value_expr)?),
                expr.span,
            )),
        }
    }

    pub fn exec(&mut self, block: &Block) -> Result<Value, ControlFlow> {
        let mut result = Value::Nil;
        for expr in &block.exprs {
            result = self.eval(expr)?;
        }
        Ok(result)
    }

    fn eval_unary(&mut self, op: &UnaryOp, rhs: &Expr, span: Span) -> Result<Value, ControlFlow> {
        let val = self.eval(rhs)?;
        match op {
            UnaryOp::Neg => match val {
                Value::Number(n) => Ok(Value::Number((-n.value).into())),
                v => Err(ControlFlow::error(
                    format!("unary '-' requires a number, got {}", v.type_name()),
                    span,
                )),
            },
            UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
        }
    }

    fn eval_index(&mut self, object: &Expr, key: &Expr, span: Span) -> Result<Value, ControlFlow> {
        let obj = self.eval(object)?;
        let Value::Table(table) = obj else {
            return Err(ControlFlow::error(
                format!("cannot index a {}", obj.type_name()),
                span,
            ));
        };
        let key_val = self.eval(key)?;
        Ok(table.get(&key_val))
    }

    fn eval_try_catch(
        &mut self,
        body: &Block,
        catches: &[CatchArm],
        else_branch: Option<&Block>,
        span: Span,
    ) -> Result<Value, ControlFlow> {
        match self.exec(body) {
            Ok(value) => {
                if let Some(branch) = else_branch {
                    self.exec(branch)
                } else {
                    Ok(value)
                }
            }

            Err(ControlFlow {
                kind: ControlFlowKind::Error(error_val),
                span: error_span,
            }) => {
                for arm in catches {

                    let matches = match (&arm.kind_filter, &error_val) {
                        (None, _) => true,
                        (Some(filter), Value::Table(t)) =>  self.eval(filter)? == t.get(&"error".into()),
                        _ => false,
                    };

                    if matches {
                        let mut inner = self.inner_scope();
                        inner.env.define(
                            Value::String(arm.binding.clone()),
                            error_val.clone(),
                            span,
                        )?;
                        return inner.exec(&arm.body);
                    }
                }

                Err(ControlFlow {
                    kind: ControlFlowKind::Error(error_val),
                    span: error_span,
                })
            }

            other => other,
        }
    }
}
