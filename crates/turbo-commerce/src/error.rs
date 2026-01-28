//! Commerce error types.

use thiserror::Error;

/// Errors that can occur in e-commerce operations.
#[derive(Error, Debug)]
pub enum CommerceError {
    /// Product not found.
    #[error("Product not found: {0}")]
    ProductNotFound(String),

    /// Variant not found.
    #[error("Variant not found: {0}")]
    VariantNotFound(String),

    /// Category not found.
    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    /// Cart not found.
    #[error("Cart not found: {0}")]
    CartNotFound(String),

    /// Order not found.
    #[error("Order not found: {0}")]
    OrderNotFound(String),

    /// Item not in cart.
    #[error("Item not in cart: {0}")]
    ItemNotInCart(String),

    /// Insufficient inventory.
    #[error("Insufficient inventory for {product_id}: requested {requested}, available {available}")]
    InsufficientInventory {
        product_id: String,
        requested: i64,
        available: i64,
    },

    /// Invalid quantity.
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(i64),

    /// Invalid checkout state transition.
    #[error("Invalid checkout transition from {from} to {to}")]
    InvalidCheckoutTransition { from: String, to: String },

    /// Checkout incomplete.
    #[error("Checkout incomplete: missing {0}")]
    CheckoutIncomplete(String),

    /// Invalid discount code.
    #[error("Invalid discount code: {0}")]
    InvalidDiscountCode(String),

    /// Discount expired.
    #[error("Discount expired: {0}")]
    DiscountExpired(String),

    /// Discount usage limit reached.
    #[error("Discount usage limit reached: {0}")]
    DiscountUsageLimitReached(String),

    /// Currency mismatch.
    #[error("Currency mismatch: expected {expected}, got {got}")]
    CurrencyMismatch { expected: String, got: String },

    /// Arithmetic overflow.
    #[error("Arithmetic overflow in money calculation")]
    Overflow,

    /// Quantity exceeds maximum allowed.
    #[error("Quantity {0} exceeds maximum allowed ({1})")]
    QuantityExceedsLimit(i64, i64),

    /// Database error.
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Cache error.
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<turbo_db::DbError> for CommerceError {
    fn from(e: turbo_db::DbError) -> Self {
        CommerceError::DatabaseError(e.to_string())
    }
}

impl From<turbo_cache::CacheError> for CommerceError {
    fn from(e: turbo_cache::CacheError) -> Self {
        CommerceError::CacheError(e.to_string())
    }
}

impl From<serde_json::Error> for CommerceError {
    fn from(e: serde_json::Error) -> Self {
        CommerceError::SerializationError(e.to_string())
    }
}
