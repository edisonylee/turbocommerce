//! Data access layer with dependency tagging and timeouts.
//!
//! This crate provides:
//! - `FetchClient` - Platform fetch with automatic timeout/retry
//! - `DependencyTag` - Semantic dependency categories
//! - `TimeoutConfig` - Per-dependency timeouts
//! - `RetryPolicy` - Retry strategies

mod client;
mod dependency;
mod retry;
mod timeout;

pub use client::*;
pub use dependency::*;
pub use retry::*;
pub use timeout::*;
