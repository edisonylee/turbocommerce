//! Type-safe SQLite database layer for TurboCommerce.
//!
//! Provides a simple, ergonomic API for working with Spin's SQLite database
//! with type-safe query results.
//!
//! # Example
//!
//! ```rust,ignore
//! use turbo_db::{Db, params};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Product {
//!     id: i64,
//!     name: String,
//!     price: f64,
//! }
//!
//! // In a server function
//! let db = Db::open_default()?;
//!
//! // Insert data
//! db.execute(
//!     "INSERT INTO products (name, price) VALUES (?, ?)",
//!     params!["Rust Book", 49.99]
//! )?;
//!
//! // Query with typed results
//! let products: Vec<Product> = db.query_as(
//!     "SELECT id, name, price FROM products WHERE price < ?",
//!     params![100.0]
//! )?;
//! ```

mod db;
mod error;
mod types;

pub use db::Db;
pub use error::DbError;
pub use types::{QueryResult, Row, Value};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{params, Db, DbError, QueryResult, Row, Value};
}

/// Create a parameter list for SQL queries.
///
/// # Example
///
/// ```rust,ignore
/// use turbo_db::params;
///
/// let params = params!["value1", 42, 3.14];
/// ```
#[macro_export]
macro_rules! params {
    () => {
        &[]
    };
    ($($param:expr),+ $(,)?) => {
        &[$($crate::Value::from($param)),+]
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_empty() {
        let p: &[Value] = params![];
        assert!(p.is_empty());
    }

    #[test]
    fn test_params_single() {
        let p = params![42];
        assert_eq!(p.len(), 1);
        assert!(matches!(&p[0], Value::Integer(42)));
    }

    #[test]
    fn test_params_multiple() {
        let p = params!["hello", 42, 3.14];
        assert_eq!(p.len(), 3);
        assert!(matches!(&p[0], Value::Text(s) if s == "hello"));
        assert!(matches!(&p[1], Value::Integer(42)));
        assert!(matches!(&p[2], Value::Real(f) if (*f - 3.14).abs() < 0.01));
    }

    #[test]
    fn test_params_trailing_comma() {
        let p = params!["a", "b",];
        assert_eq!(p.len(), 2);
    }
}
