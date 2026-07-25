use std::fmt;

/// 缓存子系统错误类型。
#[derive(Debug)]
pub enum CacheError {
    /// 序列化 / 反序列化失败。
    Serialization(String),
    /// 底层存储连接失败。
    Connection(String),
    /// 缓存键构造失败（例如 SQL 为空）。
    InvalidKey(String),
    /// 内部错误。
    Internal(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Serialization(msg) => write!(f, "Cache serialization error: {}", msg),
            CacheError::Connection(msg) => write!(f, "Cache connection error: {}", msg),
            CacheError::InvalidKey(msg) => write!(f, "Cache invalid key: {}", msg),
            CacheError::Internal(msg) => write!(f, "Cache internal error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}
