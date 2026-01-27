//! Observability infrastructure for the edge streaming SSR platform.
//!
//! This crate provides:
//! - `RequestId` - Unique request identifier with trace context
//! - `StructuredLogger` - Structured logging with request context
//! - `Metrics` - Platform-level timing metrics
//! - `ReplayRecorder` / `ReplayPlayer` - Local debugging support

mod logging;
mod metrics;
mod replay;
mod span;

pub use logging::*;
pub use metrics::*;
pub use replay::*;
pub use span::*;

// Re-export RequestId and TimingContext from edge-core for convenience
pub use edge_core::{RequestId, TimingContext};
