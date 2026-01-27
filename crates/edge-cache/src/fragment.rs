//! Fragment caching with stampede protection.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::key::FragmentKey;
use crate::policy::SectionCachePolicy;

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache operation errors.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Cache miss - item not found.
    #[error("cache miss")]
    Miss,

    /// Cache entry has expired.
    #[error("cache entry expired")]
    Expired,

    /// Failed to serialize/deserialize cache entry.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Backend storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Lock acquisition failed (stampede protection).
    #[error("failed to acquire lock: {0}")]
    LockFailed(String),

    /// Operation timed out.
    #[error("operation timed out")]
    Timeout,
}

/// Status of a cache lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheStatus {
    /// Fresh cache hit.
    Hit,
    /// Cache miss.
    Miss,
    /// Stale hit (serving while revalidating).
    Stale,
    /// Bypass - caching disabled.
    Bypass,
    /// Error during cache operation.
    Error,
}

impl std::fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hit => write!(f, "HIT"),
            Self::Miss => write!(f, "MISS"),
            Self::Stale => write!(f, "STALE"),
            Self::Bypass => write!(f, "BYPASS"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A cached fragment entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFragment {
    /// The cached HTML content.
    pub content: String,
    /// When the entry was created.
    pub created_at: u64,
    /// Time-to-live in seconds.
    pub ttl_secs: u64,
    /// Cache tags for invalidation.
    pub tags: Vec<String>,
    /// ETag for conditional requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

impl CachedFragment {
    /// Create a new cached fragment.
    pub fn new(content: impl Into<String>, ttl: Duration) -> Self {
        Self {
            content: content.into(),
            created_at: current_timestamp(),
            ttl_secs: ttl.as_secs(),
            tags: Vec::new(),
            etag: None,
        }
    }

    /// Add cache tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set ETag.
    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    /// Check if the entry has expired.
    pub fn is_expired(&self) -> bool {
        let now = current_timestamp();
        now > self.created_at + self.ttl_secs
    }

    /// Get remaining TTL in seconds.
    pub fn remaining_ttl(&self) -> u64 {
        let now = current_timestamp();
        let expires_at = self.created_at + self.ttl_secs;
        if now >= expires_at {
            0
        } else {
            expires_at - now
        }
    }

    /// Get age in seconds.
    pub fn age(&self) -> u64 {
        let now = current_timestamp();
        now.saturating_sub(self.created_at)
    }
}

/// Result of a cache get operation with metadata.
#[derive(Debug)]
pub struct CacheGetResult {
    /// The cached fragment (if found).
    pub fragment: Option<CachedFragment>,
    /// Cache status.
    pub status: CacheStatus,
    /// Whether revalidation is needed.
    pub needs_revalidation: bool,
}

impl CacheGetResult {
    /// Create a hit result.
    pub fn hit(fragment: CachedFragment) -> Self {
        Self {
            fragment: Some(fragment),
            status: CacheStatus::Hit,
            needs_revalidation: false,
        }
    }

    /// Create a stale result.
    pub fn stale(fragment: CachedFragment) -> Self {
        Self {
            fragment: Some(fragment),
            status: CacheStatus::Stale,
            needs_revalidation: true,
        }
    }

    /// Create a miss result.
    pub fn miss() -> Self {
        Self {
            fragment: None,
            status: CacheStatus::Miss,
            needs_revalidation: false,
        }
    }

    /// Create a bypass result.
    pub fn bypass() -> Self {
        Self {
            fragment: None,
            status: CacheStatus::Bypass,
            needs_revalidation: false,
        }
    }

    /// Create an error result.
    pub fn error() -> Self {
        Self {
            fragment: None,
            status: CacheStatus::Error,
            needs_revalidation: false,
        }
    }
}

/// Fragment cache backend trait.
#[async_trait]
pub trait FragmentCacheBackend: Send + Sync {
    /// Get a cached fragment.
    async fn get(&self, key: &str) -> CacheResult<Option<CachedFragment>>;

    /// Store a fragment in the cache.
    async fn set(&self, key: &str, fragment: CachedFragment) -> CacheResult<()>;

    /// Delete a cached fragment.
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Invalidate all entries with a given tag.
    async fn invalidate_tag(&self, tag: &str) -> CacheResult<u64>;

    /// Try to acquire a lock for stampede protection.
    async fn try_lock(&self, key: &str, ttl: Duration) -> CacheResult<bool>;

    /// Release a lock.
    async fn unlock(&self, key: &str) -> CacheResult<()>;
}

/// Fragment cache with stampede protection.
pub struct FragmentCache<B: FragmentCacheBackend> {
    backend: Arc<B>,
    /// Grace period for stale-while-revalidate.
    stale_grace_period: Duration,
    /// Lock TTL for stampede protection.
    lock_ttl: Duration,
}

impl<B: FragmentCacheBackend> FragmentCache<B> {
    /// Create a new fragment cache.
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            stale_grace_period: Duration::from_secs(60),
            lock_ttl: Duration::from_secs(30),
        }
    }

    /// Set the stale grace period.
    pub fn with_stale_grace_period(mut self, duration: Duration) -> Self {
        self.stale_grace_period = duration;
        self
    }

    /// Set the lock TTL.
    pub fn with_lock_ttl(mut self, duration: Duration) -> Self {
        self.lock_ttl = duration;
        self
    }

    /// Get a cached fragment.
    pub async fn get(
        &self,
        key: &FragmentKey,
        policy: &SectionCachePolicy,
    ) -> CacheGetResult {
        if !policy.enabled {
            return CacheGetResult::bypass();
        }

        let key_str = key.as_str();

        match self.backend.get(&key_str).await {
            Ok(Some(fragment)) => {
                if fragment.is_expired() {
                    // Check if within stale grace period
                    let stale_ok = policy.stale_on_error
                        && fragment.age() < policy.ttl.as_secs() + self.stale_grace_period.as_secs();

                    if stale_ok {
                        CacheGetResult::stale(fragment)
                    } else {
                        CacheGetResult::miss()
                    }
                } else {
                    CacheGetResult::hit(fragment)
                }
            }
            Ok(None) => CacheGetResult::miss(),
            Err(_) => CacheGetResult::error(),
        }
    }

    /// Store a fragment in the cache.
    pub async fn set(
        &self,
        key: &FragmentKey,
        content: String,
        policy: &SectionCachePolicy,
    ) -> CacheResult<()> {
        if !policy.enabled {
            return Ok(());
        }

        let fragment = CachedFragment::new(content, policy.ttl)
            .with_tags(policy.tags.clone());

        self.backend.set(&key.as_str(), fragment).await
    }

    /// Get or compute a fragment with stampede protection.
    ///
    /// If the cache is empty or expired, only one caller will compute the new value
    /// while others either wait or receive stale data.
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: &FragmentKey,
        policy: &SectionCachePolicy,
        compute: F,
    ) -> Result<(String, CacheStatus), CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, CacheError>>,
    {
        if !policy.enabled {
            let content = compute().await?;
            return Ok((content, CacheStatus::Bypass));
        }

        let key_str = key.as_str();
        let lock_key = format!("lock:{}", key_str);

        // Try to get from cache first
        let cache_result = self.get(key, policy).await;

        match cache_result.status {
            CacheStatus::Hit => {
                // Fresh hit, return immediately
                Ok((cache_result.fragment.unwrap().content, CacheStatus::Hit))
            }
            CacheStatus::Stale => {
                // Stale hit - try to acquire lock for revalidation
                let got_lock = self.backend.try_lock(&lock_key, self.lock_ttl).await.unwrap_or(false);

                if got_lock {
                    // We got the lock, revalidate
                    match compute().await {
                        Ok(content) => {
                            let _ = self.set(key, content.clone(), policy).await;
                            let _ = self.backend.unlock(&lock_key).await;
                            Ok((content, CacheStatus::Miss))
                        }
                        Err(_) => {
                            // Computation failed, return stale
                            let _ = self.backend.unlock(&lock_key).await;
                            Ok((cache_result.fragment.unwrap().content, CacheStatus::Stale))
                        }
                    }
                } else {
                    // Someone else is revalidating, return stale
                    Ok((cache_result.fragment.unwrap().content, CacheStatus::Stale))
                }
            }
            CacheStatus::Miss | CacheStatus::Error => {
                // Cache miss - try to acquire lock
                let got_lock = self.backend.try_lock(&lock_key, self.lock_ttl).await.unwrap_or(false);

                if got_lock {
                    // We got the lock, compute
                    match compute().await {
                        Ok(content) => {
                            let _ = self.set(key, content.clone(), policy).await;
                            let _ = self.backend.unlock(&lock_key).await;
                            Ok((content, CacheStatus::Miss))
                        }
                        Err(e) => {
                            let _ = self.backend.unlock(&lock_key).await;
                            Err(e)
                        }
                    }
                } else {
                    // Someone else is computing, we have to compute too (no stale data)
                    let content = compute().await?;
                    Ok((content, CacheStatus::Miss))
                }
            }
            CacheStatus::Bypass => {
                // Caching disabled
                let content = compute().await?;
                Ok((content, CacheStatus::Bypass))
            }
        }
    }

    /// Invalidate all entries with a given tag.
    pub async fn invalidate_tag(&self, tag: &str) -> CacheResult<u64> {
        self.backend.invalidate_tag(tag).await
    }

    /// Delete a specific entry.
    pub async fn delete(&self, key: &FragmentKey) -> CacheResult<()> {
        self.backend.delete(&key.as_str()).await
    }
}

/// In-memory fragment cache backend (for development/testing).
#[derive(Default)]
pub struct InMemoryBackend {
    // In a real implementation, this would use proper concurrent data structures.
    // For WASM single-threaded environment, simple structures work.
}

impl InMemoryBackend {
    /// Create a new in-memory backend.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl FragmentCacheBackend for InMemoryBackend {
    async fn get(&self, _key: &str) -> CacheResult<Option<CachedFragment>> {
        // Stub implementation - in real code would use a HashMap
        Ok(None)
    }

    async fn set(&self, _key: &str, _fragment: CachedFragment) -> CacheResult<()> {
        // Stub implementation
        Ok(())
    }

    async fn delete(&self, _key: &str) -> CacheResult<()> {
        Ok(())
    }

    async fn invalidate_tag(&self, _tag: &str) -> CacheResult<u64> {
        Ok(0)
    }

    async fn try_lock(&self, _key: &str, _ttl: Duration) -> CacheResult<bool> {
        // In single-threaded WASM, always succeed
        Ok(true)
    }

    async fn unlock(&self, _key: &str) -> CacheResult<()> {
        Ok(())
    }
}

// Helper to get current timestamp (seconds since epoch)
fn current_timestamp() -> u64 {
    // In WASM environment, we'd use a platform-specific time source
    // For now, use a simple approximation
    0
}
