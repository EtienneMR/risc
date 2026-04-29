//! Tree-walking interpreter: evaluates an Ast against a lexical Env chain.
//! Interpreter owns a reference to Runtime (for module loading) and the current Env.
//! repl_mode allows let-rebinding so REPL sessions can redefine variables freely.
//! Signals (Return, Break, Continue, Error) propagate as Err(Signal) through the call stack.
//! call_function handles native functions, user functions (with default args), and require.

use std::{
    collections::{HashMap, hash_map::Entry},
    mem,
    rc::Rc,
};

use crate::{
    ast::{Ast, BinaryOp, NodeId, NodeKind, ParamKind, Program, UnaryOp},
    error::NativeError,
    runtime::Runtime,
    source::Span,
    value::{
        CallContext, EnvRef, Function, Signal, SignalKind, StrRef, Table, TableKey, UserFunction,
        Value,
    },
};

pub struct Interpreter {
    env: EnvRef,
    runtime: Rc<Runtime>,
    repl_mode: bool,
}

impl Interpreter {
    pub fn new(runtime: Rc<Runtime>, repl_mode: bool, env: EnvRef) -> Self {
        Self {
            env,
            runtime,
            repl_mode,
        }
    }

    pub fn eval(&mut self, program: Program) -> Result<Value, Signal> {
        let ast = Rc::new(program.ast);
        let mut value = Value::Nil;
        for root in program.roots {
            match self.eval_node(&ast, root) {
                Ok(val) => value = val,
                Err(signal) => {
                    let signal = signal.reject_loop_control();
                    match signal.kind {
                        SignalKind::Return(val) => return Ok(val),
                        _ => return Err(signal),
                    }
                }
            }
        }
        Ok(value)
    }

    fn with_env<T>(&mut self, env: EnvRef, f: impl FnOnce(&mut Self) -> T) -> T {
        let old_env = mem::replace(&mut self.env, env);
        let out = f(self);
        self.env = old_env;
        out
    }

    fn with_scope<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let env = self.env.inner();
        self.with_env(env, f)
    }

    fn eval_node(&mut self, ast: &Rc<Ast>, node_id: NodeId) -> Result<Value, Signal> {
        let node = ast.get(node_id);
        let span = node.span;

        match &node.kind {
            NodeKind::Nil => Ok(Value::Nil),
            NodeKind::Boolean(b) => Ok(Value::Boolean(*b)),
            NodeKind::Number(n) => Ok(Value::Number(*n)),
            NodeKind::String(s) => Ok(Value::String(Rc::from(s.as_str()))),

            NodeKind::Identifier(name) => self
                .env
                .get(Rc::from(name.as_str()))
                .map_err(|e| Signal::from_error(e, span)),

            NodeKind::Block { nodes } => self.with_scope(|this| {
                let mut value = Value::Nil;
                for node in nodes {
                    value = this.eval_node(ast, *node)?;
                }
                Ok(value)
            }),

            NodeKind::Unary { op, right } => {
                let right_val = self.eval_node(ast, *right)?;
                match op {
                    UnaryOp::Neg => right_val.op_neg().map_err(|e| Signal::from_error(e, span)),
                    UnaryOp::Not => right_val.op_not().map_err(|e| Signal::from_error(e, span)),
                }
            }

            NodeKind::Binary { op, left, right } => self.eval_binary(ast, *op, *left, *right, span),

            NodeKind::Call {
                callee,
                args,
                last_is_rest,
            } => {
                let callee_val = self.eval_node(ast, *callee)?;

                let mut positional: Vec<Value> = Vec::new();
                let mut named: HashMap<StrRef, Value> = HashMap::new();

                for arg in args {
                    let value = self.eval_node(ast, arg.value)?;
                    match &arg.name {
                        None => positional.push(value),
                        Some(n) => {
                            named.insert(Rc::from(n.as_str()), value);
                        }
                    }
                }

                if *last_is_rest {
                    let rest = positional
                        .pop()
                        .expect("a positional param should exist to have a rest param");

                    let Value::Table(table) = rest else {
                        return Err(Signal::from_error(
                            NativeError::new(
                                "type error",
                                format!("can not use a {} as a rest param", rest.type_name()),
                            ),
                            span,
                        ));
                    };

                    let mut new_positional: Vec<(i64, Value)> = Vec::new();

                    for (key, value) in table.entries() {
                        match key {
                            TableKey::Integer(i) => new_positional.push((i, value)),
                            TableKey::String(s) => {
                                if let Entry::Occupied(e) = named.entry(s) {
                                    return Err(Signal::from_error(
                                        NativeError::new(
                                            "type error",
                                            format!(
                                                "argument '{}' supplied both by name and by rest",
                                                e.key()
                                            ),
                                        ),
                                        span,
                                    ));
                                }
                            }
                            k => {
                                return Err(Signal::from_error(
                                    NativeError::new(
                                        "type error",
                                        format!(
                                            "can not use a {} as a rest param key",
                                            Value::from(k).type_name()
                                        ),
                                    ),
                                    span,
                                ));
                            }
                        }
                    }

                    new_positional.sort_unstable_by_key(|(k, _)| *k);
                    positional.extend(new_positional.into_iter().map(|(_, v)| v));
                }

                self.call_function(callee_val, positional, named, span)
            }

            NodeKind::Index { object, key } => {
                let obj_val = self.eval_node(ast, *object)?;
                let key_val = self.eval_node(ast, *key)?;

                match obj_val {
                    Value::Table(table) => {
                        let key_type = key_val.type_name();
                        let table_key = TableKey::try_from(key_val).map_err(|_| {
                            Signal::from_error(
                                NativeError::new(
                                    "type error",
                                    format!("table keys must be boolean, integer, or string, got {key_type}"),
                                ),
                                span,
                            )
                        })?;
                        Ok(table.get(&table_key).unwrap_or(Value::Nil))
                    }
                    _ => Err(Signal::from_error(
                        NativeError::new(
                            "type error",
                            format!("cannot index {}", obj_val.type_name()),
                        ),
                        span,
                    )),
                }
            }

            NodeKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_node(ast, *condition)?;
                if cond_val.to_boolean() {
                    self.eval_node(ast, *then_branch)
                } else if let Some(else_id) = else_branch {
                    self.eval_node(ast, *else_id)
                } else {
                    Ok(Value::Nil)
                }
            }

            NodeKind::While { condition, body } => {
                let mut value = Value::Nil;
                loop {
                    let cond_val = self.eval_node(ast, *condition)?;
                    if !cond_val.to_boolean() {
                        break;
                    }

                    match self.eval_node(ast, *body) {
                        Ok(v) => value = v,
                        Err(signal) => match signal.kind {
                            SignalKind::Break(v) => return Ok(v),
                            SignalKind::Continue => continue,
                            _ => return Err(signal),
                        },
                    }
                }
                Ok(value)
            }

            NodeKind::For {
                identifier,
                iterator,
                body,
            } => {
                let iter_val = self.eval_node(ast, *iterator)?;
                let mut value = Value::Nil;

                loop {
                    let next_val =
                        self.call_function(iter_val.clone(), Vec::new(), HashMap::new(), span)?;

                    if next_val == Value::Nil {
                        break;
                    }

                    let result = self.with_scope(|this| {
                        this.env
                            .define(Rc::from(identifier.as_str()), next_val)
                            .map_err(|e| Signal::from_error(e, span))?;
                        this.eval_node(ast, *body)
                    });

                    match result {
                        Ok(v) => value = v,
                        Err(signal) => match signal.kind {
                            SignalKind::Break(v) => return Ok(v),
                            SignalKind::Continue => {}
                            _ => return Err(signal),
                        },
                    }
                }
                Ok(value)
            }

            NodeKind::TryCatch {
                body,
                catches,
                else_branch,
            } => match self.eval_node(ast, *body) {
                Ok(val) => {
                    if let Some(else_id) = else_branch {
                        self.eval_node(ast, *else_id)
                    } else {
                        Ok(val)
                    }
                }
                Err(signal) => match &signal.kind {
                    SignalKind::Error { kind, message } => {
                        for catch_arm in catches {
                            let matches = if let Some(filter_id) = catch_arm.kind_filter {
                                let filter_val = self.eval_node(ast, filter_id)?;
                                match filter_val {
                                    Value::String(s) => s.as_ref() == kind.as_ref(),
                                    _ => false,
                                }
                            } else {
                                true
                            };

                            if matches {
                                let mut error_table = Table::new();
                                error_table.set("error", Value::String(kind.clone()));
                                error_table.set("message", Value::String(message.clone()));

                                return self.with_scope(|this| {
                                    this.env
                                        .define(
                                            Rc::from(catch_arm.binding.as_str()),
                                            Value::Table(error_table),
                                        )
                                        .map_err(|e| Signal::from_error(e, span))?;
                                    this.eval_node(ast, catch_arm.body)
                                });
                            }
                        }

                        Err(signal)
                    }
                    _ => Err(signal),
                },
            },

            NodeKind::Break(val_id) => {
                let val = self.eval_node(ast, *val_id)?;
                Err(Signal {
                    kind: SignalKind::Break(val),
                    traceback: vec![span],
                })
            }
            NodeKind::Continue => Err(Signal {
                kind: SignalKind::Continue,
                traceback: vec![span],
            }),
            NodeKind::Return(val_id) => {
                let val = self.eval_node(ast, *val_id)?;
                Err(Signal {
                    kind: SignalKind::Return(val),
                    traceback: vec![span],
                })
            }

            NodeKind::Declaration { identifier, value } => {
                let val = self.eval_node(ast, *value)?;
                if self.repl_mode {
                    self.env.upsert(Rc::from(identifier.as_str()), val.clone());
                } else {
                    self.env
                        .define(Rc::from(identifier.as_str()), val.clone())
                        .map_err(|e| Signal::from_error(e, span))?;
                }
                Ok(val)
            }

            NodeKind::Function { params, body } => {
                Ok(Value::Function(Function::User(Rc::new(UserFunction {
                    params: params.clone(),
                    body: *body,
                    ast: ast.clone(),
                    env: self.env.clone(),
                }))))
            }

            NodeKind::Table(items) => {
                let mut table = Table::new();

                for item in items {
                    let key_val = self.eval_node(ast, item.key)?;
                    let value_val = self.eval_node(ast, item.value)?;

                    let table_key = TableKey::try_from(key_val).map_err(|_| {
                        Signal::from_error(
                            NativeError::new(
                                "type error",
                                "table keys must be boolean, integer, or string".to_string(),
                            ),
                            span,
                        )
                    })?;

                    table.set(table_key, value_val);
                }

                Ok(Value::Table(table))
            }
        }
    }

    fn eval_binary(
        &mut self,
        ast: &Rc<Ast>,
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
        span: Span,
    ) -> Result<Value, Signal> {
        match op {
            BinaryOp::Assign => {
                let right_val = self.eval_node(ast, right)?;
                let left_node = ast.get(left);

                match &left_node.kind {
                    NodeKind::Identifier(name) => {
                        self.env
                            .set(Rc::from(name.as_str()), right_val.clone())
                            .map_err(|e| Signal::from_error(e, span))?;
                        Ok(right_val)
                    }
                    NodeKind::Index { object, key } => {
                        let obj_val = self.eval_node(ast, *object)?;
                        let key_val = self.eval_node(ast, *key)?;

                        match obj_val {
                            Value::Table(mut table) => {
                                let table_key = TableKey::try_from(key_val).map_err(|_| {
                                    Signal::from_error(
                                        NativeError::new(
                                            "type error",
                                            "table keys must be boolean, integer, or string"
                                                .to_string(),
                                        ),
                                        span,
                                    )
                                })?;
                                table.set(table_key, right_val.clone());
                                Ok(right_val)
                            }
                            _ => Err(Signal::from_error(
                                NativeError::new(
                                    "type error",
                                    format!("cannot index {}", obj_val.type_name()),
                                ),
                                span,
                            )),
                        }
                    }
                    _ => Err(Signal::from_error(
                        NativeError::new("type error", "invalid assignment target".to_string()),
                        span,
                    )),
                }
            }

            BinaryOp::Pipe => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                self.call_function(right_val, vec![left_val], HashMap::new(), span)
            }

            BinaryOp::And => {
                let left_val = self.eval_node(ast, left)?;
                if !left_val.to_boolean() {
                    Ok(left_val)
                } else {
                    self.eval_node(ast, right)
                }
            }

            BinaryOp::Or => {
                let left_val = self.eval_node(ast, left)?;
                if left_val.to_boolean() {
                    Ok(left_val)
                } else {
                    self.eval_node(ast, right)
                }
            }

            BinaryOp::Add => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_add(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Sub => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_sub(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Mul => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_mul(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Div => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_div(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Rem => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_rem(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Eq => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                Ok(left_val.op_eq(&right_val))
            }

            BinaryOp::NotEq => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                Ok(left_val.op_ne(&right_val))
            }

            BinaryOp::Lt => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_lt(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Lte => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_lte(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Gt => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_gt(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }

            BinaryOp::Gte => {
                let left_val = self.eval_node(ast, left)?;
                let right_val = self.eval_node(ast, right)?;
                left_val
                    .op_gte(&right_val)
                    .map_err(|e| Signal::from_error(e, span))
            }
        }
    }

    fn call_function(
        &mut self,
        callee: Value,
        positional: Vec<Value>,
        named: HashMap<StrRef, Value>,
        span: Span,
    ) -> Result<Value, Signal> {
        match callee {
            Value::Function(function) => match function {
                Function::Native(native_fn) => {
                    let ctx = CallContext::new(positional, named, span, self.runtime.clone());
                    (native_fn.func)(ctx)
                }

                Function::User(user_fn) => {
                    self.call_user_fn(user_fn.as_ref(), positional, named, span)
                }
            },

            other => Err(Signal::from_error(
                NativeError::new("type error", format!("cannot call {}", other.type_name())),
                span,
            )),
        }
        .map_err(|sig| sig.add_traceback_frame(span))
    }

    fn call_user_fn(
        &mut self,
        user_fn: &UserFunction,
        positional: Vec<Value>,
        named: HashMap<StrRef, Value>,
        span: Span,
    ) -> Result<Value, Signal> {
        self.with_env(user_fn.env.inner(), |interp| {
            let mut positional = positional.into_iter();
            let mut named = named;

            for param in &user_fn.params {
                match &param.kind {
                    ParamKind::Rest => {
                        let mut rest = Table::from_vec(positional.by_ref().collect());
                        for (key, value) in mem::take(&mut named) {
                            rest.set(key.as_ref(), value);
                        }
                        interp
                            .env
                            .define(Rc::from(param.name.as_str()), Value::Table(rest))
                            .map_err(|e| Signal::from_error(e, span))?;
                    }

                    param_kind => {
                        let by_position = positional.next();
                        let by_name = named.remove(param.name.as_str());

                        let value = match (by_position, by_name) {
                            (Some(_), Some(_)) => {
                                return Err(arg_error(
                                    format!(
                                        "argument '{}' supplied both positionally and by name",
                                        param.name
                                    ),
                                    span,
                                ));
                            }
                            (Some(v), None) | (None, Some(v)) => v,
                            (None, None) => match param_kind {
                                ParamKind::Optional(default_node) => {
                                    interp.eval_node(&user_fn.ast, *default_node)?
                                }
                                ParamKind::Required => {
                                    return Err(arg_error(
                                        format!("missing argument '{}'", param.name),
                                        span,
                                    ));
                                }
                                ParamKind::Rest => unreachable!(),
                            },
                        };

                        interp
                            .env
                            .define(Rc::from(param.name.as_str()), value)
                            .map_err(|e| Signal::from_error(e, span))?;
                    }
                }
            }

            let extra_positional = positional.count();
            if extra_positional > 0 {
                return Err(arg_error(
                    format!(
                        "too many positional arguments: expected at most {}, got {} more",
                        user_fn.params.len(),
                        extra_positional
                    ),
                    span,
                ));
            }

            if let Some(unexpected_key) = named.into_keys().next() {
                return Err(arg_error(
                    format!("unexpected named argument '{}'", unexpected_key),
                    span,
                ));
            }

            match interp.eval_node(&user_fn.ast, user_fn.body) {
                Ok(value) => Ok(value),
                Err(signal) => match signal.kind {
                    SignalKind::Return(value) => Ok(value),
                    _ => Err(signal.reject_loop_control()),
                },
            }
        })
    }
}

fn arg_error(msg: impl Into<String>, span: Span) -> Signal {
    Signal::from_error(NativeError::new("argument error", msg.into()), span)
}
