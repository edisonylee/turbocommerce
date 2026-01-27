//! Search-specific data models.

use serde::{Deserialize, Serialize};
use super::SearchProduct;

/// Search query parameters.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub q: String,
    pub page: u32,
    pub per_page: u32,
    pub sort: SortOrder,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

impl SearchQuery {
    /// Parse search query from URL query string.
    pub fn from_query_string(qs: &str) -> Self {
        let mut query = SearchQuery {
            page: 1,
            per_page: 20,
            ..Default::default()
        };

        for pair in qs.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            let decoded = urlencoding_decode(value);

            match key {
                "q" => query.q = decoded,
                "page" => query.page = decoded.parse().unwrap_or(1),
                "per_page" => query.per_page = decoded.parse().unwrap_or(20).min(100),
                "sort" => query.sort = SortOrder::from_str(&decoded),
                "category" => query.category = Some(decoded),
                "min_price" => query.min_price = decoded.parse().ok(),
                "max_price" => query.max_price = decoded.parse().ok(),
                _ => {}
            }
        }

        query
    }

    /// Generate cache key for this query.
    pub fn cache_key(&self) -> String {
        format!(
            "search:{}:{}:{}:{}",
            self.q.to_lowercase().replace(' ', "_"),
            self.page,
            self.sort.as_str(),
            self.category.as_deref().unwrap_or("all")
        )
    }
}

/// Sort order for search results.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    #[default]
    Relevance,
    PriceLowToHigh,
    PriceHighToLow,
    Rating,
    Newest,
}

impl SortOrder {
    pub fn from_str(s: &str) -> Self {
        match s {
            "price_asc" => Self::PriceLowToHigh,
            "price_desc" => Self::PriceHighToLow,
            "rating" => Self::Rating,
            "newest" => Self::Newest,
            _ => Self::Relevance,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Relevance => "relevance",
            Self::PriceLowToHigh => "price_asc",
            Self::PriceHighToLow => "price_desc",
            Self::Rating => "rating",
            Self::Newest => "newest",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Relevance => "Relevance",
            Self::PriceLowToHigh => "Price: Low to High",
            Self::PriceHighToLow => "Price: High to Low",
            Self::Rating => "Customer Rating",
            Self::Newest => "Newest Arrivals",
        }
    }
}

/// Search results response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub products: Vec<SearchProduct>,
    pub total: u32,
    pub skip: u32,
    pub limit: u32,
}

/// Facet for filtering.
#[derive(Debug, Clone)]
pub struct Facet {
    pub name: String,
    pub key: String,
    pub values: Vec<FacetValue>,
}

/// A single facet value with count.
#[derive(Debug, Clone)]
pub struct FacetValue {
    pub value: String,
    pub count: u32,
    pub selected: bool,
}

/// Available facets for search.
#[derive(Debug, Clone)]
pub struct SearchFacets {
    pub categories: Facet,
    pub price_ranges: Facet,
    pub brands: Facet,
    pub ratings: Facet,
}

impl SearchFacets {
    /// Generate mock facets based on search results.
    pub fn from_results(results: &[SearchProduct], query: &SearchQuery) -> Self {
        // Count categories
        let mut category_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut brand_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for product in results {
            *category_counts.entry(product.category.clone()).or_insert(0) += 1;
            if !product.brand.is_empty() {
                *brand_counts.entry(product.brand.clone()).or_insert(0) += 1;
            }
        }

        let mut category_values: Vec<FacetValue> = category_counts
            .into_iter()
            .map(|(value, count)| FacetValue {
                selected: query.category.as_ref() == Some(&value),
                value,
                count,
            })
            .collect();
        category_values.sort_by(|a, b| b.count.cmp(&a.count));

        let mut brand_values: Vec<FacetValue> = brand_counts
            .into_iter()
            .map(|(value, count)| FacetValue {
                selected: false,
                value,
                count,
            })
            .collect();
        brand_values.sort_by(|a, b| b.count.cmp(&a.count));

        Self {
            categories: Facet {
                name: "Category".to_string(),
                key: "category".to_string(),
                values: category_values,
            },
            price_ranges: Facet {
                name: "Price".to_string(),
                key: "price".to_string(),
                values: vec![
                    FacetValue { value: "Under $25".to_string(), count: 15, selected: false },
                    FacetValue { value: "$25 - $50".to_string(), count: 23, selected: false },
                    FacetValue { value: "$50 - $100".to_string(), count: 18, selected: false },
                    FacetValue { value: "$100 - $200".to_string(), count: 12, selected: false },
                    FacetValue { value: "Over $200".to_string(), count: 8, selected: false },
                ],
            },
            brands: Facet {
                name: "Brand".to_string(),
                key: "brand".to_string(),
                values: brand_values,
            },
            ratings: Facet {
                name: "Customer Rating".to_string(),
                key: "rating".to_string(),
                values: vec![
                    FacetValue { value: "4 Stars & Up".to_string(), count: 42, selected: false },
                    FacetValue { value: "3 Stars & Up".to_string(), count: 58, selected: false },
                    FacetValue { value: "2 Stars & Up".to_string(), count: 65, selected: false },
                ],
            },
        }
    }
}

/// Simple URL decoding.
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}
