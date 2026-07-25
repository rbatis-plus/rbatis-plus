# Test and Acceptance Plan

**Status:** approved-plan gate template  
**Date:** 2026-07-24  
**Scope:** target RBatis Plus cache, interception, observability, and operations surfaces  
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
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](./OBSERVABILITY_SECURITY_OPERATIONS.md) | Observability / security / ops |
| 9 | `docs/TEST_AND_ACCEPTANCE_PLAN.md` | This document; acceptance plan |

> This document describes verification for a future implementation. The repository currently has no Cargo workspace or implementation code. Commands are templates to be enabled when the workspace exists; API names and examples are target contracts, not claims about shipped behavior.

## 1. Evidence policy and release gates

Every claim must be supported by one of:

- a reproducible automated test;
- a captured benchmark result with workload, hardware, versions, and commit;
- a reviewable trace/metric assertion;
- an operational drill record;
- an explicitly documented assumption or limitation.

A release candidate is accepted only when all mandatory layers pass, no P0/P1 defect remains, golden vectors are unchanged or deliberately versioned, and SQLite and Redis boundary tests have been run for the supported feature set.

Suggested future commands:

```bash
cargo test --workspace
cargo test --workspace --all-features
cargo test -p rbatis-plus --test acceptance
cargo test -p rbatis-plus --test golden_vectors
cargo test -p rbatis-plus-mem --test boundary
cargo test -p rbatis-plus-redis --test boundary
cargo bench --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## 2. Seven test layers

### Layer 1: static and contract checks

Validate public names, feature flags, macro expansion contracts, object safety, documentation examples, error enums, and compatibility shims. Compile target API examples with `trybuild` or equivalent. This layer must not require a live database or Redis instance.

Evidence: compiler output, generated API inventory, and compile-fail fixtures for invalid macro attributes.

### Layer 2: pure unit tests

Test canonical argument encoding, SQL normalization, cache policy selection, namespace/tag versioning, TTL and jitter calculations, transaction state machines, redaction, cardinality limits, and interceptor ordering as deterministic functions.

Evidence: branch coverage for policy decisions and property tests for key injectivity.

### Layer 3: component tests

Run the cache store, interceptor, serialization envelope, single-flight, and invalidation components against fakes or deterministic in-memory adapters. Inject get/set/remove failures, malformed envelopes, clock jumps, oversize values, and cancellation.

Evidence: state transition logs and fault matrix with expected fail-open/fail-closed behavior.

### Layer 4: database integration tests

Run real SQL against SQLite for supported SQL semantics and transaction boundaries. SQLite is a behavioral boundary, not proof of PostgreSQL/MySQL dialect compatibility. Use target database containers or CI services for dialect-specific SQL, locking, isolation, and driver behavior.

Evidence: database version, schema migration, SQL log, transaction outcome, and cache state before/after.

### Layer 5: distributed integration tests

Run Redis boundary tests with a pinned Redis version. Cover serialization, tag-version invalidation, Pub/Sub delivery, reconnect, timeout, failover, ACL denial, and eviction. Test network faults using a proxy or controlled fault injector rather than assuming localhost behavior represents production.

Evidence: Redis configuration, topology, injected fault, delivery latency, and residual stale-key observations.

### Layer 6: end-to-end acceptance tests

Exercise target APIs through CRUD, `py_sql!`, `html_sql!`, raw executor queries, pagination, tenant/dynamic-table rewriting, transactions, and Plus features. Verify the complete path from macro or caller through interceptor chain, store, decode, metrics, and trace.

Evidence: Given/When/Then result, trace ID, metric snapshot, database-call count, and cache contents.

### Layer 7: concurrency, fault, performance, and operational tests

Run controlled load, race, cancellation, restart, rollback, deployment, rollback, and recovery scenarios. Record p50/p95/p99 latency, throughput, DB amplification, hit ratio, error rate, stale-read window, memory, Redis command rate, and invalidation lag.

Evidence: versioned benchmark manifest, raw results, dashboards, runbook execution record, and pass/fail thresholds.

## 3. Given/When/Then acceptance cases

### A1: identical query hit

- **Given** a cacheable query with identical final SQL, typed arguments, namespace, datasource, tenant, and shard context
- **When** the query is executed twice before TTL expiry
- **Then** the first execution reads the database and stores an owned `rbs::Value`; the second returns the same logical value without a database read, emits one miss and one hit, and continues normal decode behavior.

### A2: typed argument separation

- **Given** values `1`, `"1"`, `1.0`, and `NULL` in otherwise identical queries
- **When** keys are generated
- **Then** all keys differ, remain stable across processes, and contain no raw sensitive argument.

### A3: rewritten SQL and interceptor order

- **Given** pagination, tenant, and dynamic-table interceptors rewrite a query
- **When** CacheIntercept builds the key
- **Then** it observes the final SQL and complete routing context; different pages, tenants, or tables cannot collide.

### A4: DML success and failure

- **Given** a populated namespace
- **When** a non-transactional insert, update, or delete succeeds
- **Then** the configured tag or namespace is invalidated after success. When the DML fails, no success-only invalidation is recorded unless the explicitly selected conservative policy says otherwise.

### A5: transaction commit

- **Given** a transaction that reads and writes cache-tagged data
- **When** it commits successfully
- **Then** transaction queries do not expose uncommitted values to shared L2, pending invalidations are applied once, and post-commit reads cannot use pre-write entries.

### A6: transaction rollback

- **Given** pending invalidations collected in a transaction
- **When** rollback succeeds
- **Then** pending invalidations are discarded and pre-transaction cache entries remain available, subject to normal TTL.

### A7: cache outage

- **Given** the cache backend times out or refuses a connection
- **When** a cacheable query executes under `FailOpen`
- **Then** the database remains authoritative, the request succeeds if the database succeeds, and a low-cardinality cache error metric and trace event are emitted. Under `FailClosed`, the documented error is returned.

### A8: concurrent miss

- **Given** N concurrent requests for one hot key
- **When** the key is absent
- **Then** single-flight policy bounds database fills as configured, all callers receive equivalent values, cancellation does not strand the lock, and stale or older fills cannot overwrite a newer invalidation generation.

### A9: decode compatibility

- **Given** a cached raw `Value`
- **When** callers request compatible target types through CRUD, `py_sql!`, `html_sql!`, and raw query APIs
- **Then** each follows the normal decode path; incompatible decode returns the same class of error as a database result.

### A10: external write boundary

- **Given** a database write performed outside RBatis Plus
- **When** no invalidation signal reaches the cache
- **Then** the test records the stale-read possibility and verifies the configured mitigation: short TTL, broadcast, CDC, manual flush, or namespace versioning. The system must not claim strong consistency without one.

## 4. Golden vectors

Golden vectors are versioned fixtures. Any change requires a migration note and a new key/envelope version; silently changing a vector is a release blocker.

| Vector | Input | Required assertion |
|---|---|---|
| KV-01 | driver `sqlite`, SQL `SELECT ... WHERE id = ?`, typed integer `1` | deterministic key digest |
| KV-02 | same SQL, string `"1"` | differs from KV-01 |
| KV-03 | same query, `NULL` | differs from KV-01 and KV-02 |
| KV-04 | same query, tenant `a` vs `b` | differs |
| KV-05 | page 1 vs page 2 after rewrite | differs |
| KV-06 | quoted/case-normalized equivalent SQL | follows documented normalization rule |
| KV-07 | sensitive argument | digest contains no plaintext secret |
| KV-08 | envelope version 1 payload | decodes exactly; unsupported version fails explicitly |
| KV-09 | tag generation 41 to 42 | old generation cannot satisfy new key |
| KV-10 | empty result with null-cache policy | stores only when enabled and uses null TTL |

Fixtures should include canonical bytes, digest algorithm, schema version, codec, and expected error for invalid data.

## 5. SQLite and Redis boundaries

SQLite tests prove local transaction ordering, rollback behavior, basic SQL, and deterministic test setup. They do not prove server isolation, row-lock semantics, network failures, Redis behavior, or every dialect's SQL rewriting.

Redis tests prove the selected Redis protocol and configured commands. They must not infer that Pub/Sub is durable: a subscriber outage can lose an invalidation. Stronger guarantees require a durable stream, CDC, or a rebuild-on-start protocol. Test both standalone Redis and the topology actually supported for production; document cluster hash-tag requirements if multi-key operations are used.

## 6. Methodology for hard cases

### Concurrency

Use deterministic barriers to create races between get/miss/fill, invalidation/fill, commit/fill, cancellation, and eviction. Repeat randomized schedules with a fixed seed. Assert linearization or documented eventual-consistency boundaries, not merely final values.

### Fault injection

Inject timeout, connection reset, malformed payload, partial Pub/Sub delivery, clock skew, process crash after DB commit and before invalidation, process crash after invalidation and before response, and Redis restart. Record whether the expected result is fail-open, fail-closed, retry, bypass, or manual recovery.

### Performance

Benchmark cold, warm, mixed, expired, hot-key, high-cardinality, large-value, and backend-outage workloads. Report environment and workload parameters. Compare database-only baseline against cache-enabled runs. Acceptance thresholds must be set per deployment; do not present local benchmark numbers as product guarantees.

## 7. Acceptance evidence checklist

- [ ] All seven layers pass.
- [ ] Golden vectors reviewed.
- [ ] SQLite and Redis boundaries explicitly recorded.
- [ ] Transaction commit, rollback, and commit-failure evidence attached.
- [ ] Concurrency and fault schedules reproducible.
- [ ] Performance baseline and regression threshold approved.
- [ ] Metrics, traces, alerts, runbooks, deployment, and rollback tested.
- [ ] Documentation examples remain marked as target APIs until implementation exists.
