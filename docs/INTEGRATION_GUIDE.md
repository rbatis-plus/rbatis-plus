# Integration Guide

**Status:** approved-plan gate template  
**Date:** 2026-07-24  
**Scope:** target integration of RBatis Plus cache and interceptor capabilities  
**Upstream baseline (current):** RBatis `master@4050edd3dad03a113b8bb4f5818a006f11f2da78`  
**Upstream baseline (previous):** RBatis `master@2df418feeab511c1899b2a110eef43228a1ad889`  
**Latest surveyed commit title:** `Limit decode fallback to single-column values`

## 0. Documentation Index

| # | Document | Role |
| - | --- | --- |
| 0 | [`/README.md`](../README.md) | Project entry, Mermaid diagram, doc index |
| 1 | [`docs/RBatis 支持二级缓存调研报告.md`](./RBatis%20支持二级缓存调研报告.md) | Upstream evidence baseline |
| 2 | [`docs/ARCHITECTURE.md`](./ARCHITECTURE.md) | Planned layering and components |
| 3 | [`docs/IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) | Phased plan, PR series, test matrix |
| 4 | [`docs/CACHE_SPECIFICATION.md`](./CACHE_SPECIFICATION.md) | Wire protocol |
| 5 | [`docs/TRANSACTION_CONSISTENCY.md`](./TRANSACTION_CONSISTENCY.md) | Transaction semantics |
| 6 | [`docs/DECISIONS.md`](./DECISIONS.md) | ADRs |
| 7 | `docs/INTEGRATION_GUIDE.md` | This document; integration gate template |
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](./OBSERVABILITY_SECURITY_OPERATIONS.md) | Observability / security / ops |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](./TEST_AND_ACCEPTANCE_PLAN.md) | Acceptance plan |

> The repository currently has no Cargo workspace or implementation code. The commands and Rust snippets below are future gate templates and target APIs. They are not evidence that these crates, traits, macros, or methods currently compile.

## 1. Integration goals

This guide defines how a future implementation should be introduced without silently changing database semantics. The recommended architecture keeps cache contracts in the core crate, concrete stores in separate crates, and policy metadata optional at the macro layer.

Target crate split:

```text
rbatis-plus              facade and re-exports (optional features select backends)
rbatis-plus-core         policy, SPI, interceptor, and shared protocols
rbatis-plus-mem          in-process store (optional)
rbatis-plus-redis        Redis store and invalidation transport (optional)
rbatis-plus-macros       declarative metadata (optional, planned)
```

The cache is cache-aside: the database remains authoritative. Cache errors, stale boundaries, transaction behavior, and external writers must be visible in configuration and documentation.

## 2. Prerequisites and future workspace gates

When the workspace exists, establish:

```bash
cargo metadata --workspace --no-deps
cargo test --workspace --all-features
cargo test -p rbatis-plus --test integration
cargo test -p rbatis-plus-mem --test integration
cargo test -p rbatis-plus-redis --test integration
```

Future CI should pin Rust, database images, Redis version, feature flags, and benchmark environment. Do not copy these commands into a current build script: there is no workspace in this repository yet.

## 3. Core integration model

The target flow is:

```text
CRUD / py_sql! / html_sql! / raw query
  -> SQL-rewriting interceptors
  -> CacheIntercept
  -> observability interceptors
  -> Executor / database
  -> normal decode<T>
```

The cache should store owned raw `rbs::Value`, not an application-specific `T`. On a hit, the normal decode path remains responsible for converting the value to the caller's type. This avoids separate cache implementations for CRUD, dynamic SQL, and raw queries.

Target SPI shape:

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
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;
    async fn clear_namespace(&self, namespace: &str) -> Result<u64, CacheError>;
}
```

Target context should expose explicit, typed information rather than infer transaction state from task IDs or executor type-name strings:

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

If compatibility requires preserving an existing interceptor trait, add a versioned context/adapter rather than changing old semantics invisibly.

## 4. First integration: in-process memory

Use memory as the first target because it removes network and serialization variables while validating policy and interceptor behavior. The store must be bounded by capacity and value size, support TTL, and expose hit/miss/eviction metrics. A mature concurrent cache implementation is preferable to a bespoke unbounded map.

Illustrative registration:

```rust
let store = MemoryCacheStore::builder()
    .max_capacity(10_000)
    .build();

let policies = StaticPolicyProvider::new()
    .namespace("activity")
    .ttl(Duration::from_secs(60));

rb.register_intercept(CacheIntercept::new(store, policies));
```

Before enabling it, prove A1-A9 in [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md). Memory scope is process-local; it is not a cross-process consistency mechanism.

## 5. Redis integration

Redis is an optional distributed backend. Recommended cache-aside flow:

```text
GET key
  miss -> database query -> SET envelope with TTL
successful write/commit -> advance tag generation and publish invalidation
```

Use a versioned envelope with codec, creation time, expiry, and payload. Validate the envelope before returning a value. Configure bounded operation timeouts, TLS, authentication, ACLs, maximum value size, and a documented failure mode.

Target configuration example:

```toml
[cache]
enabled = true
backend = "redis"
namespace = "orders"
key_version = 1
failure_mode = "fail_open"
transaction_mode = "bypass"        # bypass | defer
ttl_seconds = 60
max_value_bytes = 1048576

[cache.redis]
url = "rediss://cache.example.invalid/0"
connect_timeout_ms = 200
operation_timeout_ms = 100
tls_required = true
pubsub_channel = "rbatis-plus:invalidate:v1"
```

Redis Pub/Sub is not durable. If subscribers can be unavailable, pair it with generation reconciliation, a durable stream, CDC, or a recovery flush. Test reconnect, missed messages, duplicate messages, out-of-order delivery, and Redis restart.

## 6. Key construction and policy

A key must include the final rewritten SQL, typed canonical arguments, cache schema version, namespace, datasource/driver identity, and every result-changing context such as tenant, shard, locale, or authorization generation. Use a stable encoding and digest; do not use debug formatting.

Target conceptual format:

```text
rbatis-plus:l2:k1:{namespace-token}:{digest}
```

The digest input should preserve type distinctions. Never place raw secrets or personal data in the key. Add a golden vector before changing normalization or encoding. See the vectors in [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md).

Default policy should bypass:

- transaction queries until native commit-aware behavior is enabled;
- `FOR UPDATE`, `FOR SHARE`, and lock-sensitive statements;
- explicit bypass hints;
- non-deterministic or side-effecting queries;
- unknown or oversize results;
- queries whose tenant/routing context cannot be proven complete.

Empty results may be cached with a shorter TTL to reduce penetration, but this must be explicit and bounded.

## 7. Interceptor ordering and integration checks

Install SQL-rewriting interceptors before CacheIntercept:

```text
Tenant -> DynamicTable -> Page -> Cache -> Log/Metrics -> Database
```

The exact registration API is a target contract; the required semantic property is that CacheIntercept sees the final SQL and all key context. Add tests proving:

1. page 1 and page 2 produce different keys;
2. tenant A and tenant B cannot hit each other's values;
3. dynamic physical tables cannot collide;
4. logs/metrics distinguish hit, miss, bypass, and backend error;
5. an earlier `Return` does not trigger an incorrect cache write;
6. `after` always cleans pending state on success, failure, cancellation, and short circuit.

Document before/after traversal order. If after hooks run in reverse order, test that behavior rather than relying on intuition.

## 8. Plus feature integration

### CRUD macros

CRUD methods should continue to receive an executor and reach the common query/exec path. Cache metadata may be generated by the macro, but the interceptor remains the single execution implementation.

### `py_sql!` and `html_sql!`

Both macro families need a stable optional statement ID and policy metadata. Dynamic SQL must still key on final SQL and typed arguments. If a statement cannot produce bounded metadata, use conservative runtime policy.

Illustrative target metadata:

```rust
#[html_sql(
    cache_namespace = "activity",
    cache_ttl = "30s",
    cache_tags = ["biz_activity"]
)]
async fn list_activity(executor: &dyn Executor) -> Result<Vec<Activity>, Error>;
```

### Pagination

Include page number or offset, page size, ordering, and final rewritten limits. Invalidate list tags on writes. Do not assume a row-level update can safely preserve every list page.

### Transactions

The conservative mode bypasses shared L2 in transactions and may clear a namespace after successful DML. Native mode requires explicit begin, commit-success, rollback, transaction ID, and pending invalidation events. A transaction guard or automatic transaction helper must emit the same lifecycle events as explicit transactions.

## 9. MyBatis concept mapping for adopters

| MyBatis usage | RBatis Plus target integration |
|---|---|
| Namespace cache | Configure a namespace policy and `CacheStore` |
| `useCache="false"` | Per-query bypass policy or macro metadata |
| `flushCache="true"` | Tag/namespace invalidation after successful write or commit |
| Mapped statement ID | `statement_id` in typed intercept context |
| Executor interceptor | RBatis interceptor chain |
| Local session cache | Separate executor/transaction-local policy; do not confuse with L2 |
| Transaction manager | Lifecycle events consumed by pending invalidation |

Migration must preserve the semantic distinction between a local first-level cache, prepared-statement cache, and shared query-result cache. Prepared statements cache handles/plans; this integration caches result values and has different invalidation and security requirements.

## 10. Conservative-to-native transaction migration

### Phase 1: conservative

Enable only cache-aside outside transactions. In transactions, bypass L2 reads and writes. On successful transactional DML, clear the configured namespace if the product requires a conservative safety net. Document that rollback can cause unnecessary invalidation and that external writers are only covered by TTL or an external signal.

### Phase 2: context

Add typed executor kind, operation kind, statement ID, datasource ID, tenant/shard context, transaction ID, and explicit bypass hints. Keep the old interceptor adapter until downstream plugins migrate.

### Phase 3: lifecycle

Emit transaction begin, commit-success, and rollback events. Collect invalidation tags per transaction; merge duplicates; apply only after confirmed commit; discard on rollback. A failed commit must not be treated as committed. Maintain a namespace flush recovery operation.

### Phase 4: native rollout

Run both modes in shadow observation where feasible: compare proposed invalidations without changing cache state. Enable native mode per namespace, then per service. Monitor invalidation lag, stale reports, rollback behavior, and database load. Keep the conservative kill switch until the acceptance suite and incident drills pass.

## 11. Integration acceptance checklist

- [ ] Core SPI compiles in the future workspace.
- [ ] Memory backend passes unit/component tests.
- [ ] SQLite boundary tests pass and limitations are recorded.
- [ ] Redis boundary tests pass with pinned version and fault injection.
- [ ] CRUD, `py_sql!`, `html_sql!`, raw query, page, tenant, and dynamic-table paths are covered.
- [ ] Interceptor ordering is tested as a contract.
- [ ] Key golden vectors and envelope versions are reviewed.
- [ ] Metrics and traces obey low-cardinality and redaction rules in [OBSERVABILITY_SECURITY_OPERATIONS.md](./OBSERVABILITY_SECURITY_OPERATIONS.md).
- [ ] Security threat controls and backend ACL/TLS settings are verified.
- [ ] Deployment, kill switch, rollback, and namespace flush are rehearsed.
- [ ] Conservative-to-native transaction migration has explicit exit criteria.

## 12. Troubleshooting integration failures

- **Second query still hits the database:** verify final SQL, typed args, namespace, tenant/routing context, TTL, policy bypass, and interceptor order before inspecting the backend.
- **Different tenants collide:** disable the namespace, flush generations, inspect context propagation, and treat as a security issue.
- **Writes leave stale values:** determine whether the write committed, whether invalidation was deferred, and whether the writer was external.
- **Redis works locally but fails in CI:** compare TLS, ACL, topology, timeouts, Redis version, and injected network behavior.
- **Hit returns decode error:** compare cached raw `Value` envelope and normal database decode path; do not add a second type-specific decoder without a compatibility decision.
- **Load spikes on expiry:** inspect single-flight, TTL jitter, hot-key distribution, and backend timeout behavior.

All failures should produce reproducible evidence and feed back into [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md), not be resolved by undocumented configuration changes.
