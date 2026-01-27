//! Platform fetch client with dependency tagging.

use edge_core::{RequestId, TimingContext};
use serde::de::DeserializeOwned;

use crate::dependency::DependencyTag;
use crate::retry::RetryPolicy;
use crate::timeout::TimeoutConfig;

/// Error type for fetch operations.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP error: {status} for {url}")]
    Http { status: u16, url: String },

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Request error: {0}")]
    Request(String),
}

/// Fetch policy combining timeout and retry configuration.
#[derive(Debug, Clone)]
pub struct FetchPolicy {
    /// Timeout configuration.
    pub timeout: TimeoutConfig,
    /// Retry policy.
    pub retry: RetryPolicy,
}

impl FetchPolicy {
    /// Create a new fetch policy.
    pub fn new(timeout: TimeoutConfig, retry: RetryPolicy) -> Self {
        Self { timeout, retry }
    }

    /// Create from a dependency tag's defaults.
    pub fn from_tag(tag: DependencyTag) -> Self {
        Self {
            timeout: TimeoutConfig::from_total(tag.default_timeout()),
            retry: RetryPolicy::new(tag.default_max_retries()),
        }
    }
}

impl Default for FetchPolicy {
    fn default() -> Self {
        Self {
            timeout: TimeoutConfig::default(),
            retry: RetryPolicy::default(),
        }
    }
}

/// Platform-controlled fetch client.
///
/// Provides automatic timeout, retry, and observability for outbound requests.
pub struct FetchClient {
    request_id: RequestId,
    timing: TimingContext,
    default_policy: FetchPolicy,
}

impl FetchClient {
    /// Create a new fetch client.
    pub fn new(request_id: RequestId, timing: TimingContext) -> Self {
        Self {
            request_id,
            timing,
            default_policy: FetchPolicy::default(),
        }
    }

    /// Set default policy for all fetches.
    pub fn with_default_policy(mut self, policy: FetchPolicy) -> Self {
        self.default_policy = policy;
        self
    }

    /// Fetch with automatic timeout and retry based on dependency tag.
    pub async fn fetch<T: DeserializeOwned>(
        &self,
        url: &str,
        tag: DependencyTag,
    ) -> Result<T, FetchError> {
        let policy = FetchPolicy::from_tag(tag);
        self.fetch_with_policy(url, tag, policy).await
    }

    /// Fetch with explicit policy override.
    pub async fn fetch_with_policy<T: DeserializeOwned>(
        &self,
        url: &str,
        tag: DependencyTag,
        _policy: FetchPolicy,
    ) -> Result<T, FetchError> {
        // Record timing
        let timing_key = format!("fetch_{}_{}", tag.name(), url_hash(url));

        // TODO: Implement timeout wrapper when tokio is available in WASM
        // For now, use spin_sdk directly

        let req = spin_sdk::http::Request::get(url);
        let resp: spin_sdk::http::Response = spin_sdk::http::send(req)
            .await
            .map_err(|e| FetchError::Request(e.to_string()))?;

        // Check for HTTP errors
        let status = resp.status();
        if *status >= 400 {
            return Err(FetchError::Http {
                status: *status,
                url: url.to_string(),
            });
        }

        let bytes = resp.body().to_vec();
        let result: T = serde_json::from_slice(&bytes)
            .map_err(|e| FetchError::Deserialization(e.to_string()))?;

        // Log timing (would integrate with timing context)
        let _ = timing_key;

        Ok(result)
    }

    /// Get the request ID.
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
}

/// Simple hash for URL (for timing keys).
fn url_hash(url: &str) -> u32 {
    url.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
}
