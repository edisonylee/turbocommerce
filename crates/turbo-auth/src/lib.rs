//! Authentication module for TurboCommerce.
//!
//! Provides user authentication, session management, and authorization.

mod error;
mod user;
mod session;
mod password;
mod token;

pub use error::AuthError;
pub use user::{User, UserCredentials, Role};
pub use session::{AuthSession, SessionId};
pub use password::PasswordHasher;
pub use token::{AuthToken, TokenType};
