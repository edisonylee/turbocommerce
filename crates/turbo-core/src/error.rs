//! Error types for TurboCore.

use thiserror::Error;

/// Errors that can occur in TurboCore.
#[derive(Error, Debug)]
pub enum TurboError {
    /// Route not found.
    #[error("Route not found: {0}")]
    RouteNotFound(String),

    /// Server function error.
    #[error("Server function error: {0}")]
    ServerFnError(String),

    /// Streaming error.
    #[error("Streaming error: {0}")]
    StreamError(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Render error.
    #[error("Render error: {0}")]
    RenderError(String),
}

impl From<std::io::Error> for TurboError {
    fn from(err: std::io::Error) -> Self {
        TurboError::StreamError(err.to_string())
    }
}
