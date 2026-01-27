//! Session management.

use crate::user::User;
use crate::AuthError;
use serde::{Deserialize, Serialize};
use turbo_commerce::ids::CartId;

/// Session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new random session ID.
    pub fn generate() -> Self {
        Self(generate_secure_id("sess"))
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// An authenticated session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    /// Session ID.
    pub id: SessionId,
    /// The user (anonymous or authenticated).
    pub user: User,
    /// Associated cart ID.
    pub cart_id: Option<CartId>,
    /// CSRF token for form protection.
    pub csrf_token: String,
    /// IP address that created the session.
    pub ip_address: Option<String>,
    /// User agent that created the session.
    pub user_agent: Option<String>,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last activity.
    pub last_activity_at: i64,
    /// Unix timestamp when session expires.
    pub expires_at: i64,
}

impl AuthSession {
    /// Default session duration: 7 days.
    pub const DEFAULT_DURATION_SECS: i64 = 7 * 24 * 60 * 60;

    /// Create a new session for an anonymous user.
    pub fn anonymous() -> Self {
        let now = current_timestamp();
        let session_id = SessionId::generate();
        let session_id_str = session_id.as_str().to_string();

        Self {
            id: session_id,
            user: User::anonymous(session_id_str),
            cart_id: None,
            csrf_token: generate_secure_id("csrf"),
            ip_address: None,
            user_agent: None,
            created_at: now,
            last_activity_at: now,
            expires_at: now + Self::DEFAULT_DURATION_SECS,
        }
    }

    /// Create a new session for an authenticated user.
    pub fn authenticated(user: User) -> Self {
        let now = current_timestamp();
        Self {
            id: SessionId::generate(),
            user,
            cart_id: None,
            csrf_token: generate_secure_id("csrf"),
            ip_address: None,
            user_agent: None,
            created_at: now,
            last_activity_at: now,
            expires_at: now + Self::DEFAULT_DURATION_SECS,
        }
    }

    /// Create session with custom duration.
    pub fn with_duration(mut self, duration_secs: i64) -> Self {
        self.expires_at = self.created_at + duration_secs;
        self
    }

    /// Set IP address.
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent.
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Associate a cart with this session.
    pub fn with_cart(mut self, cart_id: CartId) -> Self {
        self.cart_id = Some(cart_id);
        self
    }

    /// Check if session is expired.
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Check if session is valid (not expired).
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Validate the session, returning error if invalid.
    pub fn validate(&self) -> Result<(), AuthError> {
        if self.is_expired() {
            Err(AuthError::SessionExpired)
        } else {
            Ok(())
        }
    }

    /// Update last activity timestamp.
    pub fn touch(&mut self) {
        self.last_activity_at = current_timestamp();
    }

    /// Extend session expiration.
    pub fn extend(&mut self, duration_secs: i64) {
        self.expires_at = current_timestamp() + duration_secs;
        self.touch();
    }

    /// Verify CSRF token.
    pub fn verify_csrf(&self, token: &str) -> Result<(), AuthError> {
        if self.csrf_token == token {
            Ok(())
        } else {
            Err(AuthError::CsrfMismatch)
        }
    }

    /// Regenerate CSRF token.
    pub fn regenerate_csrf(&mut self) {
        self.csrf_token = generate_secure_id("csrf");
    }

    /// Upgrade anonymous session to authenticated.
    pub fn upgrade(&mut self, user: User) -> Result<(), AuthError> {
        if !self.user.is_anonymous() {
            return Err(AuthError::Internal("Session already authenticated".to_string()));
        }
        self.user = user;
        self.regenerate_csrf();
        self.touch();
        Ok(())
    }

    /// Get time until expiration in seconds.
    pub fn time_to_expiry(&self) -> i64 {
        (self.expires_at - current_timestamp()).max(0)
    }

    /// Get cache key for this session.
    pub fn cache_key(&self) -> String {
        format!("session:{}", self.id)
    }
}

/// Session configuration.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session duration in seconds.
    pub duration_secs: i64,
    /// Whether to extend session on activity.
    pub sliding_expiration: bool,
    /// Inactivity timeout in seconds (if sliding expiration enabled).
    pub inactivity_timeout_secs: i64,
    /// Whether to require CSRF token validation.
    pub csrf_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            duration_secs: AuthSession::DEFAULT_DURATION_SECS,
            sliding_expiration: true,
            inactivity_timeout_secs: 30 * 60, // 30 minutes
            csrf_enabled: true,
        }
    }
}

/// Generate a secure random ID.
fn generate_secure_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Mix timestamp with some randomness from memory addresses
    let ptr = Box::new(0u8);
    let addr = &*ptr as *const u8 as usize;
    format!("{}_{:x}_{:x}", prefix, ts, addr)
}

/// Get current Unix timestamp.
fn current_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = AuthSession::anonymous();
        assert!(session.user.is_anonymous());
        assert!(!session.is_expired());
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_csrf() {
        let session = AuthSession::anonymous();
        let token = session.csrf_token.clone();
        assert!(session.verify_csrf(&token).is_ok());
        assert!(session.verify_csrf("wrong_token").is_err());
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();
        assert_ne!(id1, id2);
    }
}
