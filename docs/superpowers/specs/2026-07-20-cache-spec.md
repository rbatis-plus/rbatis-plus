# RBatis-Plus 二级缓存协议规范

- **日期**：2026-07-20
- **状态**：目标设计草案（部分已实施）
- **上游基线**：RBatis `master` @ `4050edd3dad03a113b8bb4f5818a006f11f2da78`
- **对标**：mybatis Cache SPI + mybatis-plus-enhance 缓存增强

---

## 1. 范围与非目标

本文定义 RBatis-Plus 查询结果二级缓存的目标协议，包括稳定缓存键、`rbs::Value` 负载、内存与 Redis 后端、失效标签、并发回填和版本升级规则。

**不保证：**
- 自动理解任意 SQL 的完整读写依赖
- 感知绕过 RBatis 的数据库写入
- 在没有事务生命周期钩子时提供提交边界一致性
- 通过缓存替代数据库锁、隔离级别或业务幂等
- 缓存最终泛型结果 `T`

---

## 2. 缓存键协议

### 2.1 键结构

```text
CacheKey = version:namespace:datasource_id:driver:key_prefix:hash(sql, args, ctx)
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `version` | u8 | 协议版本号，每次 wire-incompatible 变更递增 |
| `namespace` | String | 语义命名空间（如 "user.profile"） |
| `datasource_id` | String | 数据源标识 |
| `driver` | &str | 驱动名称 |
| `key_prefix` | Option<String> | 可选前缀 |
| `hash` | [u8; 32] | blake3(sql + args + ctx) |

### 2.2 哈希算法

- 默认：blake3（32 字节输出）
- 快速路径：xxhash（feature flag `xxhash`）
- 输入：SQL 文本 + 序列化参数 + 上下文字段

### 2.3 键注入性保证

不同 SQL、不同参数、不同上下文必须产生不同键。同 SQL + 同参数 + 同上下文必须产生相同键。

---

## 3. 缓存负载协议

### 3.1 Envelope 格式

```rust
pub struct CacheEnvelope {
    pub version: u8,
    pub codec: CodecKind,
    pub payload: Vec<u8>,
    pub created_at: u64,
    pub ttl: u64,
}

pub enum CodecKind {
    RbsJson,      // 默认，使用 rbs::to_vec
    RbsMsgPack,   // 可选，feature flag "msgpack"
}
```

### 3.2 负载编码

- 默认：`rbs::to_vec(&value)` → `rbs::from_slice(&bytes)`
- 可选：msgpack 编码（feature flag）

---

## 4. 缓存策略

```rust
pub struct CachePolicy {
    pub namespace: String,
    pub ttl: Duration,                     // 必需，默认 60s
    pub null_ttl: Option<Duration>,        // 可选，默认 TTL/4
    pub refresh_ahead: Option<Duration>,   // 软 TTL 刷新窗口
    pub cache_null: bool,                  // 默认 true
    pub max_value_size: Option<usize>,     // 默认 1 MiB
    pub transaction_mode: TransactionCacheMode,
    pub failure_mode: CacheFailureMode,
    pub tags: Vec<CacheTag>,
    pub key_prefix: Option<String>,
}

pub enum TransactionCacheMode {
    Bypass,     // 默认，最安全
    Defer,      // 收集失效标签，提交时应用
}

pub enum CacheFailureMode {
    FailOpen,   // 记录日志并继续，默认
    FailClosed, // 传播错误，中断查询
}
```

---

## 5. 失效标签

### 5.1 标签结构

```rust
pub struct CacheTag {
    pub namespace: String,
    pub name: String,
}
```

### 5.2 版本化失效

- 每个标签维护一个版本号：`version:tag:<ns>:<tag> -> u64`
- 失效时原子递增版本号（`INCR`）
- 缓存键构建时包含标签版本号
- 读取时比较版本号，不匹配则视为失效

### 5.3 失效操作

| 操作 | 行为 |
|---|---|
| `invalidate_tags(tags)` | 递增指定标签版本号 |
| `clear_namespace(ns)` | 递增命名空间下所有标签版本号 |
| DML 成功 | 自动失效关联标签 |
| DML 失败 | 不失效 |

---

## 6. 并发回填

### 6.1 SingleFlight

- 每个缓存键一个 `tokio::sync::Mutex`
- 并发 miss 只产生一次 DB 查询
- 其他等待者共享结果

### 6.2 TTL Jitter

- 有效 TTL = configured_ttl +/- rand(0, jitter_max)
- 默认 jitter_max = TTL 的 10%
- 防止缓存雪崩

---

## 7. 内存后端（MemoryCacheStore）

```rust
pub struct MemoryCacheStore {
    values: moka::future::Cache<CacheKey, Arc<Value>>,
    tag_versions: dashmap::DashMap<CacheTag, AtomicU64>,
    clock: Clock,
    metrics: Arc<dyn MetricsRecorder>,
}

impl MemoryCacheStore {
    pub fn builder() -> MemoryCacheStoreBuilder;
}

#[async_trait]
impl CacheStore for MemoryCacheStore {
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;
    async fn set(&self, key: CacheKey, value: Value, policy: &CachePolicy) -> Result<(), CacheError>;
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;
    async fn clear_namespace(&self, ns: &str) -> Result<u64, CacheError>;
}
```

---

## 8. Redis 后端（RedisCacheStore）

```rust
pub struct RedisCacheStore {
    client: redis::Client,
    publisher: redis::aio::PubSub,
    config: RedisCacheConfig,
    metrics: Arc<dyn MetricsRecorder>,
    singleflight: SingleFlight,
}
```

### 8.1 Wire format

```text
+--------+--------+---------+----------+
| version| codec  | payload | created  |
| u8     | u8     | bytes   | u64      |
+--------+--------+---------+----------+
```

### 8.2 Pub/Sub

- 频道：`<prefix>.bus`
- 消息：`InvalidateTags { tags, nonce }`
- 跨进程失效延迟目标：< 1s

---

## 9. 版本升级规则

| 变更类型 | 操作 |
|---|---|
| Envelope 字段变更 | 递增 `version` |
| Codec 变更 | 新增 `CodecKind` 变体 |
| 哈希算法变更 | 递增 `version` + 清空缓存 |
| 标签格式变更 | 递增 `version` + 清空缓存 |

---

## 10. 最大缓存值

- 默认：1 MiB
- 超过限制：静默绕过（FailOpen）
- 可配置：`max_value_size` in `CachePolicy`

---

## 11. 敏感参数处理

- 缓存键中不包含原始参数值
- 使用哈希值替代
- 可配置 `key_redact = ["password"]` 属性（planned）
- 日志中不记录原始参数
