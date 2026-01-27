//! Hero banner section with A/B testing support.

use crate::data::HeroContent;

/// Render the hero banner section.
pub fn render_hero(content: &HeroContent) -> String {
    let bg_style = content
        .background_image
        .as_ref()
        .map(|url| format!(r#" style="background-image: url('{}')""#, html_escape(url)))
        .unwrap_or_default();

    format!(
        r#"<section class="hero" data-section="hero" data-variant="{}"{}>
    <div class="hero-content">
        <h1 class="hero-headline">{}</h1>
        <p class="hero-subheadline">{}</p>
        <a href="{}" class="hero-cta">{}</a>
    </div>
    <div class="hero-visual">
        <div class="hero-illustration"></div>
    </div>
</section>"#,
        html_escape(&content.variant),
        bg_style,
        html_escape(&content.headline),
        html_escape(&content.subheadline),
        html_escape(&content.cta_url),
        html_escape(&content.cta_text)
    )
}

/// Render skeleton placeholder for hero.
pub fn render_hero_skeleton() -> String {
    r#"<section class="hero skeleton" data-section="hero">
    <div class="hero-content">
        <div class="skeleton-text skeleton-headline"></div>
        <div class="skeleton-text skeleton-subheadline"></div>
        <div class="skeleton-button"></div>
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
