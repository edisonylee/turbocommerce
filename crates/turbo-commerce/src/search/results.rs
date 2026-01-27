//! Search results and pagination.

use serde::{Deserialize, Serialize};

/// Pagination info.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pagination {
    /// Current page (1-indexed).
    pub page: i64,
    /// Items per page.
    pub per_page: i64,
    /// Total number of items.
    pub total: i64,
    /// Total number of pages.
    pub total_pages: i64,
    /// Whether there's a next page.
    pub has_next: bool,
    /// Whether there's a previous page.
    pub has_prev: bool,
}

impl Pagination {
    /// Create pagination info.
    pub fn new(page: i64, per_page: i64, total: i64) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }

    /// Get the offset for SQL queries.
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }

    /// Get page numbers for display (e.g., [1, 2, 3, ..., 10]).
    pub fn page_numbers(&self, max_visible: usize) -> Vec<i64> {
        if self.total_pages as usize <= max_visible {
            return (1..=self.total_pages).collect();
        }

        let half = max_visible / 2;
        let start = (self.page - half as i64).max(1);
        let end = (start + max_visible as i64 - 1).min(self.total_pages);
        let start = (end - max_visible as i64 + 1).max(1);

        (start..=end).collect()
    }

    /// Check if on first page.
    pub fn is_first(&self) -> bool {
        self.page == 1
    }

    /// Check if on last page.
    pub fn is_last(&self) -> bool {
        self.page >= self.total_pages
    }

    /// Get start item number (1-indexed).
    pub fn start_item(&self) -> i64 {
        if self.total == 0 {
            0
        } else {
            (self.page - 1) * self.per_page + 1
        }
    }

    /// Get end item number.
    pub fn end_item(&self) -> i64 {
        (self.page * self.per_page).min(self.total)
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(1, 24, 0)
    }
}

/// Search results container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults<T> {
    /// The result items.
    pub items: Vec<T>,
    /// Pagination info.
    pub pagination: Pagination,
    /// Query time in milliseconds.
    pub query_time_ms: i64,
    /// Facets (if requested).
    pub facets: Vec<Facet>,
}

impl<T> SearchResults<T> {
    /// Create new search results.
    pub fn new(items: Vec<T>, pagination: Pagination) -> Self {
        Self {
            items,
            pagination,
            query_time_ms: 0,
            facets: Vec::new(),
        }
    }

    /// Create empty results.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            pagination: Pagination::default(),
            query_time_ms: 0,
            facets: Vec::new(),
        }
    }

    /// Set query time.
    pub fn with_query_time(mut self, ms: i64) -> Self {
        self.query_time_ms = ms;
        self
    }

    /// Set facets.
    pub fn with_facets(mut self, facets: Vec<Facet>) -> Self {
        self.facets = facets;
        self
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get number of items in this page.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T: Default> Default for SearchResults<T> {
    fn default() -> Self {
        Self::empty()
    }
}

/// A facet for filtering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Facet {
    /// Facet name (e.g., "Category", "Price").
    pub name: String,
    /// Field this facet filters on.
    pub field: String,
    /// Type of facet.
    pub facet_type: FacetType,
    /// Facet values.
    pub values: Vec<FacetValue>,
}

impl Facet {
    /// Create a new terms facet.
    pub fn terms(name: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            field: field.into(),
            facet_type: FacetType::Terms,
            values: Vec::new(),
        }
    }

    /// Add a value to the facet.
    pub fn add_value(&mut self, value: impl Into<String>, count: i64, selected: bool) {
        self.values.push(FacetValue {
            value: value.into(),
            count,
            selected,
        });
    }
}

/// Type of facet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacetType {
    /// Discrete values (e.g., categories, colors).
    Terms,
    /// Numeric range (e.g., price).
    Range,
    /// Hierarchical (e.g., nested categories).
    Hierarchy,
}

/// A single facet value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetValue {
    /// The value.
    pub value: String,
    /// Number of items with this value.
    pub count: i64,
    /// Whether currently selected.
    pub selected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_basics() {
        let p = Pagination::new(2, 10, 45);
        assert_eq!(p.total_pages, 5);
        assert!(p.has_next);
        assert!(p.has_prev);
        assert_eq!(p.offset(), 10);
    }

    #[test]
    fn test_pagination_first_page() {
        let p = Pagination::new(1, 10, 45);
        assert!(!p.has_prev);
        assert!(p.has_next);
        assert!(p.is_first());
        assert!(!p.is_last());
    }

    #[test]
    fn test_pagination_last_page() {
        let p = Pagination::new(5, 10, 45);
        assert!(p.has_prev);
        assert!(!p.has_next);
        assert!(!p.is_first());
        assert!(p.is_last());
    }

    #[test]
    fn test_pagination_single_page() {
        let p = Pagination::new(1, 10, 5);
        assert_eq!(p.total_pages, 1);
        assert!(!p.has_next);
        assert!(!p.has_prev);
    }

    #[test]
    fn test_pagination_page_numbers() {
        let p = Pagination::new(5, 10, 100);
        let pages = p.page_numbers(5);
        assert_eq!(pages, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_pagination_item_range() {
        let p = Pagination::new(2, 10, 45);
        assert_eq!(p.start_item(), 11);
        assert_eq!(p.end_item(), 20);
    }

    #[test]
    fn test_search_results() {
        let items = vec![1, 2, 3];
        let pagination = Pagination::new(1, 10, 3);
        let results = SearchResults::new(items, pagination);

        assert_eq!(results.len(), 3);
        assert!(!results.is_empty());
    }
}
