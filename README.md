# TurboCommerce

A WASM-native, pure Rust e-commerce framework built on [Leptos](https://leptos.dev) and [Spin](https://spin.fermyon.dev).

## What is this?

TurboCommerce is an experimental framework for building e-commerce storefronts that run entirely on WebAssembly. It targets edge compute platforms like [Fermyon Cloud](https://www.fermyon.com/cloud) with sub-millisecond cold starts.

## Status

This repo is **early-stage and experimental**. It’s a working prototype that proves:
- Rust-only SSR + hydration on WASM (Spin)
- Server functions with SQLite + KV storage
- A real storefront flow (products, cart, checkout wiring)

## What Works

- Leptos SSR + hydration on Spin/WASI
- Product list + product detail pages
- Cart add/update/remove/clear
- SQLite data layer + KV cart storage
- Static assets served by Spin fileserver

## What’s Missing (Roadmap)

- One-command dev server with watch mode
- File-based routing (auto discovery)
- More robust data migrations
- Production-grade observability and tooling

## Architecture

```
┌─────────────────────────────────────────────┐
│            turbo-storefront                  │
│         (Leptos SSR + hydration)            │
└─────────────────┬───────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌─────────┐ ┌──────────┐ ┌─────────┐
│ turbo-  │ │  turbo-  │ │ turbo-  │
│commerce │ │   auth   │ │   sdk   │
└────┬────┘ └────┬─────┘ └─────────┘
     │           │
     └─────┬─────┘
           │
    ┌──────┴──────┐
    │             │
    ▼             ▼
┌─────────┐ ┌──────────┐
│ turbo-  │ │  turbo-  │
│   db    │ │  cache   │
│(SQLite) │ │  (KV)    │
└─────────┘ └──────────┘
```

## Crates

| Crate | Description |
|-------|-------------|
| `turbo-sdk` | Main SDK, re-exports all crates |
| `turbo-core` | Leptos integration and SSR |
| `turbo-router` | File-based routing |
| `turbo-macros` | `#[page]` and `#[api]` macros |
| `turbo-db` | SQLite wrapper for Spin |
| `turbo-cache` | Key-Value store wrapper |
| `turbo-data` | HTTP client for data fetching |
| `turbo-commerce` | E-commerce domain types (Product, Cart, Order) |
| `turbo-auth` | Authentication and sessions |

## Quick Start

### Prerequisites

- Rust 1.78+
- [Spin CLI](https://developer.fermyon.com/spin/v2/install)
- wasm32 targets: `rustup target add wasm32-unknown-unknown wasm32-wasip1`

### Run the Example Storefront

```bash
# One-command dev (builds client+server and runs Spin)
./scripts/turbo dev
```

### Live Demo

Example deployment on Fermyon Cloud: `https://turbo-storefront-ge7xdyyk.fermyon.app/`

Or manually:

```bash
cd examples/turbo-storefront

# Build client WASM
cargo build --lib --target wasm32-unknown-unknown --release --features hydrate
wasm-bindgen --target web --out-dir target/site/pkg \
  target/wasm32-unknown-unknown/release/turbo_storefront.wasm

# Build server WASM
cargo build --lib --target wasm32-wasip1 --release --no-default-features --features ssr

# Run locally with sample data
spin up --sqlite @/path/to/init.sql

# Open http://localhost:3000
```

### Deploy to Fermyon Cloud

```bash
spin cloud deploy
```

## Example: Product Page

```rust
use turbo_sdk::prelude::*;

#[component]
fn ProductPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").unwrap_or_default();

    view! {
        <Suspense fallback=|| view! { <ProductSkeleton/> }>
            <ProductDetail id=id()/>
        </Suspense>
    }
}

#[server(prefix = "/api")]
pub async fn get_product(id: String) -> Result<Option<Product>, ServerFnError> {
    use turbo_db::{Db, params};

    let db = Db::open_default()?;
    let product = db.query_optional(
        "SELECT * FROM products WHERE id = ?",
        params![id.as_str()]
    )?;

    Ok(product)
}
```

## Project Structure

```
turbocommerce/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── turbo-sdk/          # Main SDK
│   ├── turbo-core/         # Leptos + SSR
│   ├── turbo-router/       # Routing
│   ├── turbo-macros/       # Proc macros
│   ├── turbo-db/           # SQLite
│   ├── turbo-cache/        # Key-Value
│   ├── turbo-data/         # HTTP client
│   ├── turbo-commerce/     # E-commerce types
│   └── turbo-auth/         # Authentication
└── examples/
    └── turbo-storefront/   # Example store
```

## Security

TurboCommerce uses production-grade security practices:

- **Password hashing**: Argon2id (OWASP recommended)
- **Token generation**: Cryptographically secure random (CSPRNG)
- **Session IDs**: 192-bit entropy with CSPRNG
- **Arithmetic safety**: Checked operations to prevent overflow attacks

## Breaking Changes (v0.2.0)

### Cart API

Cart methods now return `Result` to handle overflow and validation errors:

```rust
// Before
let line_id = cart.add_item(variant_id, product_id, "Product", 1, price);

// After
let line_id = cart.add_item(variant_id, product_id, "Product", 1, price)?;
```

Changed methods:
- `Cart::add_item()` -> `Result<LineItemId, CommerceError>`
- `Cart::update_quantity()` -> `Result<bool, CommerceError>`
- `Cart::merge()` -> `Result<(), CommerceError>`
- `Cart::calculate_pricing()` -> `Result<CartPricing, CommerceError>`
- `LineItem::new()` -> `Result<Self, CommerceError>`

### Money API

New safe arithmetic methods (use these instead of operators for fallible code):

```rust
// Safe (returns Option)
let total = price.try_multiply(quantity)?;
let sum = Money::try_sum(prices.iter(), Currency::USD)?;

// Panics on overflow (use only when overflow is impossible)
let total = price.multiply(quantity);
```

### Session API

`SessionData<T>` now includes a `version: u64` field for optimistic concurrency.

## Status

This is an experimental project. It demonstrates server-side rendering with Leptos on WASM, SQLite/KV storage via Spin, and type-safe e-commerce domain modeling in Rust.

## License

MIT
