use std::time::Duration;

/// 缓存策略配置（对标 MyBatis-Plus `CachePolicy`）。
#[derive(Debug, Clone)]
pub struct CachePolicy {
    /// 缓存 TTL（默认 5 分钟）。
    pub ttl: Duration,
    /// 是否缓存 null 值（默认 true）。
    pub cache_null: bool,
    /// null 值的 TTL（默认 60 秒）。
    pub null_ttl: Duration,
    /// 单个值最大字节数（超过则跳过缓存，默认 512 KB）。
    pub max_value_size: usize,
    /// 事务模式：Bypass（立即写入）/ Defer（提交时写入）。
    pub transaction_mode: TransactionMode,
    /// 失败模式：Fail（抛异常）/ PassThrough（降级穿透）。
    pub failure_mode: FailureMode,
    /// 是否使用单飞（SingleFlight）防缓存击穿（默认 true）。
    pub use_singleflight: bool,
}

/// 事务缓存模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionMode {
    /// 立即写入 L2（默认）。
    Bypass,
    /// 延迟到事务提交时写入（对标 MyBatis `TransactionalCache`）。
    Defer,
}

/// 缓存失败处理模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    /// 抛出异常。
    Fail,
    /// 降级穿透，直接查询数据库。
    PassThrough,
}

impl Default for CachePolicy {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(300),
            cache_null: true,
            null_ttl: Duration::from_secs(60),
            max_value_size: 512 * 1024,
            transaction_mode: TransactionMode::Bypass,
            failure_mode: FailureMode::PassThrough,
            use_singleflight: true,
        }
    }
}
