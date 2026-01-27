//! HTTP client error types.

use thiserror::Error;

/// Errors that can occur when making HTTP requests.
#[derive(Error, Debug)]
pub enum FetchError {
    /// Failed to send the request.
    #[error("Request failed: {0}")]
    RequestError(String),

    /// Invalid URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// HTTP error response.
    #[error("HTTP {status}: {message}")]
    HttpError { status: u16, message: String },

    /// Failed to parse response body.
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Request timeout.
    #[error("Request timed out")]
    Timeout,

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    JsonError(String),
}

impl From<serde_json::Error> for FetchError {
    fn from(e: serde_json::Error) -> Self {
        FetchError::JsonError(e.to_string())
    }
}
