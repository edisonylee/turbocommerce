//! Product catalog module.
//!
//! Contains types for products, variants, categories, and inventory.

mod product;
mod category;
mod inventory;

pub use product::{Product, ProductMedia, ProductStatus, ProductType, ProductVariant, VariantOption, MediaType};
pub use category::Category;
pub use inventory::{InventoryLevel, InventoryAdjustment, AdjustmentReason};
