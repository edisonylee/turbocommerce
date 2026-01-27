//! Cache debugging headers.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::fragment::CacheStatus;
use crate::key::CacheKey;
use crate::policy::RouteCachePolicy;

/// Header names for cache debugging.
pub mod header_names {
    /// Cache status header (HIT, MISS, STALE, BYPASS).
    pub const X_CACHE_STATUS: &str = "X-Cache-Status";
    /// Cache key used for lookup.
    pub const X_CACHE_KEY: &str = "X-Cache-Key";
    /// Cache age in seconds.
    pub const X_CACHE_AGE: &str = "X-Cache-Age";
    /// Cache TTL remaining.
    pub const X_CACHE_TTL: &str = "X-Cache-TTL";
    /// Cache tags for invalidation.
    pub const X_CACHE_TAGS: &str = "X-Cache-Tags";
    /// Cache scope (public, private, none).
    pub const X_CACHE_SCOPE: &str = "X-Cache-Scope";
    /// Vary rules applied.
    pub const X_CACHE_VARY: &str = "X-Cache-Vary";
    /// Whether response was served from stale cache.
    pub const X_CACHE_STALE: &str = "X-Cache-Stale";
    /// Fragment cache statuses (for streaming).
    pub const X_FRAGMENT_CACHE: &str = "X-Fragment-Cache";
    /// Request ID for tracing.
    pub const X_REQUEST_ID: &str = "X-Request-ID";
    /// Time to first byte.
    pub const X_TTFB: &str = "X-TTFB";
}

/// Cache explain headers for debugging.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheExplainHeaders {
    /// Overall cache status.
    pub status: Option<CacheStatus>,
    /// Cache key used.
    pub cache_key: Option<String>,
    /// Age of cached response in seconds.
    pub age_secs: Option<u64>,
    /// Remaining TTL in seconds.
    pub ttl_secs: Option<u64>,
    /// Cache tags.
    pub tags: Vec<String>,
    /// Cache scope.
    pub scope: Option<String>,
    /// Vary rules applied.
    pub vary: Vec<String>,
    /// Whether response is stale.
    pub is_stale: bool,
    /// Per-fragment statuses.
    pub fragments: Vec<FragmentCacheInfo>,
}

/// Per-fragment cache information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCacheInfo {
    /// Section name.
    pub section: String,
    /// Cache status for this fragment.
    pub status: CacheStatus,
    /// Age if cached.
    pub age_secs: Option<u64>,
}

impl CacheExplainHeaders {
    /// Create new explain headers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set cache status.
    pub fn with_status(mut self, status: CacheStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Set cache key.
    pub fn with_key(mut self, key: &CacheKey) -> Self {
        self.cache_key = Some(key.as_str().to_string());
        self
    }

    /// Set age.
    pub fn with_age(mut self, age: Duration) -> Self {
        self.age_secs = Some(age.as_secs());
        self
    }

    /// Set TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_secs = Some(ttl.as_secs());
        self
    }

    /// Set tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set scope.
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Set vary rules.
    pub fn with_vary(mut self, vary: Vec<String>) -> Self {
        self.vary = vary;
        self
    }

    /// Mark as stale.
    pub fn mark_stale(mut self) -> Self {
        self.is_stale = true;
        self
    }

    /// Add fragment info.
    pub fn add_fragment(mut self, section: impl Into<String>, status: CacheStatus, age: Option<Duration>) -> Self {
        self.fragments.push(FragmentCacheInfo {
            section: section.into(),
            status,
            age_secs: age.map(|d| d.as_secs()),
        });
        self
    }

    /// Build from route cache policy.
    pub fn from_policy(policy: &RouteCachePolicy, key: &CacheKey) -> Self {
        let vary_rules: Vec<String> = policy
            .vary
            .iter()
            .map(|v| format!("{:?}", v))
            .collect();

        Self::new()
            .with_key(key)
            .with_tags(policy.tags.clone())
            .with_scope(policy.scope.cache_control_directive())
            .with_vary(vary_rules)
            .with_ttl(policy.ttl)
    }

    /// Convert to HTTP headers.
    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if let Some(status) = &self.status {
            headers.push((header_names::X_CACHE_STATUS.to_string(), status.to_string()));
        }

        if let Some(key) = &self.cache_key {
            headers.push((header_names::X_CACHE_KEY.to_string(), key.clone()));
        }

        if let Some(age) = self.age_secs {
            headers.push((header_names::X_CACHE_AGE.to_string(), age.to_string()));
        }

        if let Some(ttl) = self.ttl_secs {
            headers.push((header_names::X_CACHE_TTL.to_string(), ttl.to_string()));
        }

        if !self.tags.is_empty() {
            headers.push((header_names::X_CACHE_TAGS.to_string(), self.tags.join(", ")));
        }

        if let Some(scope) = &self.scope {
            headers.push((header_names::X_CACHE_SCOPE.to_string(), scope.clone()));
        }

        if !self.vary.is_empty() {
            headers.push((header_names::X_CACHE_VARY.to_string(), self.vary.join(", ")));
        }

        if self.is_stale {
            headers.push((header_names::X_CACHE_STALE.to_string(), "true".to_string()));
        }

        if !self.fragments.is_empty() {
            let fragment_info: Vec<String> = self
                .fragments
                .iter()
                .map(|f| format!("{}={}", f.section, f.status))
                .collect();
            headers.push((header_names::X_FRAGMENT_CACHE.to_string(), fragment_info.join(", ")));
        }

        headers
    }

    /// Convert to JSON for debugging endpoint.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Builder for cache response headers.
#[derive(Debug, Default)]
pub struct CacheHeadersBuilder {
    cache_control: Option<String>,
    vary: Option<String>,
    etag: Option<String>,
    age: Option<u64>,
    explain: Option<CacheExplainHeaders>,
    include_debug: bool,
}

impl CacheHeadersBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set Cache-Control header.
    pub fn cache_control(mut self, value: impl Into<String>) -> Self {
        self.cache_control = Some(value.into());
        self
    }

    /// Set Cache-Control from policy.
    pub fn cache_control_from_policy(mut self, policy: &RouteCachePolicy) -> Self {
        self.cache_control = Some(policy.cache_control_header());
        self
    }

    /// Set Vary header.
    pub fn vary(mut self, value: impl Into<String>) -> Self {
        self.vary = Some(value.into());
        self
    }

    /// Set Vary from policy.
    pub fn vary_from_policy(mut self, policy: &RouteCachePolicy) -> Self {
        self.vary = policy.vary_header();
        self
    }

    /// Set ETag header.
    pub fn etag(mut self, value: impl Into<String>) -> Self {
        self.etag = Some(value.into());
        self
    }

    /// Set Age header.
    pub fn age(mut self, seconds: u64) -> Self {
        self.age = Some(seconds);
        self
    }

    /// Set debug explain headers.
    pub fn explain(mut self, headers: CacheExplainHeaders) -> Self {
        self.explain = Some(headers);
        self
    }

    /// Enable debug headers in output.
    pub fn include_debug(mut self, enabled: bool) -> Self {
        self.include_debug = enabled;
        self
    }

    /// Build the headers.
    pub fn build(self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if let Some(cc) = self.cache_control {
            headers.push(("Cache-Control".to_string(), cc));
        }

        if let Some(vary) = self.vary {
            headers.push(("Vary".to_string(), vary));
        }

        if let Some(etag) = self.etag {
            headers.push(("ETag".to_string(), format!("\"{}\"", etag)));
        }

        if let Some(age) = self.age {
            headers.push(("Age".to_string(), age.to_string()));
        }

        if self.include_debug {
            if let Some(explain) = self.explain {
                headers.extend(explain.to_headers());
            }
        }

        headers
    }
}

/// Utility to check if debug headers should be included.
pub fn should_include_debug_headers(request_headers: &[(String, String)]) -> bool {
    request_headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("X-Debug-Cache") && value == "1"
    })
}

/// Generate a simple ETag from content.
pub fn generate_etag(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
