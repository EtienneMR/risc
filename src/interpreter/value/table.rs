use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::Value;

#[derive(Debug, Clone)]
pub struct Table {
    map: Rc<RefCell<HashMap<Value, Value>>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            map: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn has(&self, key: &Value) -> bool {
        self.map.borrow().get(key.into()).is_some()
    }

    pub fn get(&self, key: &Value) -> Value {
        self.map
            .borrow()
            .get(key.into())
            .cloned()
            .unwrap_or(Value::Nil)
    }

    pub fn set(&self, key: Value, value: Value) {
        self.map.borrow_mut().insert(key, value);
    }

    pub fn remove(&self, key: &Value) {
        self.map.borrow_mut().remove(key);
    }

    pub fn keys(&self) -> Vec<Value> {
        self.map.borrow_mut().keys().map(|v| v.clone()).collect()
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
impl Eq for Table {}

impl Hash for Table {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.map).hash(state);
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let map = self.map.borrow();

        if map.is_empty() {
            return write!(f, "{{}}");
        }

        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(k, _)| k.to_string());

        write!(f, "{{")?;
        for (i, (key, val)) in entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{key} = {val}")?;
        }
        write!(f, "}}")
    }
}

impl From<Vec<Value>> for Table {
    fn from(values: Vec<Value>) -> Self {
        let table = Self::new();
        for (index, value) in values.into_iter().enumerate() {
            table.set(Value::Number((index as f64).into()), value);
        }
        table
    }
}
