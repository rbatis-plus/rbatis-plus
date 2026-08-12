# RBatis-Plus 测试与验收计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 定义 RBatis-Plus 二级缓存、拦截器、可观测和运维表面的验证策略与发布门禁。

**Architecture:** 七层测试体系（静态检查 → 纯单元 → 组件 → 数据库集成 → 分布式集成 → 端到端 → 运维演练）

**Tech Stack:** Rust, cargo test, cargo bench, cargo clippy, trybuild, testcontainers-rs, redis-rs

**Related Design Doc:** `docs/superpowers/specs/2026-07-20-architecture-spec.md`

---

## 1. Evidence policy and release gates

Every claim must be supported by one of:
- a reproducible automated test
- a captured benchmark result with workload, hardware, versions, and commit
- a reviewable trace/metric assertion
- an operational drill record
- an explicitly documented assumption or limitation

A release candidate is accepted only when all mandatory layers pass, no P0/P1 defect remains, golden vectors are unchanged or deliberately versioned, and SQLite and Redis boundary tests have been run for the supported feature set.

**Commands:**
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

---

## 2. Seven test layers

### Layer 1: Static and contract checks

**Purpose:** Validate public names, feature flags, macro expansion contracts, object safety, documentation examples, error enums, and compatibility shims.

**Tasks:**
- [ ] **Step 1:** Compile target API examples with `trybuild`
- [ ] **Step 2:** Compile-fail fixtures for invalid macro attributes
- [ ] **Step 3:** Generated API inventory check
- [ ] **Step 4:** No live database or Redis required

**Evidence:** compiler output, generated API inventory, compile-fail fixtures

### Layer 2: Pure unit tests

**Purpose:** Test canonical argument encoding, SQL normalization, cache policy selection, namespace/tag versioning, TTL and jitter calculations, transaction state machines, redaction, cardinality limits, and interceptor ordering.

**Tasks:**
- [ ] **Step 1:** Policy decision branch coverage
- [ ] **Step 2:** Key injectivity property tests
- [ ] **Step 3:** TTL jitter calculation tests
- [ ] **Step 4:** Transaction state machine tests
- [ ] **Step 5:** Interceptor ordering tests

**Evidence:** branch coverage for policy decisions, property tests for key injectivity

### Layer 3: Component tests

**Purpose:** Run cache store, interceptor, serialization envelope, single-flight, and invalidation components against fakes or deterministic in-memory adapters.

**Tasks:**
- [ ] **Step 1:** Inject get/set/remove failures
- [ ] **Step 2:** Malformed envelopes
- [ ] **Step 3:** Clock jumps
- [ ] **Step 4:** Oversize values
- [ ] **Step 5:** Cancellation

**Evidence:** state transition logs and fault matrix with expected fail-open/fail-closed behavior

### Layer 4: Database integration tests

**Purpose:** Run real SQL against SQLite for supported SQL semantics and transaction boundaries.

**Tasks:**
- [ ] **Step 1:** SQLite behavioral boundary tests
- [ ] **Step 2:** Schema migration verification
- [ ] **Step 3:** SQL log capture
- [ ] **Step 4:** Transaction outcome verification
- [ ] **Step 5:** Cache state before/after verification

**Evidence:** database version, schema migration, SQL log, transaction outcome, cache state

### Layer 5: Distributed integration tests

**Purpose:** Run Redis boundary tests with a pinned Redis version.

**Tasks:**
- [ ] **Step 1:** Serialization tests
- [ ] **Step 2:** Tag-version invalidation tests
- [ ] **Step 3:** Pub/Sub delivery tests
- [ ] **Step 4:** Reconnect and timeout tests
- [ ] **Step 5:** Failover tests
- [ ] **Step 6:** ACL denial tests
- [ ] **Step 7:** Eviction tests

**Evidence:** Redis version, test commands, fault injection results

### Layer 6: End-to-end tests

**Purpose:** Full stack tests with real HTTP endpoints.

**Tasks:**
- [ ] **Step 1:** Axum integration test
- [ ] **Step 2:** Actix integration test
- [ ] **Step 3:** Cache hit/miss through HTTP
- [ ] **Step 4:** Transaction through HTTP

**Evidence:** HTTP response codes, cache state, transaction outcomes

### Layer 7: Operational drills

**Purpose:** Verify operational procedures.

**Tasks:**
- [ ] **Step 1:** Cache clear procedure
- [ ] **Step 2:** Cache warm-up procedure
- [ ] **Step 3:** Metrics export verification
- [ ] **Step 4:** Log format verification

**Evidence:** drill records, metrics output, log samples

---

## 3. Test matrix by phase

| Phase | Layer 1 | Layer 2 | Layer 3 | Layer 4 | Layer 5 | Layer 6 | Layer 7 |
|---|---|---|---|---|---|---|---|
| Phase 0 | OK | OK | — | — | — | — | — |
| Phase 1 | OK | OK | OK | OK | — | — | — |
| Phase 2 | OK | OK | OK | OK | — | — | — |
| Phase 3 | OK | OK | OK | OK | — | — | — |
| Phase 4 | OK | OK | OK | OK | OK | — | — |
| Phase 5 | OK | OK | OK | OK | OK | — | — |
| Phase 6 | OK | OK | OK | OK | OK | OK | OK |

---

## 4. Benchmark plan

### 4.1 Cache hit benchmark

```rust
// Target: cached path >= 5x speedup over PostgreSQL loopback
#[bench]
fn bench_cache_hit_1k_rows(b: &mut Bencher) {
    // Setup: populate cache with 1k rows
    // Benchmark: repeated cache hits
}
```

### 4.2 Cache miss benchmark

```rust
#[bench]
fn bench_cache_miss_1k_rows(b: &mut Bencher) {
    // Benchmark: cache miss + DB query + cache populate
}
```

### 4.3 Invalidation benchmark

```rust
#[bench]
fn bench_tag_invalidation_100_keys(b: &mut Bencher) {
    // Benchmark: invalidate 100 keys by tag
}
```

---

## 5. Golden vectors

### 5.1 Cache key golden vectors

```rust
#[test]
fn golden_cache_key_same_sql_same_args() {
    // Given: SQL "SELECT * FROM user WHERE id = ?", args [1]
    // Expected: deterministic cache key (snapshot)
}

#[test]
fn golden_cache_key_different_args() {
    // Given: SQL "SELECT * FROM user WHERE id = ?", args [1] vs [2]
    // Expected: different cache keys
}
```

### 5.2 Envelope golden vectors

```rust
#[test]
fn golden_envelope_encode_decode() {
    // Given: known Value
    // Expected: deterministic envelope bytes (snapshot)
}
```

---

## 6. CI matrix

| Job | Command | Trigger |
|---|---|---|
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Every push |
| Test (default) | `cargo test --workspace` | Every push |
| Test (all features) | `cargo test --workspace --all-features` | Every push |
| Doc | `cargo doc --no-deps --workspace` | Every push |
| Bench | `cargo bench --workspace` | Nightly |
| Redis integration | `cargo test -p rbatis-plus-redis --features redis-tests` | Nightly (REDIS_URL required) |

---

## 7. Defect severity

| Severity | Definition | Response time |
|---|---|---|
| P0 | Data loss, security breach, production down | Fix within 4 hours |
| P1 | Feature broken, no workaround | Fix within 24 hours |
| P2 | Feature degraded, workaround exists | Fix within 1 week |
| P3 | Cosmetic, documentation | Fix within 1 month |

---

## 8. Release checklist

- [ ] All mandatory test layers pass
- [ ] No P0/P1 defects open
- [ ] Golden vectors unchanged or deliberately versioned
- [ ] `cargo doc` clean across workspace
- [ ] README passes the "5 second test"
- [ ] CHANGELOG updated
- [ ] Version bumped in workspace `Cargo.toml`
- [ ] Git tag created
- [ ] crates.io publish dry-run succeeds
