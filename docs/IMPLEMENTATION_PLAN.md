# RBatis-Plus Implementation Plan

> Status: PLANNED — Implementation has not started. All crate names, module
> paths, type names, feature flags, dependencies, and PR descriptions in this
> document are proposed designs subject to change before merging upstream or
> shipping RBatis-Plus 0.1.0.

- Date: 2026-07-24
- Upstream baseline (current): RBatis `master` @ `4050edd3dad03a113b8bb4f5818a006f11f2da78`
- Upstream baseline (previous): RBatis `master` @ `2df418feeab511c1899b2a110eef43228a1ad889`
- Latest surveyed commit title: `Limit decode fallback to single-column values`
- CodeGraph current stats (baseline): 178 files, 1,740 nodes, 17,805 edges,
  192 classes, 715 functions, 655 tests; edge breakdown includes 9,366
  `CALLS` and 5,524 `TESTED_BY`; 88 flows, 14 communities.
- Product: RBatis-Plus (separate workspace)
- Boundary principle (agreed): generic, reusable hooks live upstream in
  `rbatis`; the complete cache product (backends, integrations, batteries) lives
  in RBatis-Plus.
- Companion documents in this folder:
  - [ARCHITECTURE.md](./ARCHITECTURE.md)
  - [RBatis 支持二级缓存调研报告.md](./RBatis%20支持二级缓存调研报告.md)
  - [CACHE_SPECIFICATION.md](./CACHE_SPECIFICATION.md)
  - [TRANSACTION_CONSISTENCY.md](./TRANSACTION_CONSISTENCY.md)
  - [DECISIONS.md](./DECISIONS.md)
  - [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md)
  - [OBSERVABILITY_SECURITY_OPERATIONS.md](./OBSERVABILITY_SECURITY_OPERATIONS.md)
  - [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md)

## 0. Documentation Index

| # | Document | Role |
| - | --- | --- |
| 0 | [`/README.md`](../README.md) | Project entry, Mermaid diagram, doc index |
| 1 | [`docs/RBatis 支持二级缓存调研报告.md`](./RBatis%20支持二级缓存调研报告.md) | Upstream evidence baseline |
| 2 | [`docs/ARCHITECTURE.md`](./ARCHITECTURE.md) | Planned layering and components |
| 3 | `docs/IMPLEMENTATION_PLAN.md` | This document; phased plan |
| 4 | [`docs/CACHE_SPECIFICATION.md`](./CACHE_SPECIFICATION.md) | Wire protocol |
| 5 | [`docs/TRANSACTION_CONSISTENCY.md`](./TRANSACTION_CONSISTENCY.md) | Transaction semantics |
| 6 | [`docs/DECISIONS.md`](./DECISIONS.md) | ADRs |
| 7 | [`docs/INTEGRATION_GUIDE.md`](./INTEGRATION_GUIDE.md) | Integration template |
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](./OBSERVABILITY_SECURITY_OPERATIONS.md) | Observability / security / ops |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](./TEST_AND_ACCEPTANCE_PLAN.md) | Acceptance plan |

---

## 1. Goals and Non-Goals

### 1.1 Goals

1. Add minimal, generic, opt-in execution-context and lifecycle hooks upstream
   in `rbatis` so multiple use cases (cache, tracing, metrics, audit) can be
   built without further core churn.
2. Ship a complete ORM L2 (result) cache product in RBatis-Plus, including:
   - In-process backend (planned: `moka`-based, default).
   - Redis distributed backend (planned: `redis` crate, optional feature).
   - Declarative annotation surface for SQL macros.
   - Production-grade invalidation, singleflight, jitter, observability.
3. Keep RBatis API stable. No breaking changes in non-experimental features.
4. Land the changes incrementally behind feature flags where possible.
5. Pass the test matrix in section 7 before tagging `rbatis-plus-0.1.0`.

### 1.2 Non-Goals (this milestone)

1. CDC / binlog ingestion drivers.
2. Cross-process invalidation transports other than Redis Pub/Sub.
3. Distributed lock services other than Redis (`SET NX PX` + `LUA`).
4. Query-side normalization beyond what the upstream hooks expose (no SQL
   parser in core).
5. ORM L1 (statement-level) cache; the existing `rbdc` prepared-statement cache
   remains untouched.
6. A web admin UI. Only programmatic admin APIs.
7. Pushing all observability to a specific backend (OpenTelemetry, StatsD);
   provide a `MetricsRecorder` trait only.

---

## 2. Repository Layout (Planned)

RBatis-Plus is a multi-crate workspace. The tree below is the planned layout;
files marked TBD do not exist yet.

```text
rbatis-plus/
├── Cargo.toml                          # workspace root (planned)
├── README.md                           # planned
├── LICENSE                             # planned
├── CONTRIBUTING.md                     # planned
├── docs/
│   ├── IMPLEMENTATION_PLAN.md          # this file
│   ├── ARCHITECTURE.md                 # companion
│   ├── CACHE_SPECIFICATION.md          # companion
│   ├── TRANSACTION_CONSISTENCY.md      # companion
│   ├── DECISIONS.md                    # companion
│   ├── INTEGRATION_GUIDE.md            # companion
│   ├── OBSERVABILITY_SECURITY_OPERATIONS.md  # companion
│   ├── TEST_AND_ACCEPTANCE_PLAN.md     # companion
│   └── RBatis 支持二级缓存调研报告.md  # research baseline
│   ├── rbatis-plus/                    # meta-crate: re-exports + facade
│   │   ├── Cargo.toml                  # TBD
│   │   └── src/
│   │       ├── lib.rs                  # planned
│   │       └── prelude.rs              # planned
│   │
├── rbatis-plus-core/               # policy, key builder, intercept glue
│   │   ├── Cargo.toml                  # TBD
│   │   └── src/
│   │       ├── lib.rs                  # planned
│   │       ├── policy.rs               # planned: CachePolicy
│   │       ├── key.rs                  # planned: CacheKeyBuilder
│   │       ├── provider.rs             # planned: CachePolicyProvider
│   │       ├── intercept.rs            # planned: CacheIntercept
│   │       └── metrics.rs              # planned: MetricsRecorder trait
│   │
├── rbatis-plus-mem/                # in-process backend
│   │   ├── Cargo.toml                  # TBD
│   │   └── src/
│   │       ├── lib.rs                  # planned
│   │       └── store.rs                # planned: MemoryCacheStore
│   │
├── rbatis-plus-redis/              # distributed backend (optional)
│   │   ├── Cargo.toml                  # TBD
│   │   └── src/
│   │       ├── lib.rs                  # planned
│   │       ├── store.rs                # planned: RedisCacheStore
│   │       ├── envelope.rs             # planned: binary envelope codec
│   │       ├── versioning.rs           # planned: tag version keys
│   │       └── pubsub.rs               # planned: invalidation bus
│   │
├── rbatis-plus-macros/             # declarative annotations (Phase 5)
│   │   ├── Cargo.toml                  # TBD
│   │   └── src/
│   │       ├── lib.rs                  # planned
│   │       └── cache_attr.rs           # planned: #[rbatis_plus::cache(...)]
│   │
│   └── rbatis-plus-test/               # shared test fixtures (no runtime role)
│       ├── Cargo.toml                  # TBD
│       └── src/
│           ├── lib.rs                  # planned
│           ├── fake_store.rs           # planned: FakeCacheStore
│           └── clock.rs                # planned: DeterministicClock
│
└── examples/                           # planned
    ├── memory_cache_basics.rs
    ├── redis_cache_basics.rs
    └── macro_annotations.rs
```

### 2.1 Upstream RBatis Touchpoints (planned PR series)

The following files in `rbatis` are planned to change. The list is exhaustive
for this milestone; every other file in upstream is out of scope.

| File                                       | Planned change                                    |
| ------------------------------------------ | ------------------------------------------------- |
| `src/plugin/intercept/mod.rs`              | Add `ExecutorKind`, `OperationKind`, lifecycle    |
| `src/plugin/intercept/mod.rs`              | Fix `apply_after` semantic when `Action::Return`  |
| `src/executor.rs`                          | Inject executor kind, datasource id, tx id        |
| `src/executor.rs`                          | Emit `TransactionBegin/Commit/Rollback` events   |
| `src/rbatis.rs`                            | Register interceptors in deterministic order     |
| `src/plugin/mod.rs`                        | Re-export new types                               |
| `tests/intercept_test.rs`                  | Extend with cache-relevant scenarios             |

> No public method signature in `rbatis` is removed in this series. New symbols
> are added; the existing `Intercept` trait is preserved with deprecated aliases
> where unavoidable, then removed in a later major version.

---

## 3. Public API Drafts (Planned)

These signatures are planned and may shift during implementation. They are
recorded here to anchor the implementation, the upstream PR descriptions, and
the test matrix.

### 3.1 Upstream: hook module additions (planned)

Located in `src/plugin/intercept/mod.rs` and re-exported from
`src/plugin/mod.rs`. Names are planned.

```rust
// planned: src/plugin/intercept/mod.rs

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorKind {
    Root,
    Connection,
    Transaction,
    TransactionGuard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Query,
    Exec,
    Begin,
    Commit,
    Rollback,
    SavePoint,
}

#[derive(Clone, Debug, Default)]
pub struct LifecycleContext {
    pub task_id: i64,
    pub transaction_id: Option<i64>,
    pub executor_kind: ExecutorKind,
    pub datasource_id: Option<String>,
    pub driver: Option<&'static str>,
    pub statement_id: Option<String>,
    pub tenant_id: Option<String>,
    pub shard_value: Option<String>,
    pub cache_hint: CacheHint,                 // planned: opt-in opt-out
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheHint {
    Default,
    Bypass,
    Refresh,
}

// planned: extended Intercept trait (additive only)
#[async_trait]
pub trait InterceptExt: Intercept {
    async fn on_lifecycle(
        &self,
        event: OperationKind,
        ctx: &LifecycleContext,
    ) -> Result<(), Error>;
}
```

The existing `Intercept::before/after` signatures stay byte-for-byte
unchanged. The `LifecycleContext` is exposed via a new method
`Intercept::ctx(...)` (default impl returns `None`) and via a parallel
apply_lifecycle helper, mirroring how `apply_before/apply_after` work today.

### 3.2 Upstream: apply_after semantic fix (planned)

In `src/executor.rs`, the helper currently returns `before_result` when an
`after` interceptor returns `Action::Return`. The planned behavior is to return
the modified `result` instead. Wrapped behind a default-enabled feature flag
`fix-after-return-semantic = false` for the transition window.

```rust
// planned
pub async fn apply_after<R>(
    intercepts: &[Arc<dyn Intercept>],
    args: &mut Vec<Value>,
    sql: &mut String,
    result: &mut R,
) -> Result<bool, Error>
```

> The exact helper signature may change; semantically, "an `after` interceptor
> that returns `Action::Return` must propagate its edited `result` to the
> caller, not the pre-`before` snapshot".

### 3.3 Upstream: transaction events (planned)

Public surface additions, no removals.

```rust
// planned: src/plugin/intercept/mod.rs
#[async_trait]
pub trait TransactionListener: Send + Sync {
    async fn on_begin(&self, tx_id: i64, ctx: &LifecycleContext);
    async fn on_commit(&self, tx_id: i64, ctx: &LifecycleContext) -> Result<(), Error>;
    async fn on_rollback(&self, tx_id: i64, ctx: &LifecycleContext);
}

// planned: src/rbatis.rs additional fields
pub struct RBatis {
    pub pool: Arc<OnceLock<Box<dyn Pool>>>,
    pub intercepts: Arc<SyncVec<Arc<dyn Intercept>>>,
    pub listeners: Arc<SyncVec<Arc<dyn TransactionListener>>>, // planned
    pub task_id_generator: Arc<dyn IdGenerator>,
}
```

`RBatis::new_with_listeners(...)` is the planned construction helper; existing
`RBatis::new` keeps working and installs an empty `SyncVec` listener chain.

### 3.4 RBatis-Plus-Core: types

```rust
// planned: rbatis-plus-core/src/lib.rs
pub use policy::{CachePolicy, TransactionCacheMode, CacheFailureMode};
pub use key::{CacheKey, CacheKeyBuilder, KeyHasher};
pub use provider::{CachePolicyProvider, StaticPolicyProvider};
pub use intercept::CacheIntercept;
pub use metrics::{MetricsRecorder, NoopMetricsRecorder};
pub use error::CacheError;
```

`CachePolicy` is planned (values are placeholders until implementation):

```rust
// planned: rbatis-plus-core/src/policy.rs
#[derive(Clone, Debug)]
pub struct CachePolicy {
    pub namespace: String,                 // e.g. "user.profile"
    pub ttl: Duration,                     // required, default 60s
    pub null_ttl: Option<Duration>,        // optional, default TTL/4
    pub refresh_ahead: Option<Duration>,   // soft TTL refresh window
    pub cache_null: bool,                  // default true
    pub max_value_size: Option<usize>,     // default 1 MiB
    pub transaction_mode: TransactionCacheMode,
    pub failure_mode: CacheFailureMode,
    pub tags: Vec<CacheTag>,
    pub key_prefix: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionCacheMode {
    Bypass,         // default, safest
    Defer,          // collect invalidations, apply on commit
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheFailureMode {
    FailOpen,       // log and continue, default
    FailClosed,     // propagate error, break the query
}
```

`CacheKey` is planned:

```rust
// planned: rbatis-plus-core/src/key.rs
pub struct CacheKey { /* opaque bytes */ }

pub struct CacheKeyBuilder {
    pub version: u8,                       // bumps on every wire-incompatible change
    pub namespace: String,
    pub datasource_id: String,
    pub driver: &'static str,
    pub key_prefix: Option<String>,
    pub hasher: KeyHasher,                 // planned: blake3 for memory, xxhash for fast paths
}

impl CacheKeyBuilder {
    pub fn build(&self, sql: &str, args: &[Value], ctx: &LifecycleContext) -> CacheKey;
}
```

### 3.5 RBatis-Plus-Mem: backend

```rust
// planned: rbatis-plus-mem/src/lib.rs
pub struct MemoryCacheStore {
    values: moka::future::Cache<CacheKey, Arc<Value>>,
    tag_versions: dashmap::DashMap<CacheTag, AtomicU64>,
    clock: Clock,                          // injectable for tests
    metrics: Arc<dyn MetricsRecorder>,
}

impl MemoryCacheStore {
    pub fn builder() -> MemoryCacheStoreBuilder;   // planned
}

#[async_trait]
impl CacheStore for MemoryCacheStore {
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;
    async fn set(&self, key: CacheKey, value: Value, policy: &CachePolicy) -> Result<(), CacheError>;
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;
    async fn clear_namespace(&self, ns: &str) -> Result<u64, CacheError>;
}
```

Dependency (planned): `moka = { version = "0.12", features = ["future"] }`,
`dashmap = "6"`, `blake3 = "1"`. Final versions are TBD and will be pinned in
the workspace `Cargo.toml` when the crate is implemented.

### 3.6 RBatis-Plus-Redis: backend

```rust
// planned: rbatis-plus-redis/src/lib.rs
pub struct RedisCacheStore {
    client: redis::Client,                  // planned: redis-rs async
    publisher: redis::aio::PubSub,
    config: RedisCacheConfig,               // planned
    metrics: Arc<dyn MetricsRecorder>,
    singleflight: SingleFlight,             // planned
}

#[async_trait]
impl CacheStore for RedisCacheStore { /* planned */ }
```

Features (planned): `default = ["tokio-comp"]`, optional
`"connection-manager"`, `"cluster"`, `"sentinel"` are not in scope for 0.1.0
and marked TBD.

Wire format (planned): an envelope with `version`, `codec`, `payload`. Codec
choices planned: `RbsJson` (default, uses `rbs::to_vec`), `RbsMsgPack`
optional (TBD behind feature `msgpack`).

The Redis implementation lives in the optional `rbatis-plus-redis` crate; the facade `rbatis-plus` selects it through an explicit feature. The optional test-support crate is `rbatis-plus-test` (planned) and is not a runtime backend.


To match MyBatis `<cache>` ergonomics, RBatis-Plus plans attribute macros that
emit `statement_id` plus policy metadata. These are planned and live in a
separate crate `rbatis-plus-macros` (TBD).

```rust
// planned syntax (subject to change)
#[rbatis_plus::cache(
    namespace = "user.profile",
    ttl = "60s",
    tags = ["user", "profile:by_id"],
    key_by = ["id"],
    transaction_mode = "Bypass",
)]
#[py_sql("select * from user where id = #{id}")]
async fn find_user_profile(id: i64) -> rbs::Value { /* ... */ }
```

`rbatis-plus-macros` is **out of scope for this milestone's exit criteria**;
the attribute syntax is recorded here so the upstream hooks are designed to
carry the metadata.

---

## 4. Phases 0-6

Phase numbering is shared across the implementation, integration, architecture, and acceptance documents: Phase 0 is upstream foundations; Phase 1 is the external-plugin MVP; Phase 2 enriches context; Phase 3 adds transaction lifecycle; Phase 4 adds Redis; Phase 5 adds macros; Phase 6 covers observability and polish.

Each phase lists its concrete tasks, deliverables, exit criteria, and which
crates or upstream PRs advance the state.

### 4.1 Phase 0 — Upstream foundations: hooks and semantics

Purpose: land the minimum upstream surface that all later phases consume.

Planned tasks:

1. Add `ExecutorKind`, `OperationKind`, `LifecycleContext`, `CacheHint` in
   `rbatis/src/plugin/intercept/mod.rs`. No field added to `RBatis`.
2. Implement `apply_lifecycle` helper, mirroring `apply_before` / `apply_after`.
3. Add `apply_after` semantic fix behind feature flag
     `fix-after-return-semantic`, planned default **off** during the transition window;

4. Extend `Intercept::ctx() -> Option<LifecycleContext>` with a default impl
   returning `None` so existing plugins keep compiling.
5. Tests:
   - `tests/intercept_test.rs`: deterministic order, before-return short
     circuit, after-return edits propagated.
   - `tests/lifecycle_test.rs` (planned): emit events from `RBatis` directly
     even with zero listeners.
6. Documentation in `rbatis/docs/`:
   - `intercept.md`: ordering rule (rewrite interceptors first, observers
     last).
   - `lifecycle.md`: extension contract for cache and observers.

Upstream PR title (planned): "rbatis: add lifecycle hooks and fix
`apply_after` semantics".

Exit criteria:

- CI on upstream `master` passes.
- `cargo doc` produces no broken links for the new symbols.
- Existing public `Intercept` consumers compile without modification.
- New helpers covered by at least 6 unit tests.

### 4.2 Phase 1 — External plugin MVP (in RBatis-Plus)

Purpose: prove cache value without exposing it inside the upstream crate.

Planned tasks:

1. Introduce `rbatis-plus-core/` (planned) with:
   - `CachePolicy`, `CacheFailureMode`, `TransactionCacheMode`.
   - `CacheKeyBuilder` using `blake3` (planned; hash choice TBD).
   - `StaticPolicyProvider` returning a single policy.
   - `CacheIntercept` reading lifecycle from `Intercept::ctx`.
   - `MetricsRecorder` trait + `NoopMetricsRecorder`.
2. Add `rbatis-plus-mem/` (planned) with `MemoryCacheStore` using
   Moka.
3. Wire `rbatis-plus-core` and `rbatis-plus-mem` against upstream Phase 0
   hooks via a path dependency or git dependency on a branch (the exact
   dependency choice is TBD before 0.1.0).
4. Behaviour matrix tests (planned):
   - Same SQL + same args cache hit.
   - Different args no hit.
   - TTL expiry miss.
   - Cache backend down — query still succeeds (FailOpen default).
   - DML success clears namespace.
5. Documentation in `rbatis-plus-core/README.md` (planned):
   - "MVP limitations" callout: no tx-deferred invalidation; rely on TTL for
     external writes.

Exit criteria:

- All MVP tests green on CI.
- A benchmark on synthetic 1k rows shows >= 5x speedup of cached path over
  PostgreSQL loopback at identical conditions.
- Plugin works with RBatis `master` + the Phase 0 PR applied.

### 4.3 Phase 2 — Core context enrichment

Purpose: extend the `LifecycleContext` so cache invalidation can be precise.

Planned tasks:

1. Upstream: add `datasource_id`, `driver`, `statement_id`, `tenant_id`,
   `shard_value` to `LifecycleContext` and populate them where available.
2. Upstream: thread `statement_id` from `py_sql!`, `html_sql!`, `crud!`
   attribute paths through to the executor. For macros the change is metadata-
   only; no generated code signature changes.
3. RBatis-Plus: `CacheKeyBuilder` consumes the new fields. `StaticPolicyProvider`
   gains a per-statement-ID rule.
4. Tests (planned):
   - Tenant A and B produce disjoint keyspaces.
   - Shard suffix in key prevents cross-shard hits.
   - `statement_id` propagation across `py_sql!` and `html_sql!`.

Upstream PR title (planned): "rbatis: enrich lifecycle context with statement,
tenant, shard identifiers".

Exit criteria:

- No regression in the upstream CRUD macro tests.
- RBatis-Plus compiles against the enriched `LifecycleContext`.
- A migration note published for plugin authors who read
  `LifecycleContext` fields manually.

### 4.4 Phase 3 — Transaction lifecycle and deferred invalidation

Purpose: support deferred invalidation so uncommitted writes do not leak
through the cache.

Planned tasks:

1. Upstream: add `TransactionListener` trait and `RBatis::listeners` field.
   Emit `Begin`, `Commit`, `Rollback`, `SavePoint` events from
   `RBatisTxExecutor` and `RBatisTxExecutorGuard`.
2. RBatis-Plus-Core: introduce `DeferredInvalidationMap: Mutex<TxId, HashSet<CacheTag>>`
   (planned). Plumb it through `CacheIntercept` and the transaction
   listeners.
3. `TransactionCacheMode::Defer` semantics (planned):
   - In-tx reads bypass L2 by default.
   - In-tx DML collects tags.
   - On commit, tags invalidated; if commit fails, tags discarded.
   - On rollback, tags discarded.
4. Tests (planned):
   - Read inside tx never hits shared cache.
   - Commit triggers tag invalidation.
   - Rollback discards pending tags.
   - Two concurrent transactions, only their own tags fire on their commit.
   - Guard-Drop auto-rollback path also discards.

Upstream PR title (planned): "rbatis: expose transaction lifecycle for plugin
authors".

Exit criteria:

- All transaction matrix tests pass.
- `cargo bench` shows no regression for the non-cache path.
- Public docs updated with the "Bypass vs Defer" trade-off.

### 4.5 Phase 4 — Distributed backend (Redis)

Purpose: provide a production-grade cross-process backend.

Planned tasks:

1. New crate `rbatis-plus-redis` (planned):
   - `RedisCacheStore` using `redis = "0.27"` (planned; pin in
     `Cargo.lock` when implemented).
   - Envelope codec with explicit `version` and `codec` fields.
   - Tag-version keys: `version:tag:<ns>:<tag> -> u64`, incremented atomically
     on `INCR`.
   - `SingleFlight` per process key via `tokio::sync::Mutex<HashMap<CacheKey, JoinHandle>>`.
   - TTL jitter: `effective = ttl ± rand(0, jitter_max)`.
   - Pub/Sub: cache-bus channel `<prefix>.bus` publishes
     `InvalidateTags { tags, nonce }` on invalidation.
2. Configuration (planned): builder with `url`, `key_prefix`, `pub_sub`, `jitter`,
   `singleflight_capacity`.
3. Tests (planned, integration gated behind `--features redis-tests` and an
   environment variable `REDIS_URL`):
   - Cross-process invalidation via Pub/Sub.
   - `REDIS_URL` missing -> tests skipped, not failed.
4. Security (planned): TLS supported via `redis://...?tls=true` (delegated to
   `redis` crate). Auth tokens not logged.

Exit criteria:

- All Redis tests pass against `redis:7` and `redis:7-alpine` containers on
  CI (matrix gated).
- Cross-process test demonstrates invalidation from one process observed in
  another within 1s.

### 4.6 Phase 5 — Macro annotations

Purpose: bring MyBatis-style declarative ergonomics to RBatis-Plus.

Planned tasks:

1. New crate `rbatis-plus-macros` (planned). Depends on `syn`, `quote`,
   `proc-macro2`.
2. Attribute `#[rbatis_plus::cache(...)]` (planned) for `py_sql` and
   `html_sql` macro outputs. Emits:
   - A `CachePolicyProvider` returning deterministic policy keyed by statement
     ID.
   - A `statement_id` literal passed into the executor (via
     `Intercept::ctx`'s `statement_id` field).
3. Compile-fail tests (planned): duplicate keys, empty namespace, invalid
   durations.
4. Sample crate (planned): `examples/macro_annotations.rs`.

Exit criteria:

- The macro produces no extra runtime cost on the miss path (verified via
  `cargo bench`).
- All compile-fail tests pass.

> Phase 5 is **not** required for the 0.1.0 tag. It is recorded here so the
> upstream hooks carry the metadata it needs.

### 4.7 Phase 6 — Observability, ergonomics, polish

Purpose: ship-ready quality.

Planned tasks:

1. Built-in metrics (planned): `hit`, `miss`, `store_error`,
   `invalidate_by_tag`, `invalidation_pending`, `pending_commit_size`.
   Emitted via `MetricsRecorder` only; no default exporter.
2. Optional `tracing` integration behind feature `tracing` (planned).
3. CLI-friendly admin helpers (planned): `RBatisPlusAdmin::invalidate_tags`,
   `clear_namespace`, `dump_keys_for_diagnostics`.
4. Admin endpoints remain synchronous and return counts; they do not depend on
   any specific HTTP framework.
5. Documentation: every public type has a doc comment, an example in the
   `examples/` folder, and a row in the matrix table in
   `docs/ARCHITECTURE.md`.

Exit criteria:

- `cargo doc --no-deps` shows zero warnings for `rbatis-plus` crates.
- All public items have rustdoc examples that pass `cargo test --doc`.
- README passes the "5 second test": name, one-sentence why, copy-paste
  install, copy-paste runnable example.

---

## 5. Upstream PR Plan (Planned)

The PR series is staged so each PR is reviewable on its own. Approximate
order, names are working titles.

| # | Title                                                     | Touchpoint files                                     | Phase |
| - | --------------------------------------------------------- | ---------------------------------------------------- | ----- |
| 1 | rbatis: add lifecycle hooks and fix apply_after semantics  | `intercept/mod.rs`, `executor.rs`, `rbatis.rs`       | 0     |
| 2 | rbatis: enrich lifecycle context for statement, tx, shard  | `intercept/mod.rs`, `executor.rs`, macro crates       | 2     |
| 3 | rbatis: expose transaction lifecycle for plugin authors    | `intercept/mod.rs`, `executor.rs`, `rbatis.rs`       | 3     |
| 4 | rbatis: structured interceptor ordering                  | `rbatis.rs`, `plugin/mod.rs`, tests                  | 0     |

PR body template (planned):

```text
## Summary
One paragraph stating what and why.

## Scope
List of files and behaviour changes.

## Compatibility
- Public API additions only.
- New symbols behind feature flag (if any).

## Test plan
- Commands run.
- New tests added.

## Out of scope
- Backends, integrations, batteries — those live in RBatis-Plus.
```

Reviewer checklist (planned, applied to every PR):

1. Public API is additive (no symbol removed).
2. No new required dependency that depends on a runtime other than Tokio.
3. `cargo test --workspace` green.
4. Doc coverage: every new pub symbol has a doc comment.
5. Cycle tolerance: no `tokio::main` or `smol` lock-in added in core.
6. Feature flag hygiene: each feature is additive and tested both on and off.

---

## 6. Concurrency and Consistency Posture (Planned Defaults)

Recorded up-front so reviewers do not have to infer choices later.

| Concern                       | Planned default                                     |
| ----------------------------- | --------------------------------------------------- |
| Cache visibility              | Tokio task-local reads, Moka writes are atomic      |
| Miss concurrency              | Per-key `tokio::sync::Mutex` singleflight           |
| Tag invalidation              | Versioned: bump number, never scan                  |
| TTL jitter                    | +/- 10% on the configured TTL                       |
| Maximum cached value          | 1 MiB, larger values bypass silently (FailOpen)     |
| Backend unreachable           | FailOpen with WARN metric and single error log line |
| External writes               | TTL only, plus optional Redis Pub/Sub bridge        |
| Tenant isolation              | Namespace + key prefix enforced, no cross-tenant    |
| Transactional writes          | Defer until commit, discard on rollback             |
| Read-your-write in same tx    | Optional via `CacheHint::Refresh` and `Bypass` modes|
| RBatis version skew           | Compile-time pins via workspace `Cargo.toml`        |

---

## 7. Test Matrix (Mapped to Phases)

Each row is a planned test. Tests marked `(planned)` do not yet exist; tests
marked `(exists)` are in upstream today and must continue to pass.

### 7.1 Basics (Phases 1, 6)

- (planned) Same SQL + same args hit.
- (planned) Different args miss.
- (planned) Different SQL miss.
- (planned) Type-distinct values: `1`, `"1"`, `1.0`, `NULL` produce distinct keys.
- (planned) TTL expiry forces miss.
- (planned) Empty result cacheable, shorter TTL.
- (planned) `CacheHint::Bypass` always queries DB.
- (planned) `CacheHint::Refresh` overwrites on hit.
- (planned) Backend error => FailOpen default, FailClosed opt-in.

### 7.2 Interceptor ordering (Phase 0)

- (exists) Pagination changes the cache key per page.
- (planned) Tenant id changes the cache key.
- (planned) Dynamic table name changes the cache key.
- (planned) A log interceptor placed after cache observes hit-or-miss.
- (planned) Another interceptor short-circuits with `Return`; cache stays silent.

### 7.3 DML invalidation (Phases 1, 3)

- (planned) Insert success invalidates tags.
- (planned) Update success invalidates tags.
- (planned) Delete success invalidates tags.
- (planned) DML failure does not invalidate.
- (planned) `clear_namespace` removes all matching keys.
- (planned) Bulk DML merges tags.

### 7.4 Transactions (Phase 3)

- (planned) In-tx read never reads shared cache.
- (planned) Uncommitted data never enters shared cache.
- (planned) Commit triggers invalidation.
- (planned) Rollback discards pending.
- (planned) Same tx, multiple writes merge tags.
- (planned) Guard-Drop rollback path discards.
- (planned) Commit failure does not invalidate.

### 7.5 Concurrency (Phases 1, 4)

- (planned) Concurrent misses on one key produce exactly one DB hit.
- (planned) High QPS get/set does not panic or deadlock.
- (planned) Race between set and invalidate leaves no orphaned value.
- (planned) Redis connection drop transitions to FailOpen with metric.
- (planned) Memory-pressure eviction coexists with tag invalidation.

### 7.6 Compatibility

- (exists) Upstream `tests/intercept_test.rs` remains green.
- (exists) Upstream CRUD macro tests remain green.
- (planned) An external RBatis user with no cache plugin is binary-compatible.

### 7.7 Macro string-heuristic regression tests

These tests guard against silent drift in the macro-driven strings and the
executor-side token recognition.

- (planned) Return-token contract: the macro-driven return token for a
  query path always contains the `ExecResult` discriminator when a
  `_test_exec_marker` is present; the executor routes to
  `apply_before_exec` not `apply_before_query`. Locked via golden test.
- (planned) Type-token contract: the executor resolves the path from the
  type token string (`"ExecResult"` vs `"Value"`); `CacheIntercept` must
  observe the same path it would have under direct `Executor::query` /
  `Executor::exec`. Locked via golden test.
- (planned) Pin the spelling of both token strings in a `static` assertion
  so a silent upstream rename fails CI rather than silently changing
  cache dispatch.
- (planned) Custom `Wrapper` types (planned feature) compile against
  the current return-token shape and type-token shape. Compile-fail
  tests are added in `rbatis-plus-test/src/wrapper_compile.rs` for:
  - A wrapper that returns `rbs::Value`.
  - A wrapper that returns `ExecResult`.
  - A wrapper that returns a custom `#[derive(FromExecResult)]` enum.
  - A wrapper used in an async `Mapper` trait method.
- (planned) When the macro emits a `py_sql` query, the produced tokens
  must remain unchanged before and after Phase 0 / Phase 1 changes;
  byte-equality golden test.

### 7.8 Pagination shared-state tests

`PageIntercept` keeps shared maps keyed by executor id; cache key
construction must not collide across pages or across concurrent requests.

- (planned) `PageIntercept::before` rewrites SQL to add `LIMIT/OFFSET`;
  the rewritten SQL produces distinct cache keys for pages 1, 2, 3.
- (planned) Two concurrent requests with different page numbers do not
  share a cache entry, even when the underlying `RBatisConnExecutor`
  has the same id (the inner `page_map` is shared but the SQL differs).
- (planned) A query without `LIMIT/OFFSET` whose executor id matches a
  paginated query never hits the paginated cache entry.
- (planned) After the page interceptor rewrites SQL, the cache key
  reflects the rewritten text; verify by asserting that
  `cache_key(query, page=2)` differs from `cache_key(raw_sql)` even when
  args are identical.
- (planned) Multi-tenant + pagination: tenant A page 2 does not collide
  with tenant B page 2.

### 7.9 Cloned connection `begin` test

`RBatisConnExecutor::begin` consumes the inner connection. A naive
implementation that clones the executor first and then calls `begin` on
the stale clone must fail loudly; the planned `TransactionListener` must
attach only to the original executor.

- (planned) When a `RBatisConnExecutor` is cloned and `begin` is invoked
  on the clone, the call returns an error because the original has
  already moved the connection into a transaction (or vice versa).
- (planned) `CacheIntercept` attaches its `TransactionListener` to the
  original (non-cloned) executor; cloning does not double-register
  listeners.
- (planned) `RBatisConnExecutor::begin` on the original succeeds and
  triggers `on_begin` exactly once across the original and all clones
  sharing its `Arc<RBatis>`.
- (planned) After `begin`, `commit` and `rollback` fire their
  corresponding listener events on the same `tx_id`; cloning the
  executor and calling `commit` on the clone does not double-fire.

---

## 8. Risks and Mitigations

| Risk                                                                   | Mitigation                                                                                  |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Upstream API churn breaks RBatis-Plus                                  | Lock upstream via git tag, mirror locally; submit small additive PRs                        |
| Tag-version blowup leaves stale keys                                   | TTL bounds + namespace scan nightly (planned, not in 0.1.0)                                 |
| Custom interceptors reading `apply_after` differently                  | Feature flag + major-version notes; provide `InterceptCompat` adapter                       |
| Moka/Redis major version breakage                                      | Pin in workspace `Cargo.toml`, set `cargo-deny` rules (planned for Phase 6)                 |
| Macro proc-macro compile-time breakage                                 | Compile-fail tests in Phase 5; disable macro feature by default                             |
| Missed invalidation due to dynamic SQL without tags                    | Document and ship `FailureMode::FailOpen` with a WARN; encourage explicit tags             |
| Users expect cache-on-by-default                                       | Default off; only opt-in via `CacheIntercept::install`                                       |
| Sensitive args in cache keys                                           | Hash inputs; never log raw args; allow `key_redact = ["password"]` attribute (planned)      |
| TLS misconfiguration on Redis                                          | Validate URLs in `RedisCacheStore::builder`, reject `rediss://` without TBD TLS loader       |
| Macro string-heuristic fragility (return-token contains `ExecResult`; executor recognises via type-token string) | Do not infer query/exec dispatch from string matching; drive both branches off `LifecycleContext::operation` / `ResultType`; add regression tests that lock the token strings (see §7.7) |

---

## 9. Release and Versioning Plan (Planned)

- Workspace `version = "0.1.0"` (planned, not yet released).
- Upstream PRs are listed in section 5; RBatis-Plus 0.1.0 requires PRs 1 and
  4 merged (minimum). PRs 2 and 3 may land after 0.1.0 as 0.2.x.
- SemVer:
  - 0.1.x — Phase 0+1 stable, MVP behaviour.
  - 0.2.x — Phase 2 features land; macros still off by default.
  - 0.3.x — Phase 3 features land; production transaction semantics.
  - 0.4.x — Phase 4 features land; Redis backend GA.
  - 1.0 — Phase 5+6 lands; macros GA; full test matrix green.

---

## 10. Day-One README Plan (Planned)

The README must answer, in order:

1. What RBatis-Plus is, in one sentence.
2. Why, in two sentences (L2 cache for RBatis queries).
3. Install:
   ```bash
   # planned
   cargo add rbatis-plus rbatis-plus-mem
   ```
4. Minimal example:
   ```rust
   // planned
   use rbatis_plus::prelude::*;
   use rbatis_plus_mem::MemoryCacheStore;
   use std::sync::Arc;
   use std::time::Duration;

   let store = Arc::new(MemoryCacheStore::builder().max_capacity(50_000).build().await?);
   let policy = StaticPolicyProvider::new(CachePolicy::default()
       .with_namespace("user.profile")
       .with_ttl(Duration::from_secs(60))
       .with_tags(["user"]));
   rb.install(CacheIntercept::new(store.clone(), policy));
   ```
5. Link to ARCHITECTURE.md and IMPLEMENTATION_PLAN.md.

---

## 11. Open Questions (Tracked, Not Blocking)

1. Hash choice: blake3 vs xxhash (planned: blake3 with fallback to xxhash
   behind feature flag).
2. Whether to expose `InterceptV2` now or only via the additive default impl
   (planned: additive only).
3. Exact `Intercept::ctx` lifetime return convention (planned: `Cow`).
4. Whether `Defer` mode is on by default (planned: no, `Bypass`).
5. Naming of the meta-crate (`rbatis-plus` vs `rbatis_plus`); planned:
   `rbatis-plus` for the meta-crate to mirror MyBatis-Plus naming, kebab-case
   directories only.

---

## 12. Acceptance Criteria for 0.1.0 Tag

- Phases 0 and 1 complete; Phases 2-6 partially or fully landed per phase
  scope.
- All MVP test rows in section 7.1 pass on CI.
- Upstream PRs 1 and 4 merged into `rbatis` `master`.
- `cargo doc` clean across the workspace.
- README passes the "5 second test".
- This document and `ARCHITECTURE.md` are reviewed and merged.

---

## 13. Change Log

| Date       | Author | Change                                                                                                |
| ---------- | ------ | ----------------------------------------------------------------------------------------------------- |
| 2026-07-24 | TBD    | Initial plan drafted from `RBatis 支持二级缓存调研报告.md` baseline (`2df418feeab511c1899b2a110eef43228a1ad889`). |
| 2026-07-24 | TBD    | Updated upstream baseline to `master@4050edd3dad03a113b8bb4f5818a006f11f2da78` (`Limit decode fallback to single-column values`). Added macro string-heuristic risk, custom wrapper compile tests, pagination shared-state tests, cloned connection `begin` test. Added `rbatis-plus-test` crate to layout, dependencies, and re-export surface. Refreshed CodeGraph stats (178 files, 1,740 nodes, 17,805 edges, 192 classes, 715 functions, 655 tests; 9,366 `CALLS`, 5,524 `TESTED_BY`; 88 flows, 14 communities). |
