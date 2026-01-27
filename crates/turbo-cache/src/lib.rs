//! Type-safe Key-Value caching layer for TurboCommerce.
//!
//! Provides a simple, ergonomic API for caching data in Spin's Key-Value Store
//! with automatic JSON serialization.
//!
//! # Example
//!
//! ```rust,ignore
//! use turbo_cache::Cache;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Cart {
//!     items: Vec<CartItem>,
//! }
//!
//! // In a server function
//! let cache = Cache::open_default()?;
//!
//! // Store a value
//! cache.set("cart:user123", &cart)?;
//!
//! // Retrieve a value
//! let cart: Option<Cart> = cache.get("cart:user123")?;
//!
//! // Delete a value
//! cache.delete("cart:user123")?;
//! ```

mod error;
mod kv;
mod session;

pub use error::CacheError;
pub use kv::Cache;
pub use session::{Session, SessionId};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{Cache, CacheError, Session, SessionId};
}
