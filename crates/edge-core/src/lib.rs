//! Core abstractions for the edge streaming SSR platform.
//!
//! This crate provides the fundamental types and traits:
//! - `WorkloadManifest` - Workload configuration
//! - `Workload` trait - Handler interface
//! - `RequestContext` - Typed request parameters
//! - `LifecyclePhase` - Request lifecycle tracking

mod config;
mod context;
mod lifecycle;
mod workload;

pub use config::*;
pub use context::*;
pub use lifecycle::*;
pub use workload::*;
