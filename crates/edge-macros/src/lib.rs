//! Procedural macros for workload and section definitions.
//!
//! Provides:
//! - `#[workload]` - Define a workload handler
//! - `#[section]` - Define a section renderer

use proc_macro::TokenStream;

/// Marks a function as a workload handler.
///
/// # Example
/// ```ignore
/// #[workload(name = "my-workload", version = "0.1.0")]
/// #[route("/api/products/:id")]
/// async fn handle(ctx: RequestContext, sink: StreamingSink) -> Result<()> {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn workload(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement macro
    item
}

/// Marks a function as a section renderer.
///
/// # Example
/// ```ignore
/// #[section(name = "product-hero", depends_on = [Pricing, Inventory])]
/// async fn render_hero(data: &ProductData) -> String {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn section(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Implement macro
    item
}
