//! Outbound request allowlist for host filtering.

use serde::{Deserialize, Serialize};

/// Result type for allowlist operations.
pub type AllowlistResult<T> = Result<T, AllowlistError>;

/// Errors from allowlist operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AllowlistError {
    #[error("host not allowed: {0}")]
    HostNotAllowed(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("scheme not allowed: {0}")]
    SchemeNotAllowed(String),

    #[error("port not allowed: {0}")]
    PortNotAllowed(u16),
}

/// Outbound request allowlist for controlling which hosts can be accessed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutboundAllowlist {
    /// Allowed exact hosts.
    allowed_hosts: Vec<String>,
    /// Allowed host patterns (supports * wildcard).
    allowed_patterns: Vec<String>,
    /// Explicitly denied hosts (takes precedence).
    denied_hosts: Vec<String>,
    /// Denied host patterns.
    denied_patterns: Vec<String>,
    /// Allowed schemes (default: https).
    allowed_schemes: Vec<String>,
    /// Allowed ports (empty = all ports allowed).
    allowed_ports: Vec<u16>,
    /// Whether to allow localhost/loopback.
    allow_localhost: bool,
    /// Whether to allow private IP ranges.
    allow_private_ips: bool,
    /// Default policy when no rules match.
    default_allow: bool,
}

impl OutboundAllowlist {
    /// Create a new empty allowlist (deny by default).
    pub fn new() -> Self {
        Self {
            allowed_schemes: vec!["https".to_string()],
            ..Default::default()
        }
    }

    /// Create a permissive allowlist for development.
    pub fn permissive() -> Self {
        Self {
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            allow_localhost: true,
            allow_private_ips: true,
            default_allow: true,
            ..Default::default()
        }
    }

    /// Allow a specific host.
    pub fn allow_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into().to_lowercase());
        self
    }

    /// Allow multiple hosts.
    pub fn allow_hosts(mut self, hosts: &[&str]) -> Self {
        for host in hosts {
            self.allowed_hosts.push(host.to_lowercase());
        }
        self
    }

    /// Allow a host pattern (supports * as wildcard).
    ///
    /// Examples:
    /// - `*.example.com` - matches `api.example.com`, `cdn.example.com`
    /// - `api.*.example.com` - matches `api.v1.example.com`
    pub fn allow_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.allowed_patterns.push(pattern.into().to_lowercase());
        self
    }

    /// Deny a specific host (takes precedence over allow).
    pub fn deny_host(mut self, host: impl Into<String>) -> Self {
        self.denied_hosts.push(host.into().to_lowercase());
        self
    }

    /// Deny a host pattern.
    pub fn deny_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.denied_patterns.push(pattern.into().to_lowercase());
        self
    }

    /// Set allowed schemes.
    pub fn with_schemes(mut self, schemes: &[&str]) -> Self {
        self.allowed_schemes = schemes.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Allow HTTP scheme (in addition to HTTPS).
    pub fn allow_http(mut self) -> Self {
        if !self.allowed_schemes.contains(&"http".to_string()) {
            self.allowed_schemes.push("http".to_string());
        }
        self
    }

    /// Set allowed ports.
    pub fn with_ports(mut self, ports: &[u16]) -> Self {
        self.allowed_ports = ports.to_vec();
        self
    }

    /// Allow localhost/loopback addresses.
    pub fn allow_localhost(mut self, allow: bool) -> Self {
        self.allow_localhost = allow;
        self
    }

    /// Allow private IP ranges (10.x, 172.16.x, 192.168.x).
    pub fn allow_private_ips(mut self, allow: bool) -> Self {
        self.allow_private_ips = allow;
        self
    }

    /// Set default policy (allow or deny when no rules match).
    pub fn default_allow(mut self, allow: bool) -> Self {
        self.default_allow = allow;
        self
    }

    /// Check if a URL is allowed.
    pub fn check_url(&self, url: &str) -> AllowlistResult<()> {
        let parsed = ParsedUrl::parse(url)?;

        // Check scheme
        if !self.allowed_schemes.is_empty()
            && !self.allowed_schemes.contains(&parsed.scheme.to_lowercase())
        {
            return Err(AllowlistError::SchemeNotAllowed(parsed.scheme));
        }

        // Check port
        if !self.allowed_ports.is_empty() && !self.allowed_ports.contains(&parsed.port) {
            return Err(AllowlistError::PortNotAllowed(parsed.port));
        }

        // Check host
        self.check_host(&parsed.host)
    }

    /// Check if a host is allowed.
    pub fn check_host(&self, host: &str) -> AllowlistResult<()> {
        let host_lower = host.to_lowercase();

        // Check if it's a localhost/loopback
        if is_localhost(&host_lower) {
            if !self.allow_localhost {
                return Err(AllowlistError::HostNotAllowed(host.to_string()));
            }
            return Ok(());
        }

        // Check if it's a private IP
        if is_private_ip(&host_lower) {
            if !self.allow_private_ips {
                return Err(AllowlistError::HostNotAllowed(host.to_string()));
            }
            return Ok(());
        }

        // Check denied hosts first (takes precedence)
        if self.denied_hosts.contains(&host_lower) {
            return Err(AllowlistError::HostNotAllowed(host.to_string()));
        }

        // Check denied patterns
        for pattern in &self.denied_patterns {
            if matches_pattern(&host_lower, pattern) {
                return Err(AllowlistError::HostNotAllowed(host.to_string()));
            }
        }

        // Check allowed hosts
        if self.allowed_hosts.contains(&host_lower) {
            return Ok(());
        }

        // Check allowed patterns
        for pattern in &self.allowed_patterns {
            if matches_pattern(&host_lower, pattern) {
                return Ok(());
            }
        }

        // Apply default policy
        if self.default_allow {
            Ok(())
        } else if self.allowed_hosts.is_empty() && self.allowed_patterns.is_empty() {
            // No rules defined, apply default
            if self.default_allow {
                Ok(())
            } else {
                Err(AllowlistError::HostNotAllowed(host.to_string()))
            }
        } else {
            Err(AllowlistError::HostNotAllowed(host.to_string()))
        }
    }

    /// Get a summary of the allowlist rules.
    pub fn summary(&self) -> AllowlistSummary {
        AllowlistSummary {
            allowed_hosts: self.allowed_hosts.len(),
            allowed_patterns: self.allowed_patterns.len(),
            denied_hosts: self.denied_hosts.len(),
            denied_patterns: self.denied_patterns.len(),
            allowed_schemes: self.allowed_schemes.clone(),
            allow_localhost: self.allow_localhost,
            allow_private_ips: self.allow_private_ips,
            default_allow: self.default_allow,
        }
    }
}

/// Summary of allowlist rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistSummary {
    pub allowed_hosts: usize,
    pub allowed_patterns: usize,
    pub denied_hosts: usize,
    pub denied_patterns: usize,
    pub allowed_schemes: Vec<String>,
    pub allow_localhost: bool,
    pub allow_private_ips: bool,
    pub default_allow: bool,
}

/// Simple URL parser for allowlist checking.
#[derive(Debug)]
struct ParsedUrl {
    scheme: String,
    host: String,
    port: u16,
}

impl ParsedUrl {
    fn parse(url: &str) -> AllowlistResult<Self> {
        // Find scheme
        let (scheme, rest) = url
            .split_once("://")
            .ok_or_else(|| AllowlistError::InvalidUrl("missing scheme".to_string()))?;

        // Find host and port
        let authority = rest.split('/').next().unwrap_or(rest);
        let authority = authority.split('?').next().unwrap_or(authority);

        let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
            // Check if this is an IPv6 address
            if h.contains('[') {
                // IPv6 with port: [::1]:8080
                let host = authority
                    .trim_start_matches('[')
                    .split(']')
                    .next()
                    .unwrap_or(authority);
                let port = authority
                    .rsplit(':')
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or_else(|| default_port(scheme));
                (host.to_string(), port)
            } else {
                let port = p.parse().unwrap_or_else(|_| default_port(scheme));
                (h.to_string(), port)
            }
        } else {
            (authority.to_string(), default_port(scheme))
        };

        Ok(Self {
            scheme: scheme.to_string(),
            host,
            port,
        })
    }
}

fn default_port(scheme: &str) -> u16 {
    match scheme.to_lowercase().as_str() {
        "http" => 80,
        "https" => 443,
        _ => 443,
    }
}

fn is_localhost(host: &str) -> bool {
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.starts_with("127.")
}

fn is_private_ip(host: &str) -> bool {
    // Check common private IP ranges
    if host.starts_with("10.") {
        return true;
    }
    if host.starts_with("192.168.") {
        return true;
    }
    if host.starts_with("172.") {
        // 172.16.0.0 - 172.31.255.255
        if let Some(second) = host.split('.').nth(1) {
            if let Ok(n) = second.parse::<u8>() {
                if (16..=31).contains(&n) {
                    return true;
                }
            }
        }
    }
    false
}

fn matches_pattern(host: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        return host == pattern;
    }

    // Convert glob pattern to simple matching
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 2 {
        // Single wildcard
        let prefix = parts[0];
        let suffix = parts[1];

        if prefix.is_empty() {
            // Pattern like "*.example.com"
            host.ends_with(suffix)
        } else if suffix.is_empty() {
            // Pattern like "api.*"
            host.starts_with(prefix)
        } else {
            // Pattern like "api.*.example.com"
            host.starts_with(prefix) && host.ends_with(suffix)
        }
    } else {
        // Multiple wildcards - use simple approach
        // For simplicity, just check if host contains non-wildcard parts
        host.contains(&pattern.replace('*', ""))
    }
}

/// Pre-configured allowlist for common use cases.
pub mod presets {
    use super::OutboundAllowlist;

    /// Allowlist for AWS services.
    pub fn aws() -> OutboundAllowlist {
        OutboundAllowlist::new()
            .allow_pattern("*.amazonaws.com")
            .allow_pattern("*.aws.amazon.com")
    }

    /// Allowlist for Google Cloud services.
    pub fn gcp() -> OutboundAllowlist {
        OutboundAllowlist::new()
            .allow_pattern("*.googleapis.com")
            .allow_pattern("*.google.com")
    }

    /// Allowlist for common CDNs.
    pub fn cdns() -> OutboundAllowlist {
        OutboundAllowlist::new()
            .allow_pattern("*.cloudflare.com")
            .allow_pattern("*.cloudfront.net")
            .allow_pattern("*.fastly.net")
            .allow_pattern("*.akamaized.net")
    }

    /// Allowlist for common APIs.
    pub fn common_apis() -> OutboundAllowlist {
        OutboundAllowlist::new()
            .allow_host("api.github.com")
            .allow_host("api.stripe.com")
            .allow_host("api.sendgrid.com")
            .allow_pattern("*.sentry.io")
    }
}
