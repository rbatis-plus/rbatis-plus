use crate::cache::{CachePolicy, CacheStore, CacheTag};
use std::sync::Arc;

/// 缓存拦截器（对标 MyBatis `CachingExecutor`）。
///
/// 在 Executor 的 query/exec 前后插入缓存逻辑：
/// - query 前：L1 → L2 → SingleFlight 查找，命中则跳过 DB
/// - query 后：写入 L2 + L1
/// - exec 后（INSERT/UPDATE/DELETE）：按表标签失效关联缓存
pub struct CacheIntercept {
    /// L2 缓存存储实现。
    store: Arc<dyn CacheStore>,
    /// 缓存策略。
    policy: CachePolicy,
}

impl CacheIntercept {
    /// 创建缓存拦截器。
    pub fn new(store: Arc<dyn CacheStore>, policy: CachePolicy) -> Self {
        Self { store, policy }
    }

    /// 获取缓存存储引用。
    pub fn store(&self) -> &dyn CacheStore {
        self.store.as_ref()
    }

    /// 获取缓存策略引用。
    pub fn policy(&self) -> &CachePolicy {
        &self.policy
    }

    /// 根据表名生成缓存标签（用于按表失效）。
    pub fn table_tag(table: &str) -> CacheTag {
        format!("table:{}", table)
    }
}
