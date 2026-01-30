//! Product catalog module.
//!
//! Contains types for products, variants, categories, and inventory.

mod category;
mod inventory;
mod product;

pub use category::Category;
pub use inventory::{AdjustmentReason, InventoryAdjustment, InventoryLevel};
pub use product::{
    MediaType, Product, ProductMedia, ProductStatus, ProductType, ProductVariant, VariantOption,
};
