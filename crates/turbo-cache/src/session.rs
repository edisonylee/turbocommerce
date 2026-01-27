//! Session management using Key-Value store.

use crate::{Cache, CacheError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A unique session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new random session ID.
    ///
    /// Uses a simple timestamp-based ID. In production, you'd want
    /// a more robust random ID generator.
    pub fn generate() -> Self {
        // Simple ID generation - in production use a proper UUID
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(format!("sess_{}", timestamp))
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
/// #[derive(Serialize, Deserialize, Default)]
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
    T: Serialize + DeserializeOwned + Default,
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
                self.set(id, &data)?;
                Ok(data)
            }
        }
    }

    /// Get session data if it exists.
    pub fn get(&self, id: &SessionId) -> Result<Option<T>, CacheError> {
        let key = self.session_key(id);
        Ok(self.cache.get::<SessionData<T>>(&key)?.map(|s| s.data))
    }

    /// Set session data.
    pub fn set(&self, id: &SessionId, data: &T) -> Result<(), CacheError> {
        let key = self.session_key(id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let session_data = SessionData {
            id: id.clone(),
            data: data.clone(),
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

    fn session_key(&self, id: &SessionId) -> String {
        format!("session:{}", id)
    }
}

impl<T> Session<T>
where
    T: Serialize + DeserializeOwned + Default + Clone,
{
    /// Update session data with a closure.
    pub fn update<F>(&self, id: &SessionId, f: F) -> Result<T, CacheError>
    where
        F: FnOnce(&mut T),
    {
        let mut data = self.get_or_create(id)?;
        f(&mut data);
        self.set(id, &data)?;
        Ok(data)
    }
}
