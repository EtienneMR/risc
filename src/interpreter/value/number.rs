use std::{
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Copy)]
pub struct Number {
    pub value: f64,
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self { value }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.value.to_bits() == other.value.to_bits()
    }
}
impl Eq for Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.fract() == 0.0 && self.value.abs() < 1e15 {
            write!(f, "{}", self.value as i64)
        } else {
            write!(f, "{}", self.value)
        }
    }
}
