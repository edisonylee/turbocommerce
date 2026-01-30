//! # TurboCommerce SDK
//!
//! The first open-source, WASM-native, pure Rust web framework
//! targeting enterprise e-commerce at Walmart scale.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use turbo_sdk::prelude::*;
//!
//! #[page("/")]
//! fn HomePage() -> impl IntoView {
//!     view! {
//!         <h1>"Welcome to TurboCommerce"</h1>
//!         <p>"Blazing fast e-commerce, powered by Rust + WASM"</p>
//!     }
//! }
//!
//! #[page("/product/:id")]
//! fn ProductPage() -> impl IntoView {
//!     let params = use_params_map();
//!     let id = move || params.get().get("id").unwrap_or_default();
//!
//!     view! {
//!         <Suspense fallback=|| view! { <div>"Loading..."</div> }>
//!             <ProductDetails id=id()/>
//!         </Suspense>
//!     }
//! }
//!
//! #[api]
//! pub async fn get_product(id: String) -> Result<Product, ServerFnError> {
//!     // Fetch from database
//!     Ok(product)
//! }
//! ```
//!
//! ## Features
//!
//! - **Streaming SSR**: Shell-first rendering with progressive hydration
//! - **File-based Routing**: Next.js-style routing with `#[page]` macro
//! - **Server Functions**: Type-safe RPC with `#[api]` macro
//! - **WASM-native**: Runs on Spin, Fermyon Cloud, Cloudflare Workers
//! - **E-commerce Ready**: Built-in primitives for products, cart, checkout
//!
//! ## Architecture
//!
//! TurboCommerce is built on:
//! - [Leptos](https://leptos.dev) for reactive UI and SSR
//! - [Spin](https://spin.fermyon.dev) for WASM deployment
//! - Custom streaming layer for shell-first SSR
//!
//! ## Crate Features
//!
//! - `ssr` - Enable server-side rendering (required for Spin deployment)
//! - `hydrate` - Enable client-side hydration (required for interactivity)

pub mod prelude;

// Re-export core crates
pub use turbo_auth;
pub use turbo_commerce;
pub use turbo_core;
pub use turbo_macros;
pub use turbo_router;

// Re-export Leptos essentials at the top level for convenience
pub use leptos::{view, IntoView};
pub use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};

// Re-export macros
pub use turbo_macros::{api, component, page};

// Re-export router essentials
pub use turbo_router::{
    path, use_params, use_params_map, use_query, use_query_map, Route, RouteMeta, RouteRegistry,
    Router, Routes,
};

// Re-export core types
pub use turbo_core::{TurboApp, TurboConfig, TurboError};

#[cfg(feature = "ssr")]
pub use turbo_core::{generate_shell_html, StreamConfig};
