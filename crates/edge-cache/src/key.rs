//! Cache key composition.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::policy::VaryRule;

/// A cache key uniquely identifying a cached response.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// The computed key string.
    key: String,
    /// Components that make up the key (for debugging).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<String>,
}

impl CacheKey {
    /// Create a cache key from a string.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            components: Vec::new(),
        }
    }

    /// Get the key string.
    pub fn as_str(&self) -> &str {
        &self.key
    }

    /// Get the key components (for debugging).
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

/// Context for building cache keys.
#[derive(Debug, Clone, Default)]
pub struct CacheKeyContext {
    /// Route path.
    pub path: String,
    /// Query parameters.
    pub query_params: BTreeMap<String, String>,
    /// HTTP headers.
    pub headers: BTreeMap<String, String>,
    /// Cookies.
    pub cookies: BTreeMap<String, String>,
    /// User ID (if authenticated).
    pub user_id: Option<String>,
    /// Geographic info.
    pub geo: Option<GeoContext>,
    /// Device type.
    pub device_type: Option<DeviceType>,
}

/// Geographic context.
#[derive(Debug, Clone, Default)]
pub struct GeoContext {
    /// Country code.
    pub country: Option<String>,
    /// Region/state.
    pub region: Option<String>,
    /// City.
    pub city: Option<String>,
}

/// Device type for cache variance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Bot,
    Unknown,
}

impl DeviceType {
    /// Detect device type from User-Agent header.
    pub fn from_user_agent(ua: &str) -> Self {
        let ua_lower = ua.to_lowercase();

        if ua_lower.contains("bot")
            || ua_lower.contains("crawler")
            || ua_lower.contains("spider")
        {
            return Self::Bot;
        }

        if ua_lower.contains("mobile") || ua_lower.contains("android") {
            if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
                return Self::Tablet;
            }
            return Self::Mobile;
        }

        if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            return Self::Tablet;
        }

        Self::Desktop
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Mobile => write!(f, "mobile"),
            Self::Tablet => write!(f, "tablet"),
            Self::Bot => write!(f, "bot"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Component of a cache key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyComponent {
    /// The route path.
    Route,
    /// Specific query parameters.
    QueryParams(Vec<String>),
    /// All query parameters.
    AllQueryParams,
    /// Specific header.
    Header(String),
    /// Specific cookie.
    Cookie(String),
    /// User ID.
    UserId,
    /// Country code.
    Country,
    /// Region code.
    Region,
    /// City.
    City,
    /// Device type.
    DeviceType,
    /// Custom static value.
    Custom(String),
}

/// Builder for composing cache keys.
#[derive(Debug, Clone, Default)]
pub struct CacheKeyBuilder {
    components: Vec<KeyComponent>,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl CacheKeyBuilder {
    /// Create a new cache key builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a prefix for the cache key.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set a suffix for the cache key.
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Include the route path in the key.
    pub fn route(mut self) -> Self {
        self.components.push(KeyComponent::Route);
        self
    }

    /// Include specific query parameters.
    pub fn query_params(mut self, params: &[&str]) -> Self {
        self.components.push(KeyComponent::QueryParams(
            params.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Include all query parameters.
    pub fn all_query_params(mut self) -> Self {
        self.components.push(KeyComponent::AllQueryParams);
        self
    }

    /// Include a header value.
    pub fn header(mut self, name: impl Into<String>) -> Self {
        self.components.push(KeyComponent::Header(name.into()));
        self
    }

    /// Include a cookie value.
    pub fn cookie(mut self, name: impl Into<String>) -> Self {
        self.components.push(KeyComponent::Cookie(name.into()));
        self
    }

    /// Include user ID.
    pub fn user_id(mut self) -> Self {
        self.components.push(KeyComponent::UserId);
        self
    }

    /// Include country.
    pub fn country(mut self) -> Self {
        self.components.push(KeyComponent::Country);
        self
    }

    /// Include region.
    pub fn region(mut self) -> Self {
        self.components.push(KeyComponent::Region);
        self
    }

    /// Include city.
    pub fn city(mut self) -> Self {
        self.components.push(KeyComponent::City);
        self
    }

    /// Include device type.
    pub fn device_type(mut self) -> Self {
        self.components.push(KeyComponent::DeviceType);
        self
    }

    /// Include a custom static value.
    pub fn custom(mut self, value: impl Into<String>) -> Self {
        self.components.push(KeyComponent::Custom(value.into()));
        self
    }

    /// Build from vary rules.
    pub fn from_vary_rules(rules: &[VaryRule]) -> Self {
        let mut builder = Self::new().route();

        for rule in rules {
            builder = match rule {
                VaryRule::Header(h) => builder.header(h),
                VaryRule::Cookie(c) => builder.cookie(c),
                VaryRule::QueryParam(q) => builder.query_params(&[q.as_str()]),
                VaryRule::Geo(g) => match g {
                    crate::policy::GeoGranularity::Country => builder.country(),
                    crate::policy::GeoGranularity::Region => builder.region(),
                    crate::policy::GeoGranularity::City => builder.city(),
                },
                VaryRule::DeviceType => builder.device_type(),
                VaryRule::UserId => builder.user_id(),
                VaryRule::Custom(c) => builder.custom(c),
            };
        }

        builder
    }

    /// Build the cache key from context.
    pub fn build(&self, ctx: &CacheKeyContext) -> CacheKey {
        let mut parts = Vec::new();
        let mut component_descs = Vec::new();

        if let Some(prefix) = &self.prefix {
            parts.push(prefix.clone());
        }

        for component in &self.components {
            match component {
                KeyComponent::Route => {
                    parts.push(ctx.path.clone());
                    component_descs.push(format!("route:{}", ctx.path));
                }
                KeyComponent::QueryParams(params) => {
                    for param in params {
                        if let Some(value) = ctx.query_params.get(param) {
                            parts.push(format!("{}={}", param, value));
                            component_descs.push(format!("query:{}={}", param, value));
                        }
                    }
                }
                KeyComponent::AllQueryParams => {
                    for (k, v) in &ctx.query_params {
                        parts.push(format!("{}={}", k, v));
                        component_descs.push(format!("query:{}={}", k, v));
                    }
                }
                KeyComponent::Header(name) => {
                    if let Some(value) = ctx.headers.get(&name.to_lowercase()) {
                        parts.push(format!("h:{}={}", name, value));
                        component_descs.push(format!("header:{}", name));
                    }
                }
                KeyComponent::Cookie(name) => {
                    if let Some(value) = ctx.cookies.get(name) {
                        parts.push(format!("c:{}={}", name, value));
                        component_descs.push(format!("cookie:{}", name));
                    }
                }
                KeyComponent::UserId => {
                    if let Some(user_id) = &ctx.user_id {
                        parts.push(format!("u:{}", user_id));
                        component_descs.push("user_id".to_string());
                    }
                }
                KeyComponent::Country => {
                    if let Some(geo) = &ctx.geo {
                        if let Some(country) = &geo.country {
                            parts.push(format!("geo:c:{}", country));
                            component_descs.push(format!("country:{}", country));
                        }
                    }
                }
                KeyComponent::Region => {
                    if let Some(geo) = &ctx.geo {
                        if let Some(region) = &geo.region {
                            parts.push(format!("geo:r:{}", region));
                            component_descs.push(format!("region:{}", region));
                        }
                    }
                }
                KeyComponent::City => {
                    if let Some(geo) = &ctx.geo {
                        if let Some(city) = &geo.city {
                            parts.push(format!("geo:city:{}", city));
                            component_descs.push(format!("city:{}", city));
                        }
                    }
                }
                KeyComponent::DeviceType => {
                    if let Some(device) = &ctx.device_type {
                        parts.push(format!("d:{}", device));
                        component_descs.push(format!("device:{}", device));
                    }
                }
                KeyComponent::Custom(value) => {
                    parts.push(value.clone());
                    component_descs.push(format!("custom:{}", value));
                }
            }
        }

        if let Some(suffix) = &self.suffix {
            parts.push(suffix.clone());
        }

        // Join with a separator and hash for consistent length
        let key_string = parts.join("|");
        let key = format!("{:x}", simple_hash(&key_string));

        CacheKey {
            key,
            components: component_descs,
        }
    }
}

/// Fragment cache key (includes section name).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FragmentKey {
    /// Section name.
    pub section: String,
    /// Base cache key.
    pub cache_key: CacheKey,
}

impl FragmentKey {
    /// Create a new fragment key.
    pub fn new(section: impl Into<String>, cache_key: CacheKey) -> Self {
        Self {
            section: section.into(),
            cache_key,
        }
    }

    /// Get the full key string.
    pub fn as_str(&self) -> String {
        format!("{}:{}", self.section, self.cache_key.as_str())
    }
}

impl std::fmt::Display for FragmentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Simple non-cryptographic hash for cache keys
fn simple_hash(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
