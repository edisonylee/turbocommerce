//! Prelude for convenient imports.
//!
//! ```rust,ignore
//! use turbo_core::prelude::*;
//! ```

// Leptos view macro and traits
pub use leptos::{
    prelude::*,
    view,
    IntoView,
    suspense::Suspense,
};

// Leptos meta tags
pub use leptos_meta::{provide_meta_context, Meta, Stylesheet, Title};

// Router
pub use turbo_router::prelude::*;

// TurboCore types
pub use crate::{TurboApp, TurboConfig, TurboError};

#[cfg(feature = "ssr")]
pub use crate::server::*;
