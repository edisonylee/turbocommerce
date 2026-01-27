//! Reference workload demonstrating the edge streaming SSR platform.
//!
//! This workload shows:
//! - Shell-first streaming with `StreamingSink`
//! - Parallel data fetching with `DependencyTag`
//! - Section-based rendering
//! - Structured logging and metrics
//! - Route-level cache policies and cache headers

use std::time::Duration;

use futures::future::join3;
use serde::Deserialize;
use spin_sdk::http::{Fields, IncomingRequest, OutgoingResponse, ResponseOutparam};
use spin_sdk::http_component;

// Import platform abstractions
use edge_sdk::edge_cache::{
    CacheExplainHeaders, CacheHeadersBuilder, CacheKeyBuilder, CacheKeyContext, CacheStatus,
    RouteCachePolicy, VaryRule,
};
use edge_sdk::edge_core::{Method, RequestContext, TimingContext};
use edge_sdk::edge_data::{DependencyTag, FetchClient};
use edge_sdk::edge_observability::{
    LogFormat, LogLevel, MetricsCollector, StructuredLogger,
};
use edge_sdk::edge_streaming::{HeadContent, Section, Shell, StreamingSink};

// Data models for the JSON API
#[derive(Debug, Deserialize)]
struct Todo {
    #[allow(dead_code)]
    id: u32,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct Post {
    #[allow(dead_code)]
    id: u32,
    title: String,
}

#[derive(Debug, Deserialize)]
struct User {
    #[allow(dead_code)]
    id: u32,
    name: String,
}

/// Main HTTP handler using platform abstractions.
#[http_component]
async fn handle(req: IncomingRequest, response_out: ResponseOutparam) {
    // Create platform request context
    let ctx = RequestContext::new(Method::Get, req.path_with_query().unwrap_or_default());
    let request_id = ctx.request_id.clone();

    // Create structured logger
    let logger = StructuredLogger::new(request_id.clone())
        .with_workload("hello-wasm")
        .with_route(&ctx.path)
        .with_min_level(LogLevel::Debug)
        .with_format(LogFormat::Human); // Use Human format for development

    // Create metrics collector
    let mut metrics = MetricsCollector::new(request_id.clone());
    metrics.set_workload("hello-wasm");
    metrics.set_route(&ctx.path);

    logger.info("Request started");

    // Define route cache policy - public cache with 5 minute TTL
    let cache_policy = RouteCachePolicy::public(Duration::from_secs(300))
        .with_swr(Duration::from_secs(60)) // Serve stale while revalidating for 60s
        .with_stale_if_error(Duration::from_secs(3600)) // Serve stale on error for 1h
        .vary_on(VaryRule::header("Accept-Encoding"))
        .with_tag("hello-wasm")
        .with_tag("demo");

    // Build cache key context from request
    let cache_key_ctx = CacheKeyContext {
        path: ctx.path.clone(),
        ..Default::default()
    };

    // Build cache key
    let cache_key = CacheKeyBuilder::new()
        .with_prefix("hello-wasm")
        .route()
        .build(&cache_key_ctx);

    logger.debug_builder("Cache key generated")
        .field("key", cache_key.as_str().to_string())
        .emit();

    // Check if debug headers requested
    let include_debug = req
        .headers()
        .get(&"x-debug-cache".to_string())
        .first()
        .map(|v| String::from_utf8_lossy(v) == "1")
        .unwrap_or(false);

    // Build cache explain headers
    let explain_headers = CacheExplainHeaders::from_policy(&cache_policy, &cache_key)
        .with_status(CacheStatus::Miss); // This is a dynamic response, always MISS

    // Build cache headers
    let cache_headers = CacheHeadersBuilder::new()
        .cache_control_from_policy(&cache_policy)
        .vary_from_policy(&cache_policy)
        .explain(explain_headers)
        .include_debug(include_debug)
        .build();

    // Setup HTTP response headers
    let mut header_list: Vec<(String, Vec<u8>)> = vec![
        ("content-type".to_owned(), "text/html; charset=utf-8".into()),
        ("x-request-id".to_owned(), request_id.to_string().into()),
    ];

    // Add cache headers
    for (name, value) in cache_headers {
        header_list.push((name, value.into_bytes()));
    }

    let headers = Fields::from_list(&header_list).unwrap();

    let response = OutgoingResponse::new(headers);
    response.set_status_code(200).unwrap();

    let body = response.take_body();
    response_out.set(response);

    // Create platform streaming sink
    let mut sink = StreamingSink::new(body, ctx.timing.clone());

    // Define the shell template
    let body_start = format!(
        r#"<body>
    <h1>Parallel fetch demo (Edge Platform)</h1>
    <p>Shell flushed ✅ Starting 3 fetches in parallel…</p>
    <p><small>Request ID: {}</small></p>
    <hr/>
"#,
        request_id
    );

    let shell = Shell::new(HeadContent::new("Parallel Fetches - Edge Platform"))
        .with_body_start(body_start)
        .with_body_end(
            r#"
    <hr/>
    <p>Done.</p>
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

    // 2) Define sections with their dependencies
    let _todos_section = Section::builder("todos")
        .depends_on("recommendations")
        .with_fallback("<li>Failed to load todos</li>")
        .build();

    let _posts_section = Section::builder("posts")
        .depends_on("search")
        .with_fallback("<li>Failed to load posts</li>")
        .build();

    let _users_section = Section::builder("users")
        .depends_on("profile")
        .with_fallback("<li>Failed to load users</li>")
        .build();

    // 3) Create fetch client
    let client = FetchClient::new(request_id.clone(), TimingContext::new());

    // 4) Start three fetches concurrently
    let todos_url = "https://jsonplaceholder.typicode.com/todos?_limit=5";
    let posts_url = "https://jsonplaceholder.typicode.com/posts?_limit=5";
    let users_url = "https://jsonplaceholder.typicode.com/users?_limit=3";

    logger.info("Starting parallel fetches");
    let fetch_start = std::time::Instant::now();

    let (todos_res, posts_res, users_res) = join3(
        client.fetch::<Vec<Todo>>(todos_url, DependencyTag::Recommendations),
        client.fetch::<Vec<Post>>(posts_url, DependencyTag::Search),
        client.fetch::<Vec<User>>(users_url, DependencyTag::Profile),
    )
    .await;

    let fetch_duration = fetch_start.elapsed();
    logger.info_builder("Parallel fetches complete")
        .duration_ms("fetch_duration", fetch_duration)
        .emit();

    // Record dependency metrics
    record_fetch_metric(&mut metrics, "recommendations", todos_url, &todos_res, fetch_duration);
    record_fetch_metric(&mut metrics, "search", posts_url, &posts_res, fetch_duration);
    record_fetch_metric(&mut metrics, "profile", users_url, &users_res, fetch_duration);

    // 5) Stream sections
    // Header section
    metrics.record_section_start("header");
    let _ = sink.send_section("header", "<h2>Results</h2>").await;
    metrics.record_section_sent("header", Some(17), false);

    // Todos section
    metrics.record_section_start("todos");
    let (todos_html, todos_fallback) = match todos_res {
        Ok(todos) => {
            logger.debug_builder("Rendering todos section")
                .field_i64("count", todos.len() as i64)
                .emit();
            let items: String = todos
                .iter()
                .map(|t| {
                    let status = if t.completed { "✅" } else { "⏳" };
                    format!("<li>{} {}</li>", status, escape_html(&t.title))
                })
                .collect();
            (format!("<h3>Todos</h3><ul>{}</ul>", items), false)
        }
        Err(e) => {
            logger.warn_builder("Todos fetch failed, using fallback")
                .field("error", e.to_string())
                .emit();
            (
                format!(
                    "<h3>Todos</h3><ul><li><b>Error:</b> {}</li></ul>",
                    escape_html(&e.to_string())
                ),
                true,
            )
        }
    };
    let _ = sink.send_section("todos", &todos_html).await;
    metrics.record_section_sent("todos", Some(todos_html.len()), todos_fallback);

    // Posts section
    metrics.record_section_start("posts");
    let (posts_html, posts_fallback) = match posts_res {
        Ok(posts) => {
            logger.debug_builder("Rendering posts section")
                .field_i64("count", posts.len() as i64)
                .emit();
            let items: String = posts
                .iter()
                .map(|p| format!("<li>{}</li>", escape_html(&p.title)))
                .collect();
            (format!("<h3>Posts</h3><ul>{}</ul>", items), false)
        }
        Err(e) => {
            logger.warn_builder("Posts fetch failed, using fallback")
                .field("error", e.to_string())
                .emit();
            (
                format!(
                    "<h3>Posts</h3><ul><li><b>Error:</b> {}</li></ul>",
                    escape_html(&e.to_string())
                ),
                true,
            )
        }
    };
    let _ = sink.send_section("posts", &posts_html).await;
    metrics.record_section_sent("posts", Some(posts_html.len()), posts_fallback);

    // Users section
    metrics.record_section_start("users");
    let (users_html, users_fallback) = match users_res {
        Ok(users) => {
            logger.debug_builder("Rendering users section")
                .field_i64("count", users.len() as i64)
                .emit();
            let items: String = users
                .iter()
                .map(|u| format!("<li>{}</li>", escape_html(&u.name)))
                .collect();
            (format!("<h3>Users</h3><ul>{}</ul>", items), false)
        }
        Err(e) => {
            logger.warn_builder("Users fetch failed, using fallback")
                .field("error", e.to_string())
                .emit();
            (
                format!(
                    "<h3>Users</h3><ul><li><b>Error:</b> {}</li></ul>",
                    escape_html(&e.to_string())
                ),
                true,
            )
        }
    };
    let _ = sink.send_section("users", &users_html).await;
    metrics.record_section_sent("users", Some(users_html.len()), users_fallback);

    // 6) Send shell closing
    let _ = sink.send_raw(shell.render_closing().into_bytes()).await;

    // Finalize and log metrics
    let final_metrics = metrics.finalize(Some(200));
    logger.info("Request complete");
    eprintln!("\n{}", final_metrics.to_summary());
}

/// Record fetch metrics for a dependency.
fn record_fetch_metric<T>(
    metrics: &mut MetricsCollector,
    tag: &str,
    url: &str,
    result: &Result<T, edge_sdk::edge_data::FetchError>,
    duration: std::time::Duration,
) {
    let (success, error) = match result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    metrics.record_dependency(
        tag,
        url,
        duration,
        None, // status_code not available from FetchError
        None, // response_bytes not tracked
        false,
        0,
        success,
        error,
    );
}

/// HTML escape to prevent XSS.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
