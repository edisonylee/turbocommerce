//! Database value types and conversions.

use crate::DbError;
use serde::de::DeserializeOwned;
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
                    serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(b))
                })
        }
    }
}

// Base64 encoding for blobs (minimal implementation for WASM compatibility)
mod base64 {
    pub mod engine {
        pub mod general_purpose {
            pub struct Standard;
            impl Standard {
                pub fn encode(&self, data: &[u8]) -> String {
                    const CHARS: &[u8] =
                        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Value Conversion Tests ===

    #[test]
    fn test_value_from_i32() {
        let v = Value::from(42i32);
        assert!(matches!(v, Value::Integer(42)));
    }

    #[test]
    fn test_value_from_i64() {
        let v = Value::from(9_999_999_999i64);
        assert!(matches!(v, Value::Integer(9_999_999_999)));
    }

    #[test]
    fn test_value_from_f32() {
        let v = Value::from(3.14f32);
        if let Value::Real(f) = v {
            assert!((f - 3.14).abs() < 0.01);
        } else {
            panic!("Expected Value::Real");
        }
    }

    #[test]
    fn test_value_from_f64() {
        let v = Value::from(2.718281828f64);
        assert!(matches!(v, Value::Real(f) if (f - 2.718281828).abs() < 0.0001));
    }

    #[test]
    fn test_value_from_str() {
        let v = Value::from("hello");
        assert!(matches!(v, Value::Text(s) if s == "hello"));
    }

    #[test]
    fn test_value_from_string() {
        let v = Value::from(String::from("world"));
        assert!(matches!(v, Value::Text(s) if s == "world"));
    }

    #[test]
    fn test_value_from_vec_u8() {
        let v = Value::from(vec![1u8, 2, 3]);
        assert!(matches!(v, Value::Blob(b) if b == vec![1, 2, 3]));
    }

    #[test]
    fn test_value_from_bool_true() {
        let v = Value::from(true);
        assert!(matches!(v, Value::Integer(1)));
    }

    #[test]
    fn test_value_from_bool_false() {
        let v = Value::from(false);
        assert!(matches!(v, Value::Integer(0)));
    }

    #[test]
    fn test_value_from_option_some() {
        let v = Value::from(Some(42i64));
        assert!(matches!(v, Value::Integer(42)));
    }

    #[test]
    fn test_value_from_option_none() {
        let v = Value::from(None::<i64>);
        assert!(v.is_null());
    }

    // === Value Accessor Tests ===

    #[test]
    fn test_value_as_integer() {
        let v = Value::Integer(42);
        assert_eq!(v.as_integer(), Some(42));

        let v = Value::Real(3.7);
        assert_eq!(v.as_integer(), Some(3)); // Truncates

        let v = Value::Text("hello".to_string());
        assert_eq!(v.as_integer(), None);
    }

    #[test]
    fn test_value_as_real() {
        let v = Value::Real(3.14);
        assert_eq!(v.as_real(), Some(3.14));

        let v = Value::Integer(42);
        assert_eq!(v.as_real(), Some(42.0));

        let v = Value::Null;
        assert_eq!(v.as_real(), None);
    }

    #[test]
    fn test_value_as_text() {
        let v = Value::Text("hello".to_string());
        assert_eq!(v.as_text(), Some("hello"));

        let v = Value::Integer(42);
        assert_eq!(v.as_text(), None);
    }

    #[test]
    fn test_value_as_blob() {
        let v = Value::Blob(vec![1, 2, 3]);
        assert_eq!(v.as_blob(), Some(&[1u8, 2, 3][..]));

        let v = Value::Text("hello".to_string());
        assert_eq!(v.as_blob(), Some(b"hello".as_slice()));

        let v = Value::Null;
        assert_eq!(v.as_blob(), None);
    }

    #[test]
    fn test_value_is_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Integer(0).is_null());
    }

    // === Row Tests ===

    #[test]
    fn test_row_new_and_columns() {
        let cols = vec!["id".to_string(), "name".to_string()];
        let vals = vec![Value::Integer(1), Value::Text("Alice".to_string())];
        let row = Row::new(cols.clone(), vals);

        assert_eq!(row.columns(), &cols);
    }

    #[test]
    fn test_row_get_by_name() {
        let row = Row::new(
            vec!["id".to_string(), "name".to_string()],
            vec![Value::Integer(42), Value::Text("Bob".to_string())],
        );

        assert!(matches!(row.get("id"), Some(Value::Integer(42))));
        assert!(matches!(row.get("name"), Some(Value::Text(s)) if s == "Bob"));
        assert!(row.get("missing").is_none());
    }

    #[test]
    fn test_row_get_by_index() {
        let row = Row::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Value::Integer(1), Value::Integer(2)],
        );

        assert!(matches!(row.get_index(0), Some(Value::Integer(1))));
        assert!(matches!(row.get_index(1), Some(Value::Integer(2))));
        assert!(row.get_index(2).is_none());
    }

    #[test]
    fn test_row_to_map() {
        let row = Row::new(
            vec!["x".to_string(), "y".to_string()],
            vec![Value::Integer(10), Value::Integer(20)],
        );

        let map = row.to_map();
        assert_eq!(map.len(), 2);
        assert!(matches!(map.get("x"), Some(Value::Integer(10))));
    }

    #[test]
    fn test_row_deserialize() {
        use serde::Deserialize;

        #[derive(Deserialize, Debug, PartialEq)]
        struct Person {
            id: i64,
            name: String,
        }

        let row = Row::new(
            vec!["id".to_string(), "name".to_string()],
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
        );

        let person: Person = row.deserialize().unwrap();
        assert_eq!(
            person,
            Person {
                id: 1,
                name: "Alice".to_string()
            }
        );
    }

    // === QueryResult Tests ===

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::new(vec!["id".to_string()], vec![]);
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert!(result.first().is_none());
    }

    #[test]
    fn test_query_result_with_rows() {
        let cols = vec!["id".to_string()];
        let rows = vec![
            Row::new(cols.clone(), vec![Value::Integer(1)]),
            Row::new(cols.clone(), vec![Value::Integer(2)]),
        ];
        let result = QueryResult::new(cols, rows);

        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert!(result.first().is_some());
    }

    #[test]
    fn test_query_result_iter() {
        let cols = vec!["v".to_string()];
        let rows = vec![
            Row::new(cols.clone(), vec![Value::Integer(1)]),
            Row::new(cols.clone(), vec![Value::Integer(2)]),
            Row::new(cols.clone(), vec![Value::Integer(3)]),
        ];
        let result = QueryResult::new(cols, rows);

        let count = result.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_query_result_deserialize_all() {
        use serde::Deserialize;

        #[derive(Deserialize, Debug, PartialEq)]
        struct Item {
            value: i64,
        }

        let cols = vec!["value".to_string()];
        let rows = vec![
            Row::new(cols.clone(), vec![Value::Integer(10)]),
            Row::new(cols.clone(), vec![Value::Integer(20)]),
        ];
        let result = QueryResult::new(cols, rows);

        let items: Vec<Item> = result.deserialize_all().unwrap();
        assert_eq!(items, vec![Item { value: 10 }, Item { value: 20 }]);
    }

    // === Base64 Encoding Tests ===

    #[test]
    fn test_base64_encode_hello() {
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"Hello");
        assert_eq!(encoded, "SGVsbG8=");
    }

    #[test]
    fn test_base64_encode_padding() {
        // "a" -> "YQ==" (2 padding chars)
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"a");
        assert_eq!(encoded, "YQ==");

        // "ab" -> "YWI=" (1 padding char)
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"ab");
        assert_eq!(encoded, "YWI=");

        // "abc" -> "YWJj" (no padding)
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"abc");
        assert_eq!(encoded, "YWJj");
    }
}
