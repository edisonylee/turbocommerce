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

mod error;
mod db;
mod types;

pub use error::DbError;
pub use db::Db;
pub use types::{Value, Row, QueryResult};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{Db, DbError, Value, Row, QueryResult, params};
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
