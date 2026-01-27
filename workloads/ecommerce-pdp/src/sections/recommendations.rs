//! Recommendations section renderer.

use crate::data::RecommendedProduct;

/// Render the recommendations section.
pub fn render_recommendations(products: &[RecommendedProduct]) -> String {
    if products.is_empty() {
        return render_recommendations_empty();
    }

    let items: String = products
        .iter()
        .take(4)
        .map(render_recommendation_card)
        .collect();

    format!(
        r#"<section class="product-recommendations" data-section="recommendations">
    <h2>You May Also Like</h2>
    <div class="recommendations-grid">
        {items}
    </div>
</section>"#,
        items = items
    )
}

fn render_recommendation_card(product: &RecommendedProduct) -> String {
    format!(
        r#"<article class="recommendation-card">
        <a href="/product/{id}" class="recommendation-link">
            <img src="{image}" alt="{name}" class="recommendation-image">
            <div class="recommendation-info">
                <h3 class="recommendation-name">{name}</h3>
                <p class="recommendation-price">${price:.2}</p>
                <p class="recommendation-reason">{reason}</p>
            </div>
        </a>
    </article>"#,
        id = escape_html(&product.id),
        image = escape_html(&product.image_url),
        name = escape_html(&product.name),
        price = product.price,
        reason = escape_html(&product.reason)
    )
}

fn render_recommendations_empty() -> String {
    r#"<section class="product-recommendations product-recommendations--empty" data-section="recommendations">
    <h2>You May Also Like</h2>
    <p class="recommendations-empty">No recommendations available.</p>
</section>"#
        .to_string()
}

/// Render recommendations fallback.
pub fn render_recommendations_fallback() -> String {
    r#"<section class="product-recommendations product-recommendations--fallback" data-section="recommendations">
    <h2>You May Also Like</h2>
    <p class="recommendations-loading">Loading recommendations...</p>
</section>"#
        .to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
