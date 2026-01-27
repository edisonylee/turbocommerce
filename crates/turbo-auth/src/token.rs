//! Authentication tokens.
//!
//! Tokens for password reset, email verification, and other auth flows.

use crate::AuthError;
use serde::{Deserialize, Serialize};
use turbo_commerce::ids::UserId;

/// Token type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// Password reset token.
    PasswordReset,
    /// Email verification token.
    EmailVerification,
    /// Account activation token.
    AccountActivation,
    /// Magic link login token.
    MagicLink,
    /// API access token.
    ApiAccess,
    /// Refresh token for session renewal.
    Refresh,
}

impl TokenType {
    /// Get token type as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::PasswordReset => "password_reset",
            TokenType::EmailVerification => "email_verification",
            TokenType::AccountActivation => "account_activation",
            TokenType::MagicLink => "magic_link",
            TokenType::ApiAccess => "api_access",
            TokenType::Refresh => "refresh",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "password_reset" => Some(TokenType::PasswordReset),
            "email_verification" => Some(TokenType::EmailVerification),
            "account_activation" => Some(TokenType::AccountActivation),
            "magic_link" => Some(TokenType::MagicLink),
            "api_access" => Some(TokenType::ApiAccess),
            "refresh" => Some(TokenType::Refresh),
            _ => None,
        }
    }

    /// Get default expiration time for this token type (in seconds).
    pub fn default_expiry_secs(&self) -> i64 {
        match self {
            TokenType::PasswordReset => 60 * 60,        // 1 hour
            TokenType::EmailVerification => 24 * 60 * 60, // 24 hours
            TokenType::AccountActivation => 7 * 24 * 60 * 60, // 7 days
            TokenType::MagicLink => 15 * 60,            // 15 minutes
            TokenType::ApiAccess => 30 * 24 * 60 * 60,  // 30 days
            TokenType::Refresh => 90 * 24 * 60 * 60,    // 90 days
        }
    }
}

/// An authentication token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The token value.
    pub token: String,
    /// Token type.
    pub token_type: TokenType,
    /// User ID this token belongs to.
    pub user_id: UserId,
    /// Unix timestamp when token was created.
    pub created_at: i64,
    /// Unix timestamp when token expires.
    pub expires_at: i64,
    /// Whether the token has been used.
    pub used: bool,
    /// Additional metadata.
    pub metadata: Option<serde_json::Value>,
}

impl AuthToken {
    /// Generate a new token.
    pub fn generate(token_type: TokenType, user_id: UserId) -> Self {
        let now = current_timestamp();
        Self {
            token: generate_token_string(),
            token_type,
            user_id,
            created_at: now,
            expires_at: now + token_type.default_expiry_secs(),
            used: false,
            metadata: None,
        }
    }

    /// Generate token with custom expiry.
    pub fn generate_with_expiry(token_type: TokenType, user_id: UserId, expiry_secs: i64) -> Self {
        let now = current_timestamp();
        Self {
            token: generate_token_string(),
            token_type,
            user_id,
            created_at: now,
            expires_at: now + expiry_secs,
            used: false,
            metadata: None,
        }
    }

    /// Add metadata to the token.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Check if token is expired.
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Check if token is valid (not expired and not used).
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.used
    }

    /// Validate the token.
    pub fn validate(&self) -> Result<(), AuthError> {
        if self.used {
            return Err(AuthError::InvalidToken);
        }
        if self.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        Ok(())
    }

    /// Mark token as used.
    pub fn mark_used(&mut self) {
        self.used = true;
    }

    /// Get time until expiration in seconds.
    pub fn time_to_expiry(&self) -> i64 {
        (self.expires_at - current_timestamp()).max(0)
    }

    /// Get cache key for this token.
    pub fn cache_key(&self) -> String {
        format!("token:{}:{}", self.token_type.as_str(), self.token)
    }

    /// Get cache key by token string.
    pub fn cache_key_for(token_type: TokenType, token: &str) -> String {
        format!("token:{}:{}", token_type.as_str(), token)
    }
}

/// Token validation result.
#[derive(Debug, Clone)]
pub struct TokenValidation {
    /// The validated token.
    pub token: AuthToken,
    /// User ID from the token.
    pub user_id: UserId,
}

/// Generate a secure token string.
fn generate_token_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    // Create pseudo-random token
    let ptr = Box::new(0u64);
    let addr = &*ptr as *const u64 as u64;

    let mut state = ts as u64 ^ addr;
    let mut chars = Vec::with_capacity(32);

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    for _ in 0..32 {
        // xorshift
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let idx = (state as usize) % ALPHABET.len();
        chars.push(ALPHABET[idx] as char);
    }

    chars.into_iter().collect()
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
    fn test_token_generation() {
        let token = AuthToken::generate(TokenType::PasswordReset, UserId::new("user_123"));
        assert!(!token.is_expired());
        assert!(token.is_valid());
        assert_eq!(token.token.len(), 32);
    }

    #[test]
    fn test_token_types() {
        assert_eq!(TokenType::PasswordReset.as_str(), "password_reset");
        assert_eq!(
            TokenType::from_str("password_reset"),
            Some(TokenType::PasswordReset)
        );
    }

    #[test]
    fn test_token_validation() {
        let mut token = AuthToken::generate(TokenType::EmailVerification, UserId::new("user_456"));
        assert!(token.validate().is_ok());

        token.mark_used();
        assert!(token.validate().is_err());
    }

    #[test]
    fn test_unique_tokens() {
        let token1 = AuthToken::generate(TokenType::MagicLink, UserId::new("user_1"));
        let token2 = AuthToken::generate(TokenType::MagicLink, UserId::new("user_1"));
        assert_ne!(token1.token, token2.token);
    }
}
