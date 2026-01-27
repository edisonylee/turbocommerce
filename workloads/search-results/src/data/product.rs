//! Product data models.

use serde::{Deserialize, Serialize};

/// A product in search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProduct {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub thumbnail: String,
    pub category: String,
    pub rating: f64,
    pub stock: u32,
    #[serde(default)]
    pub brand: String,
}

/// Sponsored product with ad metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsoredProduct {
    pub product: SearchProduct,
    pub ad_id: String,
    pub bid_price: f64,
    pub impression_url: String,
}

impl SponsoredProduct {
    /// Create a sponsored product from a regular product.
    pub fn from_product(product: SearchProduct, ad_id: &str) -> Self {
        Self {
            ad_id: ad_id.to_string(),
            bid_price: product.price * 0.05,
            impression_url: format!("/ads/impression/{}", ad_id),
            product,
        }
    }
}
