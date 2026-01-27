//! Marketing Landing Page - CMS-driven streaming SSR with A/B testing.
//!
//! This workload demonstrates:
//! - Long cache TTLs for static marketing content
//! - Multiple CMS blocks rendered as sections
//! - Hero banner with A/B testing (vary by experiment cookie)
//! - Newsletter signup form (no cache)

mod data;
mod sections;

use spin_sdk::http::{Fields, IncomingRequest, Method, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

use edge_sdk::edge_core::RequestContext;
use edge_sdk::edge_streaming::{HeadContent, Shell, StreamingSink};

use data::{ExperimentVariant, LandingPageContent};
use sections::{render_cta, render_features, render_hero, render_newsletter, render_testimonials};

/// Marketing Landing page handler.
#[http_component]
async fn handle_landing(req: IncomingRequest, response_out: ResponseOutparam) {
    // Only handle GET requests
    if req.method() != Method::Get {
        let headers = Fields::from_list(&[]).unwrap();
        let response = OutgoingResponse::new(headers);
        response.set_status_code(405).unwrap();
        response_out.set(response);
        return;
    }

    // Extract experiment variant from cookie
    let variant = extract_experiment_variant(&req);

    // Create platform context
    let path = req.path_with_query().unwrap_or_default();
    let ctx = RequestContext::new(edge_sdk::edge_core::Method::Get, path.clone());

    // Build response headers with cache control
    let header_list: Vec<(String, Vec<u8>)> = vec![
        ("content-type".to_owned(), "text/html; charset=utf-8".into()),
        ("x-request-id".to_owned(), ctx.request_id.to_string().into()),
        (
            "cache-control".to_owned(),
            "public, max-age=3600, stale-while-revalidate=300".into(),
        ),
        ("vary".to_owned(), "Cookie".into()),
        (
            "x-experiment-variant".to_owned(),
            variant.name().as_bytes().to_vec(),
        ),
    ];

    let headers = Fields::from_list(&header_list).unwrap();
    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);
    let mut sink = StreamingSink::new(body, ctx.timing.clone());

    // Load content for the variant
    let content = LandingPageContent::for_variant(variant);

    // Create shell with landing page styling
    let shell = create_shell();

    // Send shell first (streaming SSR)
    if let Err(e) = sink.send_shell(&shell.render_opening()).await {
        eprintln!("Failed to send shell: {}", e);
        return;
    }

    // Stream sections as they're ready
    // In a real implementation, these might be fetched from CMS in parallel

    // Hero section (varies by experiment)
    let _ = sink.send_section("hero", &render_hero(&content.hero)).await;

    // Features section (static content, long cache)
    let _ = sink
        .send_section("features", &render_features(&content.features))
        .await;

    // Testimonials section (static content, long cache)
    let _ = sink
        .send_section("testimonials", &render_testimonials(&content.testimonials))
        .await;

    // CTA section (static content, long cache)
    let _ = sink.send_section("cta", &render_cta(&content.cta)).await;

    // Newsletter section (no cache, contains form)
    let _ = sink.send_section("newsletter", &render_newsletter()).await;

    // Send closing shell with JavaScript
    let closing = format!(
        "{}\n{}",
        shell.render_closing(),
        landing_page_scripts(variant)
    );
    let _ = sink.send_section("closing", &closing).await;
}

/// Extract experiment variant from cookie.
fn extract_experiment_variant(req: &IncomingRequest) -> ExperimentVariant {
    // Look for experiment cookie
    let headers = req.headers().get(&"cookie".to_string());
    for header in headers {
        let cookie_str = String::from_utf8_lossy(&header);
        for cookie in cookie_str.split(';') {
            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "experiment" {
                return ExperimentVariant::from_cookie(Some(parts[1]));
            }
        }
    }

    // Default to control if no cookie
    ExperimentVariant::Control
}

/// Create shell for landing page.
fn create_shell() -> Shell {
    let head = HeadContent::new("Transform Your Business | EdgePlatform")
        .with_meta("viewport", "width=device-width, initial-scale=1")
        .with_meta(
            "description",
            "The all-in-one platform for modern teams. Scale without limits.",
        )
        .with_style(LANDING_STYLES);

    Shell::new(head)
        .with_body_start(
            r#"<body>
<header class="site-header">
    <nav class="nav-container">
        <a href="/" class="logo">EdgePlatform</a>
        <div class="nav-links">
            <a href="/features">Features</a>
            <a href="/pricing">Pricing</a>
            <a href="/docs">Docs</a>
            <a href="/blog">Blog</a>
        </div>
        <div class="nav-actions">
            <a href="/login" class="btn-secondary">Log in</a>
            <a href="/signup" class="btn-primary">Get Started</a>
        </div>
    </nav>
</header>
<main>
"#,
        )
        .with_body_end(
            r#"
</main>
<footer class="site-footer">
    <div class="footer-container">
        <div class="footer-brand">
            <a href="/" class="logo">EdgePlatform</a>
            <p>The all-in-one platform for modern teams.</p>
        </div>
        <div class="footer-links">
            <div class="footer-column">
                <h4>Product</h4>
                <a href="/features">Features</a>
                <a href="/pricing">Pricing</a>
                <a href="/integrations">Integrations</a>
            </div>
            <div class="footer-column">
                <h4>Resources</h4>
                <a href="/docs">Documentation</a>
                <a href="/blog">Blog</a>
                <a href="/support">Support</a>
            </div>
            <div class="footer-column">
                <h4>Company</h4>
                <a href="/about">About</a>
                <a href="/careers">Careers</a>
                <a href="/contact">Contact</a>
            </div>
        </div>
    </div>
    <div class="footer-bottom">
        <p>&copy; 2024 EdgePlatform. All rights reserved.</p>
        <div class="footer-legal">
            <a href="/privacy">Privacy</a>
            <a href="/terms">Terms</a>
        </div>
    </div>
</footer>
</body>
</html>"#
                .to_string(),
        )
}

fn landing_page_scripts(variant: ExperimentVariant) -> String {
    format!(
        r#"<script>
// Analytics - track experiment variant
window.experimentVariant = '{}';

// Newsletter form handling
document.getElementById('newsletter-form')?.addEventListener('submit', async (e) => {{
    e.preventDefault();
    const form = e.target;
    const email = form.email.value;
    const submitBtn = form.querySelector('button[type="submit"]');
    const successDiv = document.querySelector('.newsletter-success');
    const errorDiv = document.querySelector('.newsletter-error');

    submitBtn.disabled = true;
    submitBtn.textContent = 'Subscribing...';

    try {{
        // In a real implementation, this would POST to an API
        await new Promise(resolve => setTimeout(resolve, 1000));

        // Simulate success
        form.hidden = true;
        successDiv.hidden = false;
        errorDiv.hidden = true;

        // Track conversion
        if (window.gtag) {{
            gtag('event', 'newsletter_signup', {{
                'experiment_variant': window.experimentVariant
            }});
        }}
    }} catch (error) {{
        errorDiv.querySelector('.error-message').textContent = error.message || 'Something went wrong. Please try again.';
        errorDiv.hidden = false;
        submitBtn.disabled = false;
        submitBtn.textContent = 'Subscribe';
    }}
}});

// Track page view with experiment variant
if (window.gtag) {{
    gtag('event', 'page_view', {{
        'experiment_variant': window.experimentVariant
    }});
}}
</script>"#,
        variant.name()
    )
}

const LANDING_STYLES: &str = r##"
:root {
    --primary: #6366f1;
    --primary-hover: #4f46e5;
    --secondary: #1e293b;
    --bg: #ffffff;
    --bg-alt: #f8fafc;
    --text: #1e293b;
    --text-muted: #64748b;
    --border: #e2e8f0;
    --success: #22c55e;
    --gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
}

* { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    line-height: 1.6;
}

/* Header */
.site-header {
    position: sticky;
    top: 0;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid var(--border);
    z-index: 100;
}

.nav-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem 2rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.logo {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--primary);
    text-decoration: none;
}

.nav-links {
    display: flex;
    gap: 2rem;
}

.nav-links a {
    color: var(--text);
    text-decoration: none;
    font-weight: 500;
    transition: color 0.2s;
}

.nav-links a:hover {
    color: var(--primary);
}

.nav-actions {
    display: flex;
    gap: 1rem;
    align-items: center;
}

.btn-secondary {
    color: var(--text);
    text-decoration: none;
    font-weight: 500;
}

.btn-primary {
    background: var(--primary);
    color: white;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    text-decoration: none;
    font-weight: 500;
    transition: background 0.2s;
}

.btn-primary:hover {
    background: var(--primary-hover);
}

/* Hero */
.hero {
    padding: 6rem 2rem;
    background: var(--gradient);
    color: white;
    text-align: center;
}

.hero-content {
    max-width: 800px;
    margin: 0 auto;
}

.hero-headline {
    font-size: 3.5rem;
    font-weight: 800;
    margin-bottom: 1.5rem;
    line-height: 1.1;
}

.hero-subheadline {
    font-size: 1.25rem;
    opacity: 0.9;
    margin-bottom: 2rem;
}

.hero-cta {
    display: inline-block;
    background: white;
    color: var(--primary);
    padding: 1rem 2.5rem;
    border-radius: 8px;
    font-weight: 600;
    font-size: 1.125rem;
    text-decoration: none;
    transition: transform 0.2s, box-shadow 0.2s;
}

.hero-cta:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

/* Features */
.features {
    padding: 6rem 2rem;
    background: var(--bg);
}

.section-header {
    text-align: center;
    max-width: 600px;
    margin: 0 auto 4rem;
}

.section-header h2 {
    font-size: 2.5rem;
    font-weight: 700;
    margin-bottom: 1rem;
}

.section-header p {
    color: var(--text-muted);
    font-size: 1.125rem;
}

.features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
}

.feature-card {
    padding: 2rem;
    background: white;
    border-radius: 12px;
    border: 1px solid var(--border);
    transition: box-shadow 0.2s, transform 0.2s;
}

.feature-card:hover {
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
    transform: translateY(-2px);
}

.feature-icon {
    font-size: 2.5rem;
    margin-bottom: 1rem;
    display: block;
}

.feature-title {
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
}

.feature-description {
    color: var(--text-muted);
}

/* Testimonials */
.testimonials {
    padding: 6rem 2rem;
    background: var(--bg-alt);
}

.testimonials-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
}

.testimonial-card {
    padding: 2rem;
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
}

.testimonial-quote {
    font-size: 1.125rem;
    font-style: italic;
    color: var(--text);
    margin-bottom: 1.5rem;
    line-height: 1.7;
}

.testimonial-author {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.testimonial-avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
}

.testimonial-avatar.placeholder {
    background: var(--primary);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
}

.author-name {
    font-weight: 600;
}

.author-title {
    font-size: 0.875rem;
    color: var(--text-muted);
}

/* CTA Section */
.cta-section {
    padding: 6rem 2rem;
    background: var(--secondary);
    color: white;
    text-align: center;
}

.cta-content {
    max-width: 600px;
    margin: 0 auto;
}

.cta-headline {
    font-size: 2.5rem;
    font-weight: 700;
    margin-bottom: 1rem;
}

.cta-subheadline {
    font-size: 1.125rem;
    opacity: 0.8;
    margin-bottom: 2rem;
}

.cta-buttons {
    display: flex;
    gap: 1rem;
    justify-content: center;
    flex-wrap: wrap;
}

.cta-primary {
    background: var(--primary);
    color: white;
    padding: 1rem 2rem;
    border-radius: 8px;
    text-decoration: none;
    font-weight: 600;
    transition: background 0.2s;
}

.cta-primary:hover {
    background: var(--primary-hover);
}

.cta-secondary {
    background: transparent;
    color: white;
    padding: 1rem 2rem;
    border: 2px solid white;
    border-radius: 8px;
    text-decoration: none;
    font-weight: 600;
    transition: background 0.2s;
}

.cta-secondary:hover {
    background: rgba(255, 255, 255, 0.1);
}

/* Newsletter */
.newsletter {
    padding: 4rem 2rem;
    background: var(--bg-alt);
}

.newsletter-content {
    max-width: 500px;
    margin: 0 auto;
    text-align: center;
}

.newsletter h2 {
    font-size: 1.75rem;
    margin-bottom: 0.5rem;
}

.newsletter p {
    color: var(--text-muted);
    margin-bottom: 1.5rem;
}

.newsletter-form {
    margin-bottom: 1rem;
}

.form-group {
    display: flex;
    gap: 0.5rem;
}

.form-group input {
    flex: 1;
    padding: 0.875rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 1rem;
}

.form-group button {
    padding: 0.875rem 1.5rem;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
}

.form-group button:hover {
    background: var(--primary-hover);
}

.form-group button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
}

.privacy-note {
    font-size: 0.875rem;
    color: var(--text-muted);
}

.newsletter-success {
    padding: 1.5rem;
    background: #dcfce7;
    border-radius: 8px;
    color: #166534;
}

.newsletter-error {
    padding: 1.5rem;
    background: #fef2f2;
    border-radius: 8px;
    color: #991b1b;
}

.success-icon, .error-icon {
    font-size: 1.5rem;
    margin-bottom: 0.5rem;
    display: block;
}

/* Footer */
.site-footer {
    background: var(--secondary);
    color: white;
    padding: 4rem 2rem 2rem;
}

.footer-container {
    max-width: 1200px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: 2fr 3fr;
    gap: 4rem;
    margin-bottom: 3rem;
}

.footer-brand p {
    opacity: 0.7;
    margin-top: 1rem;
}

.footer-links {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2rem;
}

.footer-column h4 {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 1rem;
    opacity: 0.7;
}

.footer-column a {
    display: block;
    color: white;
    text-decoration: none;
    opacity: 0.8;
    padding: 0.25rem 0;
    transition: opacity 0.2s;
}

.footer-column a:hover {
    opacity: 1;
}

.footer-bottom {
    max-width: 1200px;
    margin: 0 auto;
    padding-top: 2rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
    opacity: 0.7;
}

.footer-legal {
    display: flex;
    gap: 1.5rem;
}

.footer-legal a {
    color: white;
    text-decoration: none;
}

/* Skeleton loading */
.skeleton .skeleton-text {
    height: 1rem;
    background: linear-gradient(90deg, #e2e8f0 25%, #f1f5f9 50%, #e2e8f0 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
    border-radius: 4px;
    margin-bottom: 0.5rem;
}

.skeleton .skeleton-headline { width: 60%; height: 2rem; }
.skeleton .skeleton-subheadline { width: 80%; height: 1.25rem; }
.skeleton .skeleton-button { width: 150px; height: 48px; border-radius: 8px; }
.skeleton .skeleton-icon { width: 48px; height: 48px; border-radius: 8px; }
.skeleton .skeleton-avatar { width: 48px; height: 48px; border-radius: 50%; }

@keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
}

/* Responsive */
@media (max-width: 768px) {
    .nav-links {
        display: none;
    }

    .hero-headline {
        font-size: 2.5rem;
    }

    .features-grid,
    .testimonials-grid {
        grid-template-columns: 1fr;
    }

    .footer-container {
        grid-template-columns: 1fr;
        gap: 3rem;
    }

    .footer-links {
        grid-template-columns: repeat(2, 1fr);
    }

    .footer-bottom {
        flex-direction: column;
        gap: 1rem;
        text-align: center;
    }

    .form-group {
        flex-direction: column;
    }
}
"##;
