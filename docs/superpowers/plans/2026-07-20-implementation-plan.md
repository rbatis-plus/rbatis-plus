# RBatis-Plus 二级缓存实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 RBatis 上游添加最小、通用、可选的执行上下文和生命周期钩子，然后在 RBatis-Plus 中交付完整的 ORM L2（结果）缓存产品。

**Architecture:** 上游 rbatis 只提供通用扩展基础；RBatis-Plus 提供 CacheStore SPI、CacheIntercept、策略、Key/Envelope 协议、内存与 Redis 后端。

**Tech Stack:** Rust, rbatis, moka, redis-rs, blake3, serde, tokio

**Related Design Doc:** `docs/superpowers/specs/2026-07-20-architecture-spec.md`

---

## 1. Goals and Non-Goals

### 1.1 Goals

1. 在上游 `rbatis` 添加最小、通用、可选的执行上下文和生命周期钩子
2. 在 RBatis-Plus 中交付完整 ORM L2 缓存产品（内存后端 + Redis 后端 + 声明式注解）
3. 保持 RBatis API 稳定，不引入破坏性变更
4. 通过 feature flag 增量落地
5. 通过测试矩阵后才打 `rbatis-plus-0.1.0` 标签

### 1.2 Non-Goals (this milestone)

1. CDC / binlog ingestion
2. Redis Pub/Sub 以外的跨进程失效传输
3. Redis 以外的分布式锁服务
4. SQL 解析器（仅利用上游 hooks）
5. ORM L1（语句级）缓存
6. Web 管理 UI
7. 特定可观测后端（OpenTelemetry, StatsD）

---

## 2. Repository Layout

```text
rbatis-plus/                          # facade crate (re-exports)
├── rbatis-plus-core/                 # 条件构造器 + 元数据 + mapper trait + 分页
│   └── src/{conditions,derive,mapper,metadata,method,page,toolkit}/
├── rbatis-plus-extension/            # 拦截器链 + 加密/签名/i18n/观察 + service
│   └── src/{crypto,i18n,inner,insert_ignore,observation,service,signature}/
├── rbatis-plus-macros/               # proc-macro derive (TableName/TableId 等)
├── rbatis-plus-generator/            # 代码生成器 + 3 模板引擎
│   └── src/{config,template,query,engine}/
├── rbatis-plus-sqlparser/            # SQL 解析 + 方言
│   └── src/{parser,rewrite,dialect}/
├── rbatis-plus-vernal/               # axum/actix 集成 + SqlRunner + Transactions
│   └── src/{axum_integration,actix_integration,state,transaction,sql_runner}/
├── tests/                            # 集成测试
└── docs/                             # 文档
```

---

## 3. Phases 0-6

### 3.1 Phase 0 — Upstream foundations: hooks and semantics

**Purpose:** 落地所有后续阶段消费的最小上游接口。

**Tasks:**
- [ ] **Step 1:** 添加 `ExecutorKind`, `OperationKind`, `LifecycleContext`, `CacheHint` 到 `rbatis/src/plugin/intercept/mod.rs`
- [ ] **Step 2:** 实现 `apply_lifecycle` helper（镜像 `apply_before` / `apply_after`）
- [ ] **Step 3:** 在 feature flag `fix-after-return-semantic` 后面修复 `apply_after` 语义
- [ ] **Step 4:** 扩展 `Intercept::ctx() -> Option<LifecycleContext>`（默认 impl 返回 `None`）
- [ ] **Step 5:** 测试：`tests/intercept_test.rs` + `tests/lifecycle_test.rs`
- [ ] **Step 6:** 文档：`rbatis/docs/intercept.md` + `lifecycle.md`

**Exit criteria:**
- CI on upstream `master` passes
- `cargo doc` produces no broken links
- Existing public `Intercept` consumers compile without modification
- New helpers covered by at least 6 unit tests

### 3.2 Phase 1 — External plugin MVP (in RBatis-Plus)

**Purpose:** 证明缓存价值，无需暴露到上游 crate。

**Tasks:**
- [ ] **Step 1:** `rbatis-plus-core/` 引入 `CachePolicy`, `CacheKeyBuilder`, `CacheIntercept`, `MetricsRecorder`
- [ ] **Step 2:** `rbatis-plus-mem/` 引入 `MemoryCacheStore`（Moka）
- [ ] **Step 3:** 行为矩阵测试（同 SQL 同 args 命中、不同 args 不命中、TTL 过期、后端降级、DML 清命名空间）
- [ ] **Step 4:** 文档：MVP 限制说明

**Exit criteria:**
- All MVP tests green on CI
- Benchmark: cached path >= 5x speedup over PostgreSQL loopback
- Plugin works with RBatis `master` + Phase 0 PR

### 3.3 Phase 2 — Core context enrichment

**Purpose:** 扩展 `LifecycleContext` 使缓存失效精确。

**Tasks:**
- [ ] **Step 1:** 上游：添加 `datasource_id`, `driver`, `statement_id`, `tenant_id`, `shard_value`
- [ ] **Step 2:** 上游：从 `py_sql!`, `html_sql!`, `crud!` 传播 `statement_id`
- [ ] **Step 3:** RBatis-Plus：`CacheKeyBuilder` 消费新字段
- [ ] **Step 4:** 测试：租户隔离、分片键、statement_id 传播

**Exit criteria:**
- No regression in upstream CRUD macro tests
- RBatis-Plus compiles against enriched `LifecycleContext`

### 3.4 Phase 3 — Transaction lifecycle and deferred invalidation

**Purpose:** 支持延迟失效，防止未提交写入泄露到缓存。

**Tasks:**
- [ ] **Step 1:** 上游：添加 `TransactionListener` trait + `RBatis::listeners` 字段
- [ ] **Step 2:** 上游：从 `RBatisTxExecutor` 和 `RBatisTxExecutorGuard` 发射 `Begin/Commit/Rollback/SavePoint` 事件
- [ ] **Step 3:** RBatis-Plus-Core：引入 `DeferredInvalidationMap`
- [ ] **Step 4:** `TransactionCacheMode::Defer` 语义实现
- [ ] **Step 5:** 测试：事务内读不命中共享缓存、提交触发失效、回滚丢弃标签

**Exit criteria:**
- All transaction matrix tests pass
- `cargo bench` shows no regression for non-cache path

### 3.5 Phase 4 — Distributed backend (Redis)

**Purpose:** 提供生产级跨进程后端。

**Tasks:**
- [ ] **Step 1:** 新 crate `rbatis-plus-redis`：`RedisCacheStore` + Envelope codec + Tag-version keys
- [ ] **Step 2:** SingleFlight + TTL jitter + Pub/Sub
- [ ] **Step 3:** 配置：builder with `url`, `key_prefix`, `pub_sub`, `jitter`
- [ ] **Step 4:** 测试：跨进程失效、连接断开降级

**Exit criteria:**
- All Redis tests pass against `redis:7` containers on CI
- Cross-process test demonstrates invalidation within 1s

### 3.6 Phase 5 — Macro annotations

**Purpose:** 带来 MyBatis 风格的声明式人体工程学。

**Tasks:**
- [ ] **Step 1:** 新 crate `rbatis-plus-macros`：`#[rbatis_plus::cache(...)]` 属性宏
- [ ] **Step 2:** Compile-fail tests
- [ ] **Step 3:** Sample crate

**Exit criteria:**
- Macro produces no extra runtime cost on miss path
- All compile-fail tests pass

> Phase 5 **not required** for 0.1.0 tag.

### 3.7 Phase 6 — Observability, ergonomics, polish

**Purpose:** 发布就绪质量。

**Tasks:**
- [ ] **Step 1:** 内建 metrics：`hit`, `miss`, `store_error`, `invalidate_by_tag`
- [ ] **Step 2:** 可选 `tracing` 集成（feature `tracing`）
- [ ] **Step 3:** CLI 管理 helpers：`invalidate_tags`, `clear_namespace`, `dump_keys_for_diagnostics`
- [ ] **Step 4:** 文档：每个公开类型有 doc comment + example

**Exit criteria:**
- `cargo doc --no-deps` shows zero warnings
- README passes the "5 second test"

---

## 4. Upstream PR Plan

| # | Title | Touchpoint files | Phase |
|---|---|---|---|
| 1 | rbatis: add lifecycle hooks and fix apply_after semantics | `intercept/mod.rs`, `executor.rs`, `rbatis.rs` | 0 |
| 2 | rbatis: enrich lifecycle context for statement, tx, shard | `intercept/mod.rs`, `executor.rs`, macro crates | 2 |
| 3 | rbatis: expose transaction lifecycle for plugin authors | `intercept/mod.rs`, `executor.rs`, `rbatis.rs` | 3 |
| 4 | rbatis: structured interceptor ordering | `rbatis.rs`, `plugin/mod.rs`, tests | 0 |

---

## 5. Concurrency and Consistency Posture

| Concern | Planned default |
|---|---|
| Cache visibility | Tokio task-local reads, Moka writes are atomic |
| Miss concurrency | Per-key `tokio::sync::Mutex` singleflight |
| Tag invalidation | Versioned: bump number, never scan |
| TTL jitter | +/- 10% on the configured TTL |
| Maximum cached value | 1 MiB, larger values bypass silently (FailOpen) |
| Backend unreachable | FailOpen with WARN metric and single error log line |
| External writes | TTL only, plus optional Redis Pub/Sub bridge |
| Tenant isolation | Namespace + key prefix enforced, no cross-tenant |
| Transactional writes | Defer until commit, discard on rollback |
| Read-your-write in same tx | Optional via `CacheHint::Refresh` and `Bypass` modes |

---

## 6. Test Matrix (Mapped to Phases)

### 6.1 Basics (Phases 1, 6)
- [ ] Same SQL + same args hit
- [ ] Different args miss
- [ ] Different SQL miss
- [ ] Type-distinct values produce distinct keys
- [ ] TTL expiry forces miss
- [ ] Empty result cacheable, shorter TTL
- [ ] `CacheHint::Bypass` always queries DB
- [ ] `CacheHint::Refresh` overwrites on hit
- [ ] Backend error => FailOpen default, FailClosed opt-in

### 6.2 Interceptor ordering (Phase 0)
- [ ] Pagination changes the cache key per page
- [ ] Tenant id changes the cache key
- [ ] Dynamic table name changes the cache key
- [ ] Log interceptor placed after cache observes hit-or-miss
- [ ] Another interceptor short-circuits with `Return`; cache stays silent

### 6.3 DML invalidation (Phases 1, 3)
- [ ] Insert success invalidates tags
- [ ] Update success invalidates tags
- [ ] Delete success invalidates tags
- [ ] DML failure does not invalidate
- [ ] `clear_namespace` removes all matching keys
- [ ] Bulk DML merges tags

### 6.4 Transactions (Phase 3)
- [ ] In-tx read never reads shared cache
- [ ] Uncommitted data never enters shared cache
- [ ] Commit triggers invalidation
- [ ] Rollback discards pending
- [ ] Same tx, multiple writes merge tags
- [ ] Guard-Drop rollback path discards
- [ ] Commit failure does not invalidate

### 6.5 Concurrency (Phases 1, 4)
- [ ] Concurrent misses on one key produce exactly one DB hit
- [ ] High QPS get/set does not panic or deadlock
- [ ] Race between set and invalidate leaves no orphaned value
- [ ] Redis connection drop transitions to FailOpen with metric
- [ ] Memory-pressure eviction coexists with tag invalidation

### 6.6 Compatibility
- [ ] Upstream `tests/intercept_test.rs` remains green
- [ ] Upstream CRUD macro tests remain green
- [ ] An external RBatis user with no cache plugin is binary-compatible

### 6.7 Macro string-heuristic regression tests
- [ ] Return-token contract locked via golden test
- [ ] Type-token contract locked via golden test
- [ ] Pin spelling of both token strings in `static` assertion
- [ ] Custom `Wrapper` types compile against current token shape
- [ ] `py_sql` query tokens unchanged before/after Phase 0/1

### 6.8 Pagination shared-state tests
- [ ] `PageIntercept::before` rewrites SQL; distinct cache keys for pages 1, 2, 3
- [ ] Two concurrent requests with different page numbers do not share cache entry
- [ ] Query without LIMIT/OFFSET never hits paginated cache entry
- [ ] Cache key reflects rewritten text after page interceptor
- [ ] Multi-tenant + pagination: tenant A page 2 does not collide with tenant B page 2

---

## 7. Release and Versioning Plan

- Workspace `version = "0.1.0"` (planned)
- SemVer:
  - 0.1.x — Phase 0+1 stable, MVP behaviour
  - 0.2.x — Phase 2 features land
  - 0.3.x — Phase 3 features land; production transaction semantics
  - 0.4.x — Phase 4 features land; Redis backend GA
  - 1.0 — Phase 5+6 lands; full test matrix green

---

## 8. Acceptance Criteria for 0.1.0 Tag

- [ ] Phases 0 and 1 complete
- [ ] All MVP test rows in section 6.1 pass on CI
- [ ] Upstream PRs 1 and 4 merged into `rbatis` `master`
- [ ] `cargo doc` clean across the workspace
- [ ] README passes the "5 second test"
- [ ] Architecture and Implementation Plan documents reviewed and merged
