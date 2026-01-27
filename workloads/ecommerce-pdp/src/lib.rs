//! E-commerce Product Detail Page - Reference workload.
//!
//! Demonstrates streaming SSR with:
//! - Multiple parallel data fetches
//! - Different cache policies per section
//! - Fallback handling for non-critical sections
//! - Personalization (recommendations vary by user)

mod data;
mod sections;

use std::time::Duration;

use futures::future::join4;
use spin_sdk::http::{Fields, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

use edge_sdk::edge_cache::{
    CacheHeadersBuilder, CacheKeyBuilder, CacheKeyContext, RouteCachePolicy, VaryRule,
};
use edge_sdk::edge_core::{Method, RequestContext, TimingContext};
use edge_sdk::edge_data::{DependencyTag, FetchClient};
use edge_sdk::edge_observability::{LogFormat, LogLevel, MetricsCollector, StructuredLogger};
use edge_sdk::edge_streaming::{HeadContent, Shell, StreamingSink};

use data::*;
use sections::*;

/// Mock API base URL (in production, this would be configured)
const API_BASE: &str = "https://jsonplaceholder.typicode.com";

/// Main HTTP handler for the PDP.
#[http_component]
async fn handle(req: IncomingRequest, response_out: ResponseOutparam) {
    // Extract product ID from path
    let path = req.path_with_query().unwrap_or_default();
    let product_id = extract_product_id(&path).unwrap_or("1");

    // Create platform context
    let ctx = RequestContext::new(Method::Get, path.clone());
    let request_id = ctx.request_id.clone();

    // Setup logging
    let logger = StructuredLogger::new(request_id.clone())
        .with_workload("ecommerce-pdp")
        .with_route(&ctx.path)
        .with_min_level(LogLevel::Debug)
        .with_format(LogFormat::Human);

    // Setup metrics
    let mut metrics = MetricsCollector::new(request_id.clone());
    metrics.set_workload("ecommerce-pdp");
    metrics.set_route(&ctx.path);

    logger.info_builder("PDP request started")
        .field("product_id", product_id.to_string())
        .emit();

    // Define route-level cache policy
    // The page shell is public, but sections have different policies
    let route_policy = RouteCachePolicy::public(Duration::from_secs(60))
        .with_swr(Duration::from_secs(30))
        .vary_on(VaryRule::header("Accept-Encoding"))
        .with_tag("pdp")
        .with_tag(format!("product:{}", product_id));

    // Build cache key
    let cache_key_ctx = CacheKeyContext {
        path: ctx.path.clone(),
        ..Default::default()
    };
    let cache_key = CacheKeyBuilder::new()
        .with_prefix("pdp")
        .route()
        .build(&cache_key_ctx);

    // Build response headers with caching
    let cache_headers = CacheHeadersBuilder::new()
        .cache_control_from_policy(&route_policy)
        .vary_from_policy(&route_policy)
        .build();

    let mut header_list: Vec<(String, Vec<u8>)> = vec![
        ("content-type".to_owned(), "text/html; charset=utf-8".into()),
        ("x-request-id".to_owned(), request_id.to_string().into()),
    ];
    for (name, value) in cache_headers {
        header_list.push((name, value.into_bytes()));
    }

    let headers = Fields::from_list(&header_list).unwrap();
    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);
    let mut sink = StreamingSink::new(body, ctx.timing.clone());

    // Define the shell template
    let shell = Shell::new(
        HeadContent::new(&format!("Product {} | E-commerce Store", product_id))
            .with_meta("viewport", "width=device-width, initial-scale=1")
            .with_style(PDP_STYLES),
    )
    .with_body_start(format!(
        r#"<body>
    <header class="site-header">
        <nav><a href="/">Home</a> / <a href="/products">Products</a> / Product {}</nav>
    </header>
    <main class="pdp-container">
        <p class="request-info">Request ID: {}</p>
"#,
        product_id, request_id
    ))
    .with_body_end(
        r#"
    </main>
    <footer class="site-footer">
        <p>Streaming SSR Demo - E-commerce PDP</p>
    </footer>
</body>
</html>"#
            .to_string(),
    );

    // 1) Send shell immediately
    if let Err(e) = sink.send_shell(&shell.render_opening()).await {
        logger.error_builder("Failed to send shell")
            .field("error", e.to_string())
            .emit();
        return;
    }
    metrics.record_shell_sent();
    logger.debug("Shell sent");

    // 2) Create fetch client and start parallel fetches
    let client = FetchClient::new(request_id.clone(), TimingContext::new());

    // In a real scenario, these would be actual API endpoints
    // For this demo, we'll use mock data via JSONPlaceholder

    logger.info("Starting parallel data fetches");

    // Fetch product, pricing, inventory, and reviews in parallel
    let (product_res, pricing_res, inventory_res, reviews_res) = join4(
        fetch_product(&client, product_id),
        fetch_pricing(&client, product_id),
        fetch_inventory(&client, product_id),
        fetch_reviews(&client, product_id),
    )
    .await;

    // 3) Stream hero section (critical - from product data)
    metrics.record_section_start("hero");
    let hero_html = match product_res {
        Ok(product) => {
            logger.debug("Rendering hero section");
            render_hero(&product)
        }
        Err(e) => {
            logger.warn_builder("Product fetch failed")
                .field("error", e.to_string())
                .emit();
            render_hero_fallback()
        }
    };
    let _ = sink.send_section("hero", &hero_html).await;
    metrics.record_section_sent("hero", Some(hero_html.len()), false);

    // 4) Stream pricing section (short cache)
    metrics.record_section_start("pricing");
    let pricing_html = match pricing_res {
        Ok(pricing) => {
            logger.debug("Rendering pricing section");
            render_pricing(&pricing)
        }
        Err(e) => {
            logger.warn_builder("Pricing fetch failed")
                .field("error", e.to_string())
                .emit();
            render_pricing_fallback()
        }
    };
    let _ = sink.send_section("pricing", &pricing_html).await;
    metrics.record_section_sent("pricing", Some(pricing_html.len()), false);

    // 5) Stream inventory section (no cache - always fresh)
    metrics.record_section_start("inventory");
    let inventory_html = match inventory_res {
        Ok(inventory) => {
            logger.debug("Rendering inventory section");
            render_inventory(&inventory)
        }
        Err(e) => {
            logger.warn_builder("Inventory fetch failed")
                .field("error", e.to_string())
                .emit();
            render_inventory_fallback()
        }
    };
    let _ = sink.send_section("inventory", &inventory_html).await;
    metrics.record_section_sent("inventory", Some(inventory_html.len()), false);

    // 6) Stream reviews section (moderate cache)
    metrics.record_section_start("reviews");
    let reviews_html = match reviews_res {
        Ok(reviews) => {
            logger.debug("Rendering reviews section");
            render_reviews(&reviews)
        }
        Err(e) => {
            logger.warn_builder("Reviews fetch failed")
                .field("error", e.to_string())
                .emit();
            render_reviews_fallback()
        }
    };
    let _ = sink.send_section("reviews", &reviews_html).await;
    metrics.record_section_sent("reviews", Some(reviews_html.len()), false);

    // 7) Fetch and stream recommendations (personalized - last since it can be slow)
    metrics.record_section_start("recommendations");
    let recs_res = fetch_recommendations(&client, product_id).await;
    let recs_html = match recs_res {
        Ok(recs) => {
            logger.debug("Rendering recommendations section");
            render_recommendations(&recs)
        }
        Err(e) => {
            logger.warn_builder("Recommendations fetch failed")
                .field("error", e.to_string())
                .emit();
            render_recommendations_fallback()
        }
    };
    let _ = sink.send_section("recommendations", &recs_html).await;
    metrics.record_section_sent("recommendations", Some(recs_html.len()), false);

    // 8) Send closing
    let _ = sink.send_raw(shell.render_closing().into_bytes()).await;

    // Finalize metrics
    let final_metrics = metrics.finalize(Some(200));
    logger.info("PDP request complete");
    eprintln!("\n{}", final_metrics.to_summary());
}

/// Extract product ID from path like /product/123
fn extract_product_id(path: &str) -> Option<&str> {
    path.strip_prefix("/product/")
        .and_then(|s| s.split('?').next())
        .and_then(|s| s.split('/').next())
}

// Mock data fetchers - in production these would call real APIs

async fn fetch_product(client: &FetchClient, product_id: &str) -> Result<Product, anyhow::Error> {
    // Using JSONPlaceholder posts as mock products
    let url = format!("{}/posts/{}", API_BASE, product_id);
    let post: serde_json::Value = client.fetch(&url, DependencyTag::Cms).await?;

    Ok(Product {
        id: product_id.to_string(),
        name: post["title"].as_str().unwrap_or("Product").to_string(),
        description: post["body"].as_str().unwrap_or("").to_string(),
        brand: "Demo Brand".to_string(),
        category: "Electronics".to_string(),
        images: vec![ProductImage {
            url: format!("https://picsum.photos/seed/{}/400/400", product_id),
            alt: "Product image".to_string(),
            is_primary: true,
        }],
        attributes: vec![
            ProductAttribute { name: "SKU".to_string(), value: format!("SKU-{}", product_id) },
            ProductAttribute { name: "Weight".to_string(), value: "1.5 kg".to_string() },
        ],
    })
}

async fn fetch_pricing(_client: &FetchClient, product_id: &str) -> Result<Pricing, anyhow::Error> {
    // Mock pricing data
    let id: u32 = product_id.parse().unwrap_or(1);
    let base_price = 99.99 + (id as f64 * 10.0);

    Ok(Pricing {
        product_id: product_id.to_string(),
        price: base_price,
        currency: "USD".to_string(),
        original_price: Some(base_price * 1.2),
        discount_percentage: Some(20),
        member_price: Some(base_price * 0.9),
    })
}

async fn fetch_inventory(_client: &FetchClient, product_id: &str) -> Result<Inventory, anyhow::Error> {
    // Mock inventory data
    let id: u32 = product_id.parse().unwrap_or(1);
    let quantity = (id * 7) % 100;

    Ok(Inventory {
        product_id: product_id.to_string(),
        in_stock: quantity > 0,
        quantity,
        warehouse: "US-WEST".to_string(),
        estimated_restock: if quantity == 0 { Some("2024-02-15".to_string()) } else { None },
    })
}

async fn fetch_reviews(client: &FetchClient, product_id: &str) -> Result<ReviewsResponse, anyhow::Error> {
    // Using JSONPlaceholder comments as mock reviews
    let url = format!("{}/posts/{}/comments", API_BASE, product_id);
    let comments: Vec<serde_json::Value> = client.fetch(&url, DependencyTag::Search).await?;

    let reviews: Vec<Review> = comments
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, c)| Review {
            id: c["id"].to_string(),
            product_id: product_id.to_string(),
            author: c["name"].as_str().unwrap_or("Anonymous").to_string(),
            rating: ((i % 5) + 1) as u8,
            title: c["name"].as_str().unwrap_or("Review").to_string(),
            body: c["body"].as_str().unwrap_or("").to_string(),
            date: "2024-01-15".to_string(),
            verified_purchase: i % 2 == 0,
            helpful_votes: (i * 3) as u32,
        })
        .collect();

    let total = reviews.len() as u32;
    let avg = reviews.iter().map(|r| r.rating as f32).sum::<f32>() / total as f32;

    Ok(ReviewsResponse {
        summary: ReviewSummary {
            product_id: product_id.to_string(),
            average_rating: avg,
            total_reviews: total,
            rating_distribution: RatingDistribution {
                five_star: 2,
                four_star: 1,
                three_star: 1,
                two_star: 0,
                one_star: 1,
            },
        },
        reviews,
        has_more: true,
    })
}

async fn fetch_recommendations(_client: &FetchClient, product_id: &str) -> Result<Vec<RecommendedProduct>, anyhow::Error> {
    // Mock recommendations
    let id: u32 = product_id.parse().unwrap_or(1);

    Ok(vec![
        RecommendedProduct {
            id: format!("{}", (id + 1) % 100),
            name: "Related Product 1".to_string(),
            price: 79.99,
            image_url: format!("https://picsum.photos/seed/{}/200/200", id + 1),
            reason: "Frequently bought together".to_string(),
        },
        RecommendedProduct {
            id: format!("{}", (id + 2) % 100),
            name: "Related Product 2".to_string(),
            price: 129.99,
            image_url: format!("https://picsum.photos/seed/{}/200/200", id + 2),
            reason: "Similar items".to_string(),
        },
        RecommendedProduct {
            id: format!("{}", (id + 3) % 100),
            name: "Related Product 3".to_string(),
            price: 59.99,
            image_url: format!("https://picsum.photos/seed/{}/200/200", id + 3),
            reason: "Customers also viewed".to_string(),
        },
    ])
}

/// CSS styles for the PDP
const PDP_STYLES: &str = r#"
* { box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 0; background: #f5f5f5; }
.site-header { background: #333; color: white; padding: 1rem 2rem; }
.site-header a { color: #88f; }
.site-footer { background: #333; color: white; padding: 2rem; text-align: center; margin-top: 2rem; }
.pdp-container { max-width: 1200px; margin: 0 auto; padding: 2rem; }
.request-info { font-size: 0.75rem; color: #666; }

/* Hero Section */
.product-hero { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; background: white; padding: 2rem; border-radius: 8px; margin-bottom: 1rem; }
.product-image-main { width: 100%; border-radius: 8px; }
.product-thumbnails { display: flex; gap: 0.5rem; margin-top: 1rem; }
.product-thumbnail { width: 60px; height: 60px; object-fit: cover; border-radius: 4px; cursor: pointer; }
.product-brand { color: #666; margin: 0; }
.product-name { font-size: 2rem; margin: 0.5rem 0; }
.product-category { color: #888; }
.product-description { margin: 1rem 0; line-height: 1.6; }
.product-attributes { list-style: none; padding: 0; }
.product-attributes li { padding: 0.25rem 0; }

/* Pricing Section */
.product-pricing { background: white; padding: 1.5rem 2rem; border-radius: 8px; margin-bottom: 1rem; }
.price-current { font-size: 2rem; font-weight: bold; color: #b12704; }
.price-original { text-decoration: line-through; color: #666; margin-left: 1rem; }
.price-discount { background: #cc0c39; color: white; padding: 0.25rem 0.5rem; border-radius: 4px; margin-left: 0.5rem; }
.member-price { margin-top: 0.5rem; padding: 0.5rem; background: #fffde7; border-radius: 4px; }

/* Inventory Section */
.product-inventory { background: white; padding: 1.5rem 2rem; border-radius: 8px; margin-bottom: 1rem; }
.stock-status { display: flex; align-items: center; gap: 0.5rem; }
.stock-indicator { width: 12px; height: 12px; border-radius: 50%; }
.stock-available .stock-indicator { background: #4caf50; }
.stock-low .stock-indicator { background: #ff9800; }
.stock-out .stock-indicator { background: #f44336; }
.btn-add-to-cart { background: #ff9900; border: none; padding: 1rem 2rem; font-size: 1rem; border-radius: 8px; cursor: pointer; margin-top: 1rem; }
.btn-notify-me { background: #2196f3; color: white; border: none; padding: 1rem 2rem; font-size: 1rem; border-radius: 8px; cursor: pointer; margin-top: 1rem; }

/* Reviews Section */
.product-reviews { background: white; padding: 2rem; border-radius: 8px; margin-bottom: 1rem; }
.reviews-summary { display: flex; gap: 2rem; margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #eee; }
.average-rating { text-align: center; }
.rating-number { font-size: 3rem; font-weight: bold; }
.rating-stars { color: #ff9800; font-size: 1.5rem; }
.rating-distribution { flex: 1; }
.rating-bar { display: flex; align-items: center; gap: 0.5rem; margin: 0.25rem 0; }
.rating-bar-track { flex: 1; height: 8px; background: #eee; border-radius: 4px; }
.rating-bar-fill { height: 100%; background: #ff9800; border-radius: 4px; }
.review { border-bottom: 1px solid #eee; padding: 1rem 0; }
.review-header { display: flex; gap: 1rem; align-items: center; margin-bottom: 0.5rem; }
.review-stars { color: #ff9800; }
.verified-badge { background: #e8f5e9; color: #2e7d32; font-size: 0.75rem; padding: 0.25rem 0.5rem; border-radius: 4px; }
.review-title { margin: 0.5rem 0; }
.review-body { color: #555; line-height: 1.6; }

/* Recommendations Section */
.product-recommendations { background: white; padding: 2rem; border-radius: 8px; }
.recommendations-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin-top: 1rem; }
.recommendation-card { border: 1px solid #eee; border-radius: 8px; overflow: hidden; }
.recommendation-link { text-decoration: none; color: inherit; }
.recommendation-image { width: 100%; aspect-ratio: 1; object-fit: cover; }
.recommendation-info { padding: 1rem; }
.recommendation-name { font-size: 0.9rem; margin: 0 0 0.5rem 0; }
.recommendation-price { font-weight: bold; color: #b12704; margin: 0; }
.recommendation-reason { font-size: 0.75rem; color: #666; margin: 0.25rem 0 0 0; }

/* Loading/Error States */
.product-hero--error, .product-pricing--loading, .product-inventory--loading,
.product-reviews--fallback, .product-recommendations--fallback {
    opacity: 0.7;
}
"#;
