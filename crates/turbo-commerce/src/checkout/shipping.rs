//! Shipping method types.

use crate::ids::ShippingMethodId;
use crate::money::Money;
use serde::{Deserialize, Serialize};

/// A shipping method option.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShippingMethod {
    /// Unique identifier.
    pub id: ShippingMethodId,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Carrier name (e.g., "UPS", "FedEx").
    pub carrier: Option<String>,
    /// Shipping price.
    pub price: Money,
    /// Minimum delivery days.
    pub min_delivery_days: Option<i32>,
    /// Maximum delivery days.
    pub max_delivery_days: Option<i32>,
}

impl ShippingMethod {
    /// Create a new shipping method.
    pub fn new(name: impl Into<String>, price: Money) -> Self {
        Self {
            id: ShippingMethodId::generate(),
            name: name.into(),
            description: None,
            carrier: None,
            price,
            min_delivery_days: None,
            max_delivery_days: None,
        }
    }

    /// Get delivery estimate string.
    pub fn delivery_estimate(&self) -> Option<String> {
        match (self.min_delivery_days, self.max_delivery_days) {
            (Some(min), Some(max)) if min == max => Some(format!("{} days", min)),
            (Some(min), Some(max)) => Some(format!("{}-{} days", min, max)),
            (Some(min), None) => Some(format!("{}+ days", min)),
            (None, Some(max)) => Some(format!("Up to {} days", max)),
            (None, None) => None,
        }
    }

    /// Check if this is free shipping.
    pub fn is_free(&self) -> bool {
        self.price.amount_cents == 0
    }
}

/// A selected shipping method with rate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShippingSelection {
    /// Selected method ID.
    pub method_id: ShippingMethodId,
    /// Method name (denormalized).
    pub method_name: String,
    /// Calculated rate.
    pub rate: Money,
    /// Carrier name.
    pub carrier: Option<String>,
    /// Delivery estimate.
    pub delivery_estimate: Option<String>,
}

impl ShippingSelection {
    /// Create from a shipping method.
    pub fn from_method(method: &ShippingMethod) -> Self {
        Self {
            method_id: method.id.clone(),
            method_name: method.name.clone(),
            rate: method.price,
            carrier: method.carrier.clone(),
            delivery_estimate: method.delivery_estimate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Currency;

    #[test]
    fn test_shipping_method() {
        let mut method = ShippingMethod::new(
            "Standard Shipping",
            Money::new(599, Currency::USD),
        );
        method.min_delivery_days = Some(5);
        method.max_delivery_days = Some(7);

        assert_eq!(method.delivery_estimate(), Some("5-7 days".to_string()));
        assert!(!method.is_free());
    }

    #[test]
    fn test_free_shipping() {
        let method = ShippingMethod::new(
            "Free Shipping",
            Money::zero(Currency::USD),
        );
        assert!(method.is_free());
    }
}
