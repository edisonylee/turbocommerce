//! Facets section - filter sidebar.

use crate::data::{SearchFacets, SearchQuery};

/// Render the facets sidebar section.
pub fn render_facets(facets: &SearchFacets, query: &SearchQuery) -> String {
    let base_url = build_base_url(query);

    let categories_html = render_facet_group(&facets.categories, &base_url);
    let price_html = render_facet_group(&facets.price_ranges, &base_url);
    let brands_html = render_facet_group(&facets.brands, &base_url);
    let ratings_html = render_facet_group(&facets.ratings, &base_url);

    let active_filters = render_active_filters(query);

    format!(
        r#"<aside class="facets-sidebar" data-section="facets">
    <div class="facets-header">
        <h2>Filters</h2>
        {active_filters}
    </div>

    {categories_html}
    {price_html}
    {brands_html}
    {ratings_html}
</aside>"#
    )
}

fn render_facet_group(facet: &crate::data::Facet, base_url: &str) -> String {
    if facet.values.is_empty() {
        return String::new();
    }

    let values_html: String = facet
        .values
        .iter()
        .take(10) // Show top 10
        .map(|v| {
            let checked = if v.selected { " checked" } else { "" };
            let _url = format!("{}{}={}", base_url, facet.key, urlencoding_encode(&v.value));
            format!(
                r#"<label class="facet-option">
            <input type="checkbox" name="{}" value="{}"{} onchange="applyFilter('{}', '{}')">
            <span class="facet-label">{}</span>
            <span class="facet-count">({})</span>
        </label>"#,
                facet.key,
                html_escape(&v.value),
                checked,
                facet.key,
                html_escape(&v.value),
                html_escape(&v.value),
                v.count
            )
        })
        .collect();

    let show_more = if facet.values.len() > 10 {
        format!(
            r#"<button class="show-more" onclick="toggleFacet('{}')">Show {} more</button>"#,
            facet.key,
            facet.values.len() - 10
        )
    } else {
        String::new()
    };

    format!(
        r#"<div class="facet-group" data-facet="{}">
        <h3 class="facet-title">{}</h3>
        <div class="facet-options">
            {}
        </div>
        {}
    </div>"#,
        facet.key, facet.name, values_html, show_more
    )
}

fn render_active_filters(query: &SearchQuery) -> String {
    let mut filters = Vec::new();

    if let Some(cat) = &query.category {
        filters.push(format!(
            r#"<span class="active-filter">
                Category: {}
                <button onclick="removeFilter('category')" aria-label="Remove filter">&times;</button>
            </span>"#,
            html_escape(cat)
        ));
    }

    if let Some(min) = query.min_price {
        filters.push(format!(
            r#"<span class="active-filter">
                Min: ${:.2}
                <button onclick="removeFilter('min_price')" aria-label="Remove filter">&times;</button>
            </span>"#,
            min
        ));
    }

    if let Some(max) = query.max_price {
        filters.push(format!(
            r#"<span class="active-filter">
                Max: ${:.2}
                <button onclick="removeFilter('max_price')" aria-label="Remove filter">&times;</button>
            </span>"#,
            max
        ));
    }

    if filters.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="active-filters">
                {}
                <button class="clear-all" onclick="clearAllFilters()">Clear all</button>
            </div>"#,
            filters.join("\n")
        )
    }
}

fn build_base_url(query: &SearchQuery) -> String {
    let mut url = format!("/search?q={}", urlencoding_encode(&query.q));
    if query.page > 1 {
        url.push_str(&format!("&page={}", query.page));
    }
    url.push('&');
    url
}

/// Render skeleton placeholder for facets.
pub fn render_facets_skeleton() -> String {
    r#"<aside class="facets-sidebar skeleton" data-section="facets">
    <div class="skeleton-facet-group">
        <div class="skeleton-text skeleton-title"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text"></div>
    </div>
    <div class="skeleton-facet-group">
        <div class="skeleton-text skeleton-title"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-text"></div>
    </div>
</aside>"#
        .to_string()
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
