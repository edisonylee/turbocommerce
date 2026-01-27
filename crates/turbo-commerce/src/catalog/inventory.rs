//! Inventory tracking types.

use crate::ids::VariantId;
use serde::{Deserialize, Serialize};

/// Inventory level for a product variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InventoryLevel {
    /// Total quantity in stock.
    pub quantity: i64,
    /// Quantity reserved for pending orders.
    pub reserved: i64,
    /// Whether to track inventory for this item.
    pub track_inventory: bool,
    /// Whether to allow orders when out of stock.
    pub allow_backorder: bool,
    /// Low stock threshold for alerts.
    pub low_stock_threshold: Option<i64>,
}

impl InventoryLevel {
    /// Create a new inventory level with tracking enabled.
    pub fn new(quantity: i64) -> Self {
        Self {
            quantity,
            reserved: 0,
            track_inventory: true,
            allow_backorder: false,
            low_stock_threshold: None,
        }
    }

    /// Create an inventory level with no tracking (infinite stock).
    pub fn untracked() -> Self {
        Self {
            quantity: 0,
            reserved: 0,
            track_inventory: false,
            allow_backorder: true,
            low_stock_threshold: None,
        }
    }

    /// Get available quantity (total minus reserved).
    pub fn available(&self) -> i64 {
        self.quantity - self.reserved
    }

    /// Check if the item is available for purchase.
    pub fn is_available(&self) -> bool {
        if !self.track_inventory {
            return true;
        }
        self.available() > 0 || self.allow_backorder
    }

    /// Check if a specific quantity is available.
    pub fn can_fulfill(&self, quantity: i64) -> bool {
        if !self.track_inventory {
            return true;
        }
        self.available() >= quantity || self.allow_backorder
    }

    /// Check if stock is low (below threshold).
    pub fn is_low_stock(&self) -> bool {
        if !self.track_inventory {
            return false;
        }
        self.low_stock_threshold
            .map(|threshold| self.available() <= threshold)
            .unwrap_or(false)
    }

    /// Check if out of stock.
    pub fn is_out_of_stock(&self) -> bool {
        self.track_inventory && self.available() <= 0
    }

    /// Reserve inventory for an order.
    pub fn reserve(&mut self, quantity: i64) -> bool {
        if !self.can_fulfill(quantity) {
            return false;
        }
        self.reserved += quantity;
        true
    }

    /// Release reserved inventory (e.g., order cancelled).
    pub fn release(&mut self, quantity: i64) {
        self.reserved = (self.reserved - quantity).max(0);
    }

    /// Commit reserved inventory (order shipped).
    pub fn commit(&mut self, quantity: i64) {
        self.reserved = (self.reserved - quantity).max(0);
        self.quantity = (self.quantity - quantity).max(0);
    }

    /// Add inventory (restock).
    pub fn restock(&mut self, quantity: i64) {
        self.quantity += quantity;
    }

    /// Adjust inventory (correction).
    pub fn adjust(&mut self, delta: i64) {
        self.quantity = (self.quantity + delta).max(0);
    }
}

/// Reason for an inventory adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjustmentReason {
    /// Sold to customer.
    Sale,
    /// Returned by customer.
    Return,
    /// Restocked from supplier.
    Restock,
    /// Manual correction.
    Correction,
    /// Reserved for pending order.
    Reserved,
    /// Released from reservation.
    Released,
    /// Damaged or lost.
    Shrinkage,
    /// Transferred to another location.
    Transfer,
}

impl AdjustmentReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdjustmentReason::Sale => "sale",
            AdjustmentReason::Return => "return",
            AdjustmentReason::Restock => "restock",
            AdjustmentReason::Correction => "correction",
            AdjustmentReason::Reserved => "reserved",
            AdjustmentReason::Released => "released",
            AdjustmentReason::Shrinkage => "shrinkage",
            AdjustmentReason::Transfer => "transfer",
        }
    }
}

/// An inventory adjustment record (for audit trail).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAdjustment {
    /// Variant that was adjusted.
    pub variant_id: VariantId,
    /// Change in quantity (positive or negative).
    pub quantity_change: i64,
    /// Reason for the adjustment.
    pub reason: AdjustmentReason,
    /// Reference ID (e.g., order ID).
    pub reference_id: Option<String>,
    /// Unix timestamp of adjustment.
    pub timestamp: i64,
}

impl InventoryAdjustment {
    pub fn new(
        variant_id: VariantId,
        quantity_change: i64,
        reason: AdjustmentReason,
    ) -> Self {
        Self {
            variant_id,
            quantity_change,
            reason,
            reference_id: None,
            timestamp: current_timestamp(),
        }
    }

    pub fn with_reference(mut self, reference_id: impl Into<String>) -> Self {
        self.reference_id = Some(reference_id.into());
        self
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
    fn test_inventory_availability() {
        let mut inv = InventoryLevel::new(10);
        assert!(inv.is_available());
        assert!(inv.can_fulfill(10));
        assert!(!inv.can_fulfill(11));

        inv.reserve(5);
        assert_eq!(inv.available(), 5);
        assert!(inv.can_fulfill(5));
        assert!(!inv.can_fulfill(6));
    }

    #[test]
    fn test_inventory_reserve_release() {
        let mut inv = InventoryLevel::new(10);

        assert!(inv.reserve(3));
        assert_eq!(inv.reserved, 3);
        assert_eq!(inv.available(), 7);

        inv.release(2);
        assert_eq!(inv.reserved, 1);
        assert_eq!(inv.available(), 9);
    }

    #[test]
    fn test_inventory_commit() {
        let mut inv = InventoryLevel::new(10);
        inv.reserve(3);
        inv.commit(3);

        assert_eq!(inv.quantity, 7);
        assert_eq!(inv.reserved, 0);
    }

    #[test]
    fn test_untracked_inventory() {
        let inv = InventoryLevel::untracked();
        assert!(inv.is_available());
        assert!(inv.can_fulfill(1000));
        assert!(!inv.is_out_of_stock());
    }

    #[test]
    fn test_low_stock() {
        let mut inv = InventoryLevel::new(5);
        inv.low_stock_threshold = Some(10);

        assert!(inv.is_low_stock());

        inv.quantity = 15;
        assert!(!inv.is_low_stock());
    }
}
