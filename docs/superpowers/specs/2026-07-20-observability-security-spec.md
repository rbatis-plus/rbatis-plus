# RBatis-Plus 可观测性、安全与运维规范

- **日期**：2026-07-20
- **状态**：目标设计草案
- **上游基线**：RBatis `master` @ `4050edd3dad03a113b8bb4f5818a006f11f2da78`

---

## 1. 可观测性契约

可观测性必须说明：请求是否使用了缓存、为什么绕过、失效是否完成、数据库是否仍是权威来源。Instrumentation 不得记录查询参数或敏感值。

### 1.1 指标

推荐低基数指标：

| 指标名 | 类型 | 标签 | 说明 |
|---|---|---|---|
| `rbatis_cache_requests_total` | Counter | `operation`, `outcome` | 请求总数 |
| `rbatis_cache_latency_seconds` | Histogram | `operation`, `outcome` | 延迟分布 |
| `rbatis_cache_db_queries_total` | Counter | `operation` | 数据库查询总数 |
| `rbatis_cache_invalidation_total` | Counter | `scope`, `outcome` | 失效总数 |
| `rbatis_cache_invalidation_lag_seconds` | Histogram | — | 失效延迟 |
| `rbatis_cache_backend_errors_total` | Counter | `operation`, `error_class` | 后端错误总数 |

其中：
- `operation` = `get` | `set` | `invalidate`
- `outcome` = `hit` | `miss` | `bypass` | `error` | `success`
- `scope` = `tag` | `namespace` | `generation`
- `error_class` = 有界枚举

### 1.2 MetricsRecorder trait

```rust
#[async_trait]
pub trait MetricsRecorder: Send + Sync {
    fn record_request(&self, operation: &str, outcome: &str);
    fn record_latency(&self, operation: &str, outcome: &str, duration: Duration);
    fn record_db_query(&self, operation: &str);
    fn record_invalidation(&self, scope: &str, outcome: &str);
    fn record_backend_error(&self, operation: &str, error_class: &str);
}

pub struct NoopMetricsRecorder;

impl MetricsRecorder for NoopMetricsRecorder {
    fn record_request(&self, _operation: &str, _outcome: &str) {}
    fn record_latency(&self, _operation: &str, _outcome: &str, _duration: Duration) {}
    fn record_db_query(&self, _operation: &str) {}
    fn record_invalidation(&self, _scope: &str, _outcome: &str) {}
    fn record_backend_error(&self, _operation: &str, _error_class: &str) {}
}
```

---

## 2. 日志

### 2.1 日志级别

| 事件 | 级别 | 说明 |
|---|---|---|
| 缓存命中 | DEBUG | 正常路径 |
| 缓存未命中 | DEBUG | 正常路径 |
| 缓存绕过 | INFO | 有意绕过 |
| 后端错误 | WARN | FailOpen 模式 |
| 失效完成 | INFO | 标签失效 |
| 配置错误 | ERROR | 启动失败 |

### 2.2 日志格式

```text
[LEVEL] module: message {key=value, ...}
```

**禁止记录：**
- 查询参数原始值
- 缓存键中的敏感字段
- 数据库连接字符串中的密码

---

## 3. 追踪（Tracing）

### 3.1 集成方式

可选 `tracing` 集成（feature flag `tracing`）：

```rust
#[cfg(feature = "tracing")]
use tracing::{info_span, Instrument};

#[cfg(feature = "tracing")]
let span = info_span!("cache_get", key = %cache_key);
result = store.get(&key).instrument(span).await;
```

### 3.2 Span 命名

| 操作 | Span 名 |
|---|---|
| 缓存读取 | `cache_get` |
| 缓存写入 | `cache_set` |
| 缓存失效 | `cache_invalidate` |
| 标签失效 | `cache_invalidate_tags` |

---

## 4. 安全

### 4.1 敏感数据处理

| 数据 | 处理方式 |
|---|---|
| 查询参数 | 哈希后存入缓存键，不记录原始值 |
| 数据库密码 | 不记录到日志 |
| 缓存值 | 与数据库一致的访问控制 |
| Redis 连接 | TLS 支持（`rediss://`） |

### 4.2 访问控制

- 缓存不绕过数据库的访问控制
- 缓存值的访问权限与数据库一致
- 不同租户的缓存键必须隔离

### 4.3 Redis 安全

```rust
// TLS 连接
let client = redis::Client::open("rediss://user:password@host:6380")?;

// ACL 支持
let client = redis::Client::open("redis://user:password@host:6379")?;
```

### 4.4 认证令牌

- 认证令牌不记录到日志
- Redis AUTH 令牌不记录到日志
- 缓存键中不包含认证信息

---

## 5. 运维

### 5.1 管理 API

```rust
pub struct RBatisPlusAdmin {
    store: Arc<dyn CacheStore>,
}

impl RBatisPlusAdmin {
    pub async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64>;
    pub async fn clear_namespace(&self, ns: &str) -> Result<u64>;
    pub async fn dump_keys_for_diagnostics(&self) -> Result<Vec<String>>;
}
```

### 5.2 缓存清理

```bash
# 清理指定命名空间
RBatisPlusAdmin::clear_namespace("user.profile").await?;

# 清理指定标签
RBatisPlusAdmin::invalidate_tags(&[CacheTag::new("user", "profile:by_id")]).await?;
```

### 5.3 缓存预热

```rust
// 预热常用查询
for id in hot_ids {
    let _ = select_user(&rb, id).await?;
}
```

---

## 6. 故障排查

### 6.1 缓存不生效

1. 检查拦截器注册顺序
2. 检查 feature flag `cache` 是否启用
3. 检查 `CachePolicy` 配置
4. 检查日志中是否有 `cache_bypass` 记录

### 6.2 缓存命中但数据过期

1. 检查 TTL 配置
2. 检查标签失效是否执行
3. 检查 Redis Pub/Sub 是否正常

### 6.3 后端错误

1. 检查 Redis 连接
2. 检查内存使用
3. 检查 `error_class` 指标

---

## 7. 监控告警建议

| 指标 | 阈值 | 说明 |
|---|---|---|
| `rbatis_cache_backend_errors_total` | > 10/min | 后端频繁错误 |
| `rbatis_cache_invalidation_lag_seconds` | > 5s | 失效延迟过高 |
| `rbatis_cache_requests_total{outcome="error"}` | > 1% | 错误率过高 |
| 内存使用 | > 80% | 内存后端容量 |

---

## 8. 性能基线

| 场景 | 目标 |
|---|---|
| 缓存命中延迟 | < 1ms (p99) |
| 缓存未命中延迟 | 与 DB 查询一致 |
| 标签失效延迟 | < 10ms (p99) |
| Redis 跨进程失效 | < 1s |
| 内存后端吞吐 | > 100k ops/sec |
