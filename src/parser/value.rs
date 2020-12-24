use crate::ValidationError;
use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, SubAssign};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    None,
    String(String),
    Number(f64),
    Bool(bool),
}

impl AddAssign<&Self> for Value {
    fn add_assign(&mut self, rhs: &Self) {
        match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => *self = Self::Number(n1 + n2),
            _ => (),
        }
    }
}

impl SubAssign<&Self> for Value {
    fn sub_assign(&mut self, rhs: &Self) {
        match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => *self = Self::Number(*n1 - n2),
            _ => *self = Self::None,
        }
    }
}

impl Value {
    pub fn same_type(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Value::Bool(_), Value::Bool(_)) => true,
            (Value::Number(_), Value::Number(_)) => true,
            (Value::String(_), Value::String(_)) => true,
            _ => false,
        }
    }

    fn from_yaml(yaml_value: serde_yaml::Value) -> Result<Self, ValidationError> {
        match yaml_value {
            serde_yaml::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_yaml::Value::String(s) => Ok(Value::String(s)),
            serde_yaml::Value::Number(n) => Ok(Value::Number(n.as_f64().unwrap())),
            _ => Err(verror!("Cannot create value from {:?}", yaml_value)),
        }
    }

    pub fn parse(text: &str) -> Result<Value, ValidationError> {
        match serde_yaml::from_str(&text) {
            Ok(r) => Self::from_yaml(r),
            Err(e) => Err(verror!("{}", e)),
        }
    }
}
