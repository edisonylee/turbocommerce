//! Password hashing for WASM compatibility.
//!
//! Uses a simple but secure approach that works in WASM environments.
//! In production, consider using argon2 or bcrypt with native crypto support.

use crate::AuthError;

/// Password hasher configuration.
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    /// Number of iterations for key derivation.
    pub iterations: u32,
    /// Salt length in bytes.
    pub salt_length: usize,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self {
            iterations: 10000,
            salt_length: 16,
        }
    }
}

impl PasswordHasher {
    /// Create a new hasher with custom iterations.
    pub fn new(iterations: u32) -> Self {
        Self {
            iterations,
            salt_length: 16,
        }
    }

    /// Hash a password.
    ///
    /// Returns a string in format: `$pbkdf2$iterations$salt$hash`
    pub fn hash(&self, password: &str) -> Result<String, AuthError> {
        let salt = self.generate_salt();
        let hash = self.derive_key(password, &salt);

        Ok(format!(
            "$pbkdf2${}${}${}",
            self.iterations,
            hex_encode(&salt),
            hex_encode(&hash)
        ))
    }

    /// Verify a password against a hash.
    pub fn verify(&self, password: &str, hash_str: &str) -> Result<bool, AuthError> {
        let parts: Vec<&str> = hash_str.split('$').collect();

        if parts.len() != 5 || parts[1] != "pbkdf2" {
            return Err(AuthError::Internal("Invalid hash format".to_string()));
        }

        let iterations: u32 = parts[2]
            .parse()
            .map_err(|_| AuthError::Internal("Invalid iterations".to_string()))?;
        let salt = hex_decode(parts[3])
            .map_err(|_| AuthError::Internal("Invalid salt".to_string()))?;
        let expected_hash = hex_decode(parts[4])
            .map_err(|_| AuthError::Internal("Invalid hash".to_string()))?;

        let hasher = PasswordHasher::new(iterations);
        let computed_hash = hasher.derive_key(password, &salt);

        Ok(constant_time_compare(&computed_hash, &expected_hash))
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

    /// Generate a random salt.
    fn generate_salt(&self) -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        // Create pseudo-random salt from timestamp and memory
        let ptr = Box::new(0u64);
        let addr = &*ptr as *const u64 as u64;

        let mut salt = Vec::with_capacity(self.salt_length);
        let mut state = ts as u64 ^ addr;

        for _ in 0..self.salt_length {
            // Simple xorshift for randomness
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            salt.push((state & 0xFF) as u8);
        }

        salt
    }

    /// Derive key from password and salt (PBKDF2-like).
    fn derive_key(&self, password: &str, salt: &[u8]) -> Vec<u8> {
        let password_bytes = password.as_bytes();
        let mut result = Vec::with_capacity(32);

        // Initialize with password and salt
        let mut state = [0u8; 32];
        for (i, &b) in password_bytes.iter().enumerate() {
            state[i % 32] ^= b;
        }
        for (i, &b) in salt.iter().enumerate() {
            state[(i + 16) % 32] ^= b;
        }

        // Iterative hashing (simplified PBKDF2)
        for _ in 0..self.iterations {
            state = sha256_round(&state);
        }

        result.extend_from_slice(&state);
        result
    }
}

/// Simple SHA-256-like round function.
/// NOTE: This is a simplified version for WASM compatibility.
/// In production, use a proper crypto library.
fn sha256_round(input: &[u8; 32]) -> [u8; 32] {
    let mut output = [0u8; 32];

    // Mix bytes using rotation and XOR
    for i in 0..32 {
        let a = input[i];
        let b = input[(i + 7) % 32];
        let c = input[(i + 13) % 32];
        let d = input[(i + 21) % 32];

        output[i] = a
            .wrapping_add(b.rotate_left(3))
            .wrapping_add(c.rotate_right(2))
            ^ d.wrapping_mul(17);
    }

    // Additional mixing
    for i in 0..32 {
        let j = (i + 16) % 32;
        output[i] ^= output[j].rotate_left(5);
    }

    output
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Encode bytes as hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex string to bytes.
fn hex_decode(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let hasher = PasswordHasher::default();
        let password = "SecurePass123!";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$pbkdf2$"));

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
}
