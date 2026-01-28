//! Cache error types.

use thiserror::Error;

/// Errors that can occur when using the cache.
#[derive(Error, Debug)]
pub enum CacheError {
    /// Failed to open the store.
    #[error("Failed to open store: {0}")]
    OpenError(String),

    /// Failed to serialize value.
    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),

    /// Failed to perform store operation.
    #[error("Store operation failed: {0}")]
    StoreError(String),

    /// Key not found.
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Concurrent modification detected.
    #[error("Concurrent modification: {0}")]
    ConcurrentModification(String),
}
