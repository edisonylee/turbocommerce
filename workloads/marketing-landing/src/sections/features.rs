//! Features section.

use crate::data::FeaturesContent;

/// Render the features section.
pub fn render_features(content: &FeaturesContent) -> String {
    let features_html: String = content
        .features
        .iter()
        .map(|f| {
            format!(
                r#"<div class="feature-card">
            <span class="feature-icon">{}</span>
            <h3 class="feature-title">{}</h3>
            <p class="feature-description">{}</p>
        </div>"#,
                html_escape(&f.icon),
                html_escape(&f.title),
                html_escape(&f.description)
            )
        })
        .collect();

    format!(
        r#"<section class="features" data-section="features">
    <div class="section-header">
        <h2>{}</h2>
        <p>{}</p>
    </div>
    <div class="features-grid">
        {}
    </div>
</section>"#,
        html_escape(&content.section_title),
        html_escape(&content.section_subtitle),
        features_html
    )
}

/// Render skeleton placeholder for features.
pub fn render_features_skeleton() -> String {
    let cards: String = (0..6)
        .map(|_| {
            r#"<div class="feature-card skeleton">
            <div class="skeleton-icon"></div>
            <div class="skeleton-text skeleton-title"></div>
            <div class="skeleton-text"></div>
            <div class="skeleton-text"></div>
        </div>"#
        })
        .collect();

    format!(
        r#"<section class="features skeleton" data-section="features">
    <div class="section-header">
        <div class="skeleton-text skeleton-headline"></div>
        <div class="skeleton-text"></div>
    </div>
    <div class="features-grid">
        {}
    </div>
</section>"#,
        cards
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
