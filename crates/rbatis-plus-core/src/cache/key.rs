use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 缓存键（对标 MyBatis CacheKey）。
///
/// 由 namespace + sql + args 组合生成唯一 digest，用于 L2 缓存查找。
#[derive(Debug, Clone)]
pub struct CacheKey {
    /// 命名空间（通常为 Mapper 全限定名）。
    pub namespace: String,
    /// SQL 语句。
    pub sql: String,
    /// 绑定参数列表。
    pub args: Vec<rbs::Value>,
    /// FNV-1a 哈希摘要（由 `new()` 自动计算）。
    pub digest: u64,
}

impl CacheKey {
    /// 构造缓存键并计算摘要。
    pub fn new(namespace: impl Into<String>, sql: impl Into<String>, args: Vec<rbs::Value>) -> Self {
        let namespace = namespace.into();
        let sql = sql.into();
        let mut hasher = DefaultHasher::new();
        namespace.hash(&mut hasher);
        sql.hash(&mut hasher);
        // 对 args 做字符串化后哈希（rbs::Value 未实现 Hash）
        for arg in &args {
            format!("{:?}", arg).hash(&mut hasher);
        }
        let digest = hasher.finish();
        Self {
            namespace,
            sql,
            args,
            digest,
        }
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

impl Eq for CacheKey {}

impl std::hash::Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digest.hash(state);
    }
}
