//! Product hero section renderer.

use crate::data::Product;

/// Render the product hero section.
pub fn render_hero(product: &Product) -> String {
    let primary_image = product
        .images
        .iter()
        .find(|img| img.is_primary)
        .or_else(|| product.images.first());

    let image_html = if let Some(img) = primary_image {
        format!(
            r#"<img src="{}" alt="{}" class="product-image-main">"#,
            escape_html(&img.url),
            escape_html(&img.alt)
        )
    } else {
        r#"<div class="product-image-placeholder">No image available</div>"#.to_string()
    };

    let thumbnails: String = product
        .images
        .iter()
        .take(5)
        .map(|img| {
            format!(
                r#"<img src="{}" alt="{}" class="product-thumbnail">"#,
                escape_html(&img.url),
                escape_html(&img.alt)
            )
        })
        .collect();

    let attributes: String = product
        .attributes
        .iter()
        .map(|attr| {
            format!(
                r#"<li><strong>{}:</strong> {}</li>"#,
                escape_html(&attr.name),
                escape_html(&attr.value)
            )
        })
        .collect();

    format!(
        r#"<section class="product-hero" data-section="hero">
    <div class="product-gallery">
        {image_html}
        <div class="product-thumbnails">{thumbnails}</div>
    </div>
    <div class="product-info">
        <p class="product-brand">{brand}</p>
        <h1 class="product-name">{name}</h1>
        <p class="product-category">{category}</p>
        <div class="product-description">{description}</div>
        <ul class="product-attributes">{attributes}</ul>
    </div>
</section>"#,
        image_html = image_html,
        thumbnails = thumbnails,
        brand = escape_html(&product.brand),
        name = escape_html(&product.name),
        category = escape_html(&product.category),
        description = escape_html(&product.description),
        attributes = attributes
    )
}

/// Render hero fallback when product data fails to load.
pub fn render_hero_fallback() -> String {
    r#"<section class="product-hero product-hero--error" data-section="hero">
    <div class="error-message">
        <p>Unable to load product information. Please try again.</p>
    </div>
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
