//! Caching infrastructure for the edge streaming SSR platform.
//!
//! This crate provides:
//! - `RouteCachePolicy` - Route-level cache configuration
//! - `SectionCachePolicy` - Section/fragment-level cache configuration
//! - `FragmentCache` - Fragment caching with stampede protection
//! - `CacheKeyBuilder` - Custom cache key composition
//! - `CacheExplainHeaders` - Debug headers for cache behavior
//!
//! # Example
//!
//! ```ignore
//! use std::time::Duration;
//! use edge_cache::{RouteCachePolicy, CacheScope, VaryRule, CacheKeyBuilder};
//!
//! // Define a public cache policy with 5 minute TTL
//! let policy = RouteCachePolicy::public(Duration::from_secs(300))
//!     .with_swr(Duration::from_secs(60))
//!     .vary_on(VaryRule::header("Accept-Language"))
//!     .with_tag("products");
//!
//! // Build cache keys
//! let builder = CacheKeyBuilder::new()
//!     .route()
//!     .query_params(&["page", "sort"])
//!     .country();
//! ```

mod fragment;
mod headers;
mod key;
mod policy;

pub use fragment::*;
pub use headers::*;
pub use key::*;
pub use policy::*;
