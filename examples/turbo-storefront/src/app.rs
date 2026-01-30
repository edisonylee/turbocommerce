//! Application components and pages.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos::server_fn::error::ServerFnError;
use serde::{Deserialize, Serialize};
use turbo_commerce::catalog::{
    InventoryLevel, MediaType, Product as CatalogProduct, ProductMedia, ProductStatus,
    ProductType, ProductVariant,
};
use turbo_commerce::cart::Cart as CommerceCart;
use turbo_commerce::ids::{CategoryId, ProductId, VariantId};
use turbo_commerce::money::{Currency, Money};

// ============================================================================
// Data Types
// ============================================================================

/// Row shape for products in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ProductRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub stock: i64,
}

/// Storefront view model built from TurboCommerce domain types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorefrontProduct {
    pub product: CatalogProduct,
    pub variant: ProductVariant,
    pub media: Vec<ProductMedia>,
}

impl StorefrontProduct {
    pub fn price_display(&self) -> String {
        self.variant.price.display()
    }

    pub fn stock_available(&self) -> i64 {
        self.variant.inventory.available()
    }

    pub fn in_stock(&self) -> bool {
        self.variant.inventory.is_available()
    }

    pub fn image_url(&self) -> Option<&str> {
        self.media
            .iter()
            .find(|m| m.media_type == MediaType::Image)
            .map(|m| m.url.as_str())
    }
}

fn now_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn map_row_to_storefront(row: ProductRow) -> StorefrontProduct {
    let now = now_timestamp();
    let product_id = ProductId::new(row.id.clone());
    let variant_id = VariantId::new(format!("{}-default", row.id));
    let sku = format!("SKU-{}", row.id.to_uppercase());
    let slug = slugify(&row.name);

    let product = CatalogProduct {
        id: product_id.clone(),
        sku,
        name: row.name.clone(),
        slug,
        description: row.description.clone(),
        short_description: None,
        status: ProductStatus::Active,
        product_type: ProductType::Simple,
        category_ids: row
            .category
            .as_ref()
            .map(|c| vec![CategoryId::new(c.clone())])
            .unwrap_or_default(),
        tags: Vec::new(),
        default_variant_id: Some(variant_id.clone()),
        metadata: serde_json::Value::Object(serde_json::Map::new()),
        created_at: now,
        updated_at: now,
    };

    let mut media = Vec::new();
    let image_url = row
        .image_url
        .clone()
        .and_then(|url| {
            let trimmed = url.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .or_else(|| default_image_for_row(&row));
    if let Some(url) = image_url {
        let mut image = ProductMedia::new_image(product_id.clone(), url);
        image.alt_text = Some(row.name.clone());
        media.push(image);
    }

    let mut variant = ProductVariant::new(
        product_id.clone(),
        format!("{}-DEFAULT", product.sku),
        Money::new(row.price_cents, Currency::USD),
    );
    variant.id = variant_id;
    variant.inventory = InventoryLevel::new(row.stock);
    variant.images = media.iter().map(|m| m.id.clone()).collect();

    StorefrontProduct {
        product,
        variant,
        media,
    }
}

fn default_image_for_row(row: &ProductRow) -> Option<String> {
    match row.id.as_str() {
        "rust-book" => return Some("/images/rust_programming_book.png".to_string()),
        "wasm-kit" => return Some("/images/wasm-dev-kit.png".to_string()),
        "edge-guide" => return Some("/images/edge_computing.png".to_string()),
        "perf-pro" => return Some("/images/spin-framework.png".to_string()),
        "turbo-course" => return Some("/images/cargo_crate_stickers.png".to_string()),
        "leptos-book" => return Some("/images/ferris_plushie.png".to_string()),
        _ => {}
    }

    let name = row.name.to_lowercase();
    if name.contains("rust programming book") {
        return Some("/images/rust_programming_book.png".to_string());
    }
    if name.contains("wasm dev kit") || name.contains("wasm development kit") {
        return Some("/images/wasm-dev-kit.png".to_string());
    }
    if name.contains("edge computing guide") {
        return Some("/images/edge_computing.png".to_string());
    }
    if name.contains("performance tuning") || name.contains("spin framework") {
        return Some("/images/spin-framework.png".to_string());
    }
    if name.contains("turbocommerce mastery") || name.contains("cargo crate") {
        return Some("/images/cargo_crate_stickers.png".to_string());
    }
    if name.contains("leptos") || name.contains("ferris") {
        return Some("/images/ferris_plushie.png".to_string());
    }

    None
}

#[cfg(feature = "ssr")]
fn ensure_image_urls(db: &turbo_db::Db) -> Result<(), ServerFnError> {
    use turbo_db::params;

    let updates = [
        ("rust-book", "/images/rust_programming_book.png"),
        ("wasm-kit", "/images/wasm-dev-kit.png"),
        ("edge-guide", "/images/edge_computing.png"),
        ("perf-pro", "/images/spin-framework.png"),
        ("turbo-course", "/images/cargo_crate_stickers.png"),
        ("leptos-book", "/images/ferris_plushie.png"),
    ];

    for (id, url) in updates {
        db.execute(
            "UPDATE products SET image_url = ? WHERE id = ? AND (image_url IS NULL OR image_url = '')",
            params![url, id],
        )
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    }

    Ok(())
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
    let cart_version = RwSignal::new(0u64);
    provide_context(cart_version);

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
            <a href="/" class="logo">"TURBO"</a>
            <nav>
                <a href="/">"Shop"</a>
                <a href="/products">"Collection"</a>
                <a href="/cart">"Cart"</a>
            </nav>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer>
            <div class="footer-content">
                <div class="footer-brand">
                    <span class="logo">"TURBO"</span>
                    <p>"Pure Rust. WASM-native. Edge-first."</p>
                </div>
                <div class="footer-links">
                    <a href="/products">"Shop"</a>
                    <a href="/cart">"Cart"</a>
                </div>
            </div>
            <div class="footer-bottom">
                <p>"Built with TurboCommerce"</p>
            </div>
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
        <section class="hero">
            <div class="hero-content">
                <span class="hero-eyebrow">"For Rustaceans"</span>
                <h1 class="hero-title">"Tools, Books & Gear for Rust Developers"</h1>
                <p class="hero-subtitle">"Everything you need to build fast, safe, and concurrent software. From essential reading to the beloved Ferris plushie."</p>
                <a href="/products" class="btn btn-primary">"Browse the Shop"</a>
            </div>
        </section>

        <section class="featured-section">
            <h2 class="section-title">"Featured"</h2>
            <leptos::suspense::Suspense fallback=move || view! { <ProductGridSkeleton/> }>
                <ProductGrid/>
            </leptos::suspense::Suspense>
        </section>
    }
}

/// Products listing page
#[component]
fn ProductsPage() -> impl IntoView {
    view! {
        <section class="collection-section">
            <h1 class="page-title">"All Products"</h1>
            <p class="page-subtitle">"Books, tools, and merch for the Rust community."</p>
            <leptos::suspense::Suspense fallback=move || view! { <ProductGridSkeleton/> }>
                <ProductGrid/>
            </leptos::suspense::Suspense>
        </section>
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
    let cart_version = use_context::<RwSignal<u64>>();
    let cart_version_update = cart_version.clone();
    let cart_version_remove = cart_version.clone();
    let cart_version_clear = cart_version.clone();
    let update_action = ServerAction::<UpdateCartItem>::new();
    let remove_action = ServerAction::<RemoveCartItem>::new();
    let clear_action = ServerAction::<ClearCart>::new();
    let update_value = update_action.value();
    let remove_value = remove_action.value();
    let clear_value = clear_action.value();

    create_effect(move |_| {
        if let (Some(sig), Some(Ok(_))) = (cart_version_update, update_value.get()) {
            sig.update(|v| *v = v.wrapping_add(1));
        }
    });
    create_effect(move |_| {
        if let (Some(sig), Some(Ok(_))) = (cart_version_remove, remove_value.get()) {
            sig.update(|v| *v = v.wrapping_add(1));
        }
    });
    create_effect(move |_| {
        if let (Some(sig), Some(Ok(_))) = (cart_version_clear, clear_value.get()) {
            sig.update(|v| *v = v.wrapping_add(1));
        }
    });
    let cart = Resource::new(
        move || (
            update_action.version().get(),
            remove_action.version().get(),
            clear_action.version().get(),
        ),
        |_| get_cart(),
    );

    view! {
        <section class="cart-section">
            <h1 class="page-title">"Your Cart"</h1>
            <leptos::suspense::Suspense fallback=move || view! { <CartSkeleton/> }>
                {move || cart.get().map(|result| match result {
                    Ok(cart) if cart.is_empty() => view! {
                        <div class="cart-empty">
                            <p>"Your cart is empty."</p>
                            <a href="/products" class="btn btn-secondary">"Continue Shopping"</a>
                        </div>
                    }.into_any(),
                    Ok(cart) => view! {
                        <CartView
                            cart=cart
                            update_action=update_action.clone()
                            remove_action=remove_action.clone()
                            clear_action=clear_action.clone()
                        />
                    }.into_any(),
                    Err(e) => view! {
                        <div class="error-message">
                            <p>"Error loading cart: " {e.to_string()}</p>
                        </div>
                    }.into_any(),
                })}
            </leptos::suspense::Suspense>
        </section>
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
        <div class="not-found">
            <h1 class="not-found-title">"404"</h1>
            <p class="not-found-message">"This page does not exist."</p>
            <a href="/" class="btn btn-secondary">"Return Home"</a>
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
fn ProductCard(product: StorefrontProduct) -> impl IntoView {
    let href = format!("/product/{}", product.product.id.as_str());
    let price = product.price_display();
    let stock_class = if product.in_stock() { "in-stock" } else { "out-of-stock" };
    let image_url = product.image_url().map(|s| s.to_string());
    let product_name = product.product.name.clone();

    view! {
        <a href=href class="product-card">
            <div class="product-image">
                {match image_url {
                    Some(url) => view! { <img src=url alt=product_name.clone()/> }.into_any(),
                    None => view! { <span class="product-index">{product.product.id.as_str().chars().last().unwrap_or('0').to_string()}</span> }.into_any(),
                }}
            </div>
            <div class="product-info">
                <h3 class="product-name">{product_name}</h3>
                <div class="product-meta">
                    <span class="product-price">{price}</span>
                    <span class=format!("stock-indicator {}", stock_class)>
                        {if product.in_stock() { "Available" } else { "Sold Out" }}
                    </span>
                </div>
            </div>
        </a>
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
                let description = p.product.description.clone().unwrap_or_else(|| "A quality item for Rust developers.".to_string());
                let product_id = p.product.id.as_str().to_string();
                let variant_id = p.variant.id.as_str().to_string();
                let product_name = p.product.name.clone();
                let image_url = p.image_url().map(|s| s.to_string());
                let stock_available = p.stock_available();
                let stock_status = if p.in_stock() {
                    format!("{} available", stock_available)
                } else {
                    "Currently unavailable".to_string()
                };
                view! {
                    <article class="product-detail">
                        <div class="product-detail-image">
                            {match image_url {
                                Some(url) => view! { <img src=url alt=product_name/> }.into_any(),
                                None => view! {
                                    <div class="product-detail-placeholder">
                                        <span class="product-detail-index">{p.product.id.as_str().chars().last().unwrap_or('0').to_string()}</span>
                                    </div>
                                }.into_any(),
                            }}
                        </div>
                        <div class="product-detail-content">
                            <header class="product-detail-header">
                                <h1 class="product-detail-title">{p.product.name.clone()}</h1>
                                <p class="product-detail-price">{price}</p>
                            </header>
                            <div class="product-detail-description">
                                <p class="drop-cap">{description}</p>
                            </div>
                            <div class="product-detail-meta">
                                <span class="stock-status">{stock_status}</span>
                            </div>
                            <div class="product-detail-actions">
                                <AddToCartButton product_id=product_id variant_id=variant_id/>
                            </div>
                        </div>
                    </article>
                }.into_any()
            },
            Ok(None) => view! {
                <div class="not-found">
                    <h1>"Not Found"</h1>
                    <p>"This piece is no longer in our collection."</p>
                    <a href="/products" class="btn btn-secondary">"Return to Collection"</a>
                </div>
            }.into_any(),
            Err(e) => view! {
                <div class="error-message">
                    <p>"Unable to load product: " {e.to_string()}</p>
                </div>
            }.into_any(),
        })}
    }
}

#[component]
fn AddToCartButton(product_id: String, variant_id: String) -> impl IntoView {
    let add_action = ServerAction::<AddToCart>::new();
    let pending = add_action.pending();
    let value = add_action.value();
    let cart_version = use_context::<RwSignal<u64>>();

    create_effect(move |_| {
        if let (Some(sig), Some(Ok(_))) = (cart_version, value.get()) {
            sig.update(|v| *v = v.wrapping_add(1));
        }
    });

    view! {
        <div class="add-to-cart-form">
            <ActionForm action=add_action>
                <input type="hidden" name="product_id" value=product_id/>
                <input type="hidden" name="variant_id" value=variant_id/>
                <input type="hidden" name="quantity" value="1"/>
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || pending.get()
                >
                    {move || if pending.get() { "Adding..." } else { "Add to Cart" }}
                </button>
            </ActionForm>
        </div>
        {move || value.get().map(|result| match result {
            Ok(_) => view! {
                <p class="message message-success">"Added to cart"</p>
            }.into_any(),
            Err(e) => view! {
                <p class="message message-error">"Error: " {e.to_string()}</p>
            }.into_any(),
        })}
    }
}

// ============================================================================
// Cart Components
// ============================================================================

#[component]
fn CartView(
    cart: CommerceCart,
    update_action: ServerAction<UpdateCartItem>,
    remove_action: ServerAction<RemoveCartItem>,
    clear_action: ServerAction<ClearCart>,
) -> impl IntoView {
    let pricing = cart.calculate_pricing().ok();
    let total = pricing
        .as_ref()
        .map(|p| p.grand_total.display())
        .unwrap_or_else(|| "$0.00".to_string());
    let item_count = cart.item_count();

    view! {
        <div class="cart-container">
            <p class="cart-count">{item_count.to_string()} " item(s)"</p>
            <div class="cart-items">
                {cart.items.into_iter().map(|item| {
                    let subtotal = item.total_price.display();
                    let price = item.unit_price.display();
                    let variant_id = item.variant_id.as_str().to_string();
                    let variant_id_for_dec = variant_id.clone();
                    let variant_id_for_inc = variant_id.clone();
                    let dec_quantity = (item.quantity - 1).to_string();
                    let inc_quantity = (item.quantity + 1).to_string();
                    let update_action = update_action.clone();
                    let remove_action = remove_action.clone();
                    view! {
                        <div class="cart-item">
                            <div class="cart-item-info">
                                <span class="cart-item-name">{item.product_name}</span>
                                <span class="cart-item-price">{price} " × " {item.quantity.to_string()}</span>
                            </div>
                            <div class="cart-item-subtotal">
                                <span>{subtotal}</span>
                            </div>
                            <div class="cart-item-actions" style="display:flex;align-items:center;gap:0.5rem;flex-wrap:nowrap;">
                                <ActionForm action=update_action.clone() style="display:inline-flex;">
                                    <input type="hidden" name="variant_id" value=variant_id_for_dec/>
                                    <input type="hidden" name="quantity" value=dec_quantity/>
                                    <button type="submit" class="btn btn-secondary">"-"</button>
                                </ActionForm>
                                <span class="cart-item-qty">{item.quantity.to_string()}</span>
                                <ActionForm action=update_action.clone() style="display:inline-flex;">
                                    <input type="hidden" name="variant_id" value=variant_id_for_inc/>
                                    <input type="hidden" name="quantity" value=inc_quantity/>
                                    <button type="submit" class="btn btn-secondary">"+"</button>
                                </ActionForm>
                                <ActionForm action=remove_action style="display:inline-flex;">
                                    <input type="hidden" name="variant_id" value=variant_id/>
                                    <button type="submit" class="btn btn-danger">"Remove"</button>
                                </ActionForm>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            {pricing.map(|p| view! {
                <div class="cart-pricing">
                    <div class="cart-pricing-row">
                        <span>"Subtotal"</span>
                        <span>{p.subtotal.display()}</span>
                    </div>
                    <div class="cart-pricing-row">
                        <span>"Discounts"</span>
                        <span>{p.discount_total.display()}</span>
                    </div>
                    <div class="cart-pricing-row">
                        <span>"Shipping"</span>
                        <span>{p.shipping_total.display()}</span>
                    </div>
                    <div class="cart-pricing-row">
                        <span>"Tax"</span>
                        <span>{p.tax_total.display()}</span>
                    </div>
                </div>
            })}
            <div class="cart-total">
                <span class="cart-total-label">"Total"</span>
                <span class="cart-total-amount">{total}</span>
            </div>
            <div class="cart-actions">
                <a href="/products" class="btn btn-secondary">"Continue Shopping"</a>
                <ClearCartButton clear_action=clear_action/>
            </div>
        </div>
    }
}

#[component]
fn ClearCartButton(clear_action: ServerAction<ClearCart>) -> impl IntoView {
    let pending = clear_action.pending();

    view! {
        <ActionForm action=clear_action>
            <button
                type="submit"
                class="btn btn-danger"
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
        <div class="product-card skeleton-card">
            <div class="skeleton skeleton-image"></div>
            <div class="product-info">
                <div class="skeleton skeleton-title"></div>
                <div class="skeleton skeleton-price"></div>
            </div>
        </div>
    }
}

#[component]
fn ProductDetailSkeleton() -> impl IntoView {
    view! {
        <article class="product-detail">
            <div class="product-detail-image">
                <div class="skeleton skeleton-detail-image"></div>
            </div>
            <div class="product-detail-content">
                <div class="skeleton skeleton-detail-title"></div>
                <div class="skeleton skeleton-detail-price"></div>
                <div class="skeleton skeleton-detail-desc"></div>
                <div class="skeleton skeleton-detail-btn"></div>
            </div>
        </article>
    }
}

#[component]
fn CartSkeleton() -> impl IntoView {
    view! {
        <div class="cart-container">
            <div class="skeleton skeleton-cart-count"></div>
            <div class="skeleton skeleton-cart-item"></div>
            <div class="skeleton skeleton-cart-item"></div>
            <div class="skeleton skeleton-cart-total"></div>
        </div>
    }
}

// ============================================================================
// Server Functions (API)
// ============================================================================

/// Get all products from the database.
#[leptos::server(prefix = "/api")]
pub async fn get_products() -> Result<Vec<StorefrontProduct>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_db::{Db, params};

        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
        ensure_image_urls(&db)?;

        let rows: Vec<ProductRow> = db.query_as(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products ORDER BY name",
            params![]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        Ok(rows.into_iter().map(map_row_to_storefront).collect())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Get a single product by ID.
#[leptos::server(prefix = "/api")]
pub async fn get_product(id: String) -> Result<Option<StorefrontProduct>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_db::{Db, params};

        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
        ensure_image_urls(&db)?;

        let product: Option<ProductRow> = db.query_optional(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products WHERE id = ?",
            params![id.as_str()]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        Ok(product.map(map_row_to_storefront))
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Get the shopping cart from KV store.
#[leptos::server(prefix = "/api")]
pub async fn get_cart() -> Result<CommerceCart, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;

        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        // For demo purposes, use a fixed session ID
        // In production, this would come from a session cookie
        let cart: CommerceCart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_else(|| CommerceCart::new("demo-session"));

        Ok(cart)
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Add an item to the cart.
#[leptos::server(prefix = "/api")]
pub async fn add_to_cart(product_id: String, variant_id: String, quantity: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;
        use turbo_db::{Db, params};

        // Get product info from database
        let db = Db::open_default()
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        let product: Option<ProductRow> = db.query_optional(
            "SELECT id, name, description, price_cents, image_url, category, stock FROM products WHERE id = ?",
            params![product_id.as_str()]
        ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

        let product = product.ok_or_else(|| ServerFnError::new("Product not found"))?;

        // Get current cart
        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        let mut cart: CommerceCart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_else(|| CommerceCart::new("demo-session"));

        if quantity <= 0 {
            return Err(ServerFnError::new("Quantity must be at least 1"));
        }

        let variant_id = VariantId::new(variant_id);
        let current_qty = cart
            .get_item_by_variant(&variant_id)
            .map(|item| item.quantity)
            .unwrap_or(0);
        let next_qty = current_qty + quantity;

        if product.stock <= 0 {
            return Err(ServerFnError::new("Item is out of stock"));
        }
        if next_qty > product.stock {
            return Err(ServerFnError::new("Requested quantity exceeds available stock"));
        }

        let unit_price = Money::new(product.price_cents, Currency::USD);
        let product_id = ProductId::new(product.id);
        cart.add_item(
            variant_id,
            product_id,
            product.name,
            quantity,
            unit_price,
        ).map_err(|e| ServerFnError::new(format!("Cart error: {}", e)))?;

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

/// Update cart item quantity by variant.
#[leptos::server(prefix = "/api")]
pub async fn update_cart_item(variant_id: String, quantity: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;
        use turbo_db::{Db, params};

        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        let mut cart: CommerceCart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_else(|| CommerceCart::new("demo-session"));

        let variant_id = VariantId::new(variant_id);
        let line_item = cart
            .get_item_by_variant(&variant_id)
            .ok_or_else(|| ServerFnError::new("Item not in cart"))?
            .clone();

        if quantity < 0 {
            return Err(ServerFnError::new("Quantity cannot be negative"));
        }

        if quantity > 0 {
            let db = Db::open_default()
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
            let product: Option<ProductRow> = db.query_optional(
                "SELECT id, name, description, price_cents, image_url, category, stock FROM products WHERE id = ?",
                params![line_item.product_id.as_str()]
            ).map_err(|e| ServerFnError::new(format!("Query error: {}", e)))?;

            let product = product.ok_or_else(|| ServerFnError::new("Product not found"))?;

            if product.stock <= 0 {
                return Err(ServerFnError::new("Item is out of stock"));
            }
            if quantity > product.stock {
                return Err(ServerFnError::new("Requested quantity exceeds available stock"));
            }
        }

        cart.update_quantity(&line_item.id, quantity)
            .map_err(|e| ServerFnError::new(format!("Cart error: {}", e)))?;

        cache.set("cart:demo-session", &cart)
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server-only function"))
    }
}

/// Remove cart item by variant.
#[leptos::server(prefix = "/api")]
pub async fn remove_cart_item(variant_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use turbo_cache::Cache;

        let cache = Cache::open_default()
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?;

        let mut cart: CommerceCart = cache.get("cart:demo-session")
            .map_err(|e| ServerFnError::new(format!("Cache error: {}", e)))?
            .unwrap_or_else(|| CommerceCart::new("demo-session"));

        let variant_id = VariantId::new(variant_id);
        let line_item = cart
            .get_item_by_variant(&variant_id)
            .ok_or_else(|| ServerFnError::new("Item not in cart"))?
            .clone();

        cart.remove_item(&line_item.id);

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
