//! Order types.

use crate::cart::LineItemProperty;
use crate::checkout::{Address, ShippingSelection};
use crate::ids::{OrderId, OrderLineItemId, ProductId, UserId, VariantId};
use crate::money::{Currency, Money};
use serde::{Deserialize, Serialize};

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OrderStatus {
    /// Order placed, awaiting processing.
    #[default]
    Pending,
    /// Order confirmed and processing.
    Confirmed,
    /// Order being prepared.
    Processing,
    /// Order shipped.
    Shipped,
    /// Order delivered.
    Delivered,
    /// Order cancelled.
    Cancelled,
    /// Order refunded.
    Refunded,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Confirmed => "confirmed",
            OrderStatus::Processing => "processing",
            OrderStatus::Shipped => "shipped",
            OrderStatus::Delivered => "delivered",
            OrderStatus::Cancelled => "cancelled",
            OrderStatus::Refunded => "refunded",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "Pending",
            OrderStatus::Confirmed => "Confirmed",
            OrderStatus::Processing => "Processing",
            OrderStatus::Shipped => "Shipped",
            OrderStatus::Delivered => "Delivered",
            OrderStatus::Cancelled => "Cancelled",
            OrderStatus::Refunded => "Refunded",
        }
    }

    /// Check if order is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Delivered | OrderStatus::Cancelled | OrderStatus::Refunded
        )
    }

    /// Check if order can be cancelled.
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending | OrderStatus::Confirmed | OrderStatus::Processing
        )
    }
}

/// Financial/payment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FinancialStatus {
    /// Payment pending.
    #[default]
    Pending,
    /// Payment authorized but not captured.
    Authorized,
    /// Payment captured/completed.
    Paid,
    /// Partially refunded.
    PartiallyRefunded,
    /// Fully refunded.
    Refunded,
    /// Payment voided.
    Voided,
}

impl FinancialStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FinancialStatus::Pending => "pending",
            FinancialStatus::Authorized => "authorized",
            FinancialStatus::Paid => "paid",
            FinancialStatus::PartiallyRefunded => "partially_refunded",
            FinancialStatus::Refunded => "refunded",
            FinancialStatus::Voided => "voided",
        }
    }
}

/// Fulfillment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FulfillmentStatus {
    /// Nothing fulfilled yet.
    #[default]
    Unfulfilled,
    /// Some items fulfilled.
    PartiallyFulfilled,
    /// All items fulfilled.
    Fulfilled,
}

impl FulfillmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FulfillmentStatus::Unfulfilled => "unfulfilled",
            FulfillmentStatus::PartiallyFulfilled => "partially_fulfilled",
            FulfillmentStatus::Fulfilled => "fulfilled",
        }
    }
}

/// A completed order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    /// Unique order identifier.
    pub id: OrderId,
    /// Human-readable order number.
    pub order_number: String,
    /// Customer user ID (None for guest).
    pub user_id: Option<UserId>,
    /// Customer email.
    pub email: String,
    /// Order status.
    pub status: OrderStatus,
    /// Payment status.
    pub financial_status: FinancialStatus,
    /// Fulfillment status.
    pub fulfillment_status: FulfillmentStatus,
    /// Items in the order.
    pub line_items: Vec<OrderLineItem>,
    /// Shipping address.
    pub shipping_address: Address,
    /// Billing address.
    pub billing_address: Address,
    /// Shipping method used.
    pub shipping_method: ShippingSelection,
    /// Subtotal before discounts.
    pub subtotal: Money,
    /// Total discount amount.
    pub discount_total: Money,
    /// Shipping cost.
    pub shipping_total: Money,
    /// Tax amount.
    pub tax_total: Money,
    /// Grand total charged.
    pub grand_total: Money,
    /// Order currency.
    pub currency: Currency,
    /// Customer note.
    pub note: Option<String>,
    /// Order tags.
    pub tags: Vec<String>,
    /// Additional metadata.
    pub metadata: serde_json::Value,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
    /// Unix timestamp when cancelled (if applicable).
    pub cancelled_at: Option<i64>,
}

impl Order {
    /// Generate a new order number.
    pub fn generate_order_number() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("ORD-{}", ts)
    }

    /// Get total item count.
    pub fn item_count(&self) -> i64 {
        self.line_items.iter().map(|i| i.quantity).sum()
    }

    /// Check if order is paid.
    pub fn is_paid(&self) -> bool {
        matches!(
            self.financial_status,
            FinancialStatus::Paid | FinancialStatus::PartiallyRefunded
        )
    }

    /// Check if order is fully fulfilled.
    pub fn is_fulfilled(&self) -> bool {
        self.fulfillment_status == FulfillmentStatus::Fulfilled
    }

    /// Cancel the order.
    pub fn cancel(&mut self) -> bool {
        if !self.status.can_cancel() {
            return false;
        }
        self.status = OrderStatus::Cancelled;
        self.cancelled_at = Some(current_timestamp());
        self.updated_at = current_timestamp();
        true
    }

    /// Update order status.
    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = current_timestamp();
    }

    /// Update financial status.
    pub fn set_financial_status(&mut self, status: FinancialStatus) {
        self.financial_status = status;
        self.updated_at = current_timestamp();
    }

    /// Update fulfillment status.
    pub fn set_fulfillment_status(&mut self, status: FulfillmentStatus) {
        self.fulfillment_status = status;
        self.updated_at = current_timestamp();
    }
}

/// A line item in an order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderLineItem {
    /// Unique line item identifier.
    pub id: OrderLineItemId,
    /// Variant ID.
    pub variant_id: VariantId,
    /// Product ID.
    pub product_id: ProductId,
    /// SKU at time of order.
    pub sku: String,
    /// Product name at time of order.
    pub name: String,
    /// Variant title (e.g., "Large / Blue").
    pub variant_title: Option<String>,
    /// Quantity ordered.
    pub quantity: i64,
    /// Unit price at time of order.
    pub unit_price: Money,
    /// Total price for this line.
    pub total_price: Money,
    /// Discount applied to this line.
    pub discount_amount: Money,
    /// Tax for this line.
    pub tax_amount: Money,
    /// Fulfillment status for this item.
    pub fulfillment_status: FulfillmentStatus,
    /// Quantity fulfilled.
    pub fulfilled_quantity: i64,
    /// Custom properties.
    pub properties: Vec<LineItemProperty>,
}

impl OrderLineItem {
    /// Check if fully fulfilled.
    pub fn is_fulfilled(&self) -> bool {
        self.fulfilled_quantity >= self.quantity
    }

    /// Get unfulfilled quantity.
    pub fn unfulfilled_quantity(&self) -> i64 {
        (self.quantity - self.fulfilled_quantity).max(0)
    }
}

/// Get current Unix timestamp.
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_can_cancel() {
        assert!(OrderStatus::Pending.can_cancel());
        assert!(OrderStatus::Confirmed.can_cancel());
        assert!(!OrderStatus::Shipped.can_cancel());
        assert!(!OrderStatus::Delivered.can_cancel());
    }

    #[test]
    fn test_order_number_generation() {
        let num1 = Order::generate_order_number();
        let _num2 = Order::generate_order_number();
        assert!(num1.starts_with("ORD-"));
        // Note: num2 generated to verify function can be called multiple times
    }
}
