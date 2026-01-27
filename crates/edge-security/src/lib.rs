//! Security infrastructure for the edge streaming SSR platform.
//!
//! This crate provides:
//! - `SandboxConfig` - Memory limits, execution timeout, resource constraints
//! - `OutboundAllowlist` - Pattern-based host filtering for outbound requests
//! - `ResourceLimits` - Max response size, fetch limits, rate limiting
//! - `ArtifactIntegrity` - Hash verification for WASM modules and assets
//!
//! # Example
//!
//! ```ignore
//! use edge_security::{SandboxConfig, OutboundAllowlist, ResourceLimits};
//!
//! // Configure sandbox
//! let sandbox = SandboxConfig::default()
//!     .with_memory_limit_mb(128)
//!     .with_execution_timeout_ms(30_000);
//!
//! // Configure outbound allowlist
//! let allowlist = OutboundAllowlist::new()
//!     .allow_host("api.example.com")
//!     .allow_pattern("*.cdn.example.com")
//!     .deny_host("internal.example.com");
//!
//! // Configure resource limits
//! let limits = ResourceLimits::default()
//!     .with_max_response_bytes(10 * 1024 * 1024)
//!     .with_max_concurrent_fetches(10);
//! ```

mod allowlist;
mod integrity;
mod limits;
mod sandbox;

pub use allowlist::*;
pub use integrity::*;
pub use limits::*;
pub use sandbox::*;
