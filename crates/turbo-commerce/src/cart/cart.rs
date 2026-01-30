//! Cart and line item types.

use crate::cart::{AppliedDiscount, CartPricing, LineItemPricing};
use crate::error::CommerceError;
use crate::ids::{CartId, LineItemId, ProductId, UserId, VariantId};
use crate::money::{Currency, Money};
use serde::{Deserialize, Serialize};

/// Maximum quantity allowed per line item.
pub const MAX_QUANTITY_PER_ITEM: i64 = 9999;

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
    ///
    /// Returns an error if:
    /// - Quantity is not positive
    /// - Adding would exceed MAX_QUANTITY_PER_ITEM
    /// - Arithmetic overflow would occur
    pub fn add_item(
        &mut self,
        variant_id: VariantId,
        product_id: ProductId,
        product_name: impl Into<String>,
        quantity: i64,
        unit_price: Money,
    ) -> Result<LineItemId, CommerceError> {
        // Validate quantity
        if quantity <= 0 {
            return Err(CommerceError::InvalidQuantity(quantity));
        }

        // Check if item already exists
        if let Some(existing) = self.items.iter_mut().find(|i| i.variant_id == variant_id) {
            let new_quantity = existing
                .quantity
                .checked_add(quantity)
                .ok_or(CommerceError::Overflow)?;

            if new_quantity > MAX_QUANTITY_PER_ITEM {
                return Err(CommerceError::QuantityExceedsLimit(
                    new_quantity,
                    MAX_QUANTITY_PER_ITEM,
                ));
            }

            existing.quantity = new_quantity;
            existing.update_total()?;
            self.updated_at = current_timestamp();
            return Ok(existing.id.clone());
        }

        // Validate new item quantity
        if quantity > MAX_QUANTITY_PER_ITEM {
            return Err(CommerceError::QuantityExceedsLimit(
                quantity,
                MAX_QUANTITY_PER_ITEM,
            ));
        }

        // Add new item
        let item = LineItem::new(variant_id, product_id, product_name, quantity, unit_price)?;
        let id = item.id.clone();
        self.items.push(item);
        self.updated_at = current_timestamp();
        Ok(id)
    }

    /// Update item quantity.
    ///
    /// If quantity is <= 0, removes the item.
    /// Returns error if quantity exceeds limit or would cause overflow.
    pub fn update_quantity(
        &mut self,
        line_item_id: &LineItemId,
        quantity: i64,
    ) -> Result<bool, CommerceError> {
        if quantity <= 0 {
            return Ok(self.remove_item(line_item_id));
        }

        if quantity > MAX_QUANTITY_PER_ITEM {
            return Err(CommerceError::QuantityExceedsLimit(
                quantity,
                MAX_QUANTITY_PER_ITEM,
            ));
        }

        if let Some(item) = self.items.iter_mut().find(|i| &i.id == line_item_id) {
            item.quantity = quantity;
            item.update_total()?;
            self.updated_at = current_timestamp();
            Ok(true)
        } else {
            Ok(false)
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
    ///
    /// Returns error if arithmetic overflow occurs.
    pub fn calculate_pricing(&self) -> Result<CartPricing, CommerceError> {
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

        let subtotal = Money::try_sum(self.items.iter().map(|i| &i.total_price), self.currency)
            .ok_or(CommerceError::Overflow)?;

        let discount_total =
            Money::try_sum(self.discounts.iter().map(|d| &d.amount), self.currency)
                .ok_or(CommerceError::Overflow)?;

        let grand_total = subtotal.try_subtract(&discount_total).ok_or_else(|| {
            CommerceError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                got: "mixed".to_string(),
            }
        })?;

        Ok(CartPricing {
            subtotal,
            discount_total,
            shipping_total: Money::zero(self.currency),
            tax_total: Money::zero(self.currency),
            grand_total,
            line_items,
        })
    }

    /// Merge another cart into this one (e.g., when user logs in).
    ///
    /// Items that would exceed quantity limits are capped at MAX_QUANTITY_PER_ITEM.
    pub fn merge(&mut self, other: Cart) -> Result<(), CommerceError> {
        for item in other.items {
            if let Some(existing) = self
                .items
                .iter_mut()
                .find(|i| i.variant_id == item.variant_id)
            {
                // Use saturating add and cap at max
                let new_quantity = existing
                    .quantity
                    .saturating_add(item.quantity)
                    .min(MAX_QUANTITY_PER_ITEM);
                existing.quantity = new_quantity;
                existing.update_total()?;
            } else {
                self.items.push(item);
            }
        }
        self.updated_at = current_timestamp();
        Ok(())
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
    ) -> Result<Self, CommerceError> {
        let total_price = unit_price
            .try_multiply(quantity)
            .ok_or(CommerceError::Overflow)?;
        Ok(Self {
            id: LineItemId::generate(),
            variant_id,
            product_id,
            product_name: product_name.into(),
            variant_name: None,
            quantity,
            unit_price,
            total_price,
            properties: Vec::new(),
        })
    }

    /// Update the total price based on quantity.
    pub fn update_total(&mut self) -> Result<(), CommerceError> {
        self.total_price = self
            .unit_price
            .try_multiply(self.quantity)
            .ok_or(CommerceError::Overflow)?;
        Ok(())
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
        )
        .unwrap();

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
        )
        .unwrap();

        cart.add_item(
            variant_id.clone(),
            ProductId::new("prod-1"),
            "Test Product",
            2,
            Money::new(1000, Currency::USD),
        )
        .unwrap();

        assert_eq!(cart.unique_item_count(), 1);
        assert_eq!(cart.item_count(), 3);
    }

    #[test]
    fn test_update_quantity() {
        let mut cart = Cart::new("session-123");
        let line_id = cart
            .add_item(
                VariantId::new("var-1"),
                ProductId::new("prod-1"),
                "Test Product",
                1,
                Money::new(1000, Currency::USD),
            )
            .unwrap();

        cart.update_quantity(&line_id, 5).unwrap();
        assert_eq!(cart.item_count(), 5);
    }

    #[test]
    fn test_remove_item() {
        let mut cart = Cart::new("session-123");
        let line_id = cart
            .add_item(
                VariantId::new("var-1"),
                ProductId::new("prod-1"),
                "Test Product",
                1,
                Money::new(1000, Currency::USD),
            )
            .unwrap();

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
        )
        .unwrap();
        cart.add_item(
            VariantId::new("var-2"),
            ProductId::new("prod-2"),
            "Product B",
            1,
            Money::new(2000, Currency::USD),
        )
        .unwrap();

        let pricing = cart.calculate_pricing().unwrap();
        assert_eq!(pricing.subtotal.amount_cents, 4000); // 2*1000 + 1*2000
        assert_eq!(pricing.grand_total.amount_cents, 4000);
    }

    #[test]
    fn test_quantity_limit() {
        let mut cart = Cart::new("session-123");
        let result = cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Test Product",
            MAX_QUANTITY_PER_ITEM + 1,
            Money::new(1000, Currency::USD),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_quantity() {
        let mut cart = Cart::new("session-123");
        let result = cart.add_item(
            VariantId::new("var-1"),
            ProductId::new("prod-1"),
            "Test Product",
            0,
            Money::new(1000, Currency::USD),
        );
        assert!(result.is_err());
    }
}
