//! Streaming primitives for shell-first SSR.
//!
//! This crate enforces shell-first streaming patterns:
//! - `StreamingSink` - Platform-controlled streaming
//! - `Shell` - Shell template abstraction
//! - `Section` - Named streamable sections
//! - `FlushPolicy` - Explicit flush control

mod flush;
mod section;
mod shell;
mod sink;

pub use flush::*;
pub use section::*;
pub use shell::*;
pub use sink::*;
