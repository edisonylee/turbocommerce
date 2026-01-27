//! Search filter types.

use crate::ids::CategoryId;
use crate::money::Money;
use serde::{Deserialize, Serialize};

/// A search filter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Filter {
    /// Filter by single category.
    Category(CategoryId),
    /// Filter by multiple categories (OR).
    Categories(Vec<CategoryId>),
    /// Filter by price range.
    PriceRange {
        min: Option<Money>,
        max: Option<Money>,
    },
    /// Only show in-stock items.
    InStock,
    /// Filter by tag.
    Tag(String),
    /// Filter by multiple tags (OR).
    Tags(Vec<String>),
    /// Filter by attribute/option (e.g., Color: Blue).
    Attribute {
        name: String,
        values: Vec<String>,
    },
    /// Filter by minimum rating.
    Rating {
        min: f64,
    },
    /// Filter by product type.
    ProductType(String),
    /// Filter by product status.
    Status(String),
    /// Full-text search in name/description.
    Text(String),
    /// Filter by SKU prefix.
    SkuPrefix(String),
    /// Filter by date range.
    DateRange {
        field: String,
        start: Option<i64>,
        end: Option<i64>,
    },
}

impl Filter {
    /// Create a category filter.
    pub fn category(id: impl Into<CategoryId>) -> Self {
        Filter::Category(id.into())
    }

    /// Create a price range filter.
    pub fn price_range(min: Option<Money>, max: Option<Money>) -> Self {
        Filter::PriceRange { min, max }
    }

    /// Create an in-stock filter.
    pub fn in_stock() -> Self {
        Filter::InStock
    }

    /// Create a tag filter.
    pub fn tag(tag: impl Into<String>) -> Self {
        Filter::Tag(tag.into())
    }

    /// Create an attribute filter.
    pub fn attribute(name: impl Into<String>, values: Vec<String>) -> Self {
        Filter::Attribute {
            name: name.into(),
            values,
        }
    }

    /// Create a text search filter.
    pub fn text(query: impl Into<String>) -> Self {
        Filter::Text(query.into())
    }

    /// Build SQL WHERE clause component.
    pub fn to_sql(&self) -> (String, Vec<String>) {
        match self {
            Filter::Category(id) => {
                ("category_id = ?".to_string(), vec![id.as_str().to_string()])
            }
            Filter::Categories(ids) => {
                let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let values = ids.iter().map(|id| id.as_str().to_string()).collect();
                (format!("category_id IN ({})", placeholders), values)
            }
            Filter::PriceRange { min, max } => {
                let mut clauses = Vec::new();
                let mut values = Vec::new();
                if let Some(min) = min {
                    clauses.push("price_cents >= ?".to_string());
                    values.push(min.amount_cents.to_string());
                }
                if let Some(max) = max {
                    clauses.push("price_cents <= ?".to_string());
                    values.push(max.amount_cents.to_string());
                }
                (clauses.join(" AND "), values)
            }
            Filter::InStock => ("quantity > 0".to_string(), vec![]),
            Filter::Tag(tag) => {
                ("id IN (SELECT product_id FROM product_tags WHERE tag = ?)".to_string(), vec![tag.clone()])
            }
            Filter::Tags(tags) => {
                let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                (
                    format!("id IN (SELECT product_id FROM product_tags WHERE tag IN ({}))", placeholders),
                    tags.clone(),
                )
            }
            Filter::Text(query) => {
                ("(name LIKE ? OR description LIKE ?)".to_string(), vec![format!("%{}%", query), format!("%{}%", query)])
            }
            Filter::Status(status) => {
                ("status = ?".to_string(), vec![status.clone()])
            }
            Filter::ProductType(pt) => {
                ("product_type = ?".to_string(), vec![pt.clone()])
            }
            Filter::SkuPrefix(prefix) => {
                ("sku LIKE ?".to_string(), vec![format!("{}%", prefix)])
            }
            _ => ("1=1".to_string(), vec![]), // No-op for unsupported filters
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::Currency;

    #[test]
    fn test_price_range_sql() {
        let filter = Filter::price_range(
            Some(Money::new(1000, Currency::USD)),
            Some(Money::new(5000, Currency::USD)),
        );
        let (sql, values) = filter.to_sql();
        assert!(sql.contains("price_cents >="));
        assert!(sql.contains("price_cents <="));
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_text_filter_sql() {
        let filter = Filter::text("rust");
        let (sql, values) = filter.to_sql();
        assert!(sql.contains("LIKE"));
        assert_eq!(values.len(), 2);
        assert!(values[0].contains("rust"));
    }
}
