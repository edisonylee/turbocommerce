//! Request context with typed parameters.

use std::collections::HashMap;

use crate::lifecycle::TimingContext;

/// Unique request identifier for tracing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(pub String);

impl RequestId {
    /// Generate a new request ID.
    pub fn generate() -> Self {
        // Simple UUID-like generation
        let id = format!(
            "{:x}-{:x}-{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            rand_simple(),
            rand_simple()
        );
        Self(id)
    }

    /// Create from an existing ID string.
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

fn rand_simple() -> u32 {
    // Simple pseudo-random for WASM (no std::random)
    static mut SEED: u32 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Extracted route parameters (e.g., `:id` from `/products/:id`).
pub type RouteParams = HashMap<String, String>;

/// Query string parameters.
pub type QueryParams = HashMap<String, String>;

/// HTTP headers.
pub type Headers = HashMap<String, String>;

/// Geographic information from edge location.
#[derive(Debug, Clone, Default)]
pub struct GeoInfo {
    /// ISO country code (e.g., "US").
    pub country: Option<String>,
    /// Region/state code.
    pub region: Option<String>,
    /// City name.
    pub city: Option<String>,
}

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Typed request context passed to workload handlers.
#[derive(Debug)]
pub struct RequestContext {
    /// Unique request identifier.
    pub request_id: RequestId,
    /// HTTP method.
    pub method: Method,
    /// Request path.
    pub path: String,
    /// Extracted route parameters.
    pub params: RouteParams,
    /// Query string parameters.
    pub query: QueryParams,
    /// HTTP headers.
    pub headers: Headers,
    /// Geographic information.
    pub geo: Option<GeoInfo>,
    /// Timing context for observability.
    pub timing: TimingContext,
}

impl RequestContext {
    /// Create a new request context.
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            request_id: RequestId::generate(),
            method,
            path: path.into(),
            params: HashMap::new(),
            query: HashMap::new(),
            headers: HashMap::new(),
            geo: None,
            timing: TimingContext::new(),
        }
    }

    /// Get a route parameter by name.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|s| s.as_str())
    }

    /// Get a query parameter by name.
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(|s| s.as_str())
    }

    /// Get a header value by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }
}
