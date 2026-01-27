//! Authentication errors.

use thiserror::Error;

/// Authentication error type.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Invalid credentials provided.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// User not found.
    #[error("user not found: {0}")]
    UserNotFound(String),

    /// User already exists.
    #[error("user already exists: {0}")]
    UserAlreadyExists(String),

    /// Session not found or expired.
    #[error("session not found or expired")]
    SessionNotFound,

    /// Session expired.
    #[error("session expired")]
    SessionExpired,

    /// Token invalid or expired.
    #[error("token invalid or expired")]
    InvalidToken,

    /// Token expired.
    #[error("token expired")]
    TokenExpired,

    /// Password too weak.
    #[error("password too weak: {0}")]
    WeakPassword(String),

    /// Email not verified.
    #[error("email not verified")]
    EmailNotVerified,

    /// Account locked.
    #[error("account locked")]
    AccountLocked,

    /// Insufficient permissions.
    #[error("insufficient permissions")]
    InsufficientPermissions,

    /// CSRF token mismatch.
    #[error("CSRF token mismatch")]
    CsrfMismatch,

    /// Cache error.
    #[error("cache error: {0}")]
    Cache(#[from] turbo_cache::CacheError),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl AuthError {
    /// Check if this is an authentication failure.
    pub fn is_auth_failure(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidCredentials
                | AuthError::SessionNotFound
                | AuthError::SessionExpired
                | AuthError::InvalidToken
                | AuthError::TokenExpired
        )
    }

    /// Check if this is a permission error.
    pub fn is_permission_error(&self) -> bool {
        matches!(self, AuthError::InsufficientPermissions)
    }
}
