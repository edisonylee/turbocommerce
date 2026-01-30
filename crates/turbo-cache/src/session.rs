//! Session management using Key-Value store.

use crate::{Cache, CacheError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Maximum retry attempts for optimistic concurrency control.
const MAX_UPDATE_RETRIES: u32 = 3;

/// A unique session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new cryptographically secure session ID.
    pub fn generate() -> Self {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        use rand::Rng;

        let bytes: [u8; 18] = rand::thread_rng().gen();
        Self(format!("sess_{}", URL_SAFE_NO_PAD.encode(bytes)))
    }

    /// Get the session ID as a string.
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

/// Session data stored in the cache.
///
/// Generic over the user data type `T`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData<T> {
    /// The session ID.
    pub id: SessionId,
    /// User-defined session data.
    pub data: T,
    /// Version for optimistic concurrency control.
    pub version: u64,
    /// When the session was created (Unix timestamp).
    pub created_at: u64,
    /// When the session was last accessed (Unix timestamp).
    pub last_accessed: u64,
}

/// Session manager for user sessions.
///
/// # Example
///
/// ```rust,ignore
/// use turbo_cache::{Session, SessionId};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Default, Clone)]
/// struct UserSession {
///     user_id: Option<String>,
///     cart_id: Option<String>,
/// }
///
/// let session = Session::<UserSession>::new()?;
///
/// // Get or create a session
/// let session_id = SessionId::from("abc123");
/// let data = session.get_or_create(&session_id)?;
///
/// // Update session data
/// let mut data = data;
/// data.user_id = Some("user456".to_string());
/// session.set(&session_id, &data)?;
/// ```
pub struct Session<T> {
    cache: Cache,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Session<T>
where
    T: Serialize + DeserializeOwned + Default + Clone,
{
    /// Create a new session manager using the default store.
    pub fn new() -> Result<Self, CacheError> {
        Ok(Self {
            cache: Cache::open_default()?,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create a new session manager using a named store.
    pub fn with_store(name: &str) -> Result<Self, CacheError> {
        Ok(Self {
            cache: Cache::open(name)?,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get session data, or create a new session if it doesn't exist.
    pub fn get_or_create(&self, id: &SessionId) -> Result<T, CacheError> {
        let key = self.session_key(id);
        match self.cache.get::<SessionData<T>>(&key)? {
            Some(session_data) => Ok(session_data.data),
            None => {
                let data = T::default();
                self.set_internal(id, &data, 1)?;
                Ok(data)
            }
        }
    }

    /// Get session data if it exists.
    pub fn get(&self, id: &SessionId) -> Result<Option<T>, CacheError> {
        let key = self.session_key(id);
        Ok(self.cache.get::<SessionData<T>>(&key)?.map(|s| s.data))
    }

    /// Get full session data including version (for advanced use).
    pub fn get_versioned(&self, id: &SessionId) -> Result<Option<SessionData<T>>, CacheError> {
        let key = self.session_key(id);
        self.cache.get::<SessionData<T>>(&key)
    }

    /// Set session data (unconditional write).
    pub fn set(&self, id: &SessionId, data: &T) -> Result<(), CacheError> {
        // Get current version or start at 1
        let key = self.session_key(id);
        let version = self
            .cache
            .get::<SessionData<T>>(&key)?
            .map(|s| s.version + 1)
            .unwrap_or(1);
        self.set_internal(id, data, version)
    }

    /// Internal set with explicit version.
    fn set_internal(&self, id: &SessionId, data: &T, version: u64) -> Result<(), CacheError> {
        let key = self.session_key(id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let session_data = SessionData {
            id: id.clone(),
            data: data.clone(),
            version,
            created_at: now,
            last_accessed: now,
        };

        self.cache.set(&key, &session_data)
    }

    /// Delete a session.
    pub fn delete(&self, id: &SessionId) -> Result<(), CacheError> {
        let key = self.session_key(id);
        self.cache.delete(&key)
    }

    /// Check if a session exists.
    pub fn exists(&self, id: &SessionId) -> Result<bool, CacheError> {
        let key = self.session_key(id);
        self.cache.exists(&key)
    }

    /// Update session data with a closure, using optimistic concurrency control.
    ///
    /// This method retries up to MAX_UPDATE_RETRIES times if a concurrent
    /// modification is detected. The closure receives the current data and
    /// should return the modified data.
    ///
    /// # Returns
    /// - `Ok(T)` - The updated data after successful write
    /// - `Err(CacheError::ConcurrentModification)` - If all retries failed
    pub fn update<F>(&self, id: &SessionId, f: F) -> Result<T, CacheError>
    where
        F: Fn(&mut T),
    {
        let key = self.session_key(id);

        for _attempt in 0..MAX_UPDATE_RETRIES {
            // Read current state
            let current = self.cache.get::<SessionData<T>>(&key)?;

            let (mut data, expected_version) = match current {
                Some(session_data) => (session_data.data, session_data.version),
                None => (T::default(), 0),
            };

            // Apply the update
            f(&mut data);

            // Try to write with incremented version
            let new_version = expected_version + 1;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let session_data = SessionData {
                id: id.clone(),
                data: data.clone(),
                version: new_version,
                created_at: now,
                last_accessed: now,
            };

            // Write the data
            self.cache.set(&key, &session_data)?;

            // Verify the write succeeded with our version
            // (In a real implementation with CAS support, this would be atomic)
            if let Some(written) = self.cache.get::<SessionData<T>>(&key)? {
                if written.version == new_version {
                    return Ok(data);
                }
                // Version mismatch - another writer got in, retry
                continue;
            }

            return Ok(data);
        }

        Err(CacheError::ConcurrentModification(
            "max retries exceeded".to_string(),
        ))
    }

    fn session_key(&self, id: &SessionId) -> String {
        format!("session:{}", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_new() {
        let id = SessionId::new("abc123");
        assert_eq!(id.as_str(), "abc123");
    }

    #[test]
    fn test_session_id_from_string() {
        let id = SessionId::from(String::from("xyz789"));
        assert_eq!(id.as_str(), "xyz789");
    }

    #[test]
    fn test_session_id_from_str() {
        let id = SessionId::from("test-session");
        assert_eq!(id.as_str(), "test-session");
    }

    #[test]
    fn test_session_id_display() {
        let id = SessionId::new("display-test");
        assert_eq!(format!("{}", id), "display-test");
    }

    #[test]
    fn test_session_id_generate_format() {
        let id = SessionId::generate();
        let s = id.as_str();

        // Should start with "sess_"
        assert!(s.starts_with("sess_"));

        // Base64 encoded 18 bytes = 24 chars, plus "sess_" = 29 chars
        assert_eq!(s.len(), 29);
    }

    #[test]
    fn test_session_id_generate_uniqueness() {
        let id1 = SessionId::generate();
        let id2 = SessionId::generate();

        // Two generated IDs should be different
        assert_ne!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_session_id_equality() {
        let id1 = SessionId::new("same");
        let id2 = SessionId::new("same");
        let id3 = SessionId::new("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_session_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SessionId::new("a"));
        set.insert(SessionId::new("b"));
        set.insert(SessionId::new("a")); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_session_id_clone() {
        let id1 = SessionId::new("cloneable");
        let id2 = id1.clone();

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_session_id_serialization() {
        let id = SessionId::new("serialize-me");
        let json = serde_json::to_string(&id).unwrap();

        assert_eq!(json, r#""serialize-me""#);

        let deserialized: SessionId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }
}
