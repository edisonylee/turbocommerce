//! Database value types and conversions.

use crate::DbError;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

/// A database value that can be used as a parameter or result.
#[derive(Debug, Clone)]
pub enum Value {
    /// Null value.
    Null,
    /// Integer value.
    Integer(i64),
    /// Real/float value.
    Real(f64),
    /// Text value.
    Text(String),
    /// Binary blob value.
    Blob(Vec<u8>),
}

impl Value {
    /// Try to get the value as an i64.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Real(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to get the value as an f64.
    pub fn as_real(&self) -> Option<f64> {
        match self {
            Value::Real(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get the value as a string.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the value as bytes.
    pub fn as_blob(&self) -> Option<&[u8]> {
        match self {
            Value::Blob(b) => Some(b),
            Value::Text(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    /// Check if the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

// Conversions from Rust types to Value
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Integer(v as i64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Real(v as f64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Blob(v)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Integer(if v { 1 } else { 0 })
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

/// A row from a query result.
#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<String>,
    values: Vec<Value>,
}

impl Row {
    /// Create a new row from columns and values.
    pub fn new(columns: Vec<String>, values: Vec<Value>) -> Self {
        Self { columns, values }
    }

    /// Get a value by column name.
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.columns
            .iter()
            .position(|c| c == column)
            .map(|i| &self.values[i])
    }

    /// Get a value by column index.
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Get the column names.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Get all values.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Convert the row to a HashMap.
    pub fn to_map(&self) -> HashMap<String, Value> {
        self.columns
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Try to deserialize the row into a type.
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, DbError> {
        // Convert row to JSON value, then deserialize
        let map: serde_json::Map<String, serde_json::Value> = self
            .columns
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (k.clone(), value_to_json(v)))
            .collect();

        let json = serde_json::Value::Object(map);
        serde_json::from_value(json).map_err(|e| DbError::DeserializeError(e.to_string()))
    }
}

/// Query result containing rows.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The column names.
    pub columns: Vec<String>,
    /// The rows.
    pub rows: Vec<Row>,
}

impl QueryResult {
    /// Create a new query result.
    pub fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self { columns, rows }
    }

    /// Get the number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the result is empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the first row.
    pub fn first(&self) -> Option<&Row> {
        self.rows.first()
    }

    /// Iterate over the rows.
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// Deserialize all rows into a vector of a type.
    pub fn deserialize_all<T: DeserializeOwned>(&self) -> Result<Vec<T>, DbError> {
        self.rows.iter().map(|row| row.deserialize()).collect()
    }
}

/// Convert a Value to a serde_json::Value.
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Integer(i) => serde_json::Value::Number((*i).into()),
        Value::Real(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::Text(s) => serde_json::Value::String(s.clone()),
        Value::Blob(b) => {
            // Try to parse as UTF-8 string, otherwise base64 encode
            String::from_utf8(b.clone())
                .map(serde_json::Value::String)
                .unwrap_or_else(|_| {
                    use base64::Engine;
                    serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(b))
                })
        }
    }
}

// Base64 encoding for blobs (minimal implementation)
mod base64 {
    pub mod engine {
        pub mod general_purpose {
            pub struct Standard;
            impl Standard {
                pub fn encode(&self, data: &[u8]) -> String {
                    // Simple base64 encoding
                    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                    let mut result = String::new();
                    for chunk in data.chunks(3) {
                        let b0 = chunk[0] as usize;
                        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
                        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

                        result.push(CHARS[b0 >> 2] as char);
                        result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
                        if chunk.len() > 1 {
                            result.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
                        } else {
                            result.push('=');
                        }
                        if chunk.len() > 2 {
                            result.push(CHARS[b2 & 0x3f] as char);
                        } else {
                            result.push('=');
                        }
                    }
                    result
                }
            }
            pub static STANDARD: Standard = Standard;
        }
    }
    pub use engine::general_purpose::STANDARD;
    pub trait Engine {
        fn encode(&self, data: &[u8]) -> String;
    }
    impl Engine for engine::general_purpose::Standard {
        fn encode(&self, data: &[u8]) -> String {
            self.encode(data)
        }
    }
}
