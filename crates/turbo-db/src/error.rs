//! Database error types.

use thiserror::Error;

/// Errors that can occur when using the database.
#[derive(Error, Debug)]
pub enum DbError {
    /// Failed to open the database.
    #[error("Failed to open database: {0}")]
    OpenError(String),

    /// Failed to execute a query.
    #[error("Query execution failed: {0}")]
    QueryError(String),

    /// Failed to deserialize a row.
    #[error("Deserialization error: {0}")]
    DeserializeError(String),

    /// Type conversion error.
    #[error("Type conversion error: {0}")]
    TypeError(String),

    /// No rows returned when one was expected.
    #[error("No rows returned")]
    NotFound,
}

impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        DbError::DeserializeError(e.to_string())
    }
}
