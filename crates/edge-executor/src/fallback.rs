//! Fallback strategies for section failures.

/// What to do when a section's dependencies fail.
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Render fallback HTML.
    RenderFallback(String),

    /// Skip the section entirely.
    Skip,

    /// Show an error message.
    ShowError,

    /// Use cached version if available.
    UseCached,

    /// Retry with degraded data.
    RetryDegraded,
}

impl FallbackStrategy {
    /// Create a fallback that renders custom HTML.
    pub fn html(html: impl Into<String>) -> Self {
        Self::RenderFallback(html.into())
    }

    /// Create a fallback that shows a user-friendly error.
    pub fn error_message(message: impl Into<String>) -> Self {
        Self::RenderFallback(format!(
            r#"<div class="section-error">{}</div>"#,
            message.into()
        ))
    }
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        Self::Skip
    }
}

/// Configuration for a section's fallback behavior.
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// The fallback strategy.
    pub strategy: FallbackStrategy,
    /// Whether to log the failure.
    pub log_failure: bool,
    /// Timeout before triggering fallback (if different from section timeout).
    pub fallback_timeout_ms: Option<u64>,
}

impl FallbackConfig {
    /// Create a new fallback configuration.
    pub fn new(strategy: FallbackStrategy) -> Self {
        Self {
            strategy,
            log_failure: true,
            fallback_timeout_ms: None,
        }
    }

    /// Set whether to log failures.
    pub fn with_logging(mut self, log: bool) -> Self {
        self.log_failure = log;
        self
    }

    /// Set a custom fallback timeout.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.fallback_timeout_ms = Some(ms);
        self
    }
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self::new(FallbackStrategy::default())
    }
}

/// Result of applying a fallback.
#[derive(Debug)]
pub enum FallbackResult {
    /// Rendered fallback HTML.
    Rendered(String),
    /// Section was skipped.
    Skipped,
    /// No fallback available, section failed.
    Failed(String),
}

/// Apply fallback strategy to get a result.
pub fn apply_fallback(config: &FallbackConfig, error: &str) -> FallbackResult {
    match &config.strategy {
        FallbackStrategy::RenderFallback(html) => FallbackResult::Rendered(html.clone()),
        FallbackStrategy::Skip => FallbackResult::Skipped,
        FallbackStrategy::ShowError => {
            FallbackResult::Rendered(format!(
                r#"<div class="section-error">Failed to load section: {}</div>"#,
                html_escape(error)
            ))
        }
        FallbackStrategy::UseCached => {
            // TODO: Integrate with cache layer
            FallbackResult::Failed("Cache not available".to_string())
        }
        FallbackStrategy::RetryDegraded => {
            // TODO: Implement degraded retry
            FallbackResult::Failed("Degraded retry not implemented".to_string())
        }
    }
}

/// Simple HTML escape for error messages.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
