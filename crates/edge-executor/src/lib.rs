//! Stream-as-ready section execution.
//!
//! This crate enables concurrent section rendering:
//! - `SectionScheduler` - Concurrent section execution
//! - `OrderingStrategy` - Out-of-order streaming
//! - `FallbackStrategy` - Section failure handling

mod fallback;
mod ordering;
mod scheduler;

pub use fallback::*;
pub use ordering::*;
pub use scheduler::*;
