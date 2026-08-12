# RBatis-Plus 全量迁移路线图

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 以 baomidou/mybatis-plus 为主蓝本、mybatis-plus-enhance 作为企业增强、mybatis-3 作为执行底层、rbatis-wrapper 已被吸收为向后兼容示例为目标，**完全迁移**为 rbatis-plus Rust workspace。

**Architecture:** 6 crate + facade workspace（rbatis-plus-core / macros / extension / sqlparser / vernal / generator）

**Tech Stack:** Rust 1.75+, rbatis (fork), serde, tokio, sqlparser-rs, proc-macro2/quote/syn, tera/handlebars/askama/maud

**Related Design Doc:** `docs/superpowers/specs/2026-07-20-architecture-spec.md`

---

## 全局约定

1. 目录/文件名 snake_case；类型 PascalCase；方法 snake_case；参数命名与 Java 一致
2. Java 子包 → Rust 同名子目录；多层嵌套只要求最后一级完全对齐
3. 注释从 Java 复制并中文化，标注对应 Java 类/方法行号
4. jackson → serde；Spring Boot → axum；Quarkus → actix；Spring 容器 → vernal；JNDI → vernal Provider
5. **一文件一对象**；禁止 lib.rs / mod.rs（除 `pub mod` 之外）/ compat.rs 堆放对象
6. annotation → 独立 proc-macro crate（rbatis-plus-macros）
7. **完全复用上游 rbatis 主接口**：`rbatis::plugin::Intercept` / `rbatis::plugin::cache::CacheStore` / `rbatis::plugin::cache::CacheIntercept` / `rbatis::plugin::transaction::TransactionListener` 等，重导出而非重写
8. 功能语义对齐；实现方式 Rust 化
9. **不动手发 PR 到 rbatis / mybatis-plus / mybatis-plus-enhance 上游**

---

## 实施阶段总览

| Stage | 内容 | 状态 | 验收 |
|-------|------|------|------|
| **iter0**（pre-flight）| 标 cached 子系统 deprecated + 整理 rbatis 主接口向上 | ✅ 部分完成 | deprecation 通过，无警告 |
| **iter1**（metadata）| TableInfo / TableFieldInfo / FieldMeta 全量 + 16 derive 拆分 | ✅ 大部分完成 | 16 derive 编译过、TableInfo 全覆盖 |
| **iter2**（conditions）| AbstractWrapper + 5 块完整 + 12 unit test | ✅ 大部分完成 | 链式 + 全部 if_* + lambda 编译 |
| **iter3**（mapper）| BaseMapper 默认 impl + 13 AbstractMethod 默认注入 | 🔀 进行中 | 9 CRUD + 14 重载字符 diff 校验 |
| **iter4**（annotation）| 16 derive 全产出 + 32 macrotest + trybuild | 🔀 进行中 | macrotest 全绿 |
| **iter5**（inner big-bang）| 6+8 inner 全量实现 | 🔀 进行中 | 14 inner × 5 unit + 5 集成 |
| **iter6**（cache 切换）| 删本地复刻 → `pub use rbatis::plugin::cache::*` | ⏳ 待开始 | 编译同型；3 backend OK |
| **iter7**（generator）| 4 模板引擎齐全 | 🔀 进行中 | 同一 MyEntity 输出 5 文件 fixture diff |
| **iter8**（vernal starter）| axum/actix 启动器 + SqlRunner + Transactions | ✅ 基础已落 | 通过 axum/actix 真实 HTTP 接入 |

---

## Stage 1 — iter0: pre-flight

### Task 0.1：标 cached 子系统 deprecated
- [x] **Step 1:** `rbatis-plus-core/src/cache/intercept.rs` 加 `#[deprecated]`
- [x] **Step 2:** `rbatis-plus-core/src/cache/store.rs` 同上
- [x] **Step 3:** `rbatis-plus-core/src/cache/policy.rs` 同上
- [x] **Step 4:** `rbatis-plus-core/src/cache/key.rs` 同上
- [x] **Step 5:** `rbatis-plus-core/src/cache/memory.rs` 同上
- [x] **Step 6:** `rbatis-plus-core/src/cache/listener.rs` 同上
- [x] **Step 7:** `rbatis-plus-core/src/cache/error.rs` 同上
- [x] **Step 8:** `cargo check --workspace` 通过
- [x] **Step 9:** Commit: `chore(deprecation): 标 iter6 待删模块 deprecated + 4 份迁移文档`

---

## Stage 2 — iter1: metadata 全量 + 16 derive sub-mod

### Task 1.1：16 derive 拆分
- [x] **Step 1:** `TableName` derive → `rbatis-plus-macros/src/derive/table_name.rs`
- [x] **Step 2:** `TableId` derive → `rbatis-plus-macros/src/derive/table_id.rs`
- [x] **Step 3:** `TableField` derive → `rbatis-plus-macros/src/derive/table_field.rs`
- [x] **Step 4:** `TableLogic` derive → `rbatis-plus-macros/src/derive/table_logic.rs`
- [x] **Step 5:** `Version` derive → `rbatis-plus-macros/src/derive/version.rs`
- [x] **Step 6:** `FieldFill` derive → `rbatis-plus-macros/src/derive/field_fill.rs`
- [x] **Step 7:** `FieldStrategy` derive → `rbatis-plus-macros/src/derive/field_strategy.rs`
- [x] **Step 8:** `IdType` enum → `rbatis-plus-core/src/derive/id_type.rs`
- [x] **Step 9:** `DbType` enum → `rbatis-plus-core/src/derive/db_type.rs`
- [x] **Step 10:** `IEnum` trait → `rbatis-plus-core/src/derive/i_enum.rs`
- [x] **Step 11:** `InterceptorIgnore` derive → `rbatis-plus-core/src/derive/interceptor_ignore.rs`
- [x] **Step 12:** `KeySequence` derive → `rbatis-plus-core/src/derive/key_sequence.rs`
- [x] **Step 13:** `OrderBy` derive → `rbatis-plus-core/src/derive/order_by.rs`
- [x] **Step 14:** `EncryptedField` derive → `rbatis-plus-core/src/derive/encrypted_field.rs`
- [x] **Step 15:** `EncryptedTable` derive → `rbatis-plus-core/src/derive/encrypted_table.rs`
- [x] **Step 16:** `I18nColumn` derive → `rbatis-plus-core/src/derive/i18n_column.rs`
- [x] **Step 17:** `SignatureField` derive → `rbatis-plus-core/src/derive/signature_field.rs`
- [x] **Step 18:** `TableSignature` derive → `rbatis-plus-core/src/derive/table_signature.rs`

### Task 1.2：metadata 全量
- [x] **Step 1:** `TableInfo` → `rbatis-plus-core/src/metadata/table_info.rs`（142 行，待补字段）
- [ ] **Step 2:** `TableFieldInfo` 拆出独立文件
- [ ] **Step 3:** `TableIdInfo` → 新建
- [ ] **Step 4:** `OrderFieldInfo` → 新建
- [ ] **Step 5:** `MetaObject` → 新建
- [ ] **Step 6:** `ColumnCache` → 新建
- [ ] **Step 7:** `TableInfoHelper` → 新建
- [ ] **Step 8:** `TableInfoHelperFactory` → 新建
- [ ] **Step 9:** Commit: `feat(metadata): TableInfo / TableFieldInfo 全量 + 16 derive 拆分`

---

## Stage 3 — iter2: conditions 全量

### Task 2.1：核心 traits + wrappers
- [x] **Step 1:** `Wrapper<T>` → `rbatis-plus-core/src/conditions/wrapper.rs`（29 行）
- [x] **Step 2:** `AbstractWrapper<T,R,Children>` → `conditions/abstract_wrapper.rs`（202 行）
- [x] **Step 3:** `QueryWrapper<T>` → `conditions/query/query_wrapper.rs`（317 行）
- [x] **Step 4:** `UpdateWrapper<T>` → `conditions/update/update_wrapper.rs`（189 行）
- [x] **Step 5:** `LambdaQueryWrapper<T>` → `conditions/query/lambda_query_wrapper.rs`（548 行）
- [x] **Step 6:** `LambdaUpdateWrapper<T>` → `conditions/update/lambda_update_wrapper.rs`（534 行）

### Task 2.2：接口 traits
- [x] **Step 1:** `Compare<R>` → `conditions/compare.rs`（209 行）
- [x] **Step 2:** `Func<R>` → `conditions/func.rs`（119 行）
- [x] **Step 3:** `Nested<R>` → `conditions/nested.rs`（137 行）
- [x] **Step 4:** `MergeSegments` → `conditions/merge_segments.rs`（113 行）
- [x] **Step 5:** `Column<F>` → `conditions/query/column.rs`（72 行）

### Task 2.3：待补文件
- [ ] **Step 1:** `AbstractLambdaWrapper<T,Children>` → 新建
- [ ] **Step 2:** `Join<R>` 从 nested.rs 拆出
- [ ] **Step 3:** `SharedString` → 新建
- [ ] **Step 4:** `ISqlSegment` → 新建
- [ ] **Step 5:** `NormalSegmentList` → 新建
- [ ] **Step 6:** `GroupBySegmentList` → 新建
- [ ] **Step 7:** `OrderBySegmentList` → 新建
- [ ] **Step 8:** `HavingSegmentList` → 新建
- [ ] **Step 9:** `ColumnSegment` → 新建
- [ ] **Step 10:** `Wrappers` 工厂方法 → 新建
- [ ] **Step 11:** `SqlKeyword` enum → 新建
- [ ] **Step 12:** `SqlLike` enum → 新建
- [ ] **Step 13:** `WrapperKeyword` enum → 新建
- [ ] **Step 14:** `Constants` → 新建
- [ ] **Step 15:** 12 unit test 覆盖全部 `if_*` 条件重载
- [ ] **Step 16:** Commit: `feat(conditions): AbstractWrapper + 5 块完整 + 12 unit test`

---

## Stage 4 — iter3: BaseMapper 默认 impl + 13 AbstractMethod

### Task 3.1：mapper trait
- [x] **Step 1:** `BaseMapper<T>` trait → `rbatis-plus-core/src/mapper/base_mapper.rs`（90+ 行）
- [ ] **Step 2:** `Mapper<T>` 标记 trait → 新建
- [ ] **Step 3:** `MapperProxyMetadata` → 新建

### Task 3.2：13 AbstractMethod 子类（已实现 15 个文件）
- [x] **Step 1:** `AbstractMethod` + `MethodResult` → `method/abstract_method.rs`
- [x] **Step 2:** `SqlMethod` enum → `method/sql_method.rs`
- [x] **Step 3:** `Insert` → `method/insert.rs`
- [x] **Step 4:** `Delete` → `method/delete.rs`
- [x] **Step 5:** `DeleteById` → `method/delete_by_id.rs`
- [x] **Step 6:** `DeleteByIds` → `method/delete_by_ids.rs`
- [x] **Step 7:** `Update` → `method/update.rs`
- [x] **Step 8:** `UpdateById` → `method/update_by_id.rs`
- [x] **Step 9:** `SelectById` → `method/select_by_id.rs`
- [x] **Step 10:** `SelectByIds` → `method/select_by_ids.rs`
- [x] **Step 11:** `SelectByMap` → `method/select_by_map.rs`
- [x] **Step 12:** `SelectCount` → `method/select_count.rs`
- [x] **Step 13:** `SelectList` → `method/select_list.rs`
- [x] **Step 14:** `SelectMaps` → `method/select_maps.rs`
- [x] **Step 15:** `SelectOne` → `method/select_one.rs`
- [x] **Step 16:** `SelectObjs` → `method/select_objs.rs`

### Task 3.3：待补 method
- [ ] **Step 1:** `DeleteByMap` → 新建
- [ ] **Step 2:** `DeleteBatchByIds` → 新建
- [ ] **Step 3:** `SelectBatchByIds` → 新建
- [ ] **Step 4:** `SelectMapsPage` → 新建
- [ ] **Step 5:** `SelectPage` → 新建
- [ ] **Step 6:** `SelectWithCursor` → 新建
- [ ] **Step 7:** Commit: `feat(mapper): BaseMapper 9 CRUD + 13 method 默认 impl`

---

## Stage 5 — iter4: 16 derive 全产出 + macrotest

### Task 4.1：macrotest + trybuild
- [ ] **Step 1:** `rbatis-plus-macros/tests/ui/<name>_fail.rs` × 16（编译失败负例）
- [ ] **Step 2:** `rbatis-plus-macros/tests/ui/<name>_pass.rs` × 16（编译通过正例）
- [ ] **Step 3:** `rbatis-plus-macros/src/attr/<name>.rs` × 16（属性解析子模块）
- [ ] **Step 4:** `rbatis-plus-macros/src/codegen/<name>.rs` × 16（代码生成子模块）
- [ ] **Step 5:** `rbatis-plus-core/src/annotation.rs`（运行时标记 enum）
- [ ] **Step 6:** Commit: `feat(macros): 16 derive 全 + 32 macrotest`

---

## Stage 6 — iter5: inner big-bang

### Task 5.1：mybatis-plus 原生 6 inner（已实现 7 个文件）
- [x] **Step 1:** `PaginationInnerInterceptor` → `inner/pagination.rs`（217 行）
- [x] **Step 2:** `TenantLineInnerInterceptor` → `inner/tenant.rs`（96 行）
- [x] **Step 3:** `DataPermissionInnerInterceptor` → `inner/data_permission.rs`（80 行）
- [x] **Step 4:** `BlockAttackInnerInterceptor` → `inner/block_attack.rs`（50 行）
- [x] **Step 5:** `DynamicTableNameInnerInterceptor` → `inner/dynamic_table_name.rs`（71 行）
- [x] **Step 6:** `OptimisticLockerInnerInterceptor` → `inner/optimistic_locker.rs`（68 行）
- [ ] **Step 7:** `IllegalSQLInnerInterceptor` → 新建
- [ ] **Step 8:** `DataChangeRecorderInnerInterceptor` → 新建
- [ ] **Step 9:** `ReplacePlaceholderInnerInterceptor` → 新建

### Task 5.2：mybatis-plus-enhance 8 inner（已实现 8 个文件）
- [x] **Step 1:** `EnhanceInnerInterceptor` → `inner/enhance_interceptor.rs`
- [x] **Step 2:** `EnhancePhase` → `inner/enhance_phase.rs`
- [x] **Step 3:** `MybatisPlusEnhanceInterceptor` → `inner/mybatis_plus_enhance_interceptor.rs`
- [x] **Step 4:** `DataEncryptionInnerInterceptor` → `crypto/data_encryption.rs`
- [x] **Step 5:** `DataDecryptionInnerInterceptor` → `crypto/data_decryption.rs`
- [x] **Step 6:** `DataSignatureInnerInterceptor` → `signature/data_signature.rs`
- [x] **Step 7:** `DataI18nInnerInterceptor` → `i18n/data_i18n.rs`
- [x] **Step 8:** `LongSqlInnerInterceptor` → `inner/long_sql.rs`
- [x] **Step 9:** `SqlObservationInnerInterceptor` → `observation/sql_observation.rs`

### Task 5.3：待补增强功能
- [ ] **Step 1:** crypto handler 全量实现
- [ ] **Step 2:** signature handler 全量实现
- [ ] **Step 3:** i18n handler 全量实现
- [ ] **Step 4:** observation sink 全量实现
- [ ] **Step 5:** insert_ignore context 全量实现
- [ ] **Step 6:** 14 inner × 5 unit test
- [ ] **Step 7:** 5 集成场景（多拦截器链顺序）
- [ ] **Step 8:** Commit: `feat(inner): 14 个 InnerInterceptor 全量 + 5 集成场景`

---

## Stage 7 — iter6: cache 切换

### Task 6.1：删本地复刻
- [ ] **Step 1:** 硬删 `rbatis-plus-core/src/cache/intercept.rs`
- [ ] **Step 2:** 硬删 `rbatis-plus-core/src/cache/store.rs`
- [ ] **Step 3:** 硬删 `rbatis-plus-core/src/cache/policy.rs`
- [ ] **Step 4:** 硬删 `rbatis-plus-core/src/cache/memory.rs`
- [ ] **Step 5:** 硬删 `rbatis-plus-core/src/cache/key.rs`
- [ ] **Step 6:** 硬删 `rbatis-plus-core/src/cache/error.rs`
- [ ] **Step 7:** 硬删 `rbatis-plus-core/src/cache/listener.rs`
- [ ] **Step 8:** `rbatis-plus-core/src/cache/mod.rs` → `pub use rbatis::plugin::cache::*;`

### Task 6.2：新增 backend
- [ ] **Step 1:** `rbatis-plus-extension/src/cache/redis_backend.rs` — RedisCacheBackend
- [ ] **Step 2:** `rbatis-plus-extension/src/cache/memcached_backend.rs` — MemcachedCacheBackend
- [ ] **Step 3:** `rbatis-plus-extension/src/cache/put_if_absent.rs` — PutIfAbsentCache 装饰器
- [ ] **Step 4:** `rbatis-plus-extension/src/cache/simple.rs` — SimpleCache 装饰器
- [ ] **Step 5:** `rbatis-plus-extension/src/cache/transactional.rs` — TransactionalCache 装饰器
- [ ] **Step 6:** Commit: `refactor(cache): 删本地复刻，re-export rbatis::plugin::cache`

---

## Stage 8 — iter7: generator 三引擎齐全

### Task 7.1：模板引擎（已实现 3 个文件）
- [x] **Step 1:** `TeraTemplateEngine` → `template/tera_engine.rs`（268 行）
- [x] **Step 2:** `HandlebarsTemplateEngine` → `template/handlebars_engine.rs`
- [x] **Step 3:** `AskamaTemplateEngine` → `template/askama_engine.rs`
- [x] **Step 4:** `MaudTemplateEngine` → `template/maud_engine.rs`
- [x] **Step 5:** `TemplateEngine` trait → `template/template_engine.rs`

### Task 7.2：配置 + 查询
- [x] **Step 1:** `DataSourceConfig` → `config/data_source.rs`（108 行）
- [x] **Step 2:** `PackageConfig` → `config/package.rs`（115 行）
- [x] **Step 3:** `StrategyConfig` → `config/strategy.rs`（156 行）
- [x] **Step 4:** `GlobalConfig` → `config/global.rs`（67 行）
- [x] **Step 5:** `TableInfo` query → `query/table_info.rs`（63 行）

### Task 7.3：待补
- [ ] **Step 1:** `TemplateConfig` → 新建
- [ ] **Step 2:** `InjectionConfig` → 新建
- [ ] **Step 3:** `FastAutoGenerator` → 新建
- [ ] **Step 4:** `DbType` query → 新建
- [ ] **Step 5:** 3 套 sql_diff fixture
- [ ] **Step 6:** Commit: `feat(generator): tera/handlebars/askama + 3 套 sql_diff fixture`

---

## Stage 9 — iter8: vernal Spring starter 对偶

### Task 8.1：axum/actix 集成（已实现 5 个文件）
- [x] **Step 1:** `axum_integration.rs` → rbatis-plus-vernal/src/
- [x] **Step 2:** `actix_integration.rs` → rbatis-plus-vernal/src/
- [x] **Step 3:** `sql_runner.rs` → rbatis-plus-vernal/src/
- [x] **Step 4:** `state.rs` → rbatis-plus-vernal/src/
- [x] **Step 5:** `transaction.rs` → rbatis-plus-vernal/src/

### Task 8.2：待补
- [ ] **Step 1:** `config.rs` → rbatis-plus-vernal/src/
- [ ] **Step 2:** `lifecycle.rs` → 监听 RBatis init
- [ ] **Step 3:** `auto_config.rs` → 仿 @MapperScan
- [ ] **Step 4:** 5 集成测试 + Axum/Actix 真实 HTTP 启动验证
- [ ] **Step 5:** Commit: `feat(vernal): mn / actix 集成 + SqlRunner + Transactions`

---

## 阶段交接检查单（每阶段收尾必做）

- [ ] 测试全绿 + all-features 编译 + 0 新增 warning
- [ ] `对象级对照表.md` 状态刷新
- [ ] `语义迁移对照表.md` 状态刷新
- [ ] `对象名称一致性检查.md` 状态刷新
- [ ] 本地 git commit（规范 message：`chore/feat/refactor/test(scope)`）
- [ ] docs/iter<N>.md：① 增/删/改清单 ② Java 行号对照 ③ TODO
- [ ] 仓库 README 更新（去除 "DESIGN ONLY" 失真声明）

---

## 风险登记

| 风险 | 影响 | 对策 |
|---|---|---|
| mybatis-plus v3.5.17 较复杂，304 类可能估多估少 | iter 完成度 | S0 末做完整 304 类盘点 |
| vernal / axum / actix 差异 | iter8 阻塞 | 阶段开始前做可用性 spike |
| testcontainers-rs 网络/沙箱限制 | iter5 集成 | 提供 in-memory fallback |
| sqlparser-rs API 在 0.62 与稍后版本差异 | iter2 + iter5 | 锁定 0.62 workspace deps |
| mybatis-plus-enhance v2 / v3 重命名 | 全部 | 以 2.0.x 为基线 |
| iter6 删本地缓存引发 break | 高 | iter0 先 deprecation；iter6 用 `pub use rbatis::plugin::cache::*` 兜底 |
| iter3 BaseMapper 与 rbatis::Executor 签名冲突 | 中 | `trait CRUD<T, E: Executor>` + blanket impl |
| iter4 16 derive 包冲突 | 中 | macrotest + trybuild 双跑 |

---

## commit 计划（>= 10 commit）

| # | Commit | Iter |
|---|---|---|
| 1 | `chore(deprecation): 标 iter6 待删模块 deprecated + 4 份迁移文档` | iter0 |
| 2 | `feat(metadata): TableInfo / TableFieldInfo 全量 + 16 derive 拆分` | iter1 |
| 3 | `feat(conditions): AbstractWrapper + 5 块完整 + 12 unit test` | iter2 |
| 4 | `feat(mapper): BaseMapper 9 CRUD + 13 method 默认 impl` | iter3 |
| 5 | `feat(macros): 16 derive 全 + 32 macrotest` | iter4 |
| 6 | `feat(inner): 14 个 InnerInterceptor 全量 + 5 集成场景` | iter5 (1/2) |
| 7 | `chore: 优化 inner 拦截器链调度` | iter5 (2/2) |
| 8 | `refactor(cache): 删本地复刻，re-export rbatis::plugin::cache` | iter6 |
| 9 | `feat(generator): tera/handlebars/askama + 3 套 sql_diff fixture` | iter7 |
| 10 | `feat(vernal): mn / actix 集成 + SqlRunner + Transactions` | iter8 |

---

## 当前实测状态（2026-07-28 校准）

| 维度 | 当前实测 | Java 目标 | 缺口 |
|---|---|---|---|
| rbatis-plus 总 LOC（含 main） | **~8,000** | 53,710 | **-45,710** |
| 子 crate 数 | 6（与 Java Maven 模块对齐） | 7 Maven modules | OK |
| `BaseMapper<T>` 接口 | trait 声明、无默认 impl | Java 接口 + `@SelectProvider` 等 | 9 方法 + 14 重载 |
| `QueryWrapper / UpdateWrapper` | 已落（核心 traits 链式 OK） | Java AbstractWrapper + 13 派生 | 12 unit 缺口 |
| `LambdaQueryWrapper / LambdaUpdateWrapper` | 已落（548 / 534 行） | Java lambda 反射 | 已基本对齐 |
| 16 个注解（`@TableName` 等） | 18 derive 已落 | Java 16 注解 | 已超额完成 |
| 8 增强层 inner | 8 落 MVP | Enhance 8 | 6 → 真实实现 |
| 6 mybatis-plus-jsqlparser inner | 落 7 但有 MVP | jsqlparser 6 | 关键实现 4 处需重做 |
| Spring/Quarkus 集成 | rbatis-plus-vernal | Spring Boot Starter | axum/actix starter + auto-config |
| 13 AbstractMethod 子类 | 16 落 | 21 derived method | 5 方法缺失 |
| Generator | 4 引擎已落 | Generator POM | 配置补全 |
| 缓存子系统 | DashMap 自实现 | mybatis `Cache` SPI | **必须迁移为复用 rbatis::plugin::cache** |

---

## 本文档未覆盖的细节

详见后续文档：

- `specs/2026-07-20-object-mapping-spec.md`：每个 Java 类落在哪个 .rs 文件 + 状态
- `specs/2026-07-20-semantic-migration-spec.md`：每个 Java 功能语义在 Rust 的对应实现方式
- `specs/2026-07-20-naming-convention-spec.md`：snake_case / PascalCase 对照表与人工校对记录
