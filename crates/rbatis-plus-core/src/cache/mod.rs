// 复用上游 rbatis 主仓的 Caffeine 化缓存实现（df87ac41），
// 不在本地重复发明，保证与上游行为一致。

pub use rbatis::plugin::cache::{
    CacheError, CacheIntercept, CacheKey, CachePolicy,
    CacheStore, CacheTag, MemoryCacheStore, SharedCacheStore,
    CacheTransactionListener, TransactionCacheMode, CacheFailureMode,
    L1Cache, SharedL1Cache, SingleFlight, TransactionalCacheBuffer,
    UseCacheFilter,
};
