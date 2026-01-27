//! HTTP response handling.

use crate::FetchError;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    /// The HTTP status code.
    pub status: u16,
    /// The response headers.
    pub headers: HashMap<String, String>,
    /// The response body.
    pub body: Vec<u8>,
}

impl Response {
    /// Create a new response.
    pub fn new(status: u16, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// Check if the response was successful (2xx status).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response was a client error (4xx status).
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response was a server error (5xx status).
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get the response body as text.
    pub fn text(&self) -> Result<String, FetchError> {
        String::from_utf8(self.body.clone())
            .map_err(|e| FetchError::ParseError(format!("Invalid UTF-8: {}", e)))
    }

    /// Parse the response body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, FetchError> {
        serde_json::from_slice(&self.body).map_err(|e| FetchError::ParseError(e.to_string()))
    }

    /// Get the raw response body.
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// Get a header value.
    pub fn header(&self, key: &str) -> Option<&str> {
        // Case-insensitive header lookup
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Get the Content-Type header.
    pub fn content_type(&self) -> Option<&str> {
        self.header("Content-Type")
    }

    /// Get the Content-Length header.
    pub fn content_length(&self) -> Option<usize> {
        self.header("Content-Length")
            .and_then(|v| v.parse().ok())
    }

    /// Convert to a Result, returning an error for non-2xx status codes.
    pub fn error_for_status(self) -> Result<Self, FetchError> {
        if self.is_success() {
            Ok(self)
        } else {
            let message = self.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(FetchError::HttpError {
                status: self.status,
                message,
            })
        }
    }
}
