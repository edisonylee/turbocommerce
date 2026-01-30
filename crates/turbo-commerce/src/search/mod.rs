//! Search module.
//!
//! Contains types for faceted search, filters, and pagination.

mod filter;
mod query;
mod results;

pub use filter::Filter;
pub use query::{SearchQuery, SortOption};
pub use results::{Pagination, SearchResults};
