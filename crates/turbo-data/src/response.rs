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
        self.header("Content-Length").and_then(|v| v.parse().ok())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_response(status: u16, body: &[u8]) -> Response {
        Response::new(status, HashMap::new(), body.to_vec())
    }

    fn make_response_with_headers(
        status: u16,
        headers: Vec<(&str, &str)>,
        body: &[u8],
    ) -> Response {
        let headers: HashMap<String, String> = headers
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Response::new(status, headers, body.to_vec())
    }

    // === Status Check Tests ===

    #[test]
    fn test_response_is_success() {
        assert!(make_response(200, b"").is_success());
        assert!(make_response(201, b"").is_success());
        assert!(make_response(299, b"").is_success());
        assert!(!make_response(199, b"").is_success());
        assert!(!make_response(300, b"").is_success());
    }

    #[test]
    fn test_response_is_client_error() {
        assert!(make_response(400, b"").is_client_error());
        assert!(make_response(404, b"").is_client_error());
        assert!(make_response(499, b"").is_client_error());
        assert!(!make_response(399, b"").is_client_error());
        assert!(!make_response(500, b"").is_client_error());
    }

    #[test]
    fn test_response_is_server_error() {
        assert!(make_response(500, b"").is_server_error());
        assert!(make_response(503, b"").is_server_error());
        assert!(make_response(599, b"").is_server_error());
        assert!(!make_response(499, b"").is_server_error());
        assert!(!make_response(600, b"").is_server_error());
    }

    // === Body Tests ===

    #[test]
    fn test_response_text() {
        let resp = make_response(200, b"Hello, World!");
        assert_eq!(resp.text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_response_text_invalid_utf8() {
        let resp = make_response(200, &[0xff, 0xfe]);
        assert!(resp.text().is_err());
    }

    #[test]
    fn test_response_json() {
        use serde::Deserialize;

        #[derive(Deserialize, Debug, PartialEq)]
        struct Data {
            value: i32,
        }

        let resp = make_response(200, br#"{"value": 42}"#);
        let data: Data = resp.json().unwrap();
        assert_eq!(data, Data { value: 42 });
    }

    #[test]
    fn test_response_json_invalid() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Data {
            value: i32,
        }

        let resp = make_response(200, b"not json");
        let result: Result<Data, _> = resp.json();
        assert!(result.is_err());
    }

    #[test]
    fn test_response_bytes() {
        let resp = make_response(200, &[1, 2, 3, 4]);
        assert_eq!(resp.bytes(), &[1, 2, 3, 4]);
    }

    // === Header Tests ===

    #[test]
    fn test_response_header() {
        let resp = make_response_with_headers(200, vec![("Content-Type", "application/json")], b"");
        assert_eq!(resp.header("Content-Type"), Some("application/json"));
    }

    #[test]
    fn test_response_header_case_insensitive() {
        let resp = make_response_with_headers(200, vec![("Content-Type", "text/html")], b"");
        assert_eq!(resp.header("content-type"), Some("text/html"));
        assert_eq!(resp.header("CONTENT-TYPE"), Some("text/html"));
    }

    #[test]
    fn test_response_header_missing() {
        let resp = make_response(200, b"");
        assert_eq!(resp.header("X-Missing"), None);
    }

    #[test]
    fn test_response_content_type() {
        let resp = make_response_with_headers(200, vec![("Content-Type", "application/json")], b"");
        assert_eq!(resp.content_type(), Some("application/json"));
    }

    #[test]
    fn test_response_content_length() {
        let resp = make_response_with_headers(200, vec![("Content-Length", "42")], b"");
        assert_eq!(resp.content_length(), Some(42));
    }

    #[test]
    fn test_response_content_length_invalid() {
        let resp = make_response_with_headers(200, vec![("Content-Length", "not-a-number")], b"");
        assert_eq!(resp.content_length(), None);
    }

    // === error_for_status Tests ===

    #[test]
    fn test_response_error_for_status_success() {
        let resp = make_response(200, b"OK");
        assert!(resp.error_for_status().is_ok());
    }

    #[test]
    fn test_response_error_for_status_client_error() {
        let resp = make_response(404, b"Not Found");
        let result = resp.error_for_status();
        assert!(result.is_err());
    }

    #[test]
    fn test_response_error_for_status_server_error() {
        let resp = make_response(500, b"Internal Server Error");
        let result = resp.error_for_status();
        assert!(result.is_err());
    }

    #[test]
    fn test_response_clone() {
        let resp = make_response(200, b"data");
        let cloned = resp.clone();
        assert_eq!(cloned.status, 200);
        assert_eq!(cloned.bytes(), b"data");
    }
}
