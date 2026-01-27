//! Product and variant types.

use crate::ids::{CategoryId, MediaId, ProductId, VariantId};
use crate::money::Money;
use crate::catalog::InventoryLevel;
use serde::{Deserialize, Serialize};

/// Product status in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProductStatus {
    /// Product is in draft mode, not visible to customers.
    Draft,
    /// Product is active and visible.
    #[default]
    Active,
    /// Product is archived, not visible but data preserved.
    Archived,
}

impl ProductStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProductStatus::Draft => "draft",
            ProductStatus::Active => "active",
            ProductStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(ProductStatus::Draft),
            "active" => Some(ProductStatus::Active),
            "archived" => Some(ProductStatus::Archived),
            _ => None,
        }
    }
}

/// Product type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProductType {
    /// Simple product with no variants.
    #[default]
    Simple,
    /// Product with variants (e.g., size, color).
    Variable,
    /// Bundle of multiple products.
    Bundle,
    /// Digital/downloadable product.
    Digital,
}

impl ProductType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProductType::Simple => "simple",
            ProductType::Variable => "variable",
            ProductType::Bundle => "bundle",
            ProductType::Digital => "digital",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "simple" => Some(ProductType::Simple),
            "variable" => Some(ProductType::Variable),
            "bundle" => Some(ProductType::Bundle),
            "digital" => Some(ProductType::Digital),
            _ => None,
        }
    }
}

/// A product in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    /// Unique product identifier.
    pub id: ProductId,
    /// Stock keeping unit (unique).
    pub sku: String,
    /// Product name.
    pub name: String,
    /// URL-friendly slug (unique).
    pub slug: String,
    /// Full description (may contain HTML/markdown).
    pub description: Option<String>,
    /// Short description for listings.
    pub short_description: Option<String>,
    /// Product visibility status.
    pub status: ProductStatus,
    /// Type of product.
    pub product_type: ProductType,
    /// Categories this product belongs to.
    pub category_ids: Vec<CategoryId>,
    /// Tags for filtering/search.
    pub tags: Vec<String>,
    /// ID of the default variant (for variable products).
    pub default_variant_id: Option<VariantId>,
    /// Additional metadata as JSON.
    pub metadata: serde_json::Value,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

impl Product {
    /// Create a new simple product.
    pub fn new(sku: impl Into<String>, name: impl Into<String>, slug: impl Into<String>) -> Self {
        let now = current_timestamp();
        Self {
            id: ProductId::generate(),
            sku: sku.into(),
            name: name.into(),
            slug: slug.into(),
            description: None,
            short_description: None,
            status: ProductStatus::Active,
            product_type: ProductType::Simple,
            category_ids: Vec::new(),
            tags: Vec::new(),
            default_variant_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the product is available for purchase.
    pub fn is_available(&self) -> bool {
        self.status == ProductStatus::Active
    }

    /// Check if this is a variable product (has variants).
    pub fn has_variants(&self) -> bool {
        self.product_type == ProductType::Variable
    }

    /// Check if this is a digital product.
    pub fn is_digital(&self) -> bool {
        self.product_type == ProductType::Digital
    }

    /// Add a category to this product.
    pub fn add_category(&mut self, category_id: CategoryId) {
        if !self.category_ids.contains(&category_id) {
            self.category_ids.push(category_id);
        }
    }

    /// Add a tag to this product.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

/// A product variant (e.g., size/color combination).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductVariant {
    /// Unique variant identifier.
    pub id: VariantId,
    /// Parent product ID.
    pub product_id: ProductId,
    /// Stock keeping unit for this variant (unique).
    pub sku: String,
    /// Variant name (e.g., "Large / Blue").
    pub name: Option<String>,
    /// Price of this variant.
    pub price: Money,
    /// Compare-at price (original price for showing discounts).
    pub compare_at_price: Option<Money>,
    /// Cost price (for profit calculations).
    pub cost: Option<Money>,
    /// Options that define this variant.
    pub options: Vec<VariantOption>,
    /// Weight in grams (for shipping).
    pub weight_grams: Option<i64>,
    /// Inventory level.
    pub inventory: InventoryLevel,
    /// Media/images for this variant.
    pub images: Vec<MediaId>,
    /// Sort order position.
    pub position: i32,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

impl ProductVariant {
    /// Create a new variant.
    pub fn new(product_id: ProductId, sku: impl Into<String>, price: Money) -> Self {
        let now = current_timestamp();
        Self {
            id: VariantId::generate(),
            product_id,
            sku: sku.into(),
            name: None,
            price,
            compare_at_price: None,
            cost: None,
            options: Vec::new(),
            weight_grams: None,
            inventory: InventoryLevel::default(),
            images: Vec::new(),
            position: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this variant is in stock.
    pub fn is_in_stock(&self) -> bool {
        self.inventory.is_available()
    }

    /// Check if this variant is on sale (has compare_at_price).
    pub fn is_on_sale(&self) -> bool {
        self.compare_at_price
            .map(|cap| cap.amount_cents > self.price.amount_cents)
            .unwrap_or(false)
    }

    /// Calculate the discount percentage if on sale.
    pub fn discount_percentage(&self) -> Option<f64> {
        self.compare_at_price.and_then(|cap| {
            if cap.amount_cents > self.price.amount_cents {
                let savings = cap.amount_cents - self.price.amount_cents;
                Some((savings as f64 / cap.amount_cents as f64) * 100.0)
            } else {
                None
            }
        })
    }

    /// Build the variant name from options.
    pub fn build_name(&self) -> String {
        if self.options.is_empty() {
            "Default".to_string()
        } else {
            self.options
                .iter()
                .map(|o| o.value.as_str())
                .collect::<Vec<_>>()
                .join(" / ")
        }
    }

    /// Add an option to this variant.
    pub fn add_option(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.options.push(VariantOption {
            name: name.into(),
            value: value.into(),
        });
    }
}

/// A variant option (e.g., Size: Large).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantOption {
    /// Option name (e.g., "Size", "Color").
    pub name: String,
    /// Option value (e.g., "Large", "Blue").
    pub value: String,
}

impl VariantOption {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Media type for product images/videos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MediaType {
    #[default]
    Image,
    Video,
    Model3d,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Image => "image",
            MediaType::Video => "video",
            MediaType::Model3d => "model3d",
        }
    }
}

/// Product media (image, video, 3D model).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductMedia {
    /// Unique media identifier.
    pub id: MediaId,
    /// Parent product ID.
    pub product_id: ProductId,
    /// Variants this media is associated with.
    pub variant_ids: Vec<VariantId>,
    /// Type of media.
    pub media_type: MediaType,
    /// URL to the media file.
    pub url: String,
    /// Alt text for accessibility.
    pub alt_text: Option<String>,
    /// Sort order position.
    pub position: i32,
    /// Image width in pixels.
    pub width: Option<i32>,
    /// Image height in pixels.
    pub height: Option<i32>,
}

impl ProductMedia {
    /// Create a new image media.
    pub fn new_image(product_id: ProductId, url: impl Into<String>) -> Self {
        Self {
            id: MediaId::generate(),
            product_id,
            variant_ids: Vec::new(),
            media_type: MediaType::Image,
            url: url.into(),
            alt_text: None,
            position: 0,
            width: None,
            height: None,
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
    fn test_product_creation() {
        let product = Product::new("SKU-001", "Test Product", "test-product");
        assert_eq!(product.sku, "SKU-001");
        assert_eq!(product.name, "Test Product");
        assert!(product.is_available());
    }

    #[test]
    fn test_variant_creation() {
        let product_id = ProductId::generate();
        let variant = ProductVariant::new(
            product_id.clone(),
            "SKU-001-L",
            Money::new(2999, Currency::USD),
        );
        assert_eq!(variant.product_id, product_id);
        assert_eq!(variant.price.amount_cents, 2999);
    }

    #[test]
    fn test_variant_on_sale() {
        let product_id = ProductId::generate();
        let mut variant = ProductVariant::new(
            product_id,
            "SKU-001",
            Money::new(2000, Currency::USD),
        );
        variant.compare_at_price = Some(Money::new(3000, Currency::USD));

        assert!(variant.is_on_sale());
        let discount = variant.discount_percentage().unwrap();
        assert!((discount - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_variant_options() {
        let product_id = ProductId::generate();
        let mut variant = ProductVariant::new(
            product_id,
            "SKU-001-L-BL",
            Money::new(2999, Currency::USD),
        );
        variant.add_option("Size", "Large");
        variant.add_option("Color", "Blue");

        assert_eq!(variant.build_name(), "Large / Blue");
    }
}
