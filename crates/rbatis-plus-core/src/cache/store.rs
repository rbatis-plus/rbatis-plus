use crate::cache::{CacheError, CacheKey};
use async_trait::async_trait;
use rbs::Value;
use std::time::Duration;

/// 缓存标签类型（用于按标签批量失效）。
pub type CacheTag = String;

/// 二级缓存存储 SPI（对标 MyBatis `Cache` 接口）。
///
/// 所有方法均为 async，适配 rbatis 的异步 Executor 体系。
/// 内置实现见 `MemoryCacheStore`；用户可自行实现 `RedisCacheStore` 等。
#[async_trait]
pub trait CacheStore: Send + Sync + 'static {
    /// 根据缓存键获取值；未命中返回 `Ok(None)`。
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;

    /// 写入缓存条目，附带 TTL 和标签列表。
    async fn set(
        &self,
        key: CacheKey,
        value: Value,
        ttl: Duration,
        tags: &[CacheTag],
    ) -> Result<(), CacheError>;

    /// 删除指定缓存键。
    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;

    /// 按标签批量失效，返回失效条目数。
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;

    /// 清空指定命名空间下的所有条目，返回清除条目数。
    async fn clear_namespace(&self, namespace: &str) -> Result<u64, CacheError>;

    /// 当前缓存条目总数。
    async fn len(&self) -> Result<usize, CacheError>;

    /// 缓存命中率（0.0 ~ 1.0）。
    async fn hit_ratio(&self) -> f64;
}
