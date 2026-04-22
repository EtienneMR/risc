//! Runtime value types: Nil, Boolean, Number, String, Table, Function, Native.
//! Env is a linked-list scope chain; Table is a shared-reference HashMap.
//! Signal carries Return/Break/Continue/Error through the interpreter as Err(Signal).
//! CallContext bundles positional and named arguments with the call Span.
//! upsert() allows REPL redefinition; define() enforces single-assignment elsewhere.

use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    fmt,
    rc::Rc,
};

use crate::{
    ast::{Ast, NodeId},
    error::NativeError,
    source::Span,
};

#[derive(Debug, Clone)]
pub struct CallContext {
    pub args: Vec<Value>,
    pub named: HashMap<StrRef, Value>,
    pub span: Span,
}

impl CallContext {
    pub fn new(args: Vec<Value>, named: HashMap<StrRef, Value>, span: Span) -> Self {
        Self { args, named, span }
    }

    pub fn get(&self, index: usize, name: &str) -> &Value {
        if let Some(v) = self.named.get(name) {
            return v;
        }
        self.args.get(index).unwrap_or(&Value::Nil)
    }

    pub fn error(&self, error: NativeError) -> Signal {
        Signal::from_error(error, self.span)
    }
}

pub type EnvRef = Rc<Env>;

#[derive(Debug, Clone)]
pub struct Env {
    parent: Option<EnvRef>,
    values: RefCell<HashMap<StrRef, Value>>,
}

impl Env {
    pub fn new() -> EnvRef {
        Rc::new(Env {
            parent: None,
            values: RefCell::new(HashMap::new()),
        })
    }

    pub fn inner(self: &EnvRef) -> EnvRef {
        Rc::new(Env {
            parent: Some(Rc::clone(self)),
            values: RefCell::new(HashMap::new()),
        })
    }

    pub fn define(self: &EnvRef, key: StrRef, value: Value) -> Result<(), NativeError> {
        match self.values.borrow_mut().entry(key) {
            Entry::Occupied(entry) => Err(NativeError::new(
                "definition error",
                format!("redefinition of key '{}' in this scope", entry.key()),
            )),
            Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    pub fn upsert(self: &EnvRef, key: StrRef, value: Value) {
        self.values.borrow_mut().insert(key, value);
    }

    pub fn get(self: &EnvRef, key: StrRef) -> Result<Value, NativeError> {
        if let Some(val) = self.values.borrow().get(&key) {
            return Ok(val.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.get(key);
        }

        return Err(NativeError::new(
            "definition error",
            format!("get of undefined key '{key}' in this scope"),
        ));
    }

    pub fn set(self: &EnvRef, key: StrRef, value: Value) -> Result<(), NativeError> {
        {
            let mut frame = self.values.borrow_mut();
            if let Some(slot) = frame.get_mut(&key) {
                *slot = value;
                return Ok(());
            }
        }
        if let Some(parent) = &self.parent {
            return parent.set(key, value);
        }
        return Err(NativeError::new(
            "definition error",
            format!("set of undefined key '{key}' in this scope"),
        ));
    }
}

#[derive(Debug)]
pub enum SignalKind {
    Error { kind: StrRef, message: StrRef },
    Break(Value),
    Continue,
    Return(Value),
}

#[derive(Debug)]
pub struct Signal {
    pub kind: SignalKind,
    pub span: Span,
}

impl Signal {
    pub fn from_error(error: NativeError, span: Span) -> Self {
        Self {
            kind: SignalKind::Error {
                kind: Rc::from(error.kind), //TODO: do not copy
                message: Rc::from(error.message),
            },
            span,
        }
    }

    pub fn reject_loop_control(self) -> Self {
        let kind = match self.kind {
            SignalKind::Break(_) => "break",
            SignalKind::Continue => "continue",
            _ => return self,
        };

        Self::from_error(
            NativeError::new(
                "loop control used outside of a loop",
                format!("control is a {kind}"),
            ),
            self.span,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(StrRef),

    Table(Table),
    Function(Function),

    Native(Native),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Table(_) => "table",
            Value::Function(_) => "function",
            Value::Native(_) => "native",
        }
    }

    pub fn as_boolean(&self) -> Result<bool, NativeError> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => Err(NativeError::new(
                "type error",
                format!("expected boolean got {}", self.type_name()),
            )),
        }
    }

    pub fn to_boolean(&self) -> bool {
        !matches!(self, Value::Nil | Value::Boolean(false))
    }

    pub fn to_number(&self) -> Result<f64, NativeError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::String(s) => s.parse::<f64>().map_err(|_| {
                NativeError::new(
                    "conversion error",
                    format!("cannot convert string '{}' to number", s),
                )
            }),
            _ => Err(NativeError::new(
                "conversion error",
                format!("cannot convert {} to number", self.type_name()),
            )),
        }
    }

    pub fn as_number(&self) -> Result<f64, NativeError> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(NativeError::new(
                "type error",
                format!("expected number got {}", self.type_name()),
            )),
        }
    }

    pub fn to_string_ref(&self) -> StrRef {
        Rc::from(format!("{}", self).as_str())
    }

    pub fn as_string_ref(&self) -> Result<StrRef, NativeError> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err(NativeError::new(
                "type error",
                format!("expected string got {}", self.type_name()),
            )),
        }
    }

    pub fn op_not(&self) -> Result<Value, NativeError> {
        Ok(Value::Boolean(self.to_boolean()))
    }

    pub fn op_neg(&self) -> Result<Value, NativeError> {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(NativeError::new(
                "type error",
                format!("cannot negate {}", self.type_name()),
            )),
        }
    }

    pub fn op_add(&self, rhs: &Value) -> Result<Value, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => {
                Ok(Value::String(Rc::from(format!("{}{}", a, b).as_str())))
            }
            _ => Err(NativeError::operation_type_error(
                "+",
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }

    pub fn op_sub(&self, rhs: &Value) -> Result<Value, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(NativeError::operation_type_error(
                "-",
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }

    pub fn op_mul(&self, rhs: &Value) -> Result<Value, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(NativeError::operation_type_error(
                "*",
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }

    pub fn op_div(&self, rhs: &Value) -> Result<Value, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    return Err(NativeError::new(
                        "math error",
                        "division by zero".to_string(),
                    ));
                }
                Ok(Value::Number(a / b))
            }
            _ => Err(NativeError::operation_type_error(
                "/",
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }

    pub fn op_rem(&self, rhs: &Value) -> Result<Value, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
            _ => Err(NativeError::operation_type_error(
                "%",
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }

    pub fn op_eq(&self, rhs: &Value) -> Value {
        Value::Boolean(self == rhs)
    }

    pub fn op_ne(&self, rhs: &Value) -> Value {
        Value::Boolean(self != rhs)
    }

    pub fn op_lt(&self, rhs: &Value) -> Result<Value, NativeError> {
        self.compare(rhs, "<")
            .map(|ord| Value::Boolean(ord.is_lt()))
    }

    pub fn op_lte(&self, rhs: &Value) -> Result<Value, NativeError> {
        self.compare(rhs, "<=")
            .map(|ord| Value::Boolean(ord.is_le()))
    }

    pub fn op_gt(&self, rhs: &Value) -> Result<Value, NativeError> {
        self.compare(rhs, ">")
            .map(|ord| Value::Boolean(ord.is_gt()))
    }

    pub fn op_gte(&self, rhs: &Value) -> Result<Value, NativeError> {
        self.compare(rhs, ">=")
            .map(|ord| Value::Boolean(ord.is_ge()))
    }

    fn compare(&self, rhs: &Value, op: &str) -> Result<std::cmp::Ordering, NativeError> {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(a.total_cmp(b)),
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b)),
            _ => Err(NativeError::operation_type_error(
                op,
                self.type_name(),
                rhs.type_name(),
            )),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            _ => write!(f, "<{}>", self.type_name()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    map: Rc<RefCell<HashMap<TableKey, Value>>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            map: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn from_vec(vec: Vec<Value>) -> Self {
        let mut this = Self::new();

        for (i, v) in vec.into_iter().enumerate() {
            this.set(TableKey::Integer(i as i64), v);
        }

        this
    }

    pub fn get(&self, key: &TableKey) -> Option<Value> {
        self.map.borrow().get(key).cloned()
    }

    pub fn set(&mut self, key: impl Into<TableKey>, value: Value) {
        let key = key.into();
        if matches!(value, Value::Nil) {
            self.map.borrow_mut().remove(&key);
        } else {
            self.map.borrow_mut().insert(key, value);
        }
    }

    pub fn entries(&self) -> Vec<(TableKey, Value)> {
        self.map
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.map.borrow().len()
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.map, &other.map)
    }
}

impl From<TableKey> for Value {
    fn from(value: TableKey) -> Self {
        match value {
            TableKey::Boolean(b) => Value::Boolean(b),
            TableKey::Integer(i) => Value::Number(i as f64),
            TableKey::String(s) => Value::String(s),
        }
    }
}

pub type StrRef = Rc<str>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TableKey {
    Boolean(bool),
    Integer(i64),
    String(StrRef),
}

impl TryFrom<Value> for TableKey {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(b) => Ok(TableKey::Boolean(b)),
            Value::Number(n) => {
                if !n.is_finite() || n.fract() != 0.0 {
                    return Err(());
                }

                let i = n as i64;

                if (i as f64) != n {
                    return Err(());
                }

                Ok(TableKey::Integer(i))
            }
            Value::String(s) => Ok(TableKey::String(s)),
            _ => Err(()),
        }
    }
}

impl From<&str> for TableKey {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Function {
    User(Rc<UserFunction>),
    Native(NativeFunction),
}

#[derive(Clone, Debug)]
pub struct FnParam {
    pub name: StrRef,
    pub default: Option<NodeId>,
}

#[derive(Clone, Debug)]
pub struct UserFunction {
    pub params: Vec<FnParam>,
    pub body: NodeId,
    pub ast: Rc<Ast>,
    pub env: EnvRef,
}

impl PartialEq for UserFunction {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && Rc::ptr_eq(&self.ast, &other.ast)
    }
}

#[derive(Clone, Debug)]
pub struct NativeFunction {
    pub name: &'static str,
    pub func: fn(CallContext) -> Result<Value, Signal>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub struct Native {
    pub data: Rc<RefCell<NativeData>>,
}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

#[derive(Debug)]
pub enum NativeData {
    Require,
}
