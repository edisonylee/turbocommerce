//! E-commerce domain types and logic for TurboCommerce.
//!
//! This crate provides production-ready types for building e-commerce applications:
//!
//! - **Catalog**: Products, variants, categories, inventory
//! - **Cart**: Shopping cart with line items, discounts, pricing
//! - **Checkout**: Multi-step checkout flow, orders
//! - **Search**: Faceted search, filters, pagination
//!
//! # Example
//!
//! ```rust,ignore
//! use turbo_commerce::prelude::*;
//!
//! // Create a product
//! let product = Product {
//!     id: ProductId::generate(),
//!     sku: "RUST-BOOK-001".to_string(),
//!     name: "Rust Programming Book".to_string(),
//!     slug: "rust-programming-book".to_string(),
//!     // ...
//! };
//!
//! // Create a cart and add items
//! let mut cart = Cart::new(session_id);
//! cart.add_item(
//!     variant.id.clone(),
//!     product.id.clone(),
//!     "Rust Programming Book".to_string(),
//!     1,
//!     Money::new(4999, Currency::USD),
//! );
//!
//! // Calculate totals
//! let pricing = cart.calculate_pricing();
//! println!("Total: {}", pricing.grand_total.display());
//! ```

pub mod error;
pub mod ids;
pub mod money;

pub mod catalog;
pub mod cart;
pub mod checkout;
pub mod search;

pub use error::CommerceError;
pub use ids::*;
pub use money::{Currency, Money};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::error::CommerceError;
    pub use crate::ids::*;
    pub use crate::money::{Currency, Money};

    // Catalog
    pub use crate::catalog::{
        Category, InventoryLevel, Product, ProductMedia, ProductStatus, ProductType,
        ProductVariant, VariantOption,
    };

    // Cart
    pub use crate::cart::{
        AppliedDiscount, Cart, CartPricing, Discount, DiscountCondition, DiscountType,
        DiscountValue, LineItem, LineItemPricing,
    };

    // Checkout
    pub use crate::checkout::{
        Address, CheckoutFlow, CheckoutStep, FinancialStatus, FulfillmentStatus, Order,
        OrderLineItem, OrderStatus, ShippingMethod, ShippingSelection,
    };

    // Search
    pub use crate::search::{Filter, Pagination, SearchQuery, SearchResults, SortOption};
}
