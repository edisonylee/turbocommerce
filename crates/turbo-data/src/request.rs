//! HTTP request builder.

use crate::FetchError;
use serde::Serialize;
use std::collections::HashMap;

/// HTTP methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl Method {
    /// Convert to HTTP method string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

/// A builder for constructing HTTP requests.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    #[allow(dead_code)] // Used in wasm32 target
    pub(crate) method: Method,
    #[allow(dead_code)] // Used in wasm32 target
    pub(crate) url: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<Vec<u8>>,
}

impl RequestBuilder {
    /// Create a new request builder.
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Add a header to the request.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add multiple headers to the request.
    pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set the request body as raw bytes.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the request body as a string.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.headers
            .entry("Content-Type".to_string())
            .or_insert_with(|| "text/plain".to_string());
        self.body = Some(text.into_bytes());
        self
    }

    /// Set the request body as JSON.
    pub fn json<T: Serialize>(mut self, value: &T) -> Result<Self, FetchError> {
        let json = serde_json::to_vec(value)?;
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json);
        Ok(self)
    }

    /// Add a bearer token authorization header.
    pub fn bearer_auth(self, token: impl AsRef<str>) -> Self {
        self.header("Authorization", format!("Bearer {}", token.as_ref()))
    }

    /// Add a basic authorization header.
    pub fn basic_auth(self, username: impl AsRef<str>, password: Option<&str>) -> Self {
        let credentials = match password {
            Some(pass) => format!("{}:{}", username.as_ref(), pass),
            None => format!("{}:", username.as_ref()),
        };
        let encoded = base64_encode(credentials.as_bytes());
        self.header("Authorization", format!("Basic {}", encoded))
    }

    /// Set the Accept header.
    pub fn accept(self, content_type: impl Into<String>) -> Self {
        self.header("Accept", content_type)
    }

    /// Set the Content-Type header.
    pub fn content_type(self, content_type: impl Into<String>) -> Self {
        self.header("Content-Type", content_type)
    }
}

/// Simple base64 encoding for auth headers.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(CHARS[b0 >> 2] as char);
        result.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    result
}
