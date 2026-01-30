//! Checkout module.
//!
//! Contains types for checkout flow, addresses, shipping, and orders.

mod address;
mod flow;
mod order;
mod shipping;

pub use address::Address;
pub use flow::{CheckoutFlow, CheckoutStep};
pub use order::{FinancialStatus, FulfillmentStatus, Order, OrderLineItem, OrderStatus};
pub use shipping::{ShippingMethod, ShippingSelection};
