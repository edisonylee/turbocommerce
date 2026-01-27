//! Workload definition and trait.

use serde::{Deserialize, Serialize};

use crate::config::RouteConfig;

/// Workload manifest - explicit configuration for a deployable unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadManifest {
    /// Unique name for this workload.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Routes this workload handles.
    pub routes: Vec<RouteConfig>,
}

impl WorkloadManifest {
    /// Create a new workload manifest.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            routes: Vec::new(),
        }
    }

    /// Add a route to this workload.
    pub fn with_route(mut self, route: RouteConfig) -> Self {
        self.routes.push(route);
        self
    }
}

/// Error type for workload operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkloadError {
    #[error("Shell not sent before sections")]
    ShellNotSent,

    #[error("Streaming error: {0}")]
    StreamError(String),

    #[error("Fetch error: {0}")]
    FetchError(#[from] anyhow::Error),

    #[error("Section '{0}' failed: {1}")]
    SectionFailed(String, String),
}
