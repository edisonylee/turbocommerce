//! TurboCommerce Core Framework
//!
//! The first open-source, WASM-native, pure Rust web framework
//! targeting enterprise e-commerce at scale.
//!
//! # Architecture
//!
//! TurboCore wraps Leptos with:
//! - Streaming SSR optimized for edge deployment
//! - File-based routing with `#[page]` macro
//! - Server functions with `#[api]` macro
//! - Integrated caching and data fetching
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use turbo_core::prelude::*;
//!
//! #[page("/")]
//! fn HomePage() -> impl IntoView {
//!     view! {
//!         <h1>"Welcome to TurboCommerce"</h1>
//!     }
//! }
//!
//! #[page("/product/:id")]
//! fn ProductPage() -> impl IntoView {
//!     let params = use_params_map();
//!     let id = params.get("id").unwrap_or_default();
//!
//!     view! {
//!         <Suspense fallback=|| view! { <ProductSkeleton/> }>
//!             <ProductHero id=id.clone()/>
//!         </Suspense>
//!     }
//! }
//! ```

pub mod prelude;
mod app;
mod error;

#[cfg(feature = "ssr")]
mod server;

pub use app::*;
pub use error::*;

#[cfg(feature = "ssr")]
pub use server::*;

// Re-export Leptos essentials
pub use leptos::{
    prelude::*,
    view,
    component,
    server as leptos_server_macro,
    IntoView,
    suspense::Suspense,
};
pub use leptos_meta::{provide_meta_context, Meta, Stylesheet, Title, MetaTags};
pub use leptos_router::components::{Route, Router, Routes};

// Re-export turbo-router
pub use turbo_router::{
    RouteMeta, RouteEntry, RouteRegistry,
    use_params, use_params_map, use_query, use_query_map,
    path,
};
