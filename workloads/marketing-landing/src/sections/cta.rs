//! Call-to-action section.

use crate::data::CtaContent;

/// Render the CTA section.
pub fn render_cta(content: &CtaContent) -> String {
    let secondary_cta = match (&content.secondary_cta_text, &content.secondary_cta_url) {
        (Some(text), Some(url)) => format!(
            r#"<a href="{}" class="cta-secondary">{}</a>"#,
            html_escape(url),
            html_escape(text)
        ),
        _ => String::new(),
    };

    format!(
        r#"<section class="cta-section" data-section="cta">
    <div class="cta-content">
        <h2 class="cta-headline">{}</h2>
        <p class="cta-subheadline">{}</p>
        <div class="cta-buttons">
            <a href="{}" class="cta-primary">{}</a>
            {}
        </div>
    </div>
</section>"#,
        html_escape(&content.headline),
        html_escape(&content.subheadline),
        html_escape(&content.primary_cta_url),
        html_escape(&content.primary_cta_text),
        secondary_cta
    )
}

/// Render skeleton placeholder for CTA.
pub fn render_cta_skeleton() -> String {
    r#"<section class="cta-section skeleton" data-section="cta">
    <div class="cta-content">
        <div class="skeleton-text skeleton-headline"></div>
        <div class="skeleton-text"></div>
        <div class="cta-buttons">
            <div class="skeleton-button"></div>
            <div class="skeleton-button secondary"></div>
        </div>
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
