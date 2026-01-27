//! TurboCommerce Example Storefront
//!
//! Demonstrates the TurboCommerce framework with:
//! - File-based routing with #[page] macro
//! - Server functions with #[api] macro
//! - Streaming SSR with Suspense
//! - Interactive components

mod app;

#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
