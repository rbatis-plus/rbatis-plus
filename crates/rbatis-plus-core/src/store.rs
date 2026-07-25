use crate::{CacheError, CacheKey, CacheTag};
use async_trait::async_trait;
use rbs::Value;
use std::fmt::Debug;
use std::time::Duration;

/// SPI for a cache store backend.
///
/// Implementations include in-process [`MemoryCacheStore`] and a planned
/// Redis backend.  All methods are async and object-safe.
#[async_trait]
pub trait CacheStore: Send + Sync + Debug {
    /// Look up a cached value by key.
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;

    /// Store a value with a given TTL and associated tags.
    async fn set(
        &self,
        key: CacheKey,
        value: Value,
        ttl: Duration,
        tags: &[CacheTag],
    ) -> Result<(), CacheError>;

    /// Remove a single entry.
    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;

    /// Invalidate all entries associated with the given tags.
    /// Returns the number of entries invalidated (best-effort).
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;

    /// Clear all entries in a namespace.
    async fn clear_namespace(&self, namespace: &str) -> Result<u64, CacheError>;

    /// Total number of entries across all namespaces.
    async fn len(&self) -> Result<usize, CacheError>;

    /// Approximate hit ratio (0.0 ..= 1.0).  Returns 0.0 if no requests yet.
    async fn hit_ratio(&self) -> f64;
}
