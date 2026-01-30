//! Shopping cart module.
//!
//! Contains types for cart, line items, pricing, and discounts.

#[allow(clippy::module_inception)]
mod cart;
mod discount;
mod pricing;

pub use cart::{Cart, LineItem, LineItemProperty, MAX_QUANTITY_PER_ITEM};
pub use discount::{AppliedDiscount, Discount, DiscountCondition, DiscountType, DiscountValue};
pub use pricing::{CartPricing, LineItemPricing};
