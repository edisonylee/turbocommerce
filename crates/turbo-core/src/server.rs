//! Server-side rendering utilities for Spin/WASI.
//!
//! This module provides the SSR infrastructure that integrates
//! Leptos with Spin's HTTP handler.

#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;

/// Generate the HTML shell for SSR.
///
/// This creates the initial HTML document that wraps the application.
/// It includes:
/// - DOCTYPE and HTML structure
/// - Meta tags and viewport
/// - CSS and JS links
/// - Hydration scripts
///
/// # Example
///
/// ```rust,ignore
/// #[cfg(feature = "ssr")]
/// pub fn shell(options: LeptosOptions) -> impl IntoView {
///     turbo_core::generate_shell(options, "my-app", "/pkg/style.css")
/// }
/// ```
#[cfg(feature = "ssr")]
pub fn generate_shell_html(app_name: &str, css_path: Option<&str>, body_html: &str) -> String {
    let css_link = css_path
        .map(|p| format!(r#"<link rel="stylesheet" href="{}">"#, p))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    {css_link}
    <title>{app_name}</title>
</head>
<body>
    {body_html}
</body>
</html>"#
    )
}

/// Server function registration helper.
///
/// In Leptos WASI, server functions must be registered explicitly.
/// This trait provides a way to collect and register them.
pub trait ServerFnRegistry {
    /// Register all server functions with the handler.
    fn register_server_fns<H>(handler: H) -> H;
}

/// Streaming response configuration.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for streaming chunks.
    pub buffer_size: usize,
    /// Flush after shell is sent.
    pub flush_after_shell: bool,
    /// Flush after each section.
    pub flush_after_section: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            flush_after_shell: true,
            flush_after_section: true,
        }
    }
}

impl StreamConfig {
    /// Create a new streaming configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the buffer size.
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable flush after shell.
    pub fn flush_shell(mut self, enabled: bool) -> Self {
        self.flush_after_shell = enabled;
        self
    }

    /// Enable or disable flush after each section.
    pub fn flush_sections(mut self, enabled: bool) -> Self {
        self.flush_after_section = enabled;
        self
    }
}
