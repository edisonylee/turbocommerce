//! Application components and pages.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos::server_fn::error::ServerFnError;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Types
// ============================================================================

/// Product from the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub stock: i64,
}

impl Product {
    /// Format the price as a dollar string.
    pub fn price_display(&self) -> String {
        format!("${:.2}", self.price_cents as f64 / 100.0)
    }
}

/// Cart item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CartItem {
    pub product_id: String,
    pub product_name: String,
    pub price_cents: i64,
    pub quantity: i64,
}

impl CartItem {
    pub fn subtotal(&self) -> i64 {
        self.price_cents * self.quantity
    }

    pub fn subtotal_display(&self) -> String {
        format!("${:.2}", self.subtotal() as f64 / 100.0)
    }
}

/// Shopping cart stored in KV.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cart {
    pub items: Vec<CartItem>,
}

impl Cart {
    pub fn total(&self) -> i64 {
        self.items.iter().map(|i| i.subtotal()).sum()
    }

    pub fn total_display(&self) -> String {
        format!("${:.2}", self.total() as f64 / 100.0)
    }

    pub fn item_count(&self) -> usize {
        self.items.iter().map(|i| i.quantity as usize).sum()
    }
}

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
            <h1>"TurboCommerce"</h1>
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
            <p style="font-size: 0.8rem; color: #888;">"Data layer: Spin SQLite + Key-Value Store"</p>
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
            <p style="font-size: 0.9rem; color: #888; margin-top: 0.5rem;">
                "Now with Spin SQLite database + Key-Value cart storage"
            </p>
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
    let cart = Resource::new(
        || (),
        |_| get_cart(),
    );

    view! {
        <h2>"Shopping Cart"</h2>
        <leptos::suspense::Suspense fallback=move || view! { <CartSkeleton/> }>
            {move || cart.get().map(|result| match result {
                Ok(cart) if cart.items.is_empty() => view! {
                    <p>"Your cart is empty."</p>
                    <a href="/products">"Continue shopping"</a>
                }.into_any(),
                Ok(cart) => view! {
                    <CartView cart=cart/>
                }.into_any(),
                Err(e) => view! {
                    <p style="color: red;">"Error loading cart: " {e.to_string()}</p>
                }.into_any(),
            })}
        </leptos::suspense::Suspense>
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
            <a href="/">"Back to Home"</a>
        </div>
    }
}

// ============================================================================
// Product Components
// ============================================================================

#[component]
fn ProductGrid() -> impl IntoView {
    let products = Resource::new(
        || (),
        |_| get_products(),
    );

    view! {
        {move || products.get().map(|result| match result {
            Ok(products) => view! {
                <div class="products">
                    {products.into_iter().map(|p| {
                        view! {
                            <ProductCard product=p/>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            }.into_any(),
            Err(e) => view! {
                <p style="color: red;">"Error loading products: " {e.to_string()}</p>
            }.into_any(),
        })}
    }
}

#[component]
fn ProductCard(product: Product) -> impl IntoView {
    let href = format!("/product/{}", product.id);
    let price = product.price_display();

    view! {
        <div class="product-card">
            <div style="width: 100%; height: 200px; background: #f0f0f0; display: flex; align-items: center; justify-content: center;">
                <span style="font-size: 3rem;">"📦"</span>
            </div>
            <div class="product-info">
                <h3>{product.name}</h3>
                <p class="price">{price}</p>
                <p style="font-size: 0.8rem; color: #666;">
                    {product.stock.to_string()} " in stock"
                </p>
                <a href=href class="btn" style="margin-top: 0.5rem; display: block; text-align: center;">
                    "View Details"
                </a>
            </div>
        </div>
    }
}

#[component]
fn ProductDetail(id: String) -> impl IntoView {
    let id_clone = id.clone();
    let product = Resource::new(
        move || id_clone.clone(),
        |id| get_product(id),
    );

    view! {
        {move || product.get().map(|result| match result {
            Ok(Some(p)) => {
                let price = p.price_display();
                let description = p.description.clone().unwrap_or_else(|| "No description available.".to_string());
                let product_id = p.id.clone();
                view! {
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
                        <div style="background: #f0f0f0; height: 400px; display: flex; align-items: center; justify-content: center; border-radius: 8px;">
                            <span style="font-size: 6rem;">"📦"</span>
                        </div>
                        <div>
                            <h1>{p.name.clone()}</h1>
                            <p class="price" style="font-size: 2rem; margin: 1rem 0;">{price}</p>
                            <p style="color: #666; margin-bottom: 1rem;">
                                {description}
                            </p>
                            <p style="color: #888; margin-bottom: 2rem;">
                                {p.stock.to_string()} " in stock"
                            </p>
                            <AddToCartButton product_id=product_id/>
                        </div>
                    </div>
                }.into_any()
            },
            Ok(None) => view! {
                <p>"Product not found"</p>
                <a href="/products">"Back to products"</a>
            }.into_any(),
            Err(e) => view! {
                <p style="color: red;">"Error loading product: " {e.to_string()}</p>
            }.into_any(),
        })}
    }
}

#[component]
fn AddToCartButton(product_id: String) -> impl IntoView {
    let add_action = ServerAction::<AddToCart>::new();
    let pending = add_action.pending();
    let value = add_action.value();

    view! {
        <ActionForm action=add_action>
            <input type="hidden" name="product_id" value=product_id/>
            <input type="hidden" name="quantity" value="1"/>
            <button
                type="submit"
                class="btn"
                disabled=move || pending.get()
            >
                {move || if pending.get() { "Adding..." } else { "Add to Cart" }}
            </button>
        </ActionForm>
        {move || value.get().map(|result| match result {
            Ok(_) => view! {
                <p style="color: green; margin-top: 0.5rem;">"Added to cart!"</p>
            }.into_any(),
            Err(e) => view! {
                <p style="color: red; margin-top: 0.5rem;">"Error: " {e.to_string()}</p>
            }.into_any(),
        })}
    }
}

// ============================================================================
// Cart Components
// ============================================================================

#[component]
fn CartView(cart: Cart) -> impl IntoView {
    let total = cart.total_display();
    let item_count = cart.item_count();

    view! {
        <div style="max-width: 600px;">
            <p style="margin-bottom: 1rem;">{item_count.to_string()} " item(s) in your cart"</p>
            {cart.items.into_iter().map(|item| {
                let subtotal = item.subtotal_display();
                let price = format!("${:.2}", item.price_cents as f64 / 100.0);
                view! {
                    <div style="display: flex; justify-content: space-between; padding: 1rem; border-bottom: 1px solid #eee;">
                        <div>
                            <strong>{item.product_name}</strong>
                            <p style="color: #666;">{price} " x " {item.quantity.to_string()}</p>
                        </div>
                        <div style="text-align: right;">
                            <strong>{subtotal}</strong>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
            <div style="display: flex; justify-content: space-between; padding: 1rem; font-size: 1.25rem;">
                <strong>"Total"</strong>
                <strong>{total}</strong>
            </div>
            <div style="margin-top: 1rem; display: flex; gap: 1rem;">
                <a href="/products" style="color: #666;">"Continue Shopping"</a>
                <ClearCartButton/>
            </div>
        </div>
    }
}

#[component]
fn ClearCartButton() -> impl IntoView {
    let clear_action = ServerAction::<ClearCart>::new();
    let pending = clear_action.pending();

    view! {
        <ActionForm action=clear_action>
            <button
                type="submit"
                style="background: #dc3545; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer;"
                disabled=move || pending.get()
            >
                {move || if pending.get() { "Clearing..." } else { "Clear Cart" }}
            </button>
        </ActionForm>
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

#[component]
fn CartSkeleton() -> impl IntoView {
    view! {
        <div style="max-width: 600px;">
            <div class="skeleton" style="width: 200px; height: 1.5rem; margin-bottom: 1rem;"></div>
            <div class="skeleton" style="width: 100%; height: 4rem; margin-bottom: 0.5rem;"></div>
            <div class="skeleton" style="width: 100%; height: 4rem; margin-bottom: 0.5rem;"></div>
            <div class="skeleton" style="width: 100%; height: 4rem;"></div>
        </div>
    }
}

// ============================================================================
// Server Functions (API)
// ============================================================================

/// Get all products from the database.
#[leptos::server(prefix = "/api")]
pub async fn get_products() -> Result<Vec<Product>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_db::{Db, params};

        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let products: Vec<Product> = db.query_as(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products ORDER BY name",
            params![]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        Ok(products)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Get a single product by ID.
#[leptos::server(prefix = "/api")]
pub async fn get_product(id: String) -> Result<Option<Product>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_db::{Db, params};

        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let product: Option<Product> = db.query_optional(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products WHERE id = ?",
            params![id.as_str()]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        Ok(product)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Get the shopping cart from KV store.
#[leptos::server(prefix = "/api")]
pub async fn get_cart() -> Result<Cart, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;

        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        // For demo purposes, use a fixed session ID
        // In production, this would come from a session cookie
        let cart: Cart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_default();

        Ok(cart)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Add an item to the cart.
#[leptos::server(prefix = "/api")]
pub async fn add_to_cart(product_id: String, quantity: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;
        use turbo_db::{Db, params};

        // Get product info from database
        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let product: Option<Product> = db.query_optional(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products WHERE id = ?",
            params![product_id.as_str()]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        let product = product.ok_or_else(|| ServerFnError::new("Product not found"))?;

        // Get current cart
        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        let mut cart: Cart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_default();

        // Check if item already in cart
        if let Some(item) = cart.items.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity += quantity;
        } else {
            cart.items.push(CartItem {
                product_id: product.id,
                product_name: product.name,
                price_cents: product.price_cents,
                quantity,
            });
        }

        // Save cart
        cache.set("cart:demo-session", &cart)
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Clear the cart.
#[leptos::server(prefix = "/api")]
pub async fn clear_cart() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;

        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        cache.delete("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}
