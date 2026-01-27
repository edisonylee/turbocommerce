//! Pricing section renderer.

use crate::data::Pricing;

/// Render the pricing section.
pub fn render_pricing(pricing: &Pricing) -> String {
    let current_price = pricing.format_price(pricing.price);

    let sale_info = if pricing.is_on_sale() {
        let original = pricing.format_price(pricing.original_price.unwrap());
        let discount = pricing.discount_percentage.unwrap();
        format!(
            r#"<span class="price-original">{}</span>
            <span class="price-discount">-{}% OFF</span>"#,
            original, discount
        )
    } else {
        String::new()
    };

    let member_price = if let Some(mp) = pricing.member_price {
        format!(
            r#"<div class="member-price">
                <span class="member-label">Member Price:</span>
                <span class="member-value">{}</span>
            </div>"#,
            pricing.format_price(mp)
        )
    } else {
        String::new()
    };

    format!(
        r#"<section class="product-pricing" data-section="pricing">
    <div class="price-main">
        <span class="price-current">{current_price}</span>
        {sale_info}
    </div>
    {member_price}
</section>"#,
        current_price = current_price,
        sale_info = sale_info,
        member_price = member_price
    )
}

/// Render pricing fallback.
pub fn render_pricing_fallback() -> String {
    r#"<section class="product-pricing product-pricing--loading" data-section="pricing">
    <div class="price-loading">
        <span class="price-placeholder">Loading price...</span>
    </div>
</section>"#
        .to_string()
}
