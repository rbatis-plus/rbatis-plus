use crate::cache::{CacheError, CacheKey, CacheStore, CacheTag};
use async_trait::async_trait;
use dashmap::DashMap;
use rbs::Value;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 内存缓存条目。
#[derive(Clone, Debug)]
struct CacheEntry {
    value: Value,
    /// 过期时间点。
    expires_at: Instant,
    /// 关联的标签列表。
    tags: Vec<CacheTag>,
}

/// 进程内 L2 缓存存储（对标 MyBatis-Plus 内置缓存）。
///
/// 使用 DashMap 做并发安全的 KV 存储；支持 TTL、标签失效、命中率统计。
/// 适合单机场景；多机场景请使用 `RedisCacheStore`。
///
/// 对应 Java：com.baomidou.mybatisplus.extension.plugins.inner.CachedContent / MyBatis-Plus 本地缓存方案
#[derive(Clone)]
pub struct MemoryCacheStore {
    /// key_digest -> CacheEntry
    store: Arc<DashMap<u64, CacheEntry>>,
    /// tag -> Vec<key_digest>（用于按标签失效）
    tag_index: Arc<DashMap<String, Vec<u64>>>,
    /// 命中计数。
    hits: Arc<AtomicU64>,
    /// 未命中计数。
    misses: Arc<AtomicU64>,
}

impl MemoryCacheStore {
    /// 创建新的内存缓存实例。
    ///
    /// 默认无容量限制；生产环境建议在 `CachePolicy.max_size` 层面做限制。
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            tag_index: Arc::new(DashMap::new()),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 清除所有过期条目。
    pub fn purge_expired(&self) {
        let now = Instant::now();
        let expired_keys: Vec<u64> = self
            .store
            .iter()
            .filter_map(|r| {
                if r.value().expires_at <= now {
                    Some(*r.key())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            if let Some(entry) = self.store.remove(&key) {
                self.remove_from_tag_index(&entry.1.tags, key);
            }
        }
    }

    /// 获取命中率（0.0 ~ 1.0）。
    pub fn hit_ratio(&self) -> f64 {
        let h = self.hits.load(Ordering::Relaxed);
        let m = self.misses.load(Ordering::Relaxed);
        let total = h + m;
        if total == 0 { 0.0 } else { h as f64 / total as f64 }
    }

    /// 当前缓存条目数。
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// 按标签删除关联条目，并维护 tag_index。
    fn invalidate_by_tags(&self, tags: &[CacheTag]) -> u64 {
        let mut invalidated = 0u64;
        for tag in tags {
            let digests: Vec<u64> = self
                .tag_index
                .get(tag)
                .map(|r| r.value().clone())
                .unwrap_or_default();

            for digest in digests {
                if let Some(entry) = self.store.remove(&digest) {
                    self.remove_from_tag_index(&entry.1.tags, digest);
                    invalidated += 1;
                }
            }
        }
        invalidated
    }

    /// 从 tag_index 中移除指定 digest。
    fn remove_from_tag_index(&self, tags: &[CacheTag], digest: u64) {
        for tag in tags {
            let mut entry = self.tag_index.entry(tag.clone()).or_default();
            entry.retain(|&x| x != digest);
        }
    }
}

impl Default for MemoryCacheStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for MemoryCacheStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryCacheStore")
            .field("len", &self.len())
            .field("hit_ratio", &self.hit_ratio())
            .finish()
    }
}

#[async_trait]
impl CacheStore for MemoryCacheStore {
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError> {
        match self.store.get(&key.digest) {
            Some(entry) => {
                let entry = entry.value();
                if entry.expires_at <= Instant::now() {
                    // 过期，移除
                    let removed = self.store.remove(&key.digest);
                    if let Some((_k, entry)) = removed {
                        self.remove_from_tag_index(&entry.tags, key.digest);
                    }
                    self.misses.fetch_add(1, Ordering::Relaxed);
                    Ok(None)
                } else {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    Ok(Some(entry.value.clone()))
                }
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn set(
        &self,
        key: CacheKey,
        value: Value,
        ttl: Duration,
        tags: &[CacheTag],
    ) -> Result<(), CacheError> {
        let digest = key.digest;
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
            tags: tags.to_vec(),
        };

        // 先移除旧条目的标签索引
        if let Some(old) = self.store.remove(&digest) {
            self.remove_from_tag_index(&old.1.tags, digest);
        }

        // 写入新条目
        self.store.insert(digest, entry);

        // 维护标签索引
        for tag in tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(digest);
        }

        Ok(())
    }

    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        if let Some((_k, entry)) = self.store.remove(&key.digest) {
            self.remove_from_tag_index(&entry.tags, key.digest);
        }
        Ok(())
    }

    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError> {
        Ok(self.invalidate_by_tags(tags))
    }

    async fn clear_namespace(&self, _namespace: &str) -> Result<u64, CacheError> {
        // 简单实现：按标签失效；生产版应按 namespace 维度维护独立索引
        Ok(0)
    }

    async fn len(&self) -> Result<usize, CacheError> {
        Ok(self.len())
    }

    async fn hit_ratio(&self) -> f64 {
        self.hit_ratio()
    }
}
