//! Key-Value store wrapper with automatic serialization.

use crate::CacheError;
use serde::{de::DeserializeOwned, Serialize};

/// Type-safe cache backed by Spin's Key-Value Store.
///
/// Provides automatic JSON serialization for any type that implements
/// `Serialize` and `DeserializeOwned`.
pub struct Cache {
    #[cfg(target_arch = "wasm32")]
    store: spin_sdk::key_value::Store,
    #[cfg(not(target_arch = "wasm32"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Cache {
    /// Open the default Key-Value store.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cache = Cache::open_default()?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn open_default() -> Result<Self, CacheError> {
        let store = spin_sdk::key_value::Store::open_default()
            .map_err(|e| CacheError::OpenError(e.to_string()))?;
        Ok(Self { store })
    }

    /// Open a named Key-Value store.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cache = Cache::open("my-store")?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn open(name: &str) -> Result<Self, CacheError> {
        let store = spin_sdk::key_value::Store::open(name)
            .map_err(|e| CacheError::OpenError(e.to_string()))?;
        Ok(Self { store })
    }

    /// Get a value from the cache.
    ///
    /// Returns `None` if the key doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cart: Option<Cart> = cache.get("cart:user123")?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        match self.store.get(key) {
            Ok(Some(bytes)) => {
                let value: T = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(CacheError::StoreError(e.to_string())),
        }
    }

    /// Set a value in the cache.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// cache.set("cart:user123", &cart)?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        let bytes = serde_json::to_vec(value)?;
        self.store
            .set(key, &bytes)
            .map_err(|e| CacheError::StoreError(e.to_string()))
    }

    /// Delete a value from the cache.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// cache.delete("cart:user123")?;
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.store
            .delete(key)
            .map_err(|e| CacheError::StoreError(e.to_string()))
    }

    /// Check if a key exists in the cache.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if cache.exists("cart:user123")? {
    ///     // Key exists
    /// }
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn exists(&self, key: &str) -> Result<bool, CacheError> {
        self.store
            .exists(key)
            .map_err(|e| CacheError::StoreError(e.to_string()))
    }

    /// Get all keys in the cache.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let keys = cache.keys()?;
    /// for key in keys {
    ///     println!("Key: {}", key);
    /// }
    /// ```
    #[cfg(target_arch = "wasm32")]
    pub fn keys(&self) -> Result<Vec<String>, CacheError> {
        self.store
            .get_keys()
            .map_err(|e| CacheError::StoreError(e.to_string()))
    }

    // Non-WASM stubs for development/testing
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_default() -> Result<Self, CacheError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(_name: &str) -> Result<Self, CacheError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get<T: DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, CacheError> {
        Ok(None)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set<T: Serialize>(&self, _key: &str, _value: &T) -> Result<(), CacheError> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn delete(&self, _key: &str) -> Result<(), CacheError> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn exists(&self, _key: &str) -> Result<bool, CacheError> {
        Ok(false)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn keys(&self) -> Result<Vec<String>, CacheError> {
        Ok(vec![])
    }
}

/// Helper to build cache keys with namespacing.
///
/// # Example
///
/// ```rust,ignore
/// let key = cache_key!("cart", user_id);
/// // Returns "cart:user123"
/// ```
#[macro_export]
macro_rules! cache_key {
    ($prefix:expr, $($part:expr),+) => {{
        let mut key = String::from($prefix);
        $(
            key.push(':');
            key.push_str(&$part.to_string());
        )+
        key
    }};
}
