//! Results section - product grid with pagination.

use crate::data::{SearchProduct, SearchQuery};

/// Render the search results grid section.
pub fn render_results(products: &[SearchProduct], query: &SearchQuery, total: u32) -> String {
    let products_html: String = products
        .iter()
        .map(|p| render_product_card(p))
        .collect();

    let pagination = render_pagination(query, total);

    format!(
        r#"<section class="search-results" data-section="results">
    <div class="product-grid">
        {}
    </div>
    {}
</section>"#,
        products_html, pagination
    )
}

fn render_product_card(product: &SearchProduct) -> String {
    let stars = render_stars(product.rating);
    let stock_class = if product.stock > 10 {
        "in-stock"
    } else if product.stock > 0 {
        "low-stock"
    } else {
        "out-of-stock"
    };

    let stock_text = if product.stock > 10 {
        "In Stock".to_string()
    } else if product.stock > 0 {
        format!("Only {} left", product.stock)
    } else {
        "Out of Stock".to_string()
    };

    format!(
        r#"<article class="product-card" data-product-id="{}">
    <a href="/product/{}" class="product-link">
        <div class="product-image">
            <img src="{}" alt="{}" loading="lazy">
        </div>
        <div class="product-info">
            <h3 class="product-title">{}</h3>
            <div class="product-rating">
                {}
                <span class="rating-value">{:.1}</span>
            </div>
            <div class="product-price">${:.2}</div>
            <div class="product-stock {}">
                {}
            </div>
        </div>
    </a>
    <button class="add-to-cart" data-product-id="{}" {}>
        Add to Cart
    </button>
</article>"#,
        product.id,
        product.id,
        html_escape(&product.thumbnail),
        html_escape(&product.title),
        html_escape(&product.title),
        stars,
        product.rating,
        product.price,
        stock_class,
        stock_text,
        product.id,
        if product.stock == 0 { "disabled" } else { "" }
    )
}

fn render_stars(rating: f64) -> String {
    let full_stars = rating.floor() as u32;
    let has_half = rating.fract() >= 0.5;
    let empty_stars = 5 - full_stars - if has_half { 1 } else { 0 };

    let mut html = String::from(r#"<span class="stars">"#);

    for _ in 0..full_stars {
        html.push_str(r#"<span class="star full">★</span>"#);
    }
    if has_half {
        html.push_str(r#"<span class="star half">★</span>"#);
    }
    for _ in 0..empty_stars {
        html.push_str(r#"<span class="star empty">☆</span>"#);
    }

    html.push_str("</span>");
    html
}

fn render_pagination(query: &SearchQuery, total: u32) -> String {
    let total_pages = (total + query.per_page - 1) / query.per_page;

    if total_pages <= 1 {
        return String::new();
    }

    let current = query.page;
    let mut pages = Vec::new();

    // Always show first page
    pages.push(1);

    // Show pages around current
    let start = current.saturating_sub(2).max(2);
    let end = (current + 2).min(total_pages - 1);

    if start > 2 {
        pages.push(0); // Ellipsis marker
    }

    for p in start..=end {
        if p > 1 && p < total_pages {
            pages.push(p);
        }
    }

    if end < total_pages - 1 {
        pages.push(0); // Ellipsis marker
    }

    // Always show last page
    if total_pages > 1 {
        pages.push(total_pages);
    }

    let base_url = format!(
        "/search?q={}&sort={}",
        urlencoding_encode(&query.q),
        query.sort.as_str()
    );

    let pages_html: String = pages
        .iter()
        .map(|&p| {
            if p == 0 {
                r#"<span class="pagination-ellipsis">...</span>"#.to_string()
            } else if p == current {
                format!(
                    r#"<span class="pagination-page current" aria-current="page">{}</span>"#,
                    p
                )
            } else {
                format!(
                    r#"<a href="{}&page={}" class="pagination-page">{}</a>"#,
                    base_url, p, p
                )
            }
        })
        .collect();

    let prev_link = if current > 1 {
        format!(
            r#"<a href="{}&page={}" class="pagination-prev" aria-label="Previous page">&larr; Prev</a>"#,
            base_url,
            current - 1
        )
    } else {
        r#"<span class="pagination-prev disabled">&larr; Prev</span>"#.to_string()
    };

    let next_link = if current < total_pages {
        format!(
            r#"<a href="{}&page={}" class="pagination-next" aria-label="Next page">Next &rarr;</a>"#,
            base_url,
            current + 1
        )
    } else {
        r#"<span class="pagination-next disabled">Next &rarr;</span>"#.to_string()
    };

    format!(
        r#"<nav class="pagination" aria-label="Search results pagination">
    {}
    <div class="pagination-pages">
        {}
    </div>
    {}
</nav>"#,
        prev_link, pages_html, next_link
    )
}

/// Render skeleton placeholder for results.
pub fn render_results_skeleton() -> String {
    let cards: String = (0..8)
        .map(|_| {
            r#"<div class="product-card skeleton">
        <div class="skeleton-image"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text short"></div>
        <div class="skeleton-text short"></div>
    </div>"#
        })
        .collect();

    format!(
        r#"<section class="search-results skeleton" data-section="results">
    <div class="product-grid">
        {}
    </div>
</section>"#,
        cards
    )
}

/// Render error state for results.
pub fn render_results_error(message: &str) -> String {
    format!(
        r#"<section class="search-results error" data-section="results">
    <div class="error-state">
        <svg class="error-icon" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <h2>Unable to load results</h2>
        <p>{}</p>
        <button onclick="location.reload()">Try Again</button>
    </div>
</section>"#,
        html_escape(message)
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn urlencoding_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
