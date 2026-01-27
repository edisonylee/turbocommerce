//! HTTP client utilities for TurboCommerce.
//!
//! Provides a simple, ergonomic API for making HTTP requests from Spin WASM
//! applications with automatic JSON handling.
//!
//! # Example
//!
//! ```rust,ignore
//! use turbo_data::{FetchClient, Method};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Deserialize)]
//! struct Product {
//!     id: String,
//!     name: String,
//!     price: f64,
//! }
//!
//! // In a server function
//! let client = FetchClient::new();
//!
//! // Simple GET request
//! let product: Product = client
//!     .get("https://api.example.com/products/123")
//!     .send()?
//!     .json()?;
//!
//! // POST with JSON body
//! #[derive(Serialize)]
//! struct CreateProduct {
//!     name: String,
//!     price: f64,
//! }
//!
//! let new_product = CreateProduct {
//!     name: "Widget".to_string(),
//!     price: 29.99,
//! };
//!
//! let created: Product = client
//!     .post("https://api.example.com/products")
//!     .json(&new_product)?
//!     .send()?
//!     .json()?;
//! ```

mod error;
mod request;
mod response;

pub use error::FetchError;
pub use request::{Method, RequestBuilder};
pub use response::Response;

/// HTTP client for making outbound requests.
///
/// This is a lightweight wrapper around Spin's HTTP client that provides
/// a convenient builder API for constructing and sending requests.
pub struct FetchClient {
    base_url: Option<String>,
    default_headers: std::collections::HashMap<String, String>,
}

impl Default for FetchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FetchClient {
    /// Create a new HTTP client.
    pub fn new() -> Self {
        Self {
            base_url: None,
            default_headers: std::collections::HashMap::new(),
        }
    }

    /// Create a client with a base URL that will be prepended to all requests.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Add a default header that will be included in all requests.
    pub fn with_default_header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Create a GET request.
    pub fn get(&self, url: impl Into<String>) -> ClientRequestBuilder {
        self.request(Method::Get, url)
    }

    /// Create a POST request.
    pub fn post(&self, url: impl Into<String>) -> ClientRequestBuilder {
        self.request(Method::Post, url)
    }

    /// Create a PUT request.
    pub fn put(&self, url: impl Into<String>) -> ClientRequestBuilder {
        self.request(Method::Put, url)
    }

    /// Create a PATCH request.
    pub fn patch(&self, url: impl Into<String>) -> ClientRequestBuilder {
        self.request(Method::Patch, url)
    }

    /// Create a DELETE request.
    pub fn delete(&self, url: impl Into<String>) -> ClientRequestBuilder {
        self.request(Method::Delete, url)
    }

    /// Create a request with a custom method.
    pub fn request(&self, method: Method, url: impl Into<String>) -> ClientRequestBuilder {
        let url = url.into();
        let full_url = match &self.base_url {
            Some(base) => {
                if url.starts_with("http://") || url.starts_with("https://") {
                    url
                } else {
                    format!("{}{}", base.trim_end_matches('/'), url)
                }
            }
            None => url,
        };

        let mut builder = RequestBuilder::new(method, full_url);
        for (key, value) in &self.default_headers {
            builder = builder.header(key.clone(), value.clone());
        }

        ClientRequestBuilder { builder }
    }
}

/// A request builder bound to a client.
pub struct ClientRequestBuilder {
    builder: RequestBuilder,
}

impl ClientRequestBuilder {
    /// Add a header to the request.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.builder = self.builder.header(key, value);
        self
    }

    /// Set the request body as raw bytes.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.builder = self.builder.body(body);
        self
    }

    /// Set the request body as a string.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.builder = self.builder.text(text);
        self
    }

    /// Set the request body as JSON.
    pub fn json<T: serde::Serialize>(mut self, value: &T) -> Result<Self, FetchError> {
        self.builder = self.builder.json(value)?;
        Ok(self)
    }

    /// Add a bearer token authorization header.
    pub fn bearer_auth(mut self, token: impl AsRef<str>) -> Self {
        self.builder = self.builder.bearer_auth(token);
        self
    }

    /// Add a basic authorization header.
    pub fn basic_auth(mut self, username: impl AsRef<str>, password: Option<&str>) -> Self {
        self.builder = self.builder.basic_auth(username, password);
        self
    }

    /// Send the request and return the response.
    #[cfg(target_arch = "wasm32")]
    pub fn send(self) -> Result<Response, FetchError> {
        use spin_sdk::http::{Method as SpinMethod, Request};

        let method = match self.builder.method {
            Method::Get => SpinMethod::Get,
            Method::Post => SpinMethod::Post,
            Method::Put => SpinMethod::Put,
            Method::Patch => SpinMethod::Patch,
            Method::Delete => SpinMethod::Delete,
            Method::Head => SpinMethod::Head,
            Method::Options => SpinMethod::Options,
        };

        let mut request = Request::builder();
        request.method(method);
        request.uri(&self.builder.url);

        for (key, value) in &self.builder.headers {
            request.header(key.as_str(), value.as_str());
        }

        let request = if let Some(body) = self.builder.body {
            request.body(body).map_err(|e| FetchError::RequestError(e.to_string()))?
        } else {
            request.build()
        };

        let response = spin_sdk::http::send(request)
            .map_err(|e| FetchError::RequestError(e.to_string()))?;

        let status = response.status();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.into_body();

        Ok(Response::new(status, headers, body))
    }

    /// Send the request and return the response (non-WASM stub).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn send(self) -> Result<Response, FetchError> {
        // Return empty response for non-WASM builds (testing/development)
        Ok(Response::new(
            200,
            std::collections::HashMap::new(),
            Vec::new(),
        ))
    }
}

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::{FetchClient, FetchError, Method, Response};
}
