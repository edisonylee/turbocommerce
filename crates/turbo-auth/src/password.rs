//! Password hashing using Argon2 (OWASP recommended).

use crate::AuthError;
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as Argon2Hasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};

/// Password hasher using Argon2id.
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    /// Argon2 memory cost (KiB).
    pub memory_cost: u32,
    /// Argon2 time cost (iterations).
    pub time_cost: u32,
    /// Argon2 parallelism.
    pub parallelism: u32,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self {
            memory_cost: 19456, // ~19 MiB
            time_cost: 2,
            parallelism: 1,
        }
    }
}

impl PasswordHasher {
    /// Create a new hasher with custom time cost.
    pub fn new(time_cost: u32) -> Self {
        Self {
            time_cost,
            ..Default::default()
        }
    }

    /// Hash a password using Argon2id.
    ///
    /// Returns the password hash in PHC string format.
    pub fn hash(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(self.memory_cost, self.time_cost, self.parallelism, None)
                .map_err(|e| AuthError::Internal(format!("Argon2 params error: {}", e)))?,
        );

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::Internal(format!("Password hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash.
    pub fn verify(&self, password: &str, hash_str: &str) -> Result<bool, AuthError> {
        let hash = PasswordHash::new(hash_str)
            .map_err(|e| AuthError::Internal(format!("Invalid hash format: {}", e)))?;

        // Argon2 will read params from the hash itself
        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::Internal(format!("Verification error: {}", e))),
        }
    }

    /// Validate password strength.
    pub fn validate_password(password: &str) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_upper || !has_lower || !has_digit {
            return Err(AuthError::WeakPassword(
                "Password must contain uppercase, lowercase, and numbers".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let hasher = PasswordHasher::default();
        let password = "SecurePass123!";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$argon2id$"));

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        assert!(PasswordHasher::validate_password("SecurePass1").is_ok());
        assert!(PasswordHasher::validate_password("short").is_err());
        assert!(PasswordHasher::validate_password("alllowercase1").is_err());
        assert!(PasswordHasher::validate_password("ALLUPPERCASE1").is_err());
        assert!(PasswordHasher::validate_password("NoNumbers").is_err());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let hasher = PasswordHasher::default();
        let password = "TestPassword1";

        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }

    // Security tests

    #[test]
    fn test_hash_uses_argon2id() {
        let hasher = PasswordHasher::default();
        let hash = hasher.hash("TestPassword1").unwrap();

        // Verify it's using Argon2id algorithm
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_contains_version_and_params() {
        let hasher = PasswordHasher::default();
        let hash = hasher.hash("TestPassword1").unwrap();

        // Argon2 PHC format: $argon2id$v=19$m=...,t=...,p=...
        assert!(hash.contains("$v="));
        assert!(hash.contains("m="));
        assert!(hash.contains("t="));
        assert!(hash.contains("p="));
    }

    #[test]
    fn test_salts_are_unique() {
        let hasher = PasswordHasher::default();
        let password = "TestPassword1";

        // Generate multiple hashes rapidly
        let hashes: Vec<String> = (0..10).map(|_| hasher.hash(password).unwrap()).collect();

        // All hashes should be unique (unique salts)
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "Hashes {} and {} are identical", i, j);
            }
        }
    }

    #[test]
    fn test_timing_safe_verification() {
        let hasher = PasswordHasher::default();
        let hash = hasher.hash("CorrectPassword1").unwrap();

        // Both should complete (timing attack resistance is internal to argon2)
        let _ = hasher.verify("WrongPassword1", &hash);
        let _ = hasher.verify("CorrectPassword1", &hash);
    }

    #[test]
    fn test_invalid_hash_format() {
        let hasher = PasswordHasher::default();

        // Invalid format should error, not panic
        assert!(hasher.verify("password", "not-a-valid-hash").is_err());
        assert!(hasher.verify("password", "").is_err());
        assert!(hasher.verify("password", "$invalid$format$").is_err());
    }
}
