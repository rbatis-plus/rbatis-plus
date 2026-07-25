# RBatis 支持二级缓存调研报告

> **文档类型**：技术调研 / 架构输入  
> **调研日期**：2026-07-24  
> **上游基线（最新）**：RBatis `master@4050edd3dad03a113b8bb4f5818a006f11f2da78`  
> **上游基线（旧）**：RBatis `2df418feeab511c1899b2a110eef43228a1ad889`  
> **最新调研提交标题**：`Limit decode fallback to single-column values`  
> **决策记录**：[RBatis-Plus 二级缓存架构决策](DECISIONS.md)
>
> **文档索引**：[结论先行](#1-结论先行) · [CodeGraph 调研结果](#2-codegraph-调研结果) · [当前架构](#3-rbatis-当前查询执行架构) · [推荐 SPI](#7-推荐-spi) · [实施路线](#13-推荐实施路线) · [最新架构总结](#17-最新架构总结) · [架构决策](DECISIONS.md)

## 0. 文档索引

RBatis-Plus 文档全集（共 10 份，含本份与根 README）：

| # | 文档 | 作用 |
| - | --- | --- |
| 0 | [`/README.md`](../README.md) | 项目门面 + Mermaid 总图 + 文档索引 |
| 1 | `docs/RBatis 支持二级缓存调研报告.md` | 本文档；RBatis L2 缓存现状证据 |
| 2 | [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) | RBatis-Plus 分层与组件架构 |
| 3 | [`docs/IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) | 分阶段实施、PR 系列、测试矩阵 |
| 4 | [`docs/CACHE_SPECIFICATION.md`](CACHE_SPECIFICATION.md) | Key/Envelope/Codec/Tag 协议 |
| 5 | [`docs/TRANSACTION_CONSISTENCY.md`](TRANSACTION_CONSISTENCY.md) | 事务读/回填/失效语义 |
| 6 | [`docs/DECISIONS.md`](DECISIONS.md) | 架构决策记录 (ADR) |
| 7 | [`docs/INTEGRATION_GUIDE.md`](INTEGRATION_GUIDE.md) | 集成与门禁模板 |
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](OBSERVABILITY_SECURITY_OPERATIONS.md) | 可观测 / 安全 / 运维 |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](TEST_AND_ACCEPTANCE_PLAN.md) | 验收计划与发布门 |

## 1. 结论先行

RBatis 当前版本没有 ORM 查询结果二级缓存。

现有代码中的“缓存”主要是：

1. `rbdc-*` 驱动层的 prepared statement cache；
2. 历史版本中的 SQL/表达式解析缓存；
3. 这些都不是 MyBatis 意义上的跨连接、跨会话查询结果二级缓存。

RBatis 已有的 `Intercept` 扩展点能够实现一个基础版二级缓存插件：

- 查询前读取缓存；
- 命中时通过 `Action::Return` 短路数据库访问；
- 查询后写缓存；
- DML 成功后失效缓存。

但是，如果目标是提供生产级、事务一致的二级缓存，不能只新增一个 `CacheIntercept`。现有拦截器 API 缺少可靠的执行上下文和事务生命周期事件，至少需要对核心层做小幅增强。

推荐采用“上游薄、Plus 厚”的边界（完整取舍见 [ADR-001](DECISIONS.md#adr-001上游保持薄rbatis-plus-承载缓存产品能力)）：

> **通用执行钩子、执行上下文和事务生命周期事件进入上游 `rbatis`；缓存 SPI、`CacheIntercept`、策略、后端和产品能力保留在 RBatis-Plus。**

建议分成：

- 上游 `rbatis`：缓存无关的通用钩子、执行上下文和事务事件；
- RBatis-Plus 核心：缓存 SPI、`CacheIntercept`、策略和稳定协议；
- RBatis-Plus 内存缓存 crate：内存缓存实现；
- RBatis-Plus Redis 缓存 crate：Redis 分布式缓存实现，通过可选 feature 显式启用；
- 默认关闭，用户显式注册。

---

## 2. CodeGraph 调研结果

我先为仓库完成了知识图谱全量构建。

**最新基线（`master@4050edd3dad03a113b8bb4f5818a006f11f2da78`）：**

- 178 个源码文件；
- 1,740 个节点；
- 17,805 条关系边；
- 192 个类；
- 715 个函数；
- 655 个测试节点；
- 9,366 条 `CALLS` 边；
- 5,524 条 `TESTED_BY` 边；
- 88 条执行流；
- 14 个代码社区。

**旧基线（`2df418feeab511c1899b2a110eef43228a1ad889`，仅供对比）：**

- 181 个源码文件；
- 1,818 个节点；
- 18,245 条关系边；
- 88 条执行流；
- 14 个代码社区。

主要架构社区包括：

| 社区 | 作用 |
|---|---|
| `src-op` | CRUD 和操作相关代码 |
| `src-activity` | RBatis 运行时/业务执行路径 |
| `src-decode` | 查询结果反序列化 |
| `intercept-before` | SQL 拦截器处理 |
| `plugin-page` | 分页插件 |
| `syntax-tree-html-node` | HTML SQL AST |
| `macros-impl` | 过程宏生成 |
| `tests-mock` | Mock 和集成测试 |

影响分析表明，如果直接修改：

- `src/plugin/intercept/mod.rs`
- `src/executor.rs`
- `src/rbatis.rs`

在 3 跳范围内会影响至少：

- 500 个图节点，完整结果实际超过 1,300；
- 118 个额外文件；
- CRUD、分页、事务、宏查询、执行器测试等大量路径。

因此二级缓存最好通过**兼容性扩展**接入，避免重写 `Executor` 或改变所有 CRUD 宏的生成结果。

---

## 3. RBatis 当前查询执行架构

### 3.1 核心对象

`RBatis` 持有三个关键字段：

```rust
pub struct RBatis {
    pub pool: Arc<OnceLock<Box<dyn Pool>>>,
    pub intercepts: Arc<SyncVec<Arc<dyn Intercept>>>,
    pub task_id_generator: Arc<dyn IdGenerator>,
}
```

位置：`src/rbatis.rs:22`

含义：

- `pool`：数据库连接池；
- `intercepts`：进程级共享的拦截器链；
- `task_id_generator`：连接/事务任务 ID 生成器。

`RBatis` 是 `Clone` 的，连接执行器会保留一个 `RBatis` 克隆及共享的拦截器链：

```rust
pub struct RBatisConnExecutor {
    pub id: i64,
    pub rb: RBatis,
    pub conn: Arc<Mutex<Box<dyn Connection>>>,
    pub intercepts: Arc<SyncVec<Arc<dyn Intercept>>>,
}
```

位置：`src/executor.rs:44`

这意味着内存缓存如果放在拦截器内部并由 `Arc` 共享，天然就是：

- 同一 `RBatis` 实例内共享；
- 跨连接共享；
- 跨事务执行器共享。

从作用域上看，这已经符合“二级缓存”的基本条件。

---

### 3.2 Executor 抽象

统一执行接口是：

```rust
pub trait Executor: RBatisRef + Send + Sync {
    fn id(&self) -> i64;
    fn name(&self) -> &str;
    fn exec(...) -> BoxFuture<'_, Result<ExecResult, Error>>;
    fn query(...) -> BoxFuture<'_, Result<Value, Error>>;
}
```

位置：`src/executor.rs:18`

主要实现：

- `RBatis`
- `RBatisConnExecutor`
- `RBatisTxExecutor`
- `RBatisTxExecutorGuard`

宏生成的 CRUD、`py_sql!`、`html_sql!` 最终都接受 `&dyn Executor`，因此缓存只要覆盖 `Executor::query/exec` 的公共路径，就不需要逐个修改 CRUD 和 SQL 宏。

---

### 3.3 查询路径

CodeGraph 得到的查询解码流为：

```text
RBatis::query_decode
  -> RBatis::exec_decode
      -> intercept::apply_before
      -> Connection::exec_decode
      -> intercept::apply_after
      -> decode
          -> decode_ref
              -> try_decode_single_column
```

相关实现：`src/executor.rs:561`、`src/executor.rs:570`

普通查询的连接执行路径是：

```text
RBatis::query
  -> RBatis::acquire
      -> RBatisConnExecutor::query
          -> apply_before
          -> Connection::exec_decode
          -> apply_after
```

`RBatisConnExecutor::query` 位于 `src/executor.rs:172`。

事务查询也使用相同拦截器：

```text
RBatisTxExecutor::query
  -> apply_before
  -> transaction connection exec_decode
  -> apply_after
```

位置：`src/executor.rs:417`

这说明一个拦截器可以统一观察：

- 普通查询；
- 连接级查询；
- 事务查询；
- CRUD 查询；
- 宏生成查询。

---

## 4. 当前拦截器能做什么

`Intercept` 的核心接口是：

```rust
async fn before(
    &self,
    task_id: i64,
    rb: &dyn Executor,
    sql: &mut String,
    args: &mut Vec<Value>,
    result: ResultType<
        &mut Result<ExecResult, Error>,
        &mut Result<Value, Error>,
    >,
) -> Result<Action, Error>;

async fn after(...) -> Result<Action, Error>;
```

位置：`src/plugin/intercept/mod.rs:68`

`Action` 有两个值：

```rust
pub enum Action {
    Next,
    Return,
}
```

位置：`src/plugin/intercept/mod.rs:32`

在 `before` 中：

- `Action::Next`：继续访问数据库；
- `Action::Return`：直接返回拦截器写入的结果。

已有测试证明拦截器可以构造结果并跳过数据库访问：

```rust
*v = Ok(ExecResult { ... });
Ok(Action::Return)
```

位置：`tests/intercept_test.rs:160`

因此查询缓存命中可以写成：

```rust
if let Some(value) = cache.get(&key).await? {
    *query_result = Ok(value);
    return Ok(Action::Return);
}
```

未命中后，`after` 可以从 `ResultType::Query` 中取得原始 `rbs::Value` 并写入缓存。

这是当前实现二级缓存最自然的接入点。

---

## 5. 当前拦截器模型的几个关键问题

### 5.1 `after` 短路返回疑似有错误

例如 `RBatisConnExecutor::query`：

```rust
if intercept::apply_after(...).await? {
    return before_result;
}
result
```

位置：`src/executor.rs:192`

`RBatis::exec_decode` 也有类似逻辑：

```rust
if intercept::apply_after(...).await? {
    return before_result.and_then(|v| decode(v));
}
```

位置：`src/executor.rs:606`

`after` 接收到的是 `result`，但当它返回 `Action::Return` 时，执行器返回的却是 `before_result`。这会使“后置拦截器修改查询结果并短路”表现异常。

缓存写入不需要在 `after` 返回 `Action::Return`，所以 MVP 可规避该问题；但如果要正式扩展拦截器能力，建议先修复为返回 `result`。

---

### 5.2 拦截器顺序会影响缓存 Key

默认拦截器顺序：

```rust
PageIntercept
LogInterceptor
```

位置：`src/rbatis.rs:44`

分页拦截器会在 `before` 中修改 SQL，例如附加 `LIMIT/OFFSET`：

`src/plugin/intercept/intercept_page.rs:65`

因此缓存拦截器应放在：

```text
SQL 重写类拦截器之后
缓存拦截器
日志/指标拦截器
```

例如：

```text
Tenant/DynamicTable/Page
  -> Cache
  -> Log
```

否则缓存 Key 可能基于未完成重写的 SQL，导致：

- 不同分页页码共用缓存；
- 多租户污染；
- 动态表名冲突；
- 读写路由上下文未包含在 Key 中。

---

### 5.3 `task_id` 不能可靠判断事务

注释说明 `task_id` “可能是 conn_id 或 tx_id”：

`src/plugin/intercept/mod.rs:73`

缓存拦截器不能仅凭：

```rust
task_id != 0
```

判断当前查询是否在事务内。

`Executor::name()` 虽能暴露实现类型名，但基于字符串区分 `RBatisTxExecutor` 很脆弱，不适合作为正式 API。

推荐增加显式上下文：

```rust
pub enum ExecutorKind {
    Root,
    Connection,
    Transaction,
    TransactionGuard,
}
```

或者：

```rust
pub trait Executor {
    fn context(&self) -> ExecutorContext;
}
```

---

### 5.4 缺少事务提交/回滚事件

事务当前的提交和回滚直接访问连接：

```rust
self.conn_executor.conn.lock().await.commit().await?;
```

位置：`src/executor.rs:308`

```rust
self.conn_executor.conn.lock().await.rollback().await?;
```

位置：`src/executor.rs:300`

它们不会经过 SQL `exec` 拦截器。因此缓存插件无法获知：

- 事务最终提交；
- 事务最终回滚；
- 何时安全地让修改操作失效缓存。

生产级二级缓存必须增加类似事件：

```rust
after_commit(tx_id)
after_rollback(tx_id)
```

或统一生命周期钩子：

```rust
enum ExecutorEvent {
    TransactionBegin,
    TransactionCommit,
    TransactionRollback,
}
```

---

### 5.5 仅凭 SQL 很难精确识别依赖表

查询 Key 比较容易生成，但 DML 后应该失效哪些缓存项，需要知道查询依赖的表。

直接解析 SQL 有以下难点：

- CTE；
- 子查询；
- JOIN；
- schema-qualified table；
- 引号与大小写；
- 数据库方言；
- `UPDATE ... FROM`；
- 动态 SQL；
- 存储过程或函数；
- 触发器带来的间接写入。

因此不建议让第一版缓存依赖“完整 SQL 解析器”。

更稳妥的是支持两种策略：

1. **保守失效**：任意成功 DML 清除指定 namespace；
2. **显式标签/表依赖**：用户或宏显式声明 query tags。

例如：

```rust
#[cache(namespace = "activity", tags = ["biz_activity"])]
async fn select_activity(...);
```

或运行时：

```rust
CachePolicy::new("activity")
    .tags(["biz_activity"])
    .ttl(Duration::from_secs(60))
```

---

## 6. 二级缓存语义建议

### 6.1 缓存对象

建议缓存数据库返回的原始：

```rust
rbs::Value
```

而不是缓存最终泛型类型 `T`。

原因：

- `Intercept` 在解码前看到的就是 `Value`；
- 所有返回类型共享同一个缓存入口；
- 不需要存放 `Any`；
- 内存后端可以直接存 `Value`；
- Redis 后端可用 JSON、MessagePack、CBOR 或专用二进制格式序列化；
- 缓存命中后继续走已有 `decode<T>`。

注意：缓存内容应克隆为 owned `Value`，不能保存拦截器收到的引用。

---

### 6.2 Cache Key

最低要求：

```text
prefix
+ cache schema version
+ namespace
+ datasource identity
+ driver type
+ normalized/final SQL
+ encoded args
+ tenant/shard/routing context
```

示意：

```text
rbatis:l2:v1:{namespace}:{datasource}:{hash}
```

其中：

```text
hash = H(
    driver
    || final_sql
    || canonical_rbs(args)
    || tenant
    || shard
)
```

关键原则：

- 使用所有 SQL 重写完成后的 SQL；
- 参数序列化必须保留类型；
- `1`、`"1"`、`1.0`、`NULL` 不得映射成相同 Key；
- 不使用 `Debug` 字符串作为稳定协议；
- Key 中不能直接泄露密码、身份证、Token 等敏感参数；
- 分布式后端需要稳定、跨进程一致的编码算法；
- 算法和格式应带版本号，便于升级。

建议采用：

```rust
struct CacheKeyBuilder {
    version: u8,
    namespace: String,
    datasource_id: String,
    context_provider: Arc<dyn CacheContextProvider>,
}
```

---

### 6.3 缓存策略

建议内建：

```rust
pub struct CachePolicy {
    pub namespace: String,
    pub ttl: Duration,
    pub refresh_ahead: Option<Duration>,
    pub cache_null: bool,
    pub null_ttl: Option<Duration>,
    pub max_value_size: Option<usize>,
    pub transaction_mode: TransactionCacheMode,
    pub failure_mode: CacheFailureMode,
}
```

推荐默认值：

- 缓存默认关闭；
- TTL 必填或给较短默认值，例如 60 秒；
- 允许缓存空数组，防止缓存穿透；
- 空结果使用更短 TTL；
- 超大结果不缓存；
- 缓存故障默认 `FailOpen`；
- 事务内默认 bypass。

---

## 7. 推荐 SPI

### 7.1 CacheStore

建议核心只定义 object-safe 异步接口：

```rust
#[async_trait]
pub trait CacheStore: Send + Sync + Debug {
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;

    async fn set(
        &self,
        key: CacheKey,
        value: Value,
        ttl: Duration,
        tags: &[CacheTag],
    ) -> Result<(), CacheError>;

    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;

    async fn invalidate_tags(
        &self,
        tags: &[CacheTag],
    ) -> Result<u64, CacheError>;

    async fn clear_namespace(
        &self,
        namespace: &str,
    ) -> Result<u64, CacheError>;
}
```

不建议把 Redis API 直接放进 `rbatis` 主 crate，否则会引入：

- Redis 客户端依赖；
- runtime/features 冲突；
- 网络连接管理；
- 序列化协议绑定；
- 不必要的编译体积。

---

### 7.2 CachePolicyProvider

不同查询通常需要不同 TTL 和 namespace：

```rust
pub trait CachePolicyProvider: Send + Sync + Debug {
    fn query_policy(
        &self,
        ctx: &QueryContext<'_>,
    ) -> Option<CachePolicy>;

    fn invalidation_policy(
        &self,
        ctx: &ExecContext<'_>,
    ) -> InvalidationPolicy;
}
```

这样可支持：

- 全局规则；
- SQL 前缀规则；
- 表/tag 规则；
- 自定义函数名或 statement ID；
- 租户上下文；
- 不缓存 `FOR UPDATE`；
- 不缓存非确定性查询。

---

### 7.3 显式 QueryContext

当前 `Intercept` 参数数量已经较多。继续追加布尔值会使 API 更难维护。

建议引入上下文：

```rust
pub struct InterceptContext<'a> {
    pub task_id: i64,
    pub executor: &'a dyn Executor,
    pub executor_kind: ExecutorKind,
    pub operation: OperationKind,
    pub sql: &'a mut String,
    pub args: &'a mut Vec<Value>,
    pub statement_id: Option<&'a str>,
    pub transaction_id: Option<i64>,
}
```

这也是未来支持：

- trace；
- metrics；
- cache；
- tenant；
- audit；
- statement ID；

更稳定的基础。

考虑到这是 breaking change，可先新增 `InterceptV2`，保留旧 `Intercept` 适配层。

---

## 8. 查询缓存算法

### 8.1 Query Before

```text
1. 判断 ResultType 是否为 Query
2. 判断 executor_kind/transaction mode
3. 读取 CachePolicy
4. 排除不可缓存 SQL
5. 使用最终 SQL + args +上下文构建 Key
6. 查询 CacheStore
7. 命中：
   - 将 Value 写入 query result
   - 返回 Action::Return
8. 未命中：
   - 记录 task_id -> pending key/policy
   - 返回 Action::Next
```

不可缓存 SQL 默认应包括：

- `SELECT ... FOR UPDATE`
- `SELECT ... FOR SHARE`
- 数据库锁相关语句
- 用户明确 bypass 的查询
- 不确定性函数查询，例如 `random()`、`now()` 等，除非显式允许
- 事务内查询，默认 bypass

---

### 8.2 Query After

```text
1. 根据 task_id 取得 pending context
2. 仅处理数据库查询成功结果
3. 判断空结果策略
4. 判断最大值大小
5. 写入 CacheStore
6. 清理 pending context
7. 始终返回 Action::Next
```

必须确保以下路径都会清理 pending 状态：

- 查询成功；
- 查询失败；
- 后续拦截器失败；
- 缓存后端失败；
- 查询被其他拦截器提前短路。

当前 `before/after` 没有 finally/error hook，因此长期看建议加入：

```rust
async fn on_error(...)
```

或者用单一 `around` 模型。

---

## 9. 写操作和失效策略

### 9.1 非事务 DML

处理 `ResultType::Exec`：

```text
before:
  计算待失效 tags/namespace

after:
  仅当数据库执行成功时失效
```

必须在执行成功后失效，不能在 `before` 直接删除，否则失败的更新也会制造不必要的缓存抖动。

---

### 9.2 事务 DML

事务中不应立即把新结果写入共享缓存，也不能在尚未提交时让其他请求看到修改。

推荐策略：

```text
事务查询：
  默认 bypass L2 cache

事务写：
  记录 pending invalidation 到 tx_id

commit 成功：
  执行 pending invalidation

rollback：
  丢弃 pending invalidation
```

即：

```rust
pending_invalidations: Map<TxId, HashSet<CacheTag>>
```

这要求核心层暴露 commit/rollback 生命周期事件。

如果暂不修改核心，可在 MVP 中采取保守模式：

> 事务内所有查询不读写缓存；事务内 DML 成功后立即清 namespace。

这不会读取未提交数据，但会造成不必要失效；而且由于事务可能回滚，缓存命中率会降低。它只适合作为过渡方案，不是最终方案。

---

### 9.3 外部写入问题

即使 RBatis 本身正确失效，数据库还可能被以下组件修改：

- 其他服务；
- 管理后台；
- 数据迁移脚本；
- 定时任务；
- 手工 SQL；
- 其他语言应用。

因此任何二级缓存都不能只依赖 RBatis 进程内 DML 拦截。

生产建议至少具备一种：

1. 短 TTL；
2. Redis Pub/Sub 失效广播；
3. 统一写入口；
4. CDC/binlog 驱动失效；
5. 版本号 namespace；
6. 管理 API 主动失效。

---

## 10. 内存后端设计

建议独立 crate 使用成熟缓存库，例如 Moka，而不是在主 crate 自己造 LRU。

能力：

- 有界容量；
- TTL/TTI；
- 并发访问；
- 淘汰策略；
- singleflight；
- 指标。

示意：

```rust
pub struct MemoryCacheStore {
    values: moka::future::Cache<CacheKey, Arc<Value>>,
    tag_index: ConcurrentMap<CacheTag, Set<CacheKey>>,
}
```

注意 tag index 和 value eviction 的一致性。简单维护 `tag -> keys` 容易产生陈旧索引。

更简单的第一版可以用**版本化 namespace/tag**：

```text
version:tag:biz_activity = 42
query key includes version 42
invalidate tag -> increment version to 43
```

优点：

- 失效为 O(1)；
- 不需要扫描所有 Key；
- Redis 易实现；
- 多节点适用。

缺点：

- 老 Key 等待 TTL 自然回收；
- Key 空间短期增加。

综合看，版本化 tag 很适合二级缓存。

---

## 11. Redis 后端设计

推荐采用 cache-aside：

```text
GET key
miss -> DB query -> SET key value EX ttl
DML commit -> INCR tag-version / publish invalidation
```

需关注：

### 缓存击穿

同一个热门 Key 失效时，不应让大量请求同时访问数据库。

可提供：

- 进程内 singleflight；
- Redis 分布式锁，可选；
- 软 TTL + 后台刷新；
- TTL jitter。

### 缓存雪崩

TTL 应加入随机抖动：

```text
effective_ttl = ttl ± jitter
```

### 缓存穿透

支持短 TTL 空结果缓存，并允许用户关闭。

### 序列化

Redis value 建议包装 envelope：

```rust
struct CacheEnvelope {
    version: u16,
    created_at: i64,
    expires_at: i64,
    codec: CacheCodec,
    payload: Vec<u8>,
}
```

不要把某一编码永久写死，否则升级 `rbs::Value` 结构后可能出现兼容问题。

---

## 12. 为什么不建议直接内置到 CRUD 宏

也可以给 `crud!`、`py_sql!`、`html_sql!` 增加缓存注解，但不建议把它作为唯一入口。

原因：

- 用户可以直接调用 `Executor::query`；
- 宏级缓存会重复实现执行语义；
- 普通 CRUD、动态 SQL、HTML SQL 可能产生多套缓存路径；
- 缓存失效仍需要覆盖底层 `exec`；
- 事务一致性仍然需要核心事件。

更好的分工是：

- 缓存执行统一位于执行器/拦截器层；
- 宏只生成 `statement_id`、namespace、tags、policy 元数据。

例如未来支持：

```rust
#[py_sql(
    cache_namespace = "activity",
    cache_ttl = "60s",
    cache_tags = ["biz_activity"]
)]
async fn select_activity(...);
```

宏将元数据传入执行上下文，实际缓存仍由统一插件完成。

---

## 13. 推荐实施路线

## 阶段 0：修复拦截器基础问题

优先处理：

1. `apply_after` 返回 `Action::Return` 时错误返回 `before_result`；
2. 增加拦截器顺序测试；
3. 增加查询短路测试；
4. 增加 after 修改结果测试；
5. 明确 `Action::Return` 在 before/after 中的语义。

涉及：

- `src/plugin/intercept/mod.rs`
- `src/executor.rs`
- `tests/intercept_test.rs`
- `tests/intercept_extended_test.rs`

---

## 阶段 1：外部插件 MVP

不改变公共 API，先验证价值：

- `CacheStore`；
- 内存实现；
- `CacheIntercept`；
- SQL + args Key；
- TTL；
- 查询命中短路；
- 查询成功回填；
- 任意成功 DML 清空 namespace；
- 事务查询默认 bypass；
- fail-open；
- 基础 hit/miss 指标。

局限要明确写入文档：

- 不保证事务提交后才精确失效；
- 不做 SQL 表依赖解析；
- 外部写入只能依靠 TTL；
- 需要用户正确安排拦截器顺序。

---

## 阶段 2：核心上下文增强

增加：

- `ExecutorKind`；
- `OperationKind`；
- `statement_id`；
- `datasource_id`；
- `transaction_id`；
- cache bypass hint；
- tenant/shard context。

建议使用新增 V2 API，避免立即破坏已有插件。

---

## 阶段 3：事务生命周期

增加：

- begin event；
- commit-success event；
- rollback event；
- transaction pending invalidations；
- commit 后批量失效；
- rollback 丢弃；
- 事务查询默认不读写 L2。

这是达到生产级一致性的关键阶段。

---

## 阶段 4：分布式后端

新增独立 Redis crate：

- 稳定 Key 协议；
- 序列化 envelope；
- tag 版本；
- Pub/Sub；
- TTL jitter；
- singleflight；
- 可观测指标；
- 手动失效 API。

---

## 阶段 5：宏元数据

为 `crud!`、`py_sql!`、`html_sql!` 提供可选缓存配置：

- statement ID；
- namespace；
- tags；
- TTL；
- bypass；
- flush tags。

这一阶段才能逐步接近 MyBatis `<cache>` / `useCache` / `flushCache` 的使用体验。

---

## 14. 测试矩阵

至少需要覆盖以下场景。

### 基础行为

- 相同 SQL 和参数第二次命中；
- 参数不同不命中；
- SQL 不同不命中；
- 参数类型不同不冲突；
- TTL 到期重新查询；
- 空结果缓存；
- 禁止缓存时始终查询数据库；
- 缓存后端异常时 fail-open。

### 拦截器顺序

- 分页 SQL 不同页 Key 不同；
- 动态表名不同 Key 不同；
- 多租户上下文不同 Key 不同；
- 日志拦截器是否观察到命中；
- 其他拦截器提前 `Return` 时不错误回填。

### DML 失效

- insert 成功后失效；
- update 成功后失效；
- delete 成功后失效；
- DML 失败不失效或按策略处理；
- namespace 清除；
- tag 清除；
- 批量 DML。

### 事务

- 事务内查询默认 bypass；
- 未提交数据不进入共享缓存；
- commit 后失效；
- rollback 不失效；
- 一个事务多次写合并 tags；
- guard drop/自动事务路径；
- commit 失败不执行失效；
- rollback 失败的保守处理。

### 并发

- 同 Key 并发 miss 只访问一次数据库；
- 高并发 get/set；
- 失效和回填竞争；
- 写操作与旧查询同时进行；
- Redis 节点短暂不可用；
- 大 Value 和容量淘汰。

---

## 15. 现有文档与历史线索

v4 使用指南确认：

- CRUD 宏统一接受执行器；
- 事务通过 `RBatisTxExecutor`；
- 拦截器是官方扩展机制；
- 文档没有二级缓存章节或公开 API。

仓库搜索结果：

- 当前源码没有 query cache、Redis cache、Moka、LRU 结果缓存模块；
- GitHub 没有明确的二级缓存 issue；
- 历史 `6c0ac8eb` 提交曾新增 `StatementCache<T>`，它是 prepared statement LRU，并不是 ORM 结果缓存；
- GitHub issue #603 也讨论的是 MySQL statement cache capacity，仍属于驱动层 prepared statements。

因此不能把已有 statement cache 当成二级缓存复用。两者分别位于：

```text
Prepared statement cache:
SQL -> 已准备语句句柄

ORM L2 cache:
SQL + args + context -> 查询结果 Value
```

---

## 16. 最终推荐架构

```text
CRUD / py_sql / html_sql / raw query
                  |
                  v
           Executor::query
                  |
                  v
       SQL rewriting interceptors
    Page / Tenant / Dynamic Table
                  |
                  v
            CacheIntercept
        +-------------------+
        | policy / key      |
        | transaction mode  |
        | metrics           |
        +-------------------+
           | hit       | miss
           v           v
       cached Value   DB Connection
                         |
                         v
                    query Value
                         |
                         v
                  CacheStore::set
                         |
                         v
                      decode<T>
```

写路径：

```text
Executor::exec
    |
    v
DB write
    |
    +-- non-transaction success --> invalidate tag/namespace
    |
    +-- transaction success -----> collect pending invalidation
                                      |
                                      +-- commit --> invalidate
                                      +-- rollback -> discard
```

---

# 总体评价

RBatis 的执行器和拦截器架构已经为二级缓存提供了约 70% 的基础：

- 统一查询入口；
- 原始 `Value` 结果；
- before 短路；
- after 回填；
- 全局共享拦截器；
- CRUD 和动态 SQL 最终都经过 Executor。

剩余最重要的 30% 是：

1. 可靠的事务识别；
2. commit/rollback 生命周期；
3. 稳定的 cache key 上下文；
4. 精确或可配置的失效策略；
5. 拦截器 after 返回语义修复；
6. 分布式一致性和外部写入处理。

因此，**二级缓存技术上可行，而且拦截器是正确入口；但生产级实现应先推动上游补充通用执行上下文和事务事件，再由 RBatis-Plus 提供缓存 SPI、后端与产品能力，而不是单纯提交一个 Redis 拦截器。**具体边界与默认语义见 [架构决策记录](DECISIONS.md)。

---

## 17. 最新架构总结（基于 `4050edd3`）

本次调研在最新 `master@4050edd3dad03a113b8bb4f5818a006f11f2da78`（提交标题：`Limit decode fallback to single-column values`）上重复了 CodeGraph 全量构建与影响分析，关键新增证据如下。

### 17.1 RBatis 工作区分层

RBatis 仓库的源码社区组织进一步证实了"增强 `Executor` + `Intercept` + `rbs::Value` + `py_sql`"优于"重建 ORM"的策略：

- **`runtime`**：连接池、驱动装载、`RBatis` 主结构；
- **`macro-driver`**：`py_sql!` / `html_sql!` / `crud!` 入口；
- **`codegen`**：AST -> 代码生成；
- **`rbdc` / `rbdc-*`**：连接、驱动、解码器。

每一层都通过 `Executor` 接口汇合到统一路径，因此缓存只需在 `Executor` 的拦截链上增加一次挂载即可覆盖：

- 普通查询；
- 事务内查询；
- `py_sql!` 生成查询；
- `html_sql!` 生成查询；
- `crud!` 包装查询；
- 原始 SQL。

### 17.2 `Executor` 是统一入口

最新 CodeGraph 同样确认 `Executor` 是跨实现的统一入口：

```text
RBatis::query / RBatis::exec
RBatisConnExecutor::query / RBatisConnExecutor::exec
RBatisTxExecutor::query / RBatisTxExecutor::exec
RBatisTxExecutorGuard::query / RBatisTxExecutorGuard::exec
```

四个实现都通过同一条 `apply_before / apply_after` 路径，因此一个 `CacheIntercept` 即覆盖全部。

### 17.3 CRUD 构建于 `py_sql` 之上

`crud!` 宏最终展开为 `py_sql!` 调用，所有 CRUD 查询都走 `py_sql` 的执行路径。这意味着**只需在 `py_sql` 之后做缓存键归一化与拦截挂载**，CRUD 与 `py_sql` 即可自动获得缓存能力。

### 17.4 `html_sql` 具有最高 CodeGraph criticality

`html_sql` 是当前基线下 criticality 最高的执行流。它的语法树解析和模板展开涉及最深的社区耦合，所以任何扩展点（context、statement_id、tags）的接入都必须：

- 不修改 `html_sql` 的展开产物签名；
- 通过统一的执行上下文注入，避免在模板里硬编码缓存元数据；
- 优先在 `py_sql` / `crud` 上验证，再推广到 `html_sql`。

### 17.5 宏驱动层的字符串约定（关键风险）

最新代码确认了两条字符串约定，必须在缓存实现中显式考虑：

1. **返回 token 字符串包含 `ExecResult`**：宏驱动在生成代码中以 `ExecResult` 作为返回类型标志位之一。`CacheIntercept` 必须在 `ResultType::Exec` 与 `ResultType::Query` 两种返回形态上分别走"无效化"与"读/写"两套逻辑，不能用类型擦除统一处理。
2. **执行器识别基于类型 token 字符串**：当前拦截器链用 executor 的类型名（字符串匹配）来识别 `RBatis` / `RBatisConnExecutor` / `RBatisTxExecutor`。如果未来通过字符串区分来触发缓存策略，需要把这种识别迁移到 `ExecutorKind`/`OperationKind` 枚举上，避免依赖脆弱字符串。

### 17.6 `PageIntercept` 的共享状态

`PageIntercept` 内部维护的 `page_map` 是**按 executor id** 共享的全局表。同样的 executor id 在并发请求之间复用，使分页上下文自然按连接粒度隔离。缓存拦截器必须放在 `PageIntercept` 之后，才能读到 SQL 重写后的最终文本；否则分页页码会错误地命中同一缓存条目。

### 17.7 事务生命周期尚未暴露给拦截器

最新代码再次确认：`RBatisConnExecutor::begin` 接管连接，随后 `commit` / `rollback` 直接调用 `Connection::commit / rollback`，**完全绕过 `Intercept` 链**。这意味着：

- `CacheIntercept` 在不修改上游的情况下，只能采用"事务内保守失效"或"事务内读 bypass"；
- 生产级一致性需要新增 `TransactionListener`（Begin / Commit / Rollback / SavePoint）钩子。

### 17.8 `RBatisConnExecutor::begin` 的语义陷阱

`begin` 路径对连接所有权有严格要求：

- `RBatisConnExecutor::begin` 会**消费 `self.conn`**（`Arc<Mutex<Box<dyn Connection>>>`），把内部连接移交给事务；
- 如果先 `clone()` 出第二个 `RBatisConnExecutor` 再调用 `begin`，由于 `Arc<Mutex<...>>` 已移交给第一个事务，**第二个克隆体的 `begin` 将失败**；
- 因此 `CacheIntercept` 在事务场景下必须只在原始（非克隆）执行器上观察 `begin` 事件，**不能依赖克隆实例**。

### 17.9 改进方向不变

这些新证据**没有**改变 §16 的结论，而是进一步验证：

- 二级缓存的接入点仍是 `Executor` 的 `apply_before / apply_after`；
- 需要新增的最小上游能力是：`ExecutorKind` 枚举、`OperationKind` 枚举、`LifecycleContext`、`TransactionListener`、`apply_after` 语义修复；
- RBatis-Plus 侧只需扩展 `CacheIntercept` 来消费上述信号，而不是另起一套 ORM。

因此 RBatis-Plus 的实施计划保持不变：先推动上游最小钩子（Phase 0），再由 RBatis-Plus 承载缓存产品能力（Phase 1+）。

---