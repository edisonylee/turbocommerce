//! Discount and coupon types.

use crate::ids::{CategoryId, DiscountId, ProductId};
use crate::money::Money;
use serde::{Deserialize, Serialize};

/// Type of discount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscountType {
    /// Percentage off.
    Percentage,
    /// Fixed amount off.
    FixedAmount,
    /// Free shipping.
    FreeShipping,
    /// Buy X get Y free/discounted.
    BuyXGetY,
}

/// Value of the discount.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscountValue {
    /// Percentage off (0.0 - 100.0).
    Percentage(f64),
    /// Fixed amount off.
    Fixed(Money),
    /// Free shipping.
    FreeShipping,
    /// Buy X get Y at a discount.
    BuyXGetY {
        buy: i64,
        get: i64,
        discount_percent: f64,
    },
}

impl DiscountValue {
    /// Calculate the discount amount for a given subtotal.
    pub fn calculate(&self, subtotal: &Money) -> Money {
        match self {
            DiscountValue::Percentage(percent) => subtotal.percentage(*percent),
            DiscountValue::Fixed(amount) => {
                // Don't exceed subtotal
                if amount.amount_cents > subtotal.amount_cents {
                    *subtotal
                } else {
                    *amount
                }
            }
            DiscountValue::FreeShipping => Money::zero(subtotal.currency),
            DiscountValue::BuyXGetY { .. } => {
                // Complex calculation handled elsewhere
                Money::zero(subtotal.currency)
            }
        }
    }
}

/// Condition for a discount to apply.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscountCondition {
    /// Minimum purchase amount.
    MinimumPurchase(Money),
    /// Specific product must be in cart.
    ProductInCart(ProductId),
    /// Product from category must be in cart.
    CategoryInCart(CategoryId),
    /// Customer has specific tag.
    CustomerTag(String),
    /// First order only.
    FirstOrder,
    /// Minimum quantity of items.
    MinimumQuantity(i64),
    /// Specific products only.
    SpecificProducts(Vec<ProductId>),
    /// Specific categories only.
    SpecificCategories(Vec<CategoryId>),
}

/// A discount/coupon definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Discount {
    /// Unique discount identifier.
    pub id: DiscountId,
    /// Discount code (e.g., "SAVE10").
    pub code: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Type of discount.
    pub discount_type: DiscountType,
    /// Value of the discount.
    pub value: DiscountValue,
    /// Conditions that must be met.
    pub conditions: Vec<DiscountCondition>,
    /// Maximum number of uses (None = unlimited).
    pub usage_limit: Option<i64>,
    /// Current usage count.
    pub usage_count: i64,
    /// Per-customer usage limit.
    pub per_customer_limit: Option<i64>,
    /// Start date (Unix timestamp).
    pub starts_at: Option<i64>,
    /// End date (Unix timestamp).
    pub ends_at: Option<i64>,
    /// Whether discount is active.
    pub active: bool,
    /// Combine with other discounts?
    pub combinable: bool,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

impl Discount {
    /// Create a new percentage discount.
    pub fn percentage(code: impl Into<String>, name: impl Into<String>, percent: f64) -> Self {
        let now = current_timestamp();
        Self {
            id: DiscountId::generate(),
            code: code.into(),
            name: name.into(),
            description: None,
            discount_type: DiscountType::Percentage,
            value: DiscountValue::Percentage(percent),
            conditions: Vec::new(),
            usage_limit: None,
            usage_count: 0,
            per_customer_limit: None,
            starts_at: None,
            ends_at: None,
            active: true,
            combinable: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new fixed amount discount.
    pub fn fixed_amount(code: impl Into<String>, name: impl Into<String>, amount: Money) -> Self {
        let now = current_timestamp();
        Self {
            id: DiscountId::generate(),
            code: code.into(),
            name: name.into(),
            description: None,
            discount_type: DiscountType::FixedAmount,
            value: DiscountValue::Fixed(amount),
            conditions: Vec::new(),
            usage_limit: None,
            usage_count: 0,
            per_customer_limit: None,
            starts_at: None,
            ends_at: None,
            active: true,
            combinable: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a free shipping discount.
    pub fn free_shipping(code: impl Into<String>, name: impl Into<String>) -> Self {
        let now = current_timestamp();
        Self {
            id: DiscountId::generate(),
            code: code.into(),
            name: name.into(),
            description: None,
            discount_type: DiscountType::FreeShipping,
            value: DiscountValue::FreeShipping,
            conditions: Vec::new(),
            usage_limit: None,
            usage_count: 0,
            per_customer_limit: None,
            starts_at: None,
            ends_at: None,
            active: true,
            combinable: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the discount is currently valid (time-based).
    pub fn is_valid(&self) -> bool {
        if !self.active {
            return false;
        }

        let now = current_timestamp();

        if let Some(starts) = self.starts_at {
            if now < starts {
                return false;
            }
        }

        if let Some(ends) = self.ends_at {
            if now > ends {
                return false;
            }
        }

        if let Some(limit) = self.usage_limit {
            if self.usage_count >= limit {
                return false;
            }
        }

        true
    }

    /// Check if discount has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.usage_limit
            .map(|limit| self.usage_count >= limit)
            .unwrap_or(false)
    }

    /// Check if discount has expired.
    pub fn is_expired(&self) -> bool {
        self.ends_at
            .map(|ends| current_timestamp() > ends)
            .unwrap_or(false)
    }

    /// Add a minimum purchase condition.
    pub fn with_minimum_purchase(mut self, amount: Money) -> Self {
        self.conditions
            .push(DiscountCondition::MinimumPurchase(amount));
        self
    }

    /// Add a usage limit.
    pub fn with_usage_limit(mut self, limit: i64) -> Self {
        self.usage_limit = Some(limit);
        self
    }

    /// Set expiration date.
    pub fn expires_at(mut self, timestamp: i64) -> Self {
        self.ends_at = Some(timestamp);
        self
    }

    /// Increment usage count.
    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.updated_at = current_timestamp();
    }
}

/// A discount that has been applied to a cart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppliedDiscount {
    /// The discount ID.
    pub discount_id: DiscountId,
    /// The discount code used.
    pub code: String,
    /// Description for display.
    pub description: String,
    /// Amount discounted.
    pub amount: Money,
}

impl AppliedDiscount {
    /// Create from a discount and calculated amount.
    pub fn from_discount(discount: &Discount, amount: Money) -> Self {
        Self {
            discount_id: discount.id.clone(),
            code: discount.code.clone(),
            description: discount.name.clone(),
            amount,
        }
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
    use crate::money::Currency;

    #[test]
    fn test_percentage_discount() {
        let discount = Discount::percentage("SAVE10", "10% Off", 10.0);
        let subtotal = Money::new(10000, Currency::USD);
        let amount = discount.value.calculate(&subtotal);
        assert_eq!(amount.amount_cents, 1000);
    }

    #[test]
    fn test_fixed_discount() {
        let discount = Discount::fixed_amount("SAVE5", "$5 Off", Money::new(500, Currency::USD));
        let subtotal = Money::new(10000, Currency::USD);
        let amount = discount.value.calculate(&subtotal);
        assert_eq!(amount.amount_cents, 500);
    }

    #[test]
    fn test_fixed_discount_capped() {
        let discount =
            Discount::fixed_amount("SAVE100", "$100 Off", Money::new(10000, Currency::USD));
        let subtotal = Money::new(5000, Currency::USD);
        let amount = discount.value.calculate(&subtotal);
        // Capped at subtotal
        assert_eq!(amount.amount_cents, 5000);
    }

    #[test]
    fn test_discount_validity() {
        let mut discount = Discount::percentage("TEST", "Test", 10.0);
        assert!(discount.is_valid());

        discount.active = false;
        assert!(!discount.is_valid());
    }

    #[test]
    fn test_discount_usage_limit() {
        let mut discount = Discount::percentage("TEST", "Test", 10.0).with_usage_limit(5);

        discount.usage_count = 4;
        assert!(discount.is_valid());

        discount.usage_count = 5;
        assert!(!discount.is_valid());
    }
}
