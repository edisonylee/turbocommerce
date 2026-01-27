//! Search header section - query info and result count.

use crate::data::{SearchQuery, SortOrder};

/// Render the search header section.
pub fn render_search_header(query: &SearchQuery, total_results: u32) -> String {
    let sort_options = [
        SortOrder::Relevance,
        SortOrder::PriceLowToHigh,
        SortOrder::PriceHighToLow,
        SortOrder::Rating,
        SortOrder::Newest,
    ];

    let sort_html: String = sort_options
        .iter()
        .map(|opt| {
            let selected = if std::mem::discriminant(opt) == std::mem::discriminant(&query.sort) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                opt.as_str(),
                selected,
                opt.display_name()
            )
        })
        .collect();

    let query_display = if query.q.is_empty() {
        "All Products".to_string()
    } else {
        format!("\"{}\"", html_escape(&query.q))
    };

    let result_text = if total_results == 1 {
        "1 result".to_string()
    } else {
        format!("{} results", total_results)
    };

    format!(
        r#"<section class="search-header" data-section="search-header">
    <div class="search-info">
        <h1>Search Results for {}</h1>
        <p class="result-count">{}</p>
    </div>
    <div class="search-controls">
        <div class="sort-control">
            <label for="sort">Sort by:</label>
            <select id="sort" name="sort" onchange="updateSort(this.value)">
                {}
            </select>
        </div>
        <div class="view-control">
            <button class="view-btn active" data-view="grid" aria-label="Grid view">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <rect x="1" y="1" width="6" height="6"/>
                    <rect x="9" y="1" width="6" height="6"/>
                    <rect x="1" y="9" width="6" height="6"/>
                    <rect x="9" y="9" width="6" height="6"/>
                </svg>
            </button>
            <button class="view-btn" data-view="list" aria-label="List view">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <rect x="1" y="1" width="14" height="3"/>
                    <rect x="1" y="6" width="14" height="3"/>
                    <rect x="1" y="11" width="14" height="3"/>
                </svg>
            </button>
        </div>
    </div>
</section>"#,
        query_display, result_text, sort_html
    )
}

/// Render skeleton placeholder for header.
pub fn render_header_skeleton() -> String {
    r#"<section class="search-header skeleton" data-section="search-header">
    <div class="search-info">
        <div class="skeleton-text skeleton-title"></div>
        <div class="skeleton-text skeleton-count"></div>
    </div>
    <div class="search-controls">
        <div class="skeleton-select"></div>
    </div>
</section>"#
        .to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
