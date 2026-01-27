//! Prelude for convenient imports.
//!
//! ```rust,ignore
//! use turbo_sdk::prelude::*;
//! ```
//!
//! This imports all commonly used items:
//! - View macros: `view!`
//! - Traits: `IntoView`
//! - Routing: `use_params_map`, `use_query_map`, `path!`
//! - Components: `Router`, `Routes`, `Route`, `Suspense`
//! - Meta: `Title`, `Meta`, `Stylesheet`
//! - Macros: `#[page]`, `#[api]`, `#[component]`

// Leptos view macro and core traits
pub use leptos::{
    prelude::*,
    view,
    IntoView,
    suspense::Suspense,
};

// Meta tags
pub use leptos_meta::{provide_meta_context, Meta, Stylesheet, Title};

// Router
pub use turbo_router::{
    Route, Router, Routes,
    use_params, use_params_map, use_query, use_query_map,
    path,
};

// Macros
pub use turbo_macros::{page, api, component};

// Core types
pub use turbo_core::{TurboApp, TurboConfig, TurboError};

#[cfg(feature = "ssr")]
pub use turbo_core::{StreamConfig, generate_shell_html};
