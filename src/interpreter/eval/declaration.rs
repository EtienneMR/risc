use crate::interpreter::{
    Interpreter,
    control_flow::ControlFlow,
    value::{Value, function::Function, table::Table},
};
use crate::parser::ast::{Block, Expr, TableRow};
use crate::source::Span;

impl Interpreter {
    pub fn eval_table(&mut self, rows: &[TableRow], _span: Span) -> Result<Value, ControlFlow> {
        let table = Table::new();
        for row in rows {
            table.set(self.eval(&row.key)?, self.eval(&row.value)?);
        }
        Ok(Value::Table(table))
    }

    pub fn eval_bind(
        &mut self,
        identifier: &str,
        value: &Expr,
        span: Span,
    ) -> Result<Value, ControlFlow> {
        let val = self.eval(value)?;
        self.env
            .define(Value::String(identifier.to_string()), val.clone(), span)?;
        Ok(val)
    }

    pub fn eval_function(&mut self, params: &[String], body: Block) -> Result<Value, ControlFlow> {
        Ok(Value::Function(Function::new(
            params.to_vec(),
            body,
            self.env.clone(),
        )))
    }
}
