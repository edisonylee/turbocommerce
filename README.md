# TurboCommerce

A WASM-native, pure Rust e-commerce framework built on [Leptos](https://leptos.dev) and [Spin](https://spin.fermyon.dev).

## What is this?

TurboCommerce is an experimental framework for building e-commerce storefronts that run entirely on WebAssembly. It targets edge compute platforms like [Fermyon Cloud](https://www.fermyon.com/cloud) with sub-millisecond cold starts.

**This is not a Vercel competitor.** TurboCommerce is a framework (like Next.js), not a hosting platform. It runs on Fermyon Cloud, which is the hosting platform.

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

## Status

This is an experimental project. It demonstrates:

- Server-side rendering with Leptos on WASM
- SQLite and KV storage via Spin SDK
- Type-safe e-commerce domain modeling in Rust
- Sub-millisecond cold starts on edge compute

## License

MIT
