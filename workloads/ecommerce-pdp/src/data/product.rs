//! Product data models.

use serde::{Deserialize, Serialize};

/// Product information from the CMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub brand: String,
    pub category: String,
    pub images: Vec<ProductImage>,
    pub attributes: Vec<ProductAttribute>,
}

/// Product image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    pub url: String,
    pub alt: String,
    #[serde(default)]
    pub is_primary: bool,
}

/// Product attribute (color, size, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAttribute {
    pub name: String,
    pub value: String,
}

/// Live pricing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub product_id: String,
    pub price: f64,
    pub currency: String,
    #[serde(default)]
    pub original_price: Option<f64>,
    #[serde(default)]
    pub discount_percentage: Option<u8>,
    #[serde(default)]
    pub member_price: Option<f64>,
}

impl Pricing {
    /// Check if the product is on sale.
    pub fn is_on_sale(&self) -> bool {
        self.original_price.is_some() && self.discount_percentage.is_some()
    }

    /// Format price with currency.
    pub fn format_price(&self, price: f64) -> String {
        match self.currency.as_str() {
            "USD" => format!("${:.2}", price),
            "EUR" => format!("{:.2}", price),
            "GBP" => format!("{:.2}", price),
            _ => format!("{:.2} {}", price, self.currency),
        }
    }
}

/// Inventory/stock information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub product_id: String,
    pub in_stock: bool,
    pub quantity: u32,
    pub warehouse: String,
    #[serde(default)]
    pub estimated_restock: Option<String>,
}

impl Inventory {
    /// Get stock status message.
    pub fn status_message(&self) -> &'static str {
        if !self.in_stock {
            "Out of Stock"
        } else if self.quantity <= 5 {
            "Low Stock - Order Soon!"
        } else if self.quantity <= 20 {
            "In Stock"
        } else {
            "In Stock - Ships Today"
        }
    }

    /// Get status CSS class.
    pub fn status_class(&self) -> &'static str {
        if !self.in_stock {
            "stock-out"
        } else if self.quantity <= 5 {
            "stock-low"
        } else {
            "stock-available"
        }
    }
}

/// Recommendation item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedProduct {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub image_url: String,
    pub reason: String, // "frequently bought together", "similar items", etc.
}
