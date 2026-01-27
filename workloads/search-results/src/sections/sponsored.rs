//! Sponsored results section.

use crate::data::SponsoredProduct;

/// Render sponsored products section.
pub fn render_sponsored(products: &[SponsoredProduct]) -> String {
    if products.is_empty() {
        return String::new();
    }

    let products_html: String = products
        .iter()
        .map(|sp| render_sponsored_card(sp))
        .collect();

    format!(
        r#"<section class="sponsored-results" data-section="sponsored">
    <div class="sponsored-header">
        <span class="sponsored-label">Sponsored</span>
    </div>
    <div class="sponsored-grid">
        {}
    </div>
</section>"#,
        products_html
    )
}

fn render_sponsored_card(sp: &SponsoredProduct) -> String {
    let p = &sp.product;

    format!(
        r#"<article class="sponsored-card" data-ad-id="{}" data-impression-url="{}">
    <span class="ad-badge">Ad</span>
    <a href="/product/{}?ref=sponsored&ad_id={}" class="product-link">
        <div class="product-image">
            <img src="{}" alt="{}" loading="lazy">
        </div>
        <div class="product-info">
            <h3 class="product-title">{}</h3>
            <div class="product-price">${:.2}</div>
        </div>
    </a>
</article>"#,
        html_escape(&sp.ad_id),
        html_escape(&sp.impression_url),
        p.id,
        html_escape(&sp.ad_id),
        html_escape(&p.thumbnail),
        html_escape(&p.title),
        html_escape(&p.title),
        p.price
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
