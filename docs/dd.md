
### 最终推荐架构


```markdown
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

```markdown
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

```markdown
RBatis
└─ 提供可靠的底层扩展点
├─ 执行上下文
├─ 事务生命周期事件
├─ 拦截器语义修复
└─ statement/query metadata

RBatis-Plus
└─ 提供开箱即用的二级缓存
├─ 内存缓存
├─ Redis 缓存
├─ 注解/宏配置
├─ 自动失效
├─ 多租户隔离
├─ 防击穿/雪崩/穿透
├─ 监控指标
└─ Spring/MyBatis-Plus 风格开发体验
```

```markdown
┌─────────────────────────────────────────────┐
│                RBatis-Plus                  │
│                                             │
│  #[cacheable] / #[cache_evict]              │
│  Mapper metadata / Entity metadata          │
│  Cache policy / Namespace / Tags            │
│  Tenant / Logic delete / Optimistic lock    │
│  Memory / Redis / Multi-level cache         │
│  Singleflight / Metrics / Refresh-ahead     │
└──────────────────────┬──────────────────────┘
                       │ public extension API
┌──────────────────────▼──────────────────────┐
│                  RBatis                     │
│                                             │
│  ExecutorContext                            │
│  Intercept result semantics                 │
│  Query/Exec interception                    │
│  Transaction commit/rollback events         │
│  Generic statement metadata                 │
└──────────────────────┬──────────────────────┘
                       │
                 rbdc / database
```

```markdown
rbatis-plus-core
├── mapper
├── metadata
├── interceptor
└── transaction context

rbatis-plus-cache
├── Cache
├── CacheManager
├── CacheKeyBuilder
├── CachePolicy
├── CacheContext
├── CacheInterceptor
├── CacheTransactionListener
└── metrics

rbatis-plus-cache-memory
└── MokaCache

rbatis-plus-cache-redis
├── RedisCache
├── codec
├── tag version
└── pub/sub

rbatis-plus-macros
├── cacheable
├── cache_put
├── cache_evict
└── cache_bypass
```

```markdown
RBatis 上游
├── 修复 Intercept after 返回语义
├── ExecutorKind / ExecutionContext
├── 事务 begin/commit/rollback 生命周期
└── 通用 statement metadata

RBatis-Plus
├── CacheStore / CacheManager / CachePolicy
├── CacheInterceptor
├── Key 与 Envelope 协议
├── Memory 缓存
├── Redis 分布式缓存
├── 声明式缓存宏
├── 多租户/动态表名/逻辑删除/乐观锁联动
└── 防穿透、击穿、雪崩和可观测性
```

```markdown
┌─────────────────────────────────────────────┐
│                 用户代码                    │
│ CRUD / raw_sql / py_sql / html_sql          │
└──────────────────────┬──────────────────────┘
                       │ 编译期
┌──────────────────────▼──────────────────────┐
│          rbatis-macro-driver                │
│ 函数签名分析 / 参数提取 / query-exec 判断   │
└──────────────────────┬──────────────────────┘
                       │ 调用
┌──────────────────────▼──────────────────────┐
│             rbatis-codegen                  │
│ PySQL / HTML AST / Expression / Token Gen   │
└──────────────────────┬──────────────────────┘
                       │ 生成 Rust
┌──────────────────────▼──────────────────────┐
│              Executor                      │
│ RBatis / Conn / Tx / TxGuard               │
└───────────────┬─────────────────────────────┘
                │
        ┌───────▼────────┐
        │ Intercept Chain │
        │ Page / Log / ...│
        └───────┬────────┘
                │
┌───────────────▼─────────────────────────────┐
│                 rbdc                        │
│ Pool / Driver / Connection / ExecResult     │
└───────────────┬─────────────────────────────┘
                │
┌───────────────▼─────────────────────────────┐
│ MySQL / PostgreSQL / SQLite / MSSQL / ...   │
└─────────────────────────────────────────────┘

查询返回：
DB Row → rbs::Value → rbatis::decode<T> → 用户类型
```

```markdown
┌────────────────────────────────────────────────────────────┐
│                       User Code                            │
│ Mapper / SqlSession / SqlSessionFactory                    │
└──────────────────────────────┬─────────────────────────────┘
                               │
┌──────────────────────────────▼─────────────────────────────┐
│               binding / session / defaults                  │
│ MapperProxy → MapperMethod → SqlCommand / MethodSignature │
│ DefaultSqlSession → DefaultSqlSessionFactory                │
│ Configuration (alias / typeHandler / mapper / cache / interceptor)│
└──────────────────────────────┬─────────────────────────────┘
                               │ plugin
┌──────────────────────────────▼─────────────────────────────┐
│                    plugin.Interceptor                      │
│  - Interceptor / Intercepts / Signature / Invocation        │
│  - Plugin (JDK dynamic proxy) / InterceptorChain            │
│  - 只能代理：Executor / ParameterHandler /                  │
│              ResultSetHandler / StatementHandler            │
└──────────────────────────────┬─────────────────────────────┘
                               │
┌──────────────────────────────▼─────────────────────────────┐
│                         executor                            │
│ CachingExecutor                                              │
│  ├─ TransactionalCacheManager  ←  二级缓存提交/回滚       │
│  └─ delegate: BaseExecutor                                   │
│       ├─ SimpleExecutor                                       │
│       ├─ ReuseExecutor                                       │
│       └─ BatchExecutor                                       │
│            │                                                 │
│            ▼                                                 │
│  StatementHandler                                            │
│   ├─ SimpleStatementHandler                                  │
│   ├─ PreparedStatementHandler                                │
│   ├─ CallableStatementHandler                                │
│   └─ RoutingStatementHandler                                 │
│  ParameterHandler → DefaultParameterHandler                   │
│  ResultSetHandler  → DefaultResultSetHandler                  │
└──────────────────────────────┬─────────────────────────────┘
                               │
┌──────────────────────────────▼─────────────────────────────┐
│            scripting / xmltags / builder                    │
│ XMLConfigBuilder / XMLMapperBuilder / XMLStatementBuilder   │
│ MapperAnnotationBuilder / MapperBuilderAssistant            │
│ XMLLanguageDriver / XMLScriptBuilder                        │
│ If/Where/Choose/ForEach/Set/Trim/VarDecl SqlNode           │
│ RawSqlSource / DynamicSqlSource / ProviderSqlSource         │
│ BoundSql / ParameterMapping / ResultMapping / ResultMap      │
│ Cache SPI + 装饰器（LRU/FIFO/Soft/Weak/Blocking/...）       │
└──────────────────────────────┬─────────────────────────────┘
                               │
┌──────────────────────────────▼─────────────────────────────┐
│            datasource / transaction / jdbc                 │
│ PooledDataSource / UnpooledDataSource / JNDI                │
│ JdbcTransaction / ManagedTransaction                        │
│ JDBC PreparedStatement / ResultSet / Connection              │
└────────────────────────────────────────────────────────────┘
```

```markdown
```

```markdown
```