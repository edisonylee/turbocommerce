//! Application components and pages.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos::server_fn::error::ServerFnError;

// ============================================================================
// Shell (SSR entry point)
// ============================================================================

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos::view;
    use leptos::hydration::{AutoReload, HydrationScripts};

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() root=""/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

// ============================================================================
// App Component
// ============================================================================

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let fallback = || view! { <NotFound/> }.into_view();

    view! {
        <Stylesheet id="leptos" href="/pkg/turbo_storefront.css"/>
        <Meta name="description" content="TurboCommerce - Blazing fast e-commerce powered by Rust + WASM"/>
        <Title text="TurboCommerce Store"/>

        <Router>
            <Header/>
            <main>
                <Routes fallback>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("/products") view=ProductsPage/>
                    <Route path=path!("/product/:id") view=ProductPage/>
                    <Route path=path!("/cart") view=CartPage/>
                    <Route path=path!("/*any") view=NotFound/>
                </Routes>
            </main>
            <Footer/>
        </Router>
    }
}

// ============================================================================
// Layout Components
// ============================================================================

#[component]
fn Header() -> impl IntoView {
    view! {
        <header>
            <h1>"⚡ TurboCommerce"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/products">"Products"</a>
                <a href="/cart">"Cart"</a>
            </nav>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer>
            <p>"Built with TurboCommerce - Pure Rust, WASM-native, Walmart-scale"</p>
        </footer>
    }
}

// ============================================================================
// Pages
// ============================================================================

/// Home page with hero section
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="hero">
            <h2>"Welcome to TurboCommerce"</h2>
            <p>"The first open-source, WASM-native e-commerce framework"</p>
            <a href="/products" class="btn" style="margin-top: 1rem; display: inline-block;">
                "Browse Products"
            </a>
        </div>

        <h2>"Featured Products"</h2>
        <leptos::suspense::Suspense fallback=move || view! { <ProductGridSkeleton/> }>
            <ProductGrid/>
        </leptos::suspense::Suspense>
    }
}

/// Products listing page
#[component]
fn ProductsPage() -> impl IntoView {
    view! {
        <h2>"All Products"</h2>
        <leptos::suspense::Suspense fallback=move || view! { <ProductGridSkeleton/> }>
            <ProductGrid/>
        </leptos::suspense::Suspense>
    }
}

/// Single product page
#[component]
fn ProductPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let id = move || params.get().get("id").unwrap_or_default();

    view! {
        <leptos::suspense::Suspense fallback=move || view! { <ProductDetailSkeleton/> }>
            <ProductDetail id=id()/>
        </leptos::suspense::Suspense>
    }
}

/// Shopping cart page
#[component]
fn CartPage() -> impl IntoView {
    view! {
        <h2>"Shopping Cart"</h2>
        <p>"Your cart is empty. "</p>
        <a href="/products">"Continue shopping →"</a>
    }
}

/// 404 page
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(resp) = use_context::<leptos_wasi::response::ResponseOptions>() {
            resp.set_status(leptos_wasi::prelude::StatusCode::NOT_FOUND);
        }
    }

    view! {
        <div style="text-align: center; padding: 4rem;">
            <h1>"404"</h1>
            <p>"Page not found"</p>
            <a href="/">"← Back to Home"</a>
        </div>
    }
}

// ============================================================================
// Product Components
// ============================================================================

#[component]
fn ProductGrid() -> impl IntoView {
    // In a real app, this would fetch from a server function
    let products = vec![
        ("1", "Rust Programming Book", "$49.99"),
        ("2", "WASM Development Kit", "$99.99"),
        ("3", "Edge Computing Guide", "$39.99"),
        ("4", "Performance Tuning Pro", "$79.99"),
    ];

    view! {
        <div class="products">
            {products.into_iter().map(|(id, name, price)| {
                view! {
                    <ProductCard id=id.to_string() name=name.to_string() price=price.to_string()/>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn ProductCard(id: String, name: String, price: String) -> impl IntoView {
    let href = format!("/product/{}", id);

    view! {
        <div class="product-card">
            <div style="width: 100%; height: 200px; background: #f0f0f0; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 3rem;">"📦"</span>
            </div>
            <div class="product-info">
                <h3>{name}</h3>
                <p class="price">{price}</p>
                <a href=href class="btn" style="margin-top: 0.5rem; display: block; text-align: center;">
                    "View Details"
                </a>
            </div>
        </div>
    }
}

#[component]
fn ProductDetail(id: String) -> impl IntoView {
    // In a real app, this would fetch product data
    let name = format!("Product {}", id);
    let price = "$99.99";

    view! {
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
            <div style="background: #f0f0f0; height: 400px; display: flex; align-items: center; justify-content: center; border-radius: 8px;">
                <span style="font-size: 6rem;">"📦"</span>
            </div>
            <div>
                <h1>{name}</h1>
                <p class="price" style="font-size: 2rem; margin: 1rem 0;">{price}</p>
                <p style="color: #666; margin-bottom: 2rem;">
                    "This is a high-quality product built with Rust and deployed on WASM. "
                    "It features blazing fast performance and type-safe guarantees."
                </p>
                <button class="btn">"Add to Cart"</button>
            </div>
        </div>
    }
}

// ============================================================================
// Skeleton Components (Loading States)
// ============================================================================

#[component]
fn ProductGridSkeleton() -> impl IntoView {
    view! {
        <div class="products">
            <ProductCardSkeleton/>
            <ProductCardSkeleton/>
            <ProductCardSkeleton/>
            <ProductCardSkeleton/>
        </div>
    }
}

#[component]
fn ProductCardSkeleton() -> impl IntoView {
    view! {
        <div class="product-card">
            <div class="skeleton" style="width: 100%; height: 200px;"></div>
            <div class="product-info">
                <div class="skeleton" style="width: 80%; height: 1.5rem; margin-bottom: 0.5rem;"></div>
                <div class="skeleton" style="width: 40%; height: 1.25rem;"></div>
            </div>
        </div>
    }
}

#[component]
fn ProductDetailSkeleton() -> impl IntoView {
    view! {
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
            <div class="skeleton" style="height: 400px; border-radius: 8px;"></div>
            <div>
                <div class="skeleton" style="width: 60%; height: 2rem; margin-bottom: 1rem;"></div>
                <div class="skeleton" style="width: 30%; height: 2rem; margin-bottom: 2rem;"></div>
                <div class="skeleton" style="width: 100%; height: 4rem; margin-bottom: 1rem;"></div>
                <div class="skeleton" style="width: 150px; height: 3rem;"></div>
            </div>
        </div>
    }
}

// ============================================================================
// Server Functions (API)
// ============================================================================

#[leptos::server(prefix = "/api")]
pub async fn get_products() -> Result<Vec<(String, String, String)>, ServerFnError> {
    // Simulate database fetch
    Ok(vec![
        ("1".to_string(), "Rust Programming Book".to_string(), "$49.99".to_string()),
        ("2".to_string(), "WASM Development Kit".to_string(), "$99.99".to_string()),
        ("3".to_string(), "Edge Computing Guide".to_string(), "$39.99".to_string()),
        ("4".to_string(), "Performance Tuning Pro".to_string(), "$79.99".to_string()),
    ])
}

#[leptos::server(prefix = "/api")]
pub async fn get_product(id: String) -> Result<(String, String, String), ServerFnError> {
    // Simulate database fetch
    Ok((id.clone(), format!("Product {}", id), "$99.99".to_string()))
}
