//! Resource limits for workload execution.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

/// Resource limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum response body size in bytes.
    pub max_response_bytes: u64,
    /// Maximum request body size in bytes.
    pub max_request_bytes: u64,
    /// Maximum number of concurrent outbound fetches.
    pub max_concurrent_fetches: u32,
    /// Maximum total outbound fetches per request.
    pub max_total_fetches: u32,
    /// Maximum size of a single fetch response.
    pub max_fetch_response_bytes: u64,
    /// Maximum total bytes fetched per request.
    pub max_total_fetch_bytes: u64,
    /// Maximum number of response headers.
    pub max_response_headers: u32,
    /// Maximum size of a single header value.
    pub max_header_value_bytes: u32,
    /// Maximum URL length for outbound requests.
    pub max_url_length: u32,
    /// Rate limit: requests per second.
    pub rate_limit_rps: Option<u32>,
    /// Rate limit: burst size.
    pub rate_limit_burst: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_response_bytes: 10 * 1024 * 1024,      // 10 MB
            max_request_bytes: 1 * 1024 * 1024,        // 1 MB
            max_concurrent_fetches: 10,
            max_total_fetches: 50,
            max_fetch_response_bytes: 5 * 1024 * 1024, // 5 MB per fetch
            max_total_fetch_bytes: 50 * 1024 * 1024,   // 50 MB total
            max_response_headers: 100,
            max_header_value_bytes: 8 * 1024,          // 8 KB
            max_url_length: 2048,
            rate_limit_rps: None,
            rate_limit_burst: None,
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create strict limits for untrusted workloads.
    pub fn strict() -> Self {
        Self {
            max_response_bytes: 1 * 1024 * 1024,       // 1 MB
            max_request_bytes: 256 * 1024,             // 256 KB
            max_concurrent_fetches: 3,
            max_total_fetches: 10,
            max_fetch_response_bytes: 512 * 1024,      // 512 KB per fetch
            max_total_fetch_bytes: 5 * 1024 * 1024,    // 5 MB total
            max_response_headers: 50,
            max_header_value_bytes: 4 * 1024,          // 4 KB
            max_url_length: 1024,
            rate_limit_rps: Some(10),
            rate_limit_burst: Some(20),
        }
    }

    /// Create permissive limits for development.
    pub fn development() -> Self {
        Self {
            max_response_bytes: 100 * 1024 * 1024,     // 100 MB
            max_request_bytes: 10 * 1024 * 1024,       // 10 MB
            max_concurrent_fetches: 50,
            max_total_fetches: 200,
            max_fetch_response_bytes: 50 * 1024 * 1024, // 50 MB per fetch
            max_total_fetch_bytes: 500 * 1024 * 1024,   // 500 MB total
            max_response_headers: 500,
            max_header_value_bytes: 64 * 1024,          // 64 KB
            max_url_length: 8192,
            rate_limit_rps: None,
            rate_limit_burst: None,
        }
    }

    /// Set maximum response body size in bytes.
    pub fn with_max_response_bytes(mut self, bytes: u64) -> Self {
        self.max_response_bytes = bytes;
        self
    }

    /// Set maximum response body size in megabytes.
    pub fn with_max_response_mb(mut self, mb: u64) -> Self {
        self.max_response_bytes = mb * 1024 * 1024;
        self
    }

    /// Set maximum request body size.
    pub fn with_max_request_bytes(mut self, bytes: u64) -> Self {
        self.max_request_bytes = bytes;
        self
    }

    /// Set maximum concurrent fetches.
    pub fn with_max_concurrent_fetches(mut self, max: u32) -> Self {
        self.max_concurrent_fetches = max;
        self
    }

    /// Set maximum total fetches per request.
    pub fn with_max_total_fetches(mut self, max: u32) -> Self {
        self.max_total_fetches = max;
        self
    }

    /// Set maximum fetch response size.
    pub fn with_max_fetch_response_bytes(mut self, bytes: u64) -> Self {
        self.max_fetch_response_bytes = bytes;
        self
    }

    /// Set maximum total fetch bytes.
    pub fn with_max_total_fetch_bytes(mut self, bytes: u64) -> Self {
        self.max_total_fetch_bytes = bytes;
        self
    }

    /// Set rate limit (requests per second).
    pub fn with_rate_limit(mut self, rps: u32, burst: u32) -> Self {
        self.rate_limit_rps = Some(rps);
        self.rate_limit_burst = Some(burst);
        self
    }

    /// Validate the limits configuration.
    pub fn validate(&self) -> Result<(), LimitsError> {
        if self.max_response_bytes == 0 {
            return Err(LimitsError::InvalidLimit("max_response_bytes cannot be 0".into()));
        }
        if self.max_concurrent_fetches == 0 {
            return Err(LimitsError::InvalidLimit("max_concurrent_fetches cannot be 0".into()));
        }
        if self.max_fetch_response_bytes > self.max_total_fetch_bytes {
            return Err(LimitsError::InvalidLimit(
                "max_fetch_response_bytes cannot exceed max_total_fetch_bytes".into(),
            ));
        }
        Ok(())
    }
}

/// Errors from limit violations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LimitsError {
    #[error("invalid limit configuration: {0}")]
    InvalidLimit(String),

    #[error("response size exceeded: {used} / {limit} bytes")]
    ResponseSizeExceeded { used: u64, limit: u64 },

    #[error("request size exceeded: {used} / {limit} bytes")]
    RequestSizeExceeded { used: u64, limit: u64 },

    #[error("concurrent fetch limit exceeded: {current} / {limit}")]
    ConcurrentFetchExceeded { current: u32, limit: u32 },

    #[error("total fetch limit exceeded: {count} / {limit}")]
    TotalFetchExceeded { count: u32, limit: u32 },

    #[error("fetch response size exceeded: {size} / {limit} bytes")]
    FetchResponseSizeExceeded { size: u64, limit: u64 },

    #[error("total fetch bytes exceeded: {used} / {limit} bytes")]
    TotalFetchBytesExceeded { used: u64, limit: u64 },

    #[error("too many response headers: {count} / {limit}")]
    TooManyHeaders { count: u32, limit: u32 },

    #[error("header value too large: {size} / {limit} bytes")]
    HeaderValueTooLarge { size: u32, limit: u32 },

    #[error("URL too long: {length} / {limit} characters")]
    UrlTooLong { length: u32, limit: u32 },

    #[error("rate limit exceeded")]
    RateLimitExceeded,
}

/// Tracker for resource usage during request handling.
#[derive(Debug, Default)]
pub struct ResourceTracker {
    response_bytes: AtomicU64,
    request_bytes: AtomicU64,
    concurrent_fetches: AtomicUsize,
    total_fetches: AtomicUsize,
    total_fetch_bytes: AtomicU64,
    header_count: AtomicUsize,
}

impl ResourceTracker {
    /// Create a new resource tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add response bytes.
    pub fn add_response_bytes(&self, bytes: u64) -> u64 {
        self.response_bytes.fetch_add(bytes, Ordering::SeqCst) + bytes
    }

    /// Get current response bytes.
    pub fn response_bytes(&self) -> u64 {
        self.response_bytes.load(Ordering::SeqCst)
    }

    /// Check response size against limit.
    pub fn check_response_size(&self, limits: &ResourceLimits) -> Result<(), LimitsError> {
        let used = self.response_bytes();
        if used > limits.max_response_bytes {
            return Err(LimitsError::ResponseSizeExceeded {
                used,
                limit: limits.max_response_bytes,
            });
        }
        Ok(())
    }

    /// Add request bytes.
    pub fn add_request_bytes(&self, bytes: u64) -> u64 {
        self.request_bytes.fetch_add(bytes, Ordering::SeqCst) + bytes
    }

    /// Check request size against limit.
    pub fn check_request_size(&self, limits: &ResourceLimits) -> Result<(), LimitsError> {
        let used = self.request_bytes.load(Ordering::SeqCst);
        if used > limits.max_request_bytes {
            return Err(LimitsError::RequestSizeExceeded {
                used,
                limit: limits.max_request_bytes,
            });
        }
        Ok(())
    }

    /// Start a fetch (increment concurrent counter).
    pub fn start_fetch(&self, limits: &ResourceLimits) -> Result<FetchGuard<'_>, LimitsError> {
        let current = self.concurrent_fetches.fetch_add(1, Ordering::SeqCst) + 1;
        if current > limits.max_concurrent_fetches as usize {
            self.concurrent_fetches.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitsError::ConcurrentFetchExceeded {
                current: current as u32,
                limit: limits.max_concurrent_fetches,
            });
        }

        let total = self.total_fetches.fetch_add(1, Ordering::SeqCst) + 1;
        if total > limits.max_total_fetches as usize {
            self.concurrent_fetches.fetch_sub(1, Ordering::SeqCst);
            self.total_fetches.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitsError::TotalFetchExceeded {
                count: total as u32,
                limit: limits.max_total_fetches,
            });
        }

        Ok(FetchGuard { tracker: self })
    }

    /// Add fetch response bytes.
    pub fn add_fetch_bytes(&self, bytes: u64, limits: &ResourceLimits) -> Result<(), LimitsError> {
        if bytes > limits.max_fetch_response_bytes {
            return Err(LimitsError::FetchResponseSizeExceeded {
                size: bytes,
                limit: limits.max_fetch_response_bytes,
            });
        }

        let total = self.total_fetch_bytes.fetch_add(bytes, Ordering::SeqCst) + bytes;
        if total > limits.max_total_fetch_bytes {
            return Err(LimitsError::TotalFetchBytesExceeded {
                used: total,
                limit: limits.max_total_fetch_bytes,
            });
        }

        Ok(())
    }

    /// Check URL length.
    pub fn check_url_length(&self, url: &str, limits: &ResourceLimits) -> Result<(), LimitsError> {
        let length = url.len() as u32;
        if length > limits.max_url_length {
            return Err(LimitsError::UrlTooLong {
                length,
                limit: limits.max_url_length,
            });
        }
        Ok(())
    }

    /// Check header value size.
    pub fn check_header_value(
        &self,
        value: &str,
        limits: &ResourceLimits,
    ) -> Result<(), LimitsError> {
        let size = value.len() as u32;
        if size > limits.max_header_value_bytes {
            return Err(LimitsError::HeaderValueTooLarge {
                size,
                limit: limits.max_header_value_bytes,
            });
        }
        Ok(())
    }

    /// Add a response header.
    pub fn add_header(&self, limits: &ResourceLimits) -> Result<(), LimitsError> {
        let count = self.header_count.fetch_add(1, Ordering::SeqCst) + 1;
        if count > limits.max_response_headers as usize {
            return Err(LimitsError::TooManyHeaders {
                count: count as u32,
                limit: limits.max_response_headers,
            });
        }
        Ok(())
    }

    /// Get usage summary.
    pub fn summary(&self) -> ResourceUsageSummary {
        ResourceUsageSummary {
            response_bytes: self.response_bytes.load(Ordering::SeqCst),
            request_bytes: self.request_bytes.load(Ordering::SeqCst),
            concurrent_fetches: self.concurrent_fetches.load(Ordering::SeqCst) as u32,
            total_fetches: self.total_fetches.load(Ordering::SeqCst) as u32,
            total_fetch_bytes: self.total_fetch_bytes.load(Ordering::SeqCst),
            header_count: self.header_count.load(Ordering::SeqCst) as u32,
        }
    }
}

/// Guard that decrements concurrent fetch count when dropped.
pub struct FetchGuard<'a> {
    tracker: &'a ResourceTracker,
}

impl<'a> Drop for FetchGuard<'a> {
    fn drop(&mut self) {
        self.tracker.concurrent_fetches.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Summary of resource usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSummary {
    pub response_bytes: u64,
    pub request_bytes: u64,
    pub concurrent_fetches: u32,
    pub total_fetches: u32,
    pub total_fetch_bytes: u64,
    pub header_count: u32,
}

impl ResourceUsageSummary {
    /// Check if any limits were approached (> 80% usage).
    pub fn approaching_limits(&self, limits: &ResourceLimits) -> Vec<String> {
        let mut warnings = Vec::new();

        let response_pct = (self.response_bytes as f64 / limits.max_response_bytes as f64) * 100.0;
        if response_pct > 80.0 {
            warnings.push(format!("response_bytes at {:.1}%", response_pct));
        }

        let fetch_pct = (self.total_fetches as f64 / limits.max_total_fetches as f64) * 100.0;
        if fetch_pct > 80.0 {
            warnings.push(format!("total_fetches at {:.1}%", fetch_pct));
        }

        let fetch_bytes_pct =
            (self.total_fetch_bytes as f64 / limits.max_total_fetch_bytes as f64) * 100.0;
        if fetch_bytes_pct > 80.0 {
            warnings.push(format!("total_fetch_bytes at {:.1}%", fetch_bytes_pct));
        }

        warnings
    }
}

/// Simple token bucket rate limiter.
#[derive(Debug)]
pub struct RateLimiter {
    /// Tokens per second.
    rate: f64,
    /// Maximum burst size.
    burst: f64,
    /// Current tokens available.
    tokens: std::sync::Mutex<f64>,
    /// Last refill time.
    last_refill: std::sync::Mutex<std::time::Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(rate_per_second: u32, burst: u32) -> Self {
        Self {
            rate: rate_per_second as f64,
            burst: burst as f64,
            tokens: std::sync::Mutex::new(burst as f64),
            last_refill: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    /// Try to acquire a token. Returns true if allowed.
    pub fn try_acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        let mut last_refill = self.last_refill.lock().unwrap();

        // Refill tokens based on elapsed time
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        *tokens = (*tokens + elapsed * self.rate).min(self.burst);
        *last_refill = now;

        // Try to consume a token
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Check without consuming (for metrics).
    pub fn available_tokens(&self) -> f64 {
        let tokens = self.tokens.lock().unwrap();
        let last_refill = self.last_refill.lock().unwrap();

        let elapsed = std::time::Instant::now()
            .duration_since(*last_refill)
            .as_secs_f64();
        (*tokens + elapsed * self.rate).min(self.burst)
    }
}
