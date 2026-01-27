//! File-based routing for TurboCommerce framework.
//!
//! This crate provides Next.js-style file-based routing on top of Leptos Router:
//!
//! ```text
//! pages/
//! ├── index.rs        -> /
//! ├── about.rs        -> /about
//! ├── product/
//! │   ├── index.rs    -> /product
//! │   └── [id].rs     -> /product/:id
//! └── blog/
//!     └── [...slug].rs -> /blog/*slug
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use turbo_router::prelude::*;
//!
//! // Define routes with #[page] macro
//! #[page("/")]
//! fn HomePage() -> impl IntoView {
//!     view! { <h1>"Home"</h1> }
//! }
//!
//! #[page("/product/:id")]
//! fn ProductPage() -> impl IntoView {
//!     let params = use_params_map();
//!     let id = params.get("id").unwrap_or_default();
//!     view! { <h1>"Product: " {id}</h1> }
//! }
//! ```

pub mod prelude;
mod route;

pub use route::*;

// Re-export leptos_router essentials
pub use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{use_params, use_params_map, use_query, use_query_map},
    path,
    ParamSegment, StaticSegment, WildcardSegment,
};
