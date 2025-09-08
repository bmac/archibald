//! Value types for SQL parameters

use serde::{Deserialize, Serialize};

/// A SQL value that can be used as a parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// 32-bit integer
    I32(i32),
    /// 64-bit integer
    I64(i64),
    /// 32-bit float
    F32(f32),
    /// 64-bit float
    F64(f64),
    /// String value
    String(String),
    /// Bytes value
    Bytes(Vec<u8>),
    /// JSON value
    Json(serde_json::Value),
    /// Array of values
    Array(Vec<Value>),
    /// Subquery placeholder (actual subquery stored separately)
    SubqueryPlaceholder,
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Get the SQL type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Bool(_) => "BOOLEAN",
            Value::I32(_) => "INTEGER",
            Value::I64(_) => "BIGINT",
            Value::F32(_) => "REAL",
            Value::F64(_) => "DOUBLE PRECISION",
            Value::String(_) => "TEXT",
            Value::Bytes(_) => "BYTEA",
            Value::Json(_) => "JSON",
            Value::Array(_) => "ARRAY",
            Value::SubqueryPlaceholder => "SUBQUERY",
        }
    }

    /// Extract array values if this is an Array variant
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

// Implement From for common types
impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}

impl From<i32> for Value {
    fn from(val: i32) -> Self {
        Value::I32(val)
    }
}

impl From<i64> for Value {
    fn from(val: i64) -> Self {
        Value::I64(val)
    }
}

impl From<f32> for Value {
    fn from(val: f32) -> Self {
        Value::F32(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::F64(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::String(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Value::String(val.to_string())
    }
}

impl From<Vec<u8>> for Value {
    fn from(val: Vec<u8>) -> Self {
        Value::Bytes(val)
    }
}

impl From<serde_json::Value> for Value {
    fn from(val: serde_json::Value) -> Self {
        Value::Json(val)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(vals: Vec<T>) -> Self {
        Value::Array(vals.into_iter().map(|v| v.into()).collect())
    }
}

impl<T> From<&[T]> for Value
where
    T: Clone + Into<Value>,
{
    fn from(vals: &[T]) -> Self {
        Value::Array(vals.iter().cloned().map(|v| v.into()).collect())
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => val.into(),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        assert_eq!(Value::from(42i32), Value::I32(42));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from("hello"), Value::String("hello".to_string()));
        assert_eq!(Value::from(()), Value::Null);
    }

    #[test]
    fn test_array_conversion() {
        let arr = vec![1, 2, 3];
        let value = Value::from(arr);
        assert_eq!(
            value,
            Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)])
        );
    }

    #[test]
    fn test_slice_conversion() {
        let arr: &[i32] = &[1, 2, 3];
        let value = Value::from(arr);
        assert_eq!(
            value,
            Value::Array(vec![Value::I32(1), Value::I32(2), Value::I32(3)])
        );
    }

    #[test]
    fn test_option_conversion() {
        assert_eq!(Value::from(Some(42i32)), Value::I32(42));
        assert_eq!(Value::from(None::<i32>), Value::Null);
    }

    #[test]
    fn test_is_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::I32(42).is_null());
    }

    #[test]
    fn test_type_names() {
        assert_eq!(Value::I32(42).type_name(), "INTEGER");
        assert_eq!(Value::String("test".to_string()).type_name(), "TEXT");
        assert_eq!(Value::Bool(true).type_name(), "BOOLEAN");
        assert_eq!(Value::Null.type_name(), "NULL");
    }
}
