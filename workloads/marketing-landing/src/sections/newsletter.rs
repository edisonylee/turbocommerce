//! Newsletter signup section.

/// Render the newsletter signup form.
pub fn render_newsletter() -> String {
    r#"<section class="newsletter" data-section="newsletter">
    <div class="newsletter-content">
        <h2>Stay in the loop</h2>
        <p>Get the latest updates, tips, and exclusive offers delivered to your inbox.</p>
        <form class="newsletter-form" action="/api/newsletter" method="POST" id="newsletter-form">
            <div class="form-group">
                <input
                    type="email"
                    name="email"
                    placeholder="Enter your email"
                    required
                    aria-label="Email address"
                >
                <button type="submit">Subscribe</button>
            </div>
            <p class="privacy-note">
                We respect your privacy. Unsubscribe at any time.
            </p>
        </form>
        <div class="newsletter-success" hidden>
            <span class="success-icon">✓</span>
            <p>Thanks for subscribing! Check your inbox for confirmation.</p>
        </div>
        <div class="newsletter-error" hidden>
            <span class="error-icon">!</span>
            <p class="error-message"></p>
        </div>
    </div>
</section>"#
        .to_string()
}

/// Render skeleton placeholder for newsletter.
pub fn render_newsletter_skeleton() -> String {
    r#"<section class="newsletter skeleton" data-section="newsletter">
    <div class="newsletter-content">
        <div class="skeleton-text skeleton-headline"></div>
        <div class="skeleton-text"></div>
        <div class="skeleton-form">
            <div class="skeleton-input"></div>
            <div class="skeleton-button"></div>
        </div>
    </div>
</section>"#
        .to_string()
}
