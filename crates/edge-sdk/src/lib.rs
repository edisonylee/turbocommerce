//! Public SDK for the edge streaming SSR platform.
//!
//! This crate re-exports all platform functionality:
//!
//! ```ignore
//! use edge_sdk::prelude::*;
//!
//! #[workload(name = "my-workload")]
//! async fn handle(ctx: RequestContext, sink: StreamingSink) -> Result<()> {
//!     let logger = StructuredLogger::new(ctx.request_id.clone());
//!     logger.info("Handling request");
//!
//!     sink.send_shell(&shell)?;
//!
//!     let data = ctx.fetch_client()
//!         .fetch::<MyData>(url, DependencyTag::Cms)
//!         .await?;
//!
//!     sink.send_section("content", &render(&data))?;
//!     Ok(())
//! }
//! ```

pub use edge_cache;
pub use edge_core;
pub use edge_data;
pub use edge_executor;
pub use edge_macros::*;
pub use edge_observability;
pub use edge_security;
pub use edge_streaming;

/// Prelude for convenient imports.
pub mod prelude {
    pub use edge_cache::*;
    pub use edge_core::*;
    pub use edge_data::*;
    pub use edge_executor::*;
    pub use edge_macros::*;
    pub use edge_observability::*;
    pub use edge_security::*;
    pub use edge_streaming::*;
}
