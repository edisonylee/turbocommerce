//! Route and workload configuration.

use serde::{Deserialize, Serialize};

/// Configuration for a single route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route pattern (e.g., "/products/:id").
    pub pattern: String,
    /// Handler function name.
    pub handler: String,
    /// HTTP methods this route accepts.
    #[serde(default = "default_methods")]
    pub methods: Vec<String>,
}

fn default_methods() -> Vec<String> {
    vec!["GET".to_string()]
}

impl RouteConfig {
    /// Create a new route configuration.
    pub fn new(pattern: impl Into<String>, handler: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            handler: handler.into(),
            methods: default_methods(),
        }
    }

    /// Set allowed HTTP methods.
    pub fn with_methods(mut self, methods: Vec<&str>) -> Self {
        self.methods = methods.into_iter().map(String::from).collect();
        self
    }
}

// Note: CacheScope is now defined in edge-cache crate with more variants
