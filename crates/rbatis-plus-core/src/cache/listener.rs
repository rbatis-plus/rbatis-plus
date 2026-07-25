use crate::cache::CacheStore;
use std::sync::Arc;

/// 缓存事务监听器（对标 MyBatis `TransactionalCacheManager`）。
///
/// 在事务提交时 flush 延迟写入的缓存，在事务回滚时丢弃。
pub struct CacheTransactionListener {
    store: Arc<dyn CacheStore>,
}

impl CacheTransactionListener {
    /// 创建事务缓存监听器。
    pub fn new(store: Arc<dyn CacheStore>) -> Self {
        Self { store }
    }

    /// 获取缓存存储引用。
    pub fn store(&self) -> &dyn CacheStore {
        self.store.as_ref()
    }
}
