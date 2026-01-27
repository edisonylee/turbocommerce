//! Checkout flow state machine.

use crate::ids::{CartId, CheckoutId};
use crate::checkout::{Address, ShippingSelection};
use crate::CommerceError;
use serde::{Deserialize, Serialize};

/// Steps in the checkout flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckoutStep {
    /// Cart review.
    Cart,
    /// Contact information.
    Information,
    /// Shipping address and method.
    Shipping,
    /// Payment details.
    Payment,
    /// Order review before submission.
    Review,
    /// Checkout complete.
    Complete,
}

impl CheckoutStep {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckoutStep::Cart => "cart",
            CheckoutStep::Information => "information",
            CheckoutStep::Shipping => "shipping",
            CheckoutStep::Payment => "payment",
            CheckoutStep::Review => "review",
            CheckoutStep::Complete => "complete",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CheckoutStep::Cart => "Cart",
            CheckoutStep::Information => "Information",
            CheckoutStep::Shipping => "Shipping",
            CheckoutStep::Payment => "Payment",
            CheckoutStep::Review => "Review",
            CheckoutStep::Complete => "Complete",
        }
    }

    /// Get the step number (1-indexed).
    pub fn number(&self) -> u8 {
        match self {
            CheckoutStep::Cart => 1,
            CheckoutStep::Information => 2,
            CheckoutStep::Shipping => 3,
            CheckoutStep::Payment => 4,
            CheckoutStep::Review => 5,
            CheckoutStep::Complete => 6,
        }
    }
}

/// Checkout flow state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckoutFlow {
    /// Unique checkout identifier.
    pub id: CheckoutId,
    /// Associated cart ID.
    pub cart_id: CartId,
    /// Current step.
    pub step: CheckoutStep,
    /// Completed steps.
    pub completed_steps: Vec<CheckoutStep>,
    /// Customer email.
    pub email: Option<String>,
    /// Shipping address.
    pub shipping_address: Option<Address>,
    /// Billing address (if different from shipping).
    pub billing_address: Option<Address>,
    /// Use shipping address as billing.
    pub billing_same_as_shipping: bool,
    /// Selected shipping method.
    pub shipping_method: Option<ShippingSelection>,
    /// Payment method identifier/token.
    pub payment_token: Option<String>,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
    /// Unix timestamp when checkout expires.
    pub expires_at: i64,
}

impl CheckoutFlow {
    /// Create a new checkout flow.
    pub fn new(cart_id: CartId) -> Self {
        let now = current_timestamp();
        Self {
            id: CheckoutId::generate(),
            cart_id,
            step: CheckoutStep::Cart,
            completed_steps: Vec::new(),
            email: None,
            shipping_address: None,
            billing_address: None,
            billing_same_as_shipping: true,
            shipping_method: None,
            payment_token: None,
            created_at: now,
            updated_at: now,
            expires_at: now + 3600, // 1 hour default expiry
        }
    }

    /// Check if checkout can advance to a step.
    pub fn can_advance_to(&self, step: CheckoutStep) -> bool {
        match step {
            CheckoutStep::Cart => true,
            CheckoutStep::Information => true,
            CheckoutStep::Shipping => self.email.is_some(),
            CheckoutStep::Payment => {
                self.email.is_some()
                    && self.shipping_address.as_ref().map(|a| a.is_complete()).unwrap_or(false)
                    && self.shipping_method.is_some()
            }
            CheckoutStep::Review => {
                self.can_advance_to(CheckoutStep::Payment) && self.payment_token.is_some()
            }
            CheckoutStep::Complete => self.can_advance_to(CheckoutStep::Review),
        }
    }

    /// Advance to the next step.
    pub fn advance(&mut self) -> Result<CheckoutStep, CommerceError> {
        let next = match self.step {
            CheckoutStep::Cart => CheckoutStep::Information,
            CheckoutStep::Information => CheckoutStep::Shipping,
            CheckoutStep::Shipping => CheckoutStep::Payment,
            CheckoutStep::Payment => CheckoutStep::Review,
            CheckoutStep::Review => CheckoutStep::Complete,
            CheckoutStep::Complete => {
                return Err(CommerceError::InvalidCheckoutTransition {
                    from: "complete".to_string(),
                    to: "none".to_string(),
                })
            }
        };

        if !self.can_advance_to(next) {
            return Err(CommerceError::CheckoutIncomplete(
                self.missing_for_step(next).join(", "),
            ));
        }

        if !self.completed_steps.contains(&self.step) {
            self.completed_steps.push(self.step);
        }
        self.step = next;
        self.updated_at = current_timestamp();

        Ok(next)
    }

    /// Go back to a previous step.
    pub fn go_back(&mut self) -> Result<CheckoutStep, CommerceError> {
        let prev = match self.step {
            CheckoutStep::Cart => {
                return Err(CommerceError::InvalidCheckoutTransition {
                    from: "cart".to_string(),
                    to: "none".to_string(),
                })
            }
            CheckoutStep::Information => CheckoutStep::Cart,
            CheckoutStep::Shipping => CheckoutStep::Information,
            CheckoutStep::Payment => CheckoutStep::Shipping,
            CheckoutStep::Review => CheckoutStep::Payment,
            CheckoutStep::Complete => CheckoutStep::Review,
        };

        self.step = prev;
        self.updated_at = current_timestamp();

        Ok(prev)
    }

    /// Go to a specific step (if allowed).
    pub fn go_to(&mut self, step: CheckoutStep) -> Result<(), CommerceError> {
        // Can go back to any completed step or the current step
        if step == self.step || self.completed_steps.contains(&step) {
            self.step = step;
            self.updated_at = current_timestamp();
            Ok(())
        } else if self.can_advance_to(step) && step.number() == self.step.number() + 1 {
            self.advance()?;
            Ok(())
        } else {
            Err(CommerceError::InvalidCheckoutTransition {
                from: self.step.as_str().to_string(),
                to: step.as_str().to_string(),
            })
        }
    }

    /// Get what's missing to advance to a step.
    fn missing_for_step(&self, step: CheckoutStep) -> Vec<&'static str> {
        let mut missing = Vec::new();
        match step {
            CheckoutStep::Shipping => {
                if self.email.is_none() {
                    missing.push("email");
                }
            }
            CheckoutStep::Payment => {
                if self.email.is_none() {
                    missing.push("email");
                }
                if self.shipping_address.is_none() {
                    missing.push("shipping address");
                }
                if self.shipping_method.is_none() {
                    missing.push("shipping method");
                }
            }
            CheckoutStep::Review => {
                missing.extend(self.missing_for_step(CheckoutStep::Payment));
                if self.payment_token.is_none() {
                    missing.push("payment method");
                }
            }
            _ => {}
        }
        missing
    }

    /// Set the customer email.
    pub fn set_email(&mut self, email: impl Into<String>) {
        self.email = Some(email.into());
        self.updated_at = current_timestamp();
    }

    /// Set the shipping address.
    pub fn set_shipping_address(&mut self, address: Address) {
        self.shipping_address = Some(address);
        self.updated_at = current_timestamp();
    }

    /// Set the billing address.
    pub fn set_billing_address(&mut self, address: Address) {
        self.billing_address = Some(address);
        self.billing_same_as_shipping = false;
        self.updated_at = current_timestamp();
    }

    /// Set billing same as shipping.
    pub fn set_billing_same_as_shipping(&mut self, same: bool) {
        self.billing_same_as_shipping = same;
        if same {
            self.billing_address = None;
        }
        self.updated_at = current_timestamp();
    }

    /// Set the shipping method.
    pub fn set_shipping_method(&mut self, selection: ShippingSelection) {
        self.shipping_method = Some(selection);
        self.updated_at = current_timestamp();
    }

    /// Set the payment token.
    pub fn set_payment_token(&mut self, token: impl Into<String>) {
        self.payment_token = Some(token.into());
        self.updated_at = current_timestamp();
    }

    /// Get the effective billing address.
    pub fn effective_billing_address(&self) -> Option<&Address> {
        if self.billing_same_as_shipping {
            self.shipping_address.as_ref()
        } else {
            self.billing_address.as_ref()
        }
    }

    /// Check if checkout is complete.
    pub fn is_complete(&self) -> bool {
        self.step == CheckoutStep::Complete
    }

    /// Check if checkout has expired.
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Get progress percentage.
    pub fn progress_percent(&self) -> u8 {
        ((self.step.number() as f64 / 6.0) * 100.0) as u8
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
    fn test_checkout_creation() {
        let flow = CheckoutFlow::new(CartId::new("cart-123"));
        assert_eq!(flow.step, CheckoutStep::Cart);
        assert!(flow.completed_steps.is_empty());
    }

    #[test]
    fn test_checkout_advance() {
        let mut flow = CheckoutFlow::new(CartId::new("cart-123"));

        // Can always advance from cart to information
        assert!(flow.advance().is_ok());
        assert_eq!(flow.step, CheckoutStep::Information);

        // Set email to advance to shipping
        flow.set_email("test@example.com");
        assert!(flow.advance().is_ok());
        assert_eq!(flow.step, CheckoutStep::Shipping);
    }

    #[test]
    fn test_checkout_requires_data() {
        let mut flow = CheckoutFlow::new(CartId::new("cart-123"));
        flow.step = CheckoutStep::Information;

        // Can't advance to shipping without email
        assert!(flow.advance().is_err());

        flow.set_email("test@example.com");
        assert!(flow.advance().is_ok());
    }

    #[test]
    fn test_checkout_go_back() {
        let mut flow = CheckoutFlow::new(CartId::new("cart-123"));
        flow.step = CheckoutStep::Shipping;
        flow.completed_steps = vec![CheckoutStep::Cart, CheckoutStep::Information];

        assert!(flow.go_back().is_ok());
        assert_eq!(flow.step, CheckoutStep::Information);
    }
}
