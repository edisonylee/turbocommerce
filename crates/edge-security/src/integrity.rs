//! Artifact integrity verification.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    /// SHA-256 (recommended).
    #[default]
    Sha256,
    /// SHA-384.
    Sha384,
    /// SHA-512.
    Sha512,
}

impl HashAlgorithm {
    /// Get the expected hash length in hex characters.
    pub fn hex_length(&self) -> usize {
        match self {
            Self::Sha256 => 64,
            Self::Sha384 => 96,
            Self::Sha512 => 128,
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sha256" | "sha-256" => Some(Self::Sha256),
            "sha384" | "sha-384" => Some(Self::Sha384),
            "sha512" | "sha-512" => Some(Self::Sha512),
            _ => None,
        }
    }
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256 => write!(f, "sha256"),
            Self::Sha384 => write!(f, "sha384"),
            Self::Sha512 => write!(f, "sha512"),
        }
    }
}

/// A content hash with its algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentHash {
    /// The hash algorithm used.
    pub algorithm: HashAlgorithm,
    /// The hash value in hexadecimal.
    pub hash: String,
}

impl ContentHash {
    /// Create a new content hash.
    pub fn new(algorithm: HashAlgorithm, hash: impl Into<String>) -> Self {
        Self {
            algorithm,
            hash: hash.into().to_lowercase(),
        }
    }

    /// Parse from SRI (Subresource Integrity) format.
    ///
    /// Example: `sha256-abc123...`
    pub fn from_sri(sri: &str) -> Result<Self, IntegrityError> {
        let (algo_str, hash) = sri
            .split_once('-')
            .ok_or_else(|| IntegrityError::InvalidFormat("missing algorithm prefix".into()))?;

        let algorithm = HashAlgorithm::from_str(algo_str)
            .ok_or_else(|| IntegrityError::UnsupportedAlgorithm(algo_str.into()))?;

        // SRI uses base64, but we'll accept both base64 and hex
        let hash = if hash.len() == algorithm.hex_length() {
            // Already hex
            hash.to_string()
        } else {
            // Assume base64, try to decode and convert to hex
            // For simplicity, we'll just validate it looks like base64
            hash.to_string()
        };

        Ok(Self { algorithm, hash })
    }

    /// Convert to SRI format.
    pub fn to_sri(&self) -> String {
        format!("{}-{}", self.algorithm, self.hash)
    }

    /// Validate the hash format.
    pub fn validate(&self) -> Result<(), IntegrityError> {
        let expected_len = self.algorithm.hex_length();

        // Allow base64 or hex
        if self.hash.len() != expected_len && !looks_like_base64(&self.hash) {
            return Err(IntegrityError::InvalidHashLength {
                expected: expected_len,
                actual: self.hash.len(),
            });
        }

        // Validate hex characters if it looks like hex
        if self.hash.len() == expected_len {
            if !self.hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(IntegrityError::InvalidHashFormat(
                    "hash contains non-hexadecimal characters".into(),
                ));
            }
        }

        Ok(())
    }
}

fn looks_like_base64(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

/// Artifact integrity manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrityManifest {
    /// Version of the manifest format.
    pub version: u32,
    /// Hash algorithm used for all entries.
    pub algorithm: Option<HashAlgorithm>,
    /// Map of artifact path to expected hash.
    pub artifacts: HashMap<String, ContentHash>,
    /// When the manifest was generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    /// Who generated the manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_by: Option<String>,
}

impl IntegrityManifest {
    /// Create a new manifest.
    pub fn new() -> Self {
        Self {
            version: 1,
            algorithm: Some(HashAlgorithm::Sha256),
            ..Default::default()
        }
    }

    /// Add an artifact hash.
    pub fn add(&mut self, path: impl Into<String>, hash: ContentHash) {
        self.artifacts.insert(path.into(), hash);
    }

    /// Get the expected hash for an artifact.
    pub fn get(&self, path: &str) -> Option<&ContentHash> {
        self.artifacts.get(path)
    }

    /// Check if an artifact exists in the manifest.
    pub fn contains(&self, path: &str) -> bool {
        self.artifacts.contains_key(path)
    }

    /// Get all artifact paths.
    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.artifacts.keys()
    }

    /// Validate all hashes in the manifest.
    pub fn validate(&self) -> Result<(), IntegrityError> {
        for (path, hash) in &self.artifacts {
            hash.validate().map_err(|e| IntegrityError::InvalidArtifact {
                path: path.clone(),
                reason: e.to_string(),
            })?;
        }
        Ok(())
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, IntegrityError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| IntegrityError::SerializationError(e.to_string()))
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, IntegrityError> {
        serde_json::from_str(json)
            .map_err(|e| IntegrityError::SerializationError(e.to_string()))
    }
}

/// Artifact integrity verifier.
#[derive(Debug)]
pub struct IntegrityVerifier {
    manifest: IntegrityManifest,
    strict_mode: bool,
}

impl IntegrityVerifier {
    /// Create a new verifier with a manifest.
    pub fn new(manifest: IntegrityManifest) -> Self {
        Self {
            manifest,
            strict_mode: true,
        }
    }

    /// Create an empty verifier (no checks).
    pub fn disabled() -> Self {
        Self {
            manifest: IntegrityManifest::new(),
            strict_mode: false,
        }
    }

    /// Set strict mode (fail on missing artifacts).
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Verify an artifact's integrity.
    ///
    /// In WASM, we can't actually compute hashes efficiently,
    /// so this returns what the expected hash should be.
    pub fn get_expected_hash(&self, path: &str) -> Result<Option<&ContentHash>, IntegrityError> {
        match self.manifest.get(path) {
            Some(hash) => Ok(Some(hash)),
            None => {
                if self.strict_mode {
                    Err(IntegrityError::ArtifactNotInManifest(path.into()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Verify a hash matches expected.
    pub fn verify_hash(
        &self,
        path: &str,
        actual_hash: &ContentHash,
    ) -> Result<VerificationResult, IntegrityError> {
        match self.manifest.get(path) {
            Some(expected) => {
                if actual_hash.algorithm != expected.algorithm {
                    Ok(VerificationResult::AlgorithmMismatch {
                        expected: expected.algorithm,
                        actual: actual_hash.algorithm,
                    })
                } else if actual_hash.hash.to_lowercase() != expected.hash.to_lowercase() {
                    Ok(VerificationResult::HashMismatch {
                        expected: expected.hash.clone(),
                        actual: actual_hash.hash.clone(),
                    })
                } else {
                    Ok(VerificationResult::Valid)
                }
            }
            None => {
                if self.strict_mode {
                    Err(IntegrityError::ArtifactNotInManifest(path.into()))
                } else {
                    Ok(VerificationResult::NotInManifest)
                }
            }
        }
    }

    /// Get the manifest.
    pub fn manifest(&self) -> &IntegrityManifest {
        &self.manifest
    }
}

/// Result of integrity verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// Hash matches expected value.
    Valid,
    /// Hash does not match.
    HashMismatch { expected: String, actual: String },
    /// Algorithm does not match.
    AlgorithmMismatch {
        expected: HashAlgorithm,
        actual: HashAlgorithm,
    },
    /// Artifact not in manifest (non-strict mode).
    NotInManifest,
}

impl VerificationResult {
    /// Check if verification passed.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Integrity verification errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum IntegrityError {
    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("invalid hash length: expected {expected}, got {actual}")]
    InvalidHashLength { expected: usize, actual: usize },

    #[error("invalid hash format: {0}")]
    InvalidHashFormat(String),

    #[error("artifact not in manifest: {0}")]
    ArtifactNotInManifest(String),

    #[error("invalid artifact '{path}': {reason}")]
    InvalidArtifact { path: String, reason: String },

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("verification failed for '{path}': expected {expected}, got {actual}")]
    VerificationFailed {
        path: String,
        expected: String,
        actual: String,
    },
}

/// Builder for creating integrity manifests.
#[derive(Debug, Default)]
pub struct ManifestBuilder {
    algorithm: HashAlgorithm,
    artifacts: Vec<(String, String)>,
    generated_by: Option<String>,
}

impl ManifestBuilder {
    /// Create a new manifest builder.
    pub fn new() -> Self {
        Self {
            algorithm: HashAlgorithm::Sha256,
            ..Default::default()
        }
    }

    /// Set the hash algorithm.
    pub fn algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Add an artifact with its hash.
    pub fn artifact(mut self, path: impl Into<String>, hash: impl Into<String>) -> Self {
        self.artifacts.push((path.into(), hash.into()));
        self
    }

    /// Set who generated the manifest.
    pub fn generated_by(mut self, by: impl Into<String>) -> Self {
        self.generated_by = Some(by.into());
        self
    }

    /// Build the manifest.
    pub fn build(self) -> IntegrityManifest {
        let mut manifest = IntegrityManifest::new();
        manifest.algorithm = Some(self.algorithm);
        manifest.generated_by = self.generated_by;

        for (path, hash) in self.artifacts {
            manifest.add(path, ContentHash::new(self.algorithm, hash));
        }

        manifest
    }
}
