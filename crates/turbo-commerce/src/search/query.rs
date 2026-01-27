//! Search query builder.

use crate::search::Filter;
use serde::{Deserialize, Serialize};

/// Sort options for search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortOption {
    /// Sort by relevance (default for text search).
    #[default]
    Relevance,
    /// Sort by price, low to high.
    PriceAsc,
    /// Sort by price, high to low.
    PriceDesc,
    /// Sort by name A-Z.
    NameAsc,
    /// Sort by name Z-A.
    NameDesc,
    /// Sort by newest first.
    Newest,
    /// Sort by oldest first.
    Oldest,
    /// Sort by best selling.
    BestSelling,
    /// Sort by highest rated.
    Rating,
    /// Sort by position/manual order.
    Position,
}

impl SortOption {
    /// Get SQL ORDER BY clause.
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortOption::Relevance => "created_at DESC", // Fallback for non-text search
            SortOption::PriceAsc => "price_cents ASC",
            SortOption::PriceDesc => "price_cents DESC",
            SortOption::NameAsc => "name ASC",
            SortOption::NameDesc => "name DESC",
            SortOption::Newest => "created_at DESC",
            SortOption::Oldest => "created_at ASC",
            SortOption::BestSelling => "sales_count DESC",
            SortOption::Rating => "average_rating DESC",
            SortOption::Position => "position ASC",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SortOption::Relevance => "Relevance",
            SortOption::PriceAsc => "Price: Low to High",
            SortOption::PriceDesc => "Price: High to Low",
            SortOption::NameAsc => "Name: A-Z",
            SortOption::NameDesc => "Name: Z-A",
            SortOption::Newest => "Newest",
            SortOption::Oldest => "Oldest",
            SortOption::BestSelling => "Best Selling",
            SortOption::Rating => "Highest Rated",
            SortOption::Position => "Featured",
        }
    }
}

/// A search query.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchQuery {
    /// Text query (for full-text search).
    pub query: Option<String>,
    /// Filters to apply.
    pub filters: Vec<Filter>,
    /// Sort option.
    pub sort: SortOption,
    /// Current page (1-indexed).
    pub page: i64,
    /// Items per page.
    pub per_page: i64,
    /// Whether to include facets in results.
    pub include_facets: bool,
}

impl SearchQuery {
    /// Create a new search query.
    pub fn new() -> Self {
        Self {
            query: None,
            filters: Vec::new(),
            sort: SortOption::Relevance,
            page: 1,
            per_page: 24,
            include_facets: false,
        }
    }

    /// Set the text query.
    pub fn with_query(mut self, q: impl Into<String>) -> Self {
        let q = q.into();
        if !q.is_empty() {
            self.query = Some(q.clone());
            self.filters.push(Filter::Text(q));
        }
        self
    }

    /// Add a filter.
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set sort option.
    pub fn with_sort(mut self, sort: SortOption) -> Self {
        self.sort = sort;
        self
    }

    /// Set pagination.
    pub fn with_pagination(mut self, page: i64, per_page: i64) -> Self {
        self.page = page.max(1);
        self.per_page = per_page.clamp(1, 100);
        self
    }

    /// Enable facets.
    pub fn with_facets(mut self) -> Self {
        self.include_facets = true;
        self
    }

    /// Calculate offset for SQL LIMIT/OFFSET.
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }

    /// Build SQL WHERE clause from filters.
    pub fn build_where_clause(&self) -> (String, Vec<String>) {
        if self.filters.is_empty() {
            return ("1=1".to_string(), vec![]);
        }

        let mut clauses = Vec::new();
        let mut all_values = Vec::new();

        for filter in &self.filters {
            let (clause, values) = filter.to_sql();
            if !clause.is_empty() && clause != "1=1" {
                clauses.push(format!("({})", clause));
                all_values.extend(values);
            }
        }

        if clauses.is_empty() {
            return ("1=1".to_string(), vec![]);
        }

        (clauses.join(" AND "), all_values)
    }

    /// Build full SQL query for products.
    pub fn build_sql(&self) -> (String, Vec<String>) {
        let (where_clause, values) = self.build_where_clause();
        let order_by = self.sort.to_sql();

        let sql = format!(
            "SELECT * FROM products WHERE {} ORDER BY {} LIMIT {} OFFSET {}",
            where_clause,
            order_by,
            self.per_page,
            self.offset()
        );

        (sql, values)
    }

    /// Build count SQL query.
    pub fn build_count_sql(&self) -> (String, Vec<String>) {
        let (where_clause, values) = self.build_where_clause();
        let sql = format!("SELECT COUNT(*) as count FROM products WHERE {}", where_clause);
        (sql, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::{Currency, Money};

    #[test]
    fn test_query_builder() {
        let query = SearchQuery::new()
            .with_query("rust")
            .with_filter(Filter::in_stock())
            .with_sort(SortOption::PriceAsc)
            .with_pagination(2, 10);

        assert_eq!(query.page, 2);
        assert_eq!(query.per_page, 10);
        assert_eq!(query.offset(), 10);
        assert_eq!(query.sort, SortOption::PriceAsc);
    }

    #[test]
    fn test_where_clause() {
        let query = SearchQuery::new()
            .with_filter(Filter::in_stock())
            .with_filter(Filter::price_range(
                Some(Money::new(1000, Currency::USD)),
                None,
            ));

        let (clause, _values) = query.build_where_clause();
        assert!(clause.contains("quantity > 0"));
        assert!(clause.contains("price_cents >="));
    }

    #[test]
    fn test_full_sql() {
        let query = SearchQuery::new()
            .with_filter(Filter::Status("active".to_string()))
            .with_sort(SortOption::Newest)
            .with_pagination(1, 24);

        let (sql, values) = query.build_sql();
        assert!(sql.contains("SELECT * FROM products"));
        assert!(sql.contains("status = ?"));
        assert!(sql.contains("ORDER BY created_at DESC"));
        assert!(sql.contains("LIMIT 24 OFFSET 0"));
        assert_eq!(values.len(), 1);
    }
}
