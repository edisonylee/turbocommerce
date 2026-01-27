//! Search module.
//!
//! Contains types for faceted search, filters, and pagination.

mod query;
mod filter;
mod results;

pub use query::{SearchQuery, SortOption};
pub use filter::Filter;
pub use results::{SearchResults, Pagination};
