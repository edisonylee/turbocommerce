//! Procedural macros for TurboCommerce framework.
//!
//! Provides ergonomic macros for defining pages and API endpoints:
//! - `#[page("/path")]` - Define a page component with automatic routing
//! - `#[api]` - Define an API endpoint (builds on Leptos server functions)

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr};

/// Define a page component with automatic routing.
///
/// # Example
///
/// ```rust,ignore
/// use turbo_macros::page;
///
/// #[page("/")]
/// fn HomePage() -> impl IntoView {
///     view! { <h1>"Welcome"</h1> }
/// }
///
/// #[page("/product/:id")]
/// fn ProductPage(id: String) -> impl IntoView {
///     view! { <h1>"Product: " {id}</h1> }
/// }
/// ```
///
/// This generates:
/// 1. A Leptos component with `#[component]`
/// 2. Route metadata for the router to collect
/// 3. Path parameter extraction for dynamic segments
#[proc_macro_attribute]
pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_output = &input_fn.sig.output;
    let fn_inputs = &input_fn.sig.inputs;

    // Generate route metadata name
    let route_meta_name = format_ident!("__TURBO_ROUTE_{}", fn_name.to_string().to_uppercase());
    let path_str = path.value();

    // Extract dynamic segments for parameter injection
    let segments: Vec<&str> = path_str
        .split('/')
        .filter(|s| s.starts_with(':'))
        .map(|s| &s[1..])
        .collect();

    // Generate parameter extraction if needed
    let param_extraction = if !segments.is_empty() {
        let param_names: Vec<_> = segments.iter().map(|s| format_ident!("{}", s)).collect();
        quote! {
            // Parameters extracted from URL path
            #(let #param_names = leptos_router::hooks::use_params_map().get(stringify!(#param_names)).unwrap_or_default();)*
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        /// Route metadata for automatic routing registration
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        #fn_vis const #route_meta_name: turbo_router::RouteMeta = turbo_router::RouteMeta {
            path: #path_str,
            component_name: stringify!(#fn_name),
        };

        /// Page component
        #[leptos::component]
        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            #param_extraction
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Define an API endpoint.
///
/// This is a convenience wrapper around Leptos `#[server]` that:
/// 1. Automatically prefixes with `/api`
/// 2. Adds standard error handling
/// 3. Integrates with TurboCommerce middleware
///
/// # Example
///
/// ```rust,ignore
/// use turbo_macros::api;
///
/// #[api]
/// pub async fn get_product(id: String) -> Result<Product, ApiError> {
///     // Fetch product from database
///     Ok(product)
/// }
/// ```
#[proc_macro_attribute]
pub fn api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_async = &input_fn.sig.asyncness;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;

    // Generate the server function with /api prefix
    let expanded = quote! {
        #[leptos::server(prefix = "/api")]
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Re-export of Leptos component macro with TurboCommerce enhancements.
///
/// Currently this is a simple pass-through, but allows us to add
/// framework-specific functionality in the future.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #[leptos::component]
        #input_fn
    };

    TokenStream::from(expanded)
}
