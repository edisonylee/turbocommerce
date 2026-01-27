//! Testimonials section.

use crate::data::TestimonialsContent;

/// Render the testimonials section.
pub fn render_testimonials(content: &TestimonialsContent) -> String {
    let testimonials_html: String = content
        .testimonials
        .iter()
        .map(|t| {
            let avatar = t
                .avatar_url
                .as_ref()
                .map(|url| format!(r#"<img src="{}" alt="{}" class="testimonial-avatar">"#,
                    html_escape(url), html_escape(&t.author_name)))
                .unwrap_or_else(|| {
                    let initials: String = t
                        .author_name
                        .split_whitespace()
                        .filter_map(|w| w.chars().next())
                        .take(2)
                        .collect();
                    format!(r#"<div class="testimonial-avatar placeholder">{}</div>"#, initials)
                });

            format!(
                r#"<article class="testimonial-card">
            <blockquote class="testimonial-quote">"{}"</blockquote>
            <div class="testimonial-author">
                {}
                <div class="author-info">
                    <div class="author-name">{}</div>
                    <div class="author-title">{} at {}</div>
                </div>
            </div>
        </article>"#,
                html_escape(&t.quote),
                avatar,
                html_escape(&t.author_name),
                html_escape(&t.author_title),
                html_escape(&t.company)
            )
        })
        .collect();

    format!(
        r#"<section class="testimonials" data-section="testimonials">
    <div class="section-header">
        <h2>{}</h2>
    </div>
    <div class="testimonials-grid">
        {}
    </div>
</section>"#,
        html_escape(&content.section_title),
        testimonials_html
    )
}

/// Render skeleton placeholder for testimonials.
pub fn render_testimonials_skeleton() -> String {
    let cards: String = (0..3)
        .map(|_| {
            r#"<div class="testimonial-card skeleton">
            <div class="skeleton-text"></div>
            <div class="skeleton-text"></div>
            <div class="skeleton-text short"></div>
            <div class="testimonial-author">
                <div class="skeleton-avatar"></div>
                <div class="author-info">
                    <div class="skeleton-text short"></div>
                    <div class="skeleton-text shorter"></div>
                </div>
            </div>
        </div>"#
        })
        .collect();

    format!(
        r#"<section class="testimonials skeleton" data-section="testimonials">
    <div class="section-header">
        <div class="skeleton-text skeleton-headline"></div>
    </div>
    <div class="testimonials-grid">
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
