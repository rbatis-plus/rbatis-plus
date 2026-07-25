use std::time::Duration;

/// How the cache interacts with transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransactionCacheMode {
    /// Transaction queries bypass the cache entirely (default).
    /// Transaction DML invalidates immediately on success.
    #[default]
    Bypass,
}

/// What happens when the cache backend fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheFailureMode {
    /// Continue without cache (return miss).  Default.
    #[default]
    FailOpen,
    /// Return an error to the caller.
    FailClosed,
}

/// Per-namespace cache policy.
#[derive(Debug, Clone)]
pub struct CachePolicy {
    /// Namespace / logical group.
    pub namespace: String,
    /// Time-to-live for cached entries.
    pub ttl: Duration,
    /// Whether to cache empty results (null / empty array).
    pub cache_null: bool,
    /// TTL for empty results (typically shorter than `ttl`).
    pub null_ttl: Option<Duration>,
    /// Maximum payload size in bytes (0 = unlimited).
    pub max_value_size: usize,
    /// Transaction behaviour.
    pub transaction_mode: TransactionCacheMode,
    /// Failure behaviour.
    pub failure_mode: CacheFailureMode,
}

impl CachePolicy {
    /// Create a policy with sensible defaults for the given namespace.
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            ttl: Duration::from_secs(60),
            cache_null: true,
            null_ttl: Some(Duration::from_secs(10)),
            max_value_size: 1_000_000, // 1 MB
            transaction_mode: TransactionCacheMode::Bypass,
            failure_mode: CacheFailureMode::FailOpen,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_cache_null(mut self, cache_null: bool) -> Self {
        self.cache_null = cache_null;
        self
    }
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self::new("default")
    }
}
