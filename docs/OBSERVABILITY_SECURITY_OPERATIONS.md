# Observability, Security, and Operations

**Status:** approved-plan gate template  
**Date:** 2026-07-24  
**Scope:** target RBatis Plus cache and interceptor operations  
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
| 7 | [`docs/INTEGRATION_GUIDE.md`](./INTEGRATION_GUIDE.md) | Integration gate template |
| 8 | `docs/OBSERVABILITY_SECURITY_OPERATIONS.md` | This document; observability / security / ops |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](./TEST_AND_ACCEPTANCE_PLAN.md) | Acceptance plan |

> The repository currently has no Cargo workspace or implementation. Configuration names, commands, and APIs below are target interfaces and operational templates, not implemented facts.

## 1. Observability contract

Observability must explain whether a request used the cache, why it bypassed it, whether invalidation completed, and whether the database remained authoritative. Instrumentation must never require logging query arguments or sensitive values.

### Metrics

Recommended low-cardinality metrics:

- `rbatis_cache_requests_total{operation, outcome}` where `operation` is `get|set|invalidate` and `outcome` is `hit|miss|bypass|error|success`.
- `rbatis_cache_latency_seconds{operation, outcome}` as a histogram.
- `rbatis_cache_db_queries_total{operation}`.
- `rbatis_cache_invalidation_total{scope, outcome}` where `scope` is `tag|namespace|generation`.
- `rbatis_cache_invalidation_lag_seconds`.
- `rbatis_cache_backend_errors_total{operation, error_class}` where `error_class` is a bounded enum.
- `rbatis_cache_entries` and `rbatis_cache_bytes` only for bounded backend labels.
- `rbatis_cache_singleflight_waiters` and `rbatis_cache_fill_total{outcome}`.
- `rbatis_cache_stale_guard_rejections_total{reason}`.

Do not label metrics with raw SQL, SQL hash, cache key, user ID, tenant ID, table name, statement text, exception text, Redis address, or arbitrary namespace. If per-tenant visibility is required, use an approved bounded tenant class or separate account-level telemetry. Keep operation and outcome vocabularies finite.

### Tracing

Create a span around the cache decision and, on a miss, a child span around the backend and database fill. Recommended attributes are bounded booleans/enums:

- `rbatis.cache.enabled`
- `rbatis.cache.outcome`
- `rbatis.cache.backend`
- `rbatis.cache.transaction_mode`
- `rbatis.cache.invalidation_scope`
- `rbatis.cache.key_version`
- `rbatis.cache.value_size_bucket`
- `rbatis.cache.failure_mode`

Never attach raw SQL, arguments, cache keys, tokens, credentials, personal data, or unbounded statement IDs. A redacted statement identifier may be used only when its vocabulary is controlled. Record a cryptographic key fingerprint only if approved, truncated, and explicitly documented as non-secret correlation data.

### Logging

Structured logs should include timestamp, service, environment, deployment version, trace ID, operation, bounded outcome, latency, and error class. Log the reason for bypass using a bounded enum such as `transaction`, `lock_query`, `policy`, `oversize`, `disabled`, or `backend_error`. Sampling must preserve all errors and a small proportion of hits/misses.

## 2. Threat model

### Assets

- database confidentiality, integrity, and authoritative state;
- cached query results and serialized envelopes;
- tenant and routing isolation;
- Redis credentials and network access;
- invalidation channels and generation metadata;
- trace and log data.

### Trust boundaries

1. Application process to database.
2. Application process to local memory store.
3. Application process to Redis.
4. Redis Pub/Sub or durable invalidation transport.
5. Administrative flush and deployment tooling.
6. External writers that bypass RBatis Plus.

### Threats and controls

| Threat | Control | Evidence |
|---|---|---|
| Cross-tenant cache collision | Include tenant/routing context in key; test isolation | KV-04, acceptance A3 |
| Sensitive data in keys/logs | Typed canonical encoding plus redaction; never log raw key | KV-07, log scan |
| Cache poisoning | Authenticate backend, validate envelope/version, bound size, reject malformed payload | fault test |
| Stale reads after write | Commit-aware invalidation, generation checks, short TTL, external-writer strategy | A5, A10 |
| Replay or forged invalidation | TLS/authenticated transport, ACLs, bounded command permissions, generation monotonicity | Redis security test |
| Denial of service by huge values | Maximum value size, bounded memory, admission policy, timeouts | capacity test |
| Cache stampede | Single-flight, jitter, optional distributed lock, rate limits | concurrency test |
| Partial invalidation | Namespace/generation fallback and reconciliation | fault drill |
| Credential exposure | Secret manager, environment redaction, rotation procedure | deployment review |
| Unsafe macro metadata | Validate namespace/tags/TTL; reject dynamic unbounded labels | compile/runtime tests |
| Lock-query caching | Bypass `FOR UPDATE`, `FOR SHARE`, and explicit hints | A3/A9 |
| Non-deterministic query caching | Default deny for time/random/side-effect functions; explicit opt-in | policy tests |

The cache is never the authorization boundary. Authorization and tenant filtering must remain enforced by the database/query path. A cache hit must be safe only because the key includes every context that changes authorization or result visibility.

## 3. Target configuration

Configuration is illustrative and must be finalized with the implementation.

```toml
[cache]
enabled = false
backend = "memory"                 # memory | redis
namespace = "rbatis-plus"
key_version = 1
failure_mode = "fail_open"         # fail_open | fail_closed
transaction_mode = "bypass"        # bypass | pending_invalidation
max_value_bytes = 1048576
cache_null = true
null_ttl_seconds = 5
ttl_seconds = 60
ttl_jitter_seconds = 5

[cache.redis]
url = "rediss://cache.example.invalid/0"
connect_timeout_ms = 200
operation_timeout_ms = 100
pubsub_channel = "rbatis-plus:invalidate:v1"
tls_required = true

[observability]
log_raw_sql = false
log_cache_keys = false
metrics_enabled = true
tracing_enabled = true
```

Secrets must be injected through a secret manager or protected runtime environment. Do not commit credentials or use query strings for secrets.

## 4. Target API and macro examples

These examples define intended shape only.

```rust
let policy = CachePolicy::new("activity")
    .ttl(Duration::from_secs(60))
    .tags(["biz_activity"])
    .cache_null(true)
    .failure_mode(CacheFailureMode::FailOpen);

rb.register_intercept(CacheIntercept::new(store, policy_provider));
```

```rust
#[py_sql(
    cache_namespace = "activity",
    cache_ttl = "60s",
    cache_tags = ["biz_activity"]
)]
async fn select_activity(executor: &dyn Executor, id: i64) -> Result<Activity, Error>;
```

Macros should emit bounded metadata such as `statement_id`, namespace, tags, TTL, and bypass hints. They must not implement a second cache path. Raw executor queries remain supported and use runtime policy.

## 5. Interceptor ordering

Ordering is a correctness boundary. SQL-rewriting interceptors must run before cache key construction; logging and metrics should observe the final decision. A target chain is:

```text
Tenant / routing / dynamic table / page
  -> cache decision and key construction
  -> audit / log / metrics
  -> database executor
```

Document whether “before” order is left-to-right and whether “after” order is reverse. Add a contract test for both. A cache placed before pagination can collide pages; a cache placed before tenant or routing context can leak data. If another interceptor returns early, CacheIntercept must not falsely record a database fill or write a result it did not own.

## 6. Plus feature interactions

- **CRUD macros:** use the common executor path; macro metadata is optional and must not bypass runtime policy.
- **`py_sql!` and `html_sql!`:** statement identity and final rewritten SQL must be included in policy/key decisions.
- **Pagination:** page size, offset/page number, ordering, and rewritten limit must distinguish results.
- **Dynamic tables:** physical table identity must be part of routing context or namespace.
- **Tenant interceptors:** tenant identity or an equivalent isolation generation must be included; never rely on process-local state.
- **Raw queries:** default to conservative policy when statement identity, determinism, or dependencies are unknown.
- **Locking queries:** bypass shared cache.
- **Batch writes:** invalidate after successful execution; aggregate tags where possible.
- **Transactions:** default bypass for reads and defer invalidation until commit in the native mode.

## 7. MyBatis concept mapping

| MyBatis concept | Target RBatis Plus mapping | Important difference |
|---|---|---|
| `Cache` / namespace cache | `CacheStore` plus namespace policy | Backend is an explicit SPI, not assumed built-in |
| `useCache` | query policy or macro `cache_*` metadata | Runtime safety rules can still bypass |
| `flushCache` | tag/namespace generation invalidation | Commit timing must be explicit |
| `localCacheScope` | executor/transaction-local behavior | L1 and shared L2 must not be conflated |
| mapped statement ID | bounded `statement_id` in context | Raw SQL may have no stable ID |
| `Executor` | `Executor` plus interceptors | Interceptor ordering and lifecycle are explicit |
| transaction manager | transaction events and pending invalidations | Commit/rollback hooks are required for native consistency |

This mapping is conceptual. It does not promise identical MyBatis semantics.

## 8. Runbooks

### 8.1 Cache backend outage

1. Confirm `rbatis_cache_backend_errors_total` by bounded error class.
2. Check timeout and connection saturation; do not dump keys or payloads.
3. Verify fail-open requests still reach the database and database capacity is safe.
4. If database saturation is increasing, disable cache writes or apply a documented rate limit; do not enable fail-closed without approval.
5. Restore backend connectivity and verify get/set with a synthetic non-sensitive key.
6. Compare hit ratio, latency, error rate, and DB load before declaring recovery.

### 8.2 Stale-data report

1. Capture trace ID, deployment version, bounded namespace, and timestamp.
2. Determine whether the writer was RBatis Plus or external.
3. Check commit event, invalidation generation, invalidation lag, and TTL.
4. Flush the affected namespace or advance its generation using the approved administrative command.
5. If external writes lack signaling, activate the documented mitigation and open a consistency follow-up.
6. Preserve evidence; do not manually edit Redis payloads.

### 8.3 Suspected tenant isolation issue

1. Disable the affected cache namespace immediately.
2. Preserve metrics and traces while preventing raw payload exposure.
3. Compare key-version, routing-context, and policy changes between deployments.
4. Flush all affected generations and rotate exposed credentials if backend access is suspected.
5. Treat as a security incident; perform access-log and authorization review.

### 8.4 Stampede or memory growth

1. Inspect hot-key, fill, waiter, value-size, and eviction metrics.
2. Reduce TTL refresh concurrency or enable single-flight according to policy.
3. Apply bounded admission/max-value settings; do not increase memory blindly.
4. Add TTL jitter and temporarily bypass pathological namespaces.
5. Re-run the representative load test before restoring normal policy.

### 8.5 Invalidation channel loss

1. Check subscriber health and delivery lag.
2. Determine whether the transport is lossy Pub/Sub or durable.
3. Reconcile generations from the authoritative store or run a namespace flush.
4. Restart subscribers only through the controlled deployment procedure.
5. Record the maximum possible stale-read window and notify service owners.

## 9. Deployment and rollback

### Deployment

1. Ship code with cache disabled by default.
2. Validate schema/key/envelope version compatibility in staging.
3. Enable metrics and traces before enabling reads.
4. Enable read-through for one low-risk namespace with short TTL.
5. Observe DB load, hit ratio, stale reports, error rate, and invalidation lag.
6. Gradually expand namespaces and traffic; retain a kill switch.

### Rollback

1. Disable cache reads and writes through configuration.
2. Keep database path healthy and verify capacity.
3. Flush or advance generations only if stale or poisoned data is possible.
4. Roll back the binary only after confirming the previous version understands persisted envelope/key versions, or after an explicit flush.
5. Preserve dashboards, traces, and deployment metadata for review.

Key/envelope changes require one of: backward-compatible reader, dual-read migration, or namespace version bump and flush. Never reuse a version for incompatible encoding.

## 10. Conservative-to-native transaction migration

### Conservative mode

Use while transaction lifecycle hooks are unavailable:

- bypass L2 reads and writes inside transactions;
- after successful transaction DML, clear the configured namespace conservatively;
- document that rollback can cause unnecessary invalidation and lower hit ratio;
- rely on short TTL or external invalidation for writes outside the process.

This mode prioritizes avoiding uncommitted reads over precision.

### Native mode

Migrate when explicit executor context and lifecycle events exist:

1. identify `ExecutorKind::Transaction` and `transaction_id` without string matching;
2. collect pending tags per transaction;
3. keep transaction query results out of shared L2 by default;
4. apply merged invalidations only after confirmed commit;
5. discard pending invalidations after rollback;
6. treat commit failure as not committed and do not invalidate until outcome is known;
7. make guard/drop and automatic transaction paths emit the same lifecycle events;
8. retain conservative namespace flush as a recovery fallback.

Acceptance requires commit, rollback, commit failure, nested/guarded transaction, cancellation, and process-crash evidence.

## 11. Operational ownership and review cadence

Service owners own namespace policy, TTL, tags, tenant/routing context, and external-writer inventory. Platform owners own Redis availability, credentials, dashboards, alerts, and drills. Security owners review key/log redaction, ACLs, TLS, and isolation assumptions.

Review every release that changes key version, serialization, interceptor ordering, transaction behavior, macro metadata, or failure mode. Re-run the acceptance plan at [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md).
