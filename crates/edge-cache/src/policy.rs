//! Route and section-level cache policies.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Cache scope determining who can cache the response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheScope {
    /// Cacheable by CDN and browser (shared cache).
    Public,
    /// Cacheable by browser only (private cache).
    Private,
    /// Shared by same user session (e.g., personalized but stable).
    SharedPrivate,
    /// No caching.
    #[default]
    None,
}

impl CacheScope {
    /// Get the Cache-Control directive for this scope.
    pub fn cache_control_directive(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::SharedPrivate => "private",
            Self::None => "no-store",
        }
    }

    /// Check if this scope allows any caching.
    pub fn allows_caching(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this scope allows CDN caching.
    pub fn allows_cdn_caching(&self) -> bool {
        matches!(self, Self::Public)
    }
}

/// Vary rule for cache key variance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum VaryRule {
    /// Vary by HTTP header.
    Header(String),
    /// Vary by cookie.
    Cookie(String),
    /// Vary by query parameter.
    QueryParam(String),
    /// Vary by geographic region.
    Geo(GeoGranularity),
    /// Vary by device type.
    DeviceType,
    /// Vary by authenticated user.
    UserId,
    /// Custom vary key.
    Custom(String),
}

impl VaryRule {
    /// Create a header vary rule.
    pub fn header(name: impl Into<String>) -> Self {
        Self::Header(name.into())
    }

    /// Create a cookie vary rule.
    pub fn cookie(name: impl Into<String>) -> Self {
        Self::Cookie(name.into())
    }

    /// Create a query param vary rule.
    pub fn query(name: impl Into<String>) -> Self {
        Self::QueryParam(name.into())
    }

    /// Create a geo vary rule.
    pub fn geo(granularity: GeoGranularity) -> Self {
        Self::Geo(granularity)
    }
}

/// Geographic granularity for cache variance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeoGranularity {
    /// Vary by country.
    Country,
    /// Vary by region/state.
    Region,
    /// Vary by city.
    City,
}

/// Route-level cache policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteCachePolicy {
    /// Whether caching is enabled.
    pub enabled: bool,
    /// Cache scope.
    pub scope: CacheScope,
    /// Time-to-live for cached responses.
    pub ttl: Duration,
    /// Stale-while-revalidate window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_while_revalidate: Option<Duration>,
    /// Stale-if-error window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_if_error: Option<Duration>,
    /// Vary rules for cache key.
    pub vary: Vec<VaryRule>,
    /// Whether personalization requires explicit opt-in.
    pub personalization_opt_in: bool,
    /// Custom cache tags for invalidation.
    pub tags: Vec<String>,
}

impl Default for RouteCachePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            scope: CacheScope::None,
            ttl: Duration::from_secs(0),
            stale_while_revalidate: None,
            stale_if_error: None,
            vary: Vec::new(),
            personalization_opt_in: true,
            tags: Vec::new(),
        }
    }
}

impl RouteCachePolicy {
    /// Create a new cache policy with no caching.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a public cache policy.
    pub fn public(ttl: Duration) -> Self {
        Self {
            enabled: true,
            scope: CacheScope::Public,
            ttl,
            ..Default::default()
        }
    }

    /// Create a private cache policy.
    pub fn private(ttl: Duration) -> Self {
        Self {
            enabled: true,
            scope: CacheScope::Private,
            ttl,
            ..Default::default()
        }
    }

    /// Set stale-while-revalidate window.
    pub fn with_swr(mut self, duration: Duration) -> Self {
        self.stale_while_revalidate = Some(duration);
        self
    }

    /// Set stale-if-error window.
    pub fn with_stale_if_error(mut self, duration: Duration) -> Self {
        self.stale_if_error = Some(duration);
        self
    }

    /// Add a vary rule.
    pub fn vary_on(mut self, rule: VaryRule) -> Self {
        self.vary.push(rule);
        self
    }

    /// Add multiple vary rules.
    pub fn vary_on_all(mut self, rules: Vec<VaryRule>) -> Self {
        self.vary.extend(rules);
        self
    }

    /// Add a cache tag for invalidation.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Disable personalization opt-in requirement.
    pub fn allow_personalization(mut self) -> Self {
        self.personalization_opt_in = false;
        self
    }

    /// Generate Cache-Control header value.
    pub fn cache_control_header(&self) -> String {
        if !self.enabled || self.scope == CacheScope::None {
            return "no-store".to_string();
        }

        let mut parts = vec![self.scope.cache_control_directive().to_string()];

        parts.push(format!("max-age={}", self.ttl.as_secs()));

        if let Some(swr) = self.stale_while_revalidate {
            parts.push(format!("stale-while-revalidate={}", swr.as_secs()));
        }

        if let Some(sie) = self.stale_if_error {
            parts.push(format!("stale-if-error={}", sie.as_secs()));
        }

        parts.join(", ")
    }

    /// Generate Vary header value.
    pub fn vary_header(&self) -> Option<String> {
        if self.vary.is_empty() {
            return None;
        }

        let headers: Vec<String> = self
            .vary
            .iter()
            .filter_map(|v| match v {
                VaryRule::Header(h) => Some(h.clone()),
                VaryRule::Cookie(_) => Some("Cookie".to_string()),
                VaryRule::DeviceType => Some("User-Agent".to_string()),
                _ => None,
            })
            .collect();

        if headers.is_empty() {
            None
        } else {
            Some(headers.join(", "))
        }
    }
}

/// Section-level cache policy (for fragment caching).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionCachePolicy {
    /// Whether fragment caching is enabled for this section.
    pub enabled: bool,
    /// Time-to-live for cached fragment.
    pub ttl: Duration,
    /// Whether to use stale cache on error.
    pub stale_on_error: bool,
    /// Vary rules specific to this section.
    pub vary: Vec<VaryRule>,
    /// Cache tags for this section.
    pub tags: Vec<String>,
}

impl Default for SectionCachePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            ttl: Duration::from_secs(0),
            stale_on_error: false,
            vary: Vec::new(),
            tags: Vec::new(),
        }
    }
}

impl SectionCachePolicy {
    /// Create a new section cache policy.
    pub fn new(ttl: Duration) -> Self {
        Self {
            enabled: true,
            ttl,
            ..Default::default()
        }
    }

    /// Create a disabled section cache policy.
    pub fn none() -> Self {
        Self::default()
    }

    /// Enable stale-on-error behavior.
    pub fn with_stale_on_error(mut self) -> Self {
        self.stale_on_error = true;
        self
    }

    /// Add a vary rule.
    pub fn vary_on(mut self, rule: VaryRule) -> Self {
        self.vary.push(rule);
        self
    }

    /// Add a cache tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}
