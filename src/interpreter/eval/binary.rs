use crate::interpreter::{
    Interpreter, control_flow::ControlFlow, eval::call::dispatch_call, ops::{arithmetic, compare, values_equal}, value::Value
};
use crate::parser::ast::{BinaryOp, Expr, ExprKind};
use crate::source::Span;

impl Interpreter {
    pub fn eval_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: Span,
    ) -> Result<Value, ControlFlow> {
        match op {
            BinaryOp::Assign => self.eval_assign(lhs, rhs),
            BinaryOp::Pipe => self.eval_pipe(lhs, rhs, span),
            BinaryOp::And => self.eval_and(lhs, rhs),
            BinaryOp::Or => self.eval_or(lhs, rhs),
            _ => self.eval_eager_binary(op, lhs, rhs, span),
        }
    }

    fn eval_assign(&mut self, lhs: &Expr, rhs: &Expr) -> Result<Value, ControlFlow> {
        let val = self.eval(rhs)?;
        match &lhs.kind {
            ExprKind::Identifier(name) => {
                self.env
                    .set(Value::String(name.clone()), val.clone(), lhs.span)?;
            }
            ExprKind::Index { object, key } => {
                let obj = self.eval(object)?;
                let Value::Table(table) = obj else {
                    return Err(ControlFlow::error(
                        format!("cannot index-assign into a {}", obj.type_name()),
                        lhs.span,
                    ));
                };
                let key_val = self.eval(key)?;
                table.set(key_val, val.clone());
            }
            _ => {
                return Err(ControlFlow::error(
                    format!("invalid assignment target: {:?}", lhs.kind),
                    lhs.span,
                ));
            }
        }
        Ok(val)
    }

    fn eval_pipe(&mut self, lhs: &Expr, rhs: &Expr, span: Span) -> Result<Value, ControlFlow> {
        let first_arg = self.eval(lhs)?;
        let ExprKind::Call { callee, args } = &rhs.kind else {
            return Err(ControlFlow::error(
                "right-hand side of '|>' must be a function call",
                rhs.span,
            ));
        };
        let callee_val = self.eval(callee)?;
        let mut arg_vals = vec![first_arg];
        for a in args {
            arg_vals.push(self.eval(a)?);
        }
        dispatch_call(&callee_val, arg_vals, span)
    }

    fn eval_and(&mut self, lhs: &Expr, rhs: &Expr) -> Result<Value, ControlFlow> {
        let left = self.eval(lhs)?;
        if left.is_truthy() {
            self.eval(rhs)
        } else {
            Ok(left)
        }
    }

    fn eval_or(&mut self, lhs: &Expr, rhs: &Expr) -> Result<Value, ControlFlow> {
        let left = self.eval(lhs)?;
        if left.is_truthy() {
            Ok(left)
        } else {
            self.eval(rhs)
        }
    }

    fn eval_eager_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Expr,
        rhs: &Expr,
        span: Span,
    ) -> Result<Value, ControlFlow> {
        let l = self.eval(lhs)?;
        let r = self.eval(rhs)?;

        Ok(match op {
            BinaryOp::Add => match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => Value::Number((a.value + b.value).into()),
                (Value::String(a), Value::String(b)) => Value::String(format!("{a}{b}")),
                _ => {
                    return Err(ControlFlow::error(
                        format!(
                            "'+' not defined for {} and {}",
                            l.type_name(),
                            r.type_name()
                        ),
                        span,
                    ));
                }
            },
            BinaryOp::Sub => arithmetic(l, r, span, "-", |a, b| a - b)?,
            BinaryOp::Mul => arithmetic(l, r, span, "*", |a, b| a * b)?,
            BinaryOp::Div => {
                if matches!(r, Value::Number(n) if n.value == 0.0) {
                    return Err(ControlFlow::error("division by zero", span));
                }
                arithmetic(l, r, span, "/", |a, b| a / b)?
            }
            BinaryOp::Rem => arithmetic(l, r, span, "%", |a, b| a % b)?,

            BinaryOp::Eq => Value::Bool(values_equal(&l, &r)),
            BinaryOp::NotEq => Value::Bool(!values_equal(&l, &r)),
            BinaryOp::Lt => compare(l, r, span, "<", |a, b| a < b)?,
            BinaryOp::Lte => compare(l, r, span, "<=", |a, b| a <= b)?,
            BinaryOp::Gt => compare(l, r, span, ">", |a, b| a > b)?,
            BinaryOp::Gte => compare(l, r, span, ">=", |a, b| a >= b)?,

            BinaryOp::Assign | BinaryOp::Pipe | BinaryOp::And | BinaryOp::Or => unreachable!(),
        })
    }
}
