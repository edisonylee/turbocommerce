//! Inventory section renderer.

use crate::data::Inventory;

/// Render the inventory/stock section.
pub fn render_inventory(inventory: &Inventory) -> String {
    let status_class = inventory.status_class();
    let status_message = inventory.status_message();

    let restock_info = if !inventory.in_stock {
        if let Some(ref date) = inventory.estimated_restock {
            format!(
                r#"<p class="restock-date">Expected back in stock: {}</p>"#,
                escape_html(date)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let add_to_cart = if inventory.in_stock {
        r#"<button class="btn-add-to-cart">Add to Cart</button>"#
    } else {
        r#"<button class="btn-notify-me">Notify Me When Available</button>"#
    };

    format!(
        r#"<section class="product-inventory" data-section="inventory">
    <div class="stock-status {status_class}">
        <span class="stock-indicator"></span>
        <span class="stock-message">{status_message}</span>
    </div>
    {restock_info}
    <div class="inventory-actions">
        {add_to_cart}
    </div>
</section>"#,
        status_class = status_class,
        status_message = status_message,
        restock_info = restock_info,
        add_to_cart = add_to_cart
    )
}

/// Render inventory fallback.
pub fn render_inventory_fallback() -> String {
    r#"<section class="product-inventory product-inventory--loading" data-section="inventory">
    <div class="stock-status stock-loading">
        <span class="stock-message">Checking availability...</span>
    </div>
</section>"#
        .to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
