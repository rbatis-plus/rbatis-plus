//! 二级缓存类型 re-export。
//!
//! 缓存实现位于 `rbatis-cache` crate（执行器集成层 `RbatisCacheInterceptor`
//! + `CacheBackend` SPI + `LocalBackend`/Redis/Memcached 后端），
//! 本模块仅做公共 API 透传，保持 `rbatis_plus::cache::*` 的调用方式不变。

pub use rbatis_cache::{
    CacheBackend, CacheError, CacheEnvelope, CacheFailureMode, CacheInterceptor, CacheKey,
    CacheKeyInput, CacheMetrics, CacheMetricsSnapshot, CachePolicy, CacheRequest,
    CacheTransactionListener, InvalidationStrategy, L1Cache, LocalBackend, LocalBackendConfig,
    RbatisCacheExt, RbatisCacheInterceptor, SingleFlight, SqlMetadata, StatementKind,
    TransactionCacheMode, TransactionalCacheBuffer, UseCacheFilter,
};
