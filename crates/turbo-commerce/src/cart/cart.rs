//! Cart and line item types.

use crate::ids::{CartId, LineItemId, ProductId, UserId, VariantId};
use crate::money::{Currency, Money};
use crate::cart::{AppliedDiscount, CartPricing, LineItemPricing};
use serde::{Deserialize, Serialize};

/// A shopping cart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cart {
    /// Unique cart identifier.
    pub id: CartId,
    /// Session ID for anonymous carts.
    pub session_id: String,
    /// User ID for authenticated carts.
    pub user_id: Option<UserId>,
    /// Items in the cart.
    pub items: Vec<LineItem>,
    /// Applied discounts.
    pub discounts: Vec<AppliedDiscount>,
    /// Cart currency.
    pub currency: Currency,
    /// Customer note.
    pub note: Option<String>,
    /// Additional metadata.
    pub metadata: serde_json::Value,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
    /// Unix timestamp when cart expires.
    pub expires_at: Option<i64>,
}

impl Cart {
    /// Create a new cart for a session.
    pub fn new(session_id: impl Into<String>) -> Self {
        let now = current_timestamp();
        Self {
            id: CartId::generate(),
            session_id: session_id.into(),
            user_id: None,
            items: Vec::new(),
            discounts: Vec::new(),
            currency: Currency::USD,
            note: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    /// Create a cart for an authenticated user.
    pub fn for_user(user_id: UserId, session_id: impl Into<String>) -> Self {
        let mut cart = Self::new(session_id);
        cart.user_id = Some(user_id);
        cart
    }

    /// Add an item to the cart.
    pub fn add_item(
        &mut self,
        variant_id: VariantId,
        product_id: ProductId,
        product_name: impl Into<String>,
        quantity: i64,
        unit_price: Money,
    ) -> LineItemId {
        // Check if item already exists
        if let Some(existing) = self.items.iter_mut().find(|i| i.variant_id == variant_id) {
            existing.quantity += quantity;
            existing.update_total();
            self.updated_at = current_timestamp();
            return existing.id.clone();
        }

        // Add new item
        let item = LineItem::new(variant_id, product_id, product_name, quantity, unit_price);
        let id = item.id.clone();
        self.items.push(item);
        self.updated_at = current_timestamp();
        id
    }

    /// Update item quantity.
    pub fn update_quantity(&mut self, line_item_id: &LineItemId, quantity: i64) -> bool {
        if quantity <= 0 {
            return self.remove_item(line_item_id);
        }

        if let Some(item) = self.items.iter_mut().find(|i| &i.id == line_item_id) {
            item.quantity = quantity;
            item.update_total();
            self.updated_at = current_timestamp();
            true
        } else {
            false
        }
    }

    /// Remove an item from the cart.
    pub fn remove_item(&mut self, line_item_id: &LineItemId) -> bool {
        let len_before = self.items.len();
        self.items.retain(|i| &i.id != line_item_id);
        let removed = self.items.len() < len_before;
        if removed {
            self.updated_at = current_timestamp();
        }
        removed
    }

    /// Clear all items from the cart.
    pub fn clear(&mut self) {
        self.items.clear();
        self.discounts.clear();
        self.updated_at = current_timestamp();
    }

    /// Apply a discount to the cart.
    pub fn apply_discount(&mut self, discount: AppliedDiscount) {
        // Remove any existing discount with same code
        self.discounts.retain(|d| d.code != discount.code);
        self.discounts.push(discount);
        self.updated_at = current_timestamp();
    }

    /// Remove a discount by code.
    pub fn remove_discount(&mut self, code: &str) -> bool {
        let len_before = self.discounts.len();
        self.discounts.retain(|d| d.code != code);
        let removed = self.discounts.len() < len_before;
        if removed {
            self.updated_at = current_timestamp();
        }
        removed
    }

    /// Get total item count (sum of quantities).
    pub fn item_count(&self) -> i64 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    /// Get number of unique items.
    pub fn unique_item_count(&self) -> usize {
        self.items.len()
    }

    /// Check if cart is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get an item by ID.
    pub fn get_item(&self, line_item_id: &LineItemId) -> Option<&LineItem> {
        self.items.iter().find(|i| &i.id == line_item_id)
    }

    /// Get an item by variant ID.
    pub fn get_item_by_variant(&self, variant_id: &VariantId) -> Option<&LineItem> {
        self.items.iter().find(|i| &i.variant_id == variant_id)
    }

    /// Calculate cart pricing.
    pub fn calculate_pricing(&self) -> CartPricing {
        let line_items: Vec<LineItemPricing> = self
            .items
            .iter()
            .map(|item| LineItemPricing {
                line_item_id: item.id.clone(),
                unit_price: item.unit_price,
                quantity: item.quantity,
                subtotal: item.total_price,
                discount_amount: Money::zero(self.currency),
                tax_amount: Money::zero(self.currency),
                total: item.total_price,
            })
            .collect();

        let subtotal = Money::sum(
            self.items.iter().map(|i| &i.total_price),
            self.currency,
        );

        let discount_total = Money::sum(
            self.discounts.iter().map(|d| &d.amount),
            self.currency,
        );

        CartPricing {
            subtotal,
            discount_total,
            shipping_total: Money::zero(self.currency),
            tax_total: Money::zero(self.currency),
            grand_total: subtotal.subtract(&discount_total),
            line_items,
        }
    }

    /// Merge another cart into this one (e.g., when user logs in).
    pub fn merge(&mut self, other: Cart) {
        for item in other.items {
            if let Some(existing) = self.items.iter_mut().find(|i| i.variant_id == item.variant_id)
            {
                existing.quantity += item.quantity;
                existing.update_total();
            } else {
                self.items.push(item);
            }
        }
        self.updated_at = current_timestamp();
    }

    /// Set the cart for an authenticated user.
    pub fn set_user(&mut self, user_id: UserId) {
        self.user_id = Some(user_id);
        self.updated_at = current_timestamp();
    }
}

impl Default for Cart {
    fn default() -> Self {
        Self::new("anonymous")
    }
}

/// A line item in the cart.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineItem {
    /// Unique line item identifier.
    pub id: LineItemId,
    /// Variant being purchased.
    pub variant_id: VariantId,
    /// Product ID.
    pub product_id: ProductId,
    /// Product name (denormalized for display).
    pub product_name: String,
    /// Variant name (e.g., "Large / Blue").
    pub variant_name: Option<String>,
    /// Quantity.
    pub quantity: i64,
    /// Unit price.
    pub unit_price: Money,
    /// Total price (unit_price * quantity).
    pub total_price: Money,
    /// Custom properties (e.g., gift wrapping, engraving).
    pub properties: Vec<LineItemProperty>,
}

impl LineItem {
    /// Create a new line item.
    pub fn new(
        variant_id: VariantId,
        product_id: ProductId,
        product_name: impl Into<String>,
        quantity: i64,
        unit_price: Money,
    ) -> Self {
        Self {
            id: LineItemId::generate(),
            variant_id,
            product_id,
            product_name: product_name.into(),
            variant_name: None,
            quantity,
            unit_price,
            total_price: unit_price.multiply(quantity),
            properties: Vec::new(),
        }
    }

    /// Update the total price based on quantity.
    pub fn update_total(&mut self) {
        self.total_price = self.unit_price.multiply(self.quantity);
    }

    /// Add a custom property.
    pub fn add_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.properties.push(LineItemProperty {
            name: name.into(),
            value: value.into(),
        });
    }
}

/// A custom property on a line item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineItemProperty {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: String,
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
    fn test_cart_creation() {
        let cart = Cart::new("session-123");
        assert!(cart.is_empty());
        assert_eq!(cart.session_id, "session-123");
    }

    #[test]
    fn test_add_item() {
        let mut cart = Cart::new("session-123");
        cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Test Product",
            2,
            Money::new(1000, Currency::USD),
        );

        assert_eq!(cart.item_count(), 2);
        assert_eq!(cart.unique_item_count(), 1);
    }

    #[test]
    fn test_add_same_item_increases_quantity() {
        let mut cart = Cart::new("session-123");
        let variant_id = VariantId::new("var-1");

        cart.add_item(
            variant_id.clone(),
            ProductId::new("prod-1"),
            "Test Product",
            1,
            Money::new(1000, Currency::USD),
        );

        cart.add_item(
            variant_id.clone(),
            ProductId::new("prod-1"),
            "Test Product",
            2,
            Money::new(1000, Currency::USD),
        );

        assert_eq!(cart.unique_item_count(), 1);
        assert_eq!(cart.item_count(), 3);
    }

    #[test]
    fn test_update_quantity() {
        let mut cart = Cart::new("session-123");
        let line_id = cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Test Product",
            1,
            Money::new(1000, Currency::USD),
        );

        cart.update_quantity(&line_id, 5);
        assert_eq!(cart.item_count(), 5);
    }

    #[test]
    fn test_remove_item() {
        let mut cart = Cart::new("session-123");
        let line_id = cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Test Product",
            1,
            Money::new(1000, Currency::USD),
        );

        assert!(cart.remove_item(&line_id));
        assert!(cart.is_empty());
    }

    #[test]
    fn test_pricing() {
        let mut cart = Cart::new("session-123");
        cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Product A",
            2,
            Money::new(1000, Currency::USD),
        );
        cart.add_item(
            VariantId::new("var-2"),
            ProductId::new("prod-2"),
            "Product B",
            1,
            Money::new(2000, Currency::USD),
        );

        let pricing = cart.calculate_pricing();
        assert_eq!(pricing.subtotal.amount_cents, 4000); // 2*1000 + 1*2000
        assert_eq!(pricing.grand_total.amount_cents, 4000);
    }
}
