//! User types.

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use turbo_commerce::ids::UserId;

/// User role for authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Role {
    /// Regular customer.
    #[default]
    Customer,
    /// Store staff with limited admin access.
    Staff,
    /// Store administrator.
    Admin,
    /// Super admin with full access.
    SuperAdmin,
}

impl Role {
    /// Get role as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Customer => "customer",
            Role::Staff => "staff",
            Role::Admin => "admin",
            Role::SuperAdmin => "super_admin",
        }
    }

    /// Check if this role has at least the given permission level.
    pub fn has_permission(&self, required: Role) -> bool {
        self.level() >= required.level()
    }

    /// Get permission level (higher = more permissions).
    pub fn level(&self) -> u8 {
        match self {
            Role::Customer => 0,
            Role::Staff => 1,
            Role::Admin => 2,
            Role::SuperAdmin => 3,
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "customer" => Ok(Role::Customer),
            "staff" => Ok(Role::Staff),
            "admin" => Ok(Role::Admin),
            "super_admin" => Ok(Role::SuperAdmin),
            _ => Err(()),
        }
    }
}

/// A user in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum User {
    /// Anonymous/guest user with session tracking.
    Anonymous {
        /// Session identifier.
        session_id: String,
    },
    /// Authenticated user.
    Authenticated {
        /// User ID.
        id: UserId,
        /// Email address.
        email: String,
        /// Display name.
        name: Option<String>,
        /// User roles.
        roles: Vec<Role>,
        /// Email verified status.
        email_verified: bool,
    },
}

impl User {
    /// Create a new anonymous user.
    pub fn anonymous(session_id: impl Into<String>) -> Self {
        User::Anonymous {
            session_id: session_id.into(),
        }
    }

    /// Create a new authenticated user.
    pub fn authenticated(
        id: UserId,
        email: impl Into<String>,
        name: Option<String>,
        roles: Vec<Role>,
    ) -> Self {
        User::Authenticated {
            id,
            email: email.into(),
            name,
            roles,
            email_verified: false,
        }
    }

    /// Check if user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        matches!(self, User::Authenticated { .. })
    }

    /// Check if user is anonymous.
    pub fn is_anonymous(&self) -> bool {
        matches!(self, User::Anonymous { .. })
    }

    /// Get user ID if authenticated.
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            User::Authenticated { id, .. } => Some(id),
            User::Anonymous { .. } => None,
        }
    }

    /// Get email if authenticated.
    pub fn email(&self) -> Option<&str> {
        match self {
            User::Authenticated { email, .. } => Some(email),
            User::Anonymous { .. } => None,
        }
    }

    /// Get display name.
    pub fn display_name(&self) -> &str {
        match self {
            User::Authenticated { name, email, .. } => name.as_deref().unwrap_or(email.as_str()),
            User::Anonymous { session_id } => session_id,
        }
    }

    /// Get roles.
    pub fn roles(&self) -> &[Role] {
        match self {
            User::Authenticated { roles, .. } => roles,
            User::Anonymous { .. } => &[],
        }
    }

    /// Check if user has a specific role.
    pub fn has_role(&self, role: Role) -> bool {
        self.roles().contains(&role)
    }

    /// Check if user has at least the given permission level.
    pub fn has_permission(&self, required: Role) -> bool {
        self.roles().iter().any(|r| r.has_permission(required))
    }

    /// Check if email is verified.
    pub fn is_email_verified(&self) -> bool {
        match self {
            User::Authenticated { email_verified, .. } => *email_verified,
            User::Anonymous { .. } => false,
        }
    }
}

impl Default for User {
    fn default() -> Self {
        User::anonymous(generate_session_id())
    }
}

/// Stored user credentials (for database).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    /// User ID.
    pub user_id: UserId,
    /// Email address.
    pub email: String,
    /// Hashed password.
    pub password_hash: String,
    /// Whether email is verified.
    pub email_verified: bool,
    /// Number of failed login attempts.
    pub failed_attempts: i32,
    /// Timestamp when account was locked (if applicable).
    pub locked_until: Option<i64>,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

impl UserCredentials {
    /// Create new credentials.
    pub fn new(
        user_id: UserId,
        email: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Self {
        let now = current_timestamp();
        Self {
            user_id,
            email: email.into(),
            password_hash: password_hash.into(),
            email_verified: false,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if account is locked.
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            current_timestamp() < locked_until
        } else {
            false
        }
    }

    /// Record a failed login attempt.
    pub fn record_failed_attempt(&mut self, max_attempts: i32, lock_duration_secs: i64) {
        self.failed_attempts += 1;
        self.updated_at = current_timestamp();

        if self.failed_attempts >= max_attempts {
            self.locked_until = Some(current_timestamp() + lock_duration_secs);
        }
    }

    /// Reset failed attempts (on successful login).
    pub fn reset_failed_attempts(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
        self.updated_at = current_timestamp();
    }

    /// Mark email as verified.
    pub fn verify_email(&mut self) {
        self.email_verified = true;
        self.updated_at = current_timestamp();
    }

    /// Update password hash.
    pub fn set_password_hash(&mut self, hash: impl Into<String>) {
        self.password_hash = hash.into();
        self.updated_at = current_timestamp();
    }
}

/// User profile data (public-facing).
#[allow(dead_code)] // Public API for library users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID.
    pub id: UserId,
    /// Email address.
    pub email: String,
    /// Display name.
    pub name: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// User roles.
    pub roles: Vec<Role>,
    /// Whether email is verified.
    pub email_verified: bool,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last login.
    pub last_login_at: Option<i64>,
}

#[allow(dead_code)] // Public API for library users
impl UserProfile {
    /// Create profile from user.
    pub fn from_user(user: &User, created_at: i64, last_login_at: Option<i64>) -> Option<Self> {
        match user {
            User::Authenticated {
                id,
                email,
                name,
                roles,
                email_verified,
            } => Some(Self {
                id: id.clone(),
                email: email.clone(),
                name: name.clone(),
                phone: None,
                roles: roles.clone(),
                email_verified: *email_verified,
                created_at,
                last_login_at,
            }),
            User::Anonymous { .. } => None,
        }
    }
}

/// Generate a random session ID.
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("sess_{:x}", ts)
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
    fn test_role_permissions() {
        assert!(Role::Admin.has_permission(Role::Customer));
        assert!(Role::Admin.has_permission(Role::Staff));
        assert!(Role::Admin.has_permission(Role::Admin));
        assert!(!Role::Admin.has_permission(Role::SuperAdmin));
    }

    #[test]
    fn test_anonymous_user() {
        let user = User::anonymous("test-session");
        assert!(user.is_anonymous());
        assert!(!user.is_authenticated());
        assert!(user.user_id().is_none());
    }

    #[test]
    fn test_authenticated_user() {
        let user = User::authenticated(
            UserId::new("user_123"),
            "test@example.com",
            Some("Test User".to_string()),
            vec![Role::Customer],
        );
        assert!(user.is_authenticated());
        assert!(!user.is_anonymous());
        assert!(user.user_id().is_some());
        assert_eq!(user.email(), Some("test@example.com"));
    }

    #[test]
    fn test_user_permissions() {
        let admin = User::authenticated(
            UserId::new("admin_1"),
            "admin@example.com",
            None,
            vec![Role::Admin],
        );
        assert!(admin.has_permission(Role::Customer));
        assert!(admin.has_permission(Role::Staff));
        assert!(!admin.has_permission(Role::SuperAdmin));
    }
}
