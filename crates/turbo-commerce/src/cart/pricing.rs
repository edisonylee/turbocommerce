//! Cart pricing calculations.

use crate::ids::LineItemId;
use crate::money::Money;
use serde::{Deserialize, Serialize};

/// Complete pricing breakdown for a cart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CartPricing {
    /// Subtotal before discounts.
    pub subtotal: Money,
    /// Total discount amount.
    pub discount_total: Money,
    /// Shipping cost.
    pub shipping_total: Money,
    /// Tax amount.
    pub tax_total: Money,
    /// Final total (subtotal - discounts + shipping + tax).
    pub grand_total: Money,
    /// Per-line-item pricing breakdown.
    pub line_items: Vec<LineItemPricing>,
}

impl CartPricing {
    /// Calculate the savings from discounts.
    pub fn savings(&self) -> Money {
        self.discount_total
    }

    /// Check if any discounts are applied.
    pub fn has_discounts(&self) -> bool {
        self.discount_total.amount_cents > 0
    }

    /// Get discount percentage of subtotal.
    pub fn discount_percentage(&self) -> f64 {
        if self.subtotal.amount_cents == 0 {
            return 0.0;
        }
        (self.discount_total.amount_cents as f64 / self.subtotal.amount_cents as f64) * 100.0
    }
}

/// Pricing breakdown for a single line item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineItemPricing {
    /// Line item ID.
    pub line_item_id: LineItemId,
    /// Unit price.
    pub unit_price: Money,
    /// Quantity.
    pub quantity: i64,
    /// Subtotal (unit_price * quantity).
    pub subtotal: Money,
    /// Discount applied to this item.
    pub discount_amount: Money,
    /// Tax on this item.
    pub tax_amount: Money,
    /// Final total for this item.
    pub total: Money,
}

impl LineItemPricing {
    /// Calculate effective unit price after discounts.
    pub fn effective_unit_price(&self) -> Money {
        if self.quantity == 0 {
            return self.unit_price;
        }
        Money::new(
            self.total.amount_cents / self.quantity,
            self.total.currency,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Currency;

    #[test]
    fn test_discount_percentage() {
        let pricing = CartPricing {
            subtotal: Money::new(10000, Currency::USD),
            discount_total: Money::new(1000, Currency::USD),
            shipping_total: Money::zero(Currency::USD),
            tax_total: Money::zero(Currency::USD),
            grand_total: Money::new(9000, Currency::USD),
            line_items: vec![],
        };

        assert!((pricing.discount_percentage() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_unit_price() {
        let pricing = LineItemPricing {
            line_item_id: LineItemId::new("item-1"),
            unit_price: Money::new(1000, Currency::USD),
            quantity: 2,
            subtotal: Money::new(2000, Currency::USD),
            discount_amount: Money::new(200, Currency::USD),
            tax_amount: Money::zero(Currency::USD),
            total: Money::new(1800, Currency::USD),
        };

        assert_eq!(pricing.effective_unit_price().amount_cents, 900);
    }
}
