//! Newtype IDs for type-safe identifiers.
//!
//! Using newtypes prevents accidentally mixing up different ID types,
//! e.g., passing a ProductId where a VariantId is expected.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Macro to generate newtype ID structs.
macro_rules! define_id {
    ($name:ident) => {
        /// A unique identifier.
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Create a new ID from a string.
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Generate a new unique ID.
            pub fn generate() -> Self {
                Self(generate_id())
            }

            /// Get the ID as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume and return the inner string.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

// Define all ID types
define_id!(ProductId);
define_id!(VariantId);
define_id!(CategoryId);
define_id!(CartId);
define_id!(LineItemId);
define_id!(OrderId);
define_id!(OrderLineItemId);
define_id!(DiscountId);
define_id!(ShippingMethodId);
define_id!(CheckoutId);
define_id!(AddressId);
define_id!(MediaId);
define_id!(UserId);
define_id!(SessionId);

/// Generate a unique ID using timestamp and random bytes.
fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    // Combine timestamp with atomic counter for uniqueness
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

    // Also add memory address for extra entropy
    let ptr = Box::new(0u8);
    let addr = &*ptr as *const u8 as u64;

    let combined = timestamp as u64 ^ counter ^ addr;
    format!("{:x}", combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id = ProductId::new("prod-123");
        assert_eq!(id.as_str(), "prod-123");
    }

    #[test]
    fn test_id_generation() {
        let id1 = ProductId::generate();
        let id2 = ProductId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_from_string() {
        let id: ProductId = "prod-456".into();
        assert_eq!(id.as_str(), "prod-456");
    }

    #[test]
    fn test_id_display() {
        let id = ProductId::new("prod-789");
        assert_eq!(format!("{}", id), "prod-789");
    }

    #[test]
    fn test_id_equality() {
        let id1 = ProductId::new("same");
        let id2 = ProductId::new("same");
        let id3 = ProductId::new("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
