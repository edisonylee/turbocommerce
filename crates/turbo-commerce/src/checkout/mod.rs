//! Checkout module.
//!
//! Contains types for checkout flow, addresses, shipping, and orders.

mod flow;
mod address;
mod shipping;
mod order;

pub use flow::{CheckoutFlow, CheckoutStep};
pub use address::Address;
pub use shipping::{ShippingMethod, ShippingSelection};
pub use order::{Order, OrderLineItem, OrderStatus, FinancialStatus, FulfillmentStatus};
