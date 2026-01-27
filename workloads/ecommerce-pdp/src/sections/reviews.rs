//! Reviews section renderer.

use crate::data::{Review, ReviewsResponse};

/// Render the reviews section.
pub fn render_reviews(response: &ReviewsResponse) -> String {
    let summary = &response.summary;

    // Rating distribution bars
    let distribution = &summary.rating_distribution;
    let total = summary.total_reviews;
    let distribution_html: String = (1..=5)
        .rev()
        .map(|stars| {
            let pct = distribution.percentage(stars, total);
            format!(
                r#"<div class="rating-bar">
                    <span class="rating-label">{} star</span>
                    <div class="rating-bar-track">
                        <div class="rating-bar-fill" style="width: {:.0}%"></div>
                    </div>
                    <span class="rating-count">{:.0}%</span>
                </div>"#,
                stars, pct, pct
            )
        })
        .collect();

    // Individual reviews
    let reviews_html: String = response
        .reviews
        .iter()
        .take(5)
        .map(render_single_review)
        .collect();

    let load_more = if response.has_more {
        r#"<button class="btn-load-more">Load More Reviews</button>"#
    } else {
        ""
    };

    format!(
        r#"<section class="product-reviews" data-section="reviews">
    <h2>Customer Reviews</h2>
    <div class="reviews-summary">
        <div class="average-rating">
            <span class="rating-number">{average:.1}</span>
            <span class="rating-stars">{stars}</span>
            <span class="rating-count">({total} reviews)</span>
        </div>
        <div class="rating-distribution">
            {distribution_html}
        </div>
    </div>
    <div class="reviews-list">
        {reviews_html}
    </div>
    {load_more}
</section>"#,
        average = summary.average_rating,
        stars = render_star_rating(summary.average_rating),
        total = total,
        distribution_html = distribution_html,
        reviews_html = reviews_html,
        load_more = load_more
    )
}

fn render_single_review(review: &Review) -> String {
    let verified = if review.verified_purchase {
        r#"<span class="verified-badge">Verified Purchase</span>"#
    } else {
        ""
    };

    format!(
        r#"<article class="review">
        <header class="review-header">
            <span class="review-stars">{stars}</span>
            <span class="review-author">{author}</span>
            <span class="review-date">{date}</span>
            {verified}
        </header>
        <h3 class="review-title">{title}</h3>
        <p class="review-body">{body}</p>
        <footer class="review-footer">
            <span class="helpful-count">{helpful} people found this helpful</span>
            <button class="btn-helpful">Helpful</button>
        </footer>
    </article>"#,
        stars = review.render_stars(),
        author = escape_html(&review.author),
        date = escape_html(&review.date),
        verified = verified,
        title = escape_html(&review.title),
        body = escape_html(&review.body),
        helpful = review.helpful_votes
    )
}

fn render_star_rating(rating: f32) -> String {
    let full = rating.floor() as usize;
    let half = if rating.fract() >= 0.5 { 1 } else { 0 };
    let empty = 5 - full - half;

    format!(
        "{}{}{}",
        "★".repeat(full),
        if half > 0 { "⯨" } else { "" },
        "☆".repeat(empty)
    )
}

/// Render reviews fallback.
pub fn render_reviews_fallback() -> String {
    r#"<section class="product-reviews product-reviews--fallback" data-section="reviews">
    <h2>Customer Reviews</h2>
    <p class="reviews-loading">Unable to load reviews at this time.</p>
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
