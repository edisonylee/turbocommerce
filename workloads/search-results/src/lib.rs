//! Search Results Page - Reference workload demonstrating query-driven streaming SSR.
//!
//! This workload demonstrates:
//! - Query-driven cache keys (vary by search query, page, sort)
//! - Faceted navigation with filter sidebar
//! - Pagination with streaming results
//! - Sponsored results with separate cache policy

mod data;
mod sections;

use spin_sdk::http::{Fields, IncomingRequest, Method, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

use edge_sdk::edge_core::RequestContext;
use edge_sdk::edge_streaming::{HeadContent, Shell, StreamingSink};

use data::{SearchFacets, SearchProduct, SearchQuery, SearchResults, SponsoredProduct};
use sections::{
    render_facets, render_facets_skeleton, render_results, render_results_error,
    render_search_header, render_sponsored,
};

/// Search Results page handler.
#[http_component]
async fn handle_search(req: IncomingRequest, response_out: ResponseOutparam) {
    // Only handle GET requests
    if req.method() != Method::Get {
        let headers = Fields::from_list(&[]).unwrap();
        let response = OutgoingResponse::new(headers);
        response.set_status_code(405).unwrap();
        response_out.set(response);
        return;
    }

    // Parse query parameters
    let path_with_query = req.path_with_query().unwrap_or_default();
    let query_string = path_with_query
        .split('?')
        .nth(1)
        .unwrap_or("");
    let query = SearchQuery::from_query_string(query_string);

    // Create platform context
    let ctx = RequestContext::new(
        edge_sdk::edge_core::Method::Get,
        path_with_query.clone(),
    );

    // Build response headers
    let header_list: Vec<(String, Vec<u8>)> = vec![
        ("content-type".to_owned(), "text/html; charset=utf-8".into()),
        ("x-request-id".to_owned(), ctx.request_id.to_string().into()),
        ("x-cache-key".to_owned(), query.cache_key().into()),
    ];

    let headers = Fields::from_list(&header_list).unwrap();
    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);
    let mut sink = StreamingSink::new(body, ctx.timing.clone());

    // Create shell with search page styling
    let shell = create_shell(&query);

    // Send shell first (streaming SSR)
    if let Err(e) = sink.send_shell(&shell.render_opening()).await {
        eprintln!("Failed to send shell: {}", e);
        return;
    }

    // Create layout container
    if let Err(e) = sink.send_section("layout-start", r#"<div class="search-layout"><div class="search-main">"#).await {
        eprintln!("Failed to send layout: {}", e);
        return;
    }

    // Fetch data concurrently
    let search_future = fetch_search_results(&query);
    let sponsored_future = fetch_sponsored(&query);

    // Wait for search results
    let (search_result, sponsored_result) = futures::join!(search_future, sponsored_future);

    match &search_result {
        Ok(results) => {
            // Render header with results count
            let _ = sink.send_section("search-header", &render_search_header(&query, results.total)).await;

            // Render facets
            let facets = SearchFacets::from_results(&results.products, &query);
            let _ = sink.send_section("facets", &render_facets(&facets, &query)).await;

            // Render results grid
            let _ = sink.send_section("results", &render_results(&results.products, &query, results.total)).await;
        }
        Err(e) => {
            // Render error state
            let _ = sink.send_section("search-header", &render_search_header(&query, 0)).await;
            let _ = sink.send_section("facets", &render_facets_skeleton()).await;
            let _ = sink.send_section("results", &render_results_error(&e.to_string())).await;
        }
    }

    // Close main content area and open sidebar
    let _ = sink.send_section("layout-mid", r#"</div><div class="search-sidebar">"#).await;

    // Render sponsored (in sidebar)
    if let Ok(sponsored) = sponsored_result {
        let _ = sink.send_section("sponsored", &render_sponsored(&sponsored)).await;
    }

    // Close layout
    let _ = sink.send_section("layout-end", r#"</div></div>"#).await;

    // Send closing shell with JavaScript
    let closing = format!("{}\n{}", shell.render_closing(), search_page_scripts());
    let _ = sink.send_section("closing", &closing).await;
}

/// Create shell for search page.
fn create_shell(query: &SearchQuery) -> Shell {
    let title = if query.q.is_empty() {
        "All Products - Search".to_string()
    } else {
        format!("{} - Search Results", query.q)
    };

    let head = HeadContent::new(title)
        .with_meta("viewport", "width=device-width, initial-scale=1")
        .with_meta("description", "Search results for products")
        .with_style(SEARCH_STYLES);

    Shell::new(head).with_body_start(
        r#"<body>
<header class="site-header">
    <a href="/" class="logo">EdgeStore</a>
    <form action="/search" method="GET" class="search-form">
        <input type="search" name="q" placeholder="Search products..." aria-label="Search">
        <button type="submit">Search</button>
    </form>
    <nav class="header-nav">
        <a href="/cart">Cart</a>
        <a href="/account">Account</a>
    </nav>
</header>
<main>
"#,
    )
}

/// Fetch search results from API.
async fn fetch_search_results(query: &SearchQuery) -> anyhow::Result<SearchResults> {
    let skip = (query.page - 1) * query.per_page;

    let url = if query.q.is_empty() {
        format!(
            "https://dummyjson.com/products?limit={}&skip={}",
            query.per_page, skip
        )
    } else {
        format!(
            "https://dummyjson.com/products/search?q={}&limit={}&skip={}",
            urlencoding_encode(&query.q),
            query.per_page,
            skip
        )
    };

    let req = spin_sdk::http::Request::builder()
        .method(Method::Get)
        .uri(&url)
        .header("accept", "application/json")
        .build();

    let resp: spin_sdk::http::Response = spin_sdk::http::send(req).await?;

    if *resp.status() != 200 {
        anyhow::bail!("Search API returned status {}", resp.status());
    }

    let body = resp.into_body();
    let results: SearchResults = serde_json::from_slice(&body)?;

    Ok(results)
}

/// Fetch sponsored products.
async fn fetch_sponsored(query: &SearchQuery) -> anyhow::Result<Vec<SponsoredProduct>> {
    // In a real implementation, this would call an ads service
    // For demo, we fetch some products and mark them as sponsored

    let url = format!(
        "https://dummyjson.com/products?limit=3&skip={}",
        (query.q.len() % 10) * 3 // Pseudo-random offset based on query
    );

    let req = spin_sdk::http::Request::builder()
        .method(Method::Get)
        .uri(&url)
        .header("accept", "application/json")
        .build();

    let resp: spin_sdk::http::Response = spin_sdk::http::send(req).await?;

    if *resp.status() != 200 {
        return Ok(Vec::new()); // Sponsored is non-critical, fail silently
    }

    let body = resp.into_body();

    #[derive(serde::Deserialize)]
    struct ProductsResponse {
        products: Vec<SearchProduct>,
    }

    let response: ProductsResponse = serde_json::from_slice(&body)?;

    let sponsored: Vec<SponsoredProduct> = response
        .products
        .into_iter()
        .enumerate()
        .map(|(i, p)| SponsoredProduct::from_product(p, &format!("ad-{}-{}", query.q.len(), i)))
        .collect();

    Ok(sponsored)
}

fn search_page_scripts() -> String {
    r#"<script>
function updateSort(value) {
    const url = new URL(window.location);
    url.searchParams.set('sort', value);
    url.searchParams.delete('page');
    window.location = url;
}

function applyFilter(key, value) {
    const url = new URL(window.location);
    url.searchParams.set(key, value);
    url.searchParams.delete('page');
    window.location = url;
}

function removeFilter(key) {
    const url = new URL(window.location);
    url.searchParams.delete(key);
    url.searchParams.delete('page');
    window.location = url;
}

function clearAllFilters() {
    const url = new URL(window.location);
    const q = url.searchParams.get('q');
    url.search = q ? '?q=' + encodeURIComponent(q) : '';
    window.location = url;
}

function toggleFacet(key) {
    const group = document.querySelector(`[data-facet="${key}"]`);
    if (group) {
        group.classList.toggle('expanded');
    }
}

// Track sponsored impressions
document.querySelectorAll('[data-impression-url]').forEach(el => {
    const url = el.dataset.impressionUrl;
    if (url) {
        navigator.sendBeacon(url);
    }
});

// View toggle
document.querySelectorAll('.view-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.view-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        const view = btn.dataset.view;
        document.querySelector('.product-grid')?.classList.toggle('list-view', view === 'list');
    });
});
</script>"#
        .to_string()
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

const SEARCH_STYLES: &str = r##"
:root {
    --primary: #2563eb;
    --primary-hover: #1d4ed8;
    --bg: #f8fafc;
    --card-bg: #ffffff;
    --text: #1e293b;
    --text-muted: #64748b;
    --border: #e2e8f0;
    --success: #22c55e;
    --warning: #f59e0b;
    --error: #ef4444;
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    line-height: 1.5;
}

.site-header {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 1rem 2rem;
    background: var(--card-bg);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 100;
}

.logo {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--primary);
    text-decoration: none;
}

.search-form {
    display: flex;
    flex: 1;
    max-width: 600px;
}

.search-form input {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px 0 0 8px;
    font-size: 1rem;
}

.search-form button {
    padding: 0.75rem 1.5rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 0 8px 8px 0;
    cursor: pointer;
    font-weight: 500;
}

.header-nav {
    display: flex;
    gap: 1.5rem;
}

.header-nav a {
    color: var(--text);
    text-decoration: none;
}

main {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
}

.search-layout {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 2rem;
}

.search-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
}

.search-info h1 {
    font-size: 1.5rem;
    margin-bottom: 0.25rem;
}

.result-count {
    color: var(--text-muted);
}

.search-controls {
    display: flex;
    gap: 1rem;
    align-items: center;
}

.sort-control {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.sort-control select {
    padding: 0.5rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--card-bg);
}

.view-control {
    display: flex;
    gap: 0.25rem;
}

.view-btn {
    padding: 0.5rem;
    border: 1px solid var(--border);
    background: var(--card-bg);
    cursor: pointer;
    border-radius: 4px;
}

.view-btn.active {
    background: var(--primary);
    color: white;
    border-color: var(--primary);
}

/* Facets Sidebar */
.facets-sidebar {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 1.5rem;
    height: fit-content;
    position: sticky;
    top: 100px;
}

.facets-header {
    margin-bottom: 1.5rem;
}

.facets-header h2 {
    font-size: 1.125rem;
}

.active-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.75rem;
}

.active-filter {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    background: var(--primary);
    color: white;
    border-radius: 4px;
    font-size: 0.875rem;
}

.active-filter button {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 1rem;
    padding: 0;
}

.clear-all {
    background: none;
    border: none;
    color: var(--primary);
    cursor: pointer;
    font-size: 0.875rem;
}

.facet-group {
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
}

.facet-group:last-child {
    border-bottom: none;
    margin-bottom: 0;
}

.facet-title {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
}

.facet-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0;
    cursor: pointer;
    font-size: 0.9375rem;
}

.facet-count {
    color: var(--text-muted);
    font-size: 0.875rem;
}

.show-more {
    background: none;
    border: none;
    color: var(--primary);
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.5rem 0;
}

/* Product Grid */
.product-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 1.5rem;
}

.product-grid.list-view {
    grid-template-columns: 1fr;
}

.product-card {
    background: var(--card-bg);
    border-radius: 12px;
    overflow: hidden;
    transition: box-shadow 0.2s;
}

.product-card:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.product-link {
    text-decoration: none;
    color: inherit;
    display: block;
}

.product-image {
    aspect-ratio: 1;
    overflow: hidden;
    background: #f1f5f9;
}

.product-image img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    transition: transform 0.2s;
}

.product-card:hover .product-image img {
    transform: scale(1.05);
}

.product-info {
    padding: 1rem;
}

.product-title {
    font-size: 1rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.product-rating {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
}

.stars { color: #f59e0b; }
.star.empty { color: #e2e8f0; }
.rating-value { color: var(--text-muted); font-size: 0.875rem; }

.product-price {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text);
    margin-bottom: 0.5rem;
}

.product-stock {
    font-size: 0.875rem;
}

.product-stock.in-stock { color: var(--success); }
.product-stock.low-stock { color: var(--warning); }
.product-stock.out-of-stock { color: var(--error); }

.add-to-cart {
    width: calc(100% - 2rem);
    margin: 0 1rem 1rem;
    padding: 0.75rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
}

.add-to-cart:hover:not(:disabled) {
    background: var(--primary-hover);
}

.add-to-cart:disabled {
    background: var(--border);
    cursor: not-allowed;
}

/* Pagination */
.pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.5rem;
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 1px solid var(--border);
}

.pagination-page, .pagination-prev, .pagination-next {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    text-decoration: none;
    color: var(--text);
    background: var(--card-bg);
}

.pagination-page.current {
    background: var(--primary);
    color: white;
    border-color: var(--primary);
}

.pagination-page:hover:not(.current),
.pagination-prev:hover:not(.disabled),
.pagination-next:hover:not(.disabled) {
    background: var(--bg);
}

.pagination-ellipsis {
    padding: 0.5rem;
    color: var(--text-muted);
}

.disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* Sponsored */
.search-sidebar {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.sponsored-results {
    background: var(--card-bg);
    border-radius: 12px;
    padding: 1rem;
}

.sponsored-header {
    margin-bottom: 1rem;
}

.sponsored-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-muted);
}

.sponsored-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.sponsored-card {
    position: relative;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
}

.ad-badge {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.125rem 0.375rem;
    background: var(--text-muted);
    color: white;
    font-size: 0.625rem;
    border-radius: 4px;
    text-transform: uppercase;
}

.sponsored-card .product-image {
    aspect-ratio: 1;
    max-width: 80px;
    float: left;
    margin-right: 1rem;
}

.sponsored-card .product-title {
    font-size: 0.875rem;
}

.sponsored-card .product-price {
    font-size: 1rem;
}

/* Skeleton loading */
.skeleton .skeleton-text {
    height: 1rem;
    background: linear-gradient(90deg, #e2e8f0 25%, #f1f5f9 50%, #e2e8f0 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 4px;
    margin-bottom: 0.5rem;
}

.skeleton .skeleton-title { width: 60%; height: 1.5rem; }
.skeleton .skeleton-count { width: 40%; }
.skeleton .skeleton-select { width: 120px; height: 2.5rem; }
.skeleton .skeleton-image { aspect-ratio: 1; background: #e2e8f0; }

@keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
}

/* Error state */
.error-state {
    text-align: center;
    padding: 4rem 2rem;
}

.error-icon {
    color: var(--error);
    margin-bottom: 1rem;
}

.error-state h2 {
    margin-bottom: 0.5rem;
}

.error-state p {
    color: var(--text-muted);
    margin-bottom: 1.5rem;
}

.error-state button {
    padding: 0.75rem 2rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
}

/* Responsive */
@media (max-width: 1024px) {
    .search-layout {
        grid-template-columns: 1fr;
    }

    .search-sidebar {
        order: -1;
    }

    .facets-sidebar {
        position: static;
    }
}

@media (max-width: 640px) {
    .site-header {
        flex-wrap: wrap;
        gap: 1rem;
        padding: 1rem;
    }

    .search-form {
        order: 3;
        flex-basis: 100%;
    }

    .search-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 1rem;
    }

    .product-grid {
        grid-template-columns: repeat(2, 1fr);
        gap: 1rem;
    }
}
"##;
