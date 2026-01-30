//! Authentication module for TurboCommerce.
//!
//! Provides user authentication, session management, and authorization.

mod error;
mod password;
mod session;
mod token;
mod user;

pub use error::AuthError;
pub use password::PasswordHasher;
pub use session::{AuthSession, SessionId};
pub use token::{AuthToken, TokenType};
pub use user::{Role, User, UserCredentials};
