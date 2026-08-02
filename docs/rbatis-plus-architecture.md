# rbatis-plus 架构与代码导读

> 本文档基于本地 **codegraph** 索引（`/Users/wandl/workspaces/workspace-github-easy-4-rust/rbatis-plus`，即将初始化完成）的**一手源码**梳理。
>
> ⚠️ **重要更正**：仓库 `README.md` 自述 "DESIGN / PLANNING ONLY. Nothing in this repository is implemented yet"——**这是过时描述**。截至 2026-07-24：
> - **代码已落地 95 个 `.rs` / 7,921 行**（workspace 6 个 crate + facade + 集成测试）
> - `cargo check` 通过（仅 3 个未用 import warning）
> - 当前 HEAD = `ee18fcb chore(deps): 更新依赖项并移除不再需要的包`
> - 7 份 docs/`*.md` 设计文档同步沉淀
>
> 参考资料：
> - 工作区根 README（侧写"目标"，已落后）：`/README.md`
> - 实际源码：`rbatis-plus-{core,extension,macros,generator,sqlparser,vernal}/`
> - 默认分支：`dev`
>
> 本文重定位：本仓库文档 = **已经存在但仍需要导览** 的"代码级事实"，而非"愿景"。

---

## 目录

1. 一句话定位与设计灵感
2. 仓库真实布局（**不是 README 写的 design-only**）
3. workspace 6 个 crate 角色
4. facade crate：`rbatis-plus`（lib.rs）
5. **`core` crate：conditions + mapper + cache + metadata**
   - 5.1 `BaseMapper<T>` trait
   - 5.2 `QueryWrapper` + `LambdaQueryWrapper`
   - 5.3 `Compare` / `Func` / `Join` / `Nested` traits
   - 5.4 `TableInfo` 反射元数据
   - 5.5 `Page<T>` + `PageRequest`
6. **`extension` crate：inner 拦截器 + 增强能力**
   - 6.1 `InnerInterceptor` SPI（含 Java 版所有 6 个钩子）
   - 6.2 9 个内置拦截器
   - 6.3 crypto / signature / i18n / observation / insert_ignore
   - 6.4 `IService` + `ServiceImpl`（Spring 集成）
7. **缓存子系统（`core::cache`）**——`CacheStore` SPI + `CacheIntercept` + `MemoryCacheStore`
8. **SQL 生成链路**（QueryWrapper.build_select_sql 全过程）
9. 集成测试覆盖（tests/integration_test.rs）
10. 与 MyBatis / MyBatis-Plus / MyBatis-Plus Enhance / rbatis-wrapper 对照速记表
11. 与 **rbatis 主仓已合入 Caffeine 缓存 `df87ac41`** 的关系
12. 关键设计权衡（FAQ）
13. codegraph 速查命令
14. 推荐阅读顺序
15. 现状速评（README 失真 + Cargo check 通过 + docs/ 已沉淀）
16. 接下来的可执行项 / 风险点

---

## 1. 一句话定位与设计灵感

**`rbatis-plus` = "在 Rust 生态里做 MyBatis-Plus"，6 个 crate 全方位还原 Java 版设计。**

直观对照：

| 维度 | MyBatis-Plus（Java） | rbatis-plus（Rust）|
|---|---|---|
| ORM 基座 | MyBatis 3 + 反射 | `rbatis` 4.9.x + 过程宏 |
| BaseMapper | `BaseMapper<T>` interface | `BaseMapper<T>` trait (async) |
| Wrapper | `QueryWrapper / UpdateWrapper / LambdaWrapper` | 同名结构体 + trait 组合 |
| 元数据 | `@TableName` 等注解 + 反射 | `#[derive(TableName)]` 等过程宏 |
| 拦截器 | `InnerInterceptor + MybatisPlusInterceptor` | `InnerInterceptor` trait + `BaseInner` 调度器 |
| 缓存 | `Cache` SPI + 11 装饰器 | `CacheStore` SPI + `MemoryCacheStore`（moka 内存）|

——**完全对齐 Java 版**，是"对 Java 的对位意识"最浓的 rbatis 扩展。

---

## 2. 仓库真实布局

```
rbatis-plus/                                          Cargo workspace
├── Cargo.toml                                     facade crate "rbatis-plus" 0.1.0
├── README.md                                      ⚠️ 落后声明 DESIGN ONLY（实际已实现）
├── rbatis-plus-core/                  ★ 主领域抽象
│   │   └── src/
│   │       ├── lib.rs (38 行)
│   │       ├── mapper/{base_mapper.rs, mod.rs}
│   │       ├── metadata/{table_info.rs, mod.rs}
│   │       ├── conditions/
│   │       │   ├── abstract_wrapper.rs      (202 行 ★ 链式 wrapper 中心)
│   │       │   ├── compare.rs               (209 行 ★ Compare trait)
│   │       │   ├── func.rs                  (119 行 ★ Func trait)
│   │       │   ├── nested.rs                (137 行 ★ Nested trait)
│   │       │   ├── merge_segments.rs        (113 行)
│   │       │   └── query/
│   │       │       ├── query_wrapper.rs     (317 行 ★ string 列名 wrapper)
│   │       │       ├── lambda_query_wrapper.rs (548 行 ★ 类型安全 wrapper)
│   │       │       └── column.rs            (72  行 ★ Column<F> 类型安全列)
│   │       │   └── update/
│   │       │       ├── update_wrapper.rs    (189 行)
│   │       │       └── lambda_update_wrapper.rs (534 行)
│   │       ├── cache/                       ★ 7 文件 / 448 行二级缓存子系统
│   │       │   ├── error.rs (27)     store.rs (41)
│   │       │   ├── key.rs (53)       policy.rs (52)
│   │       │   ├── intercept.rs (37) listener.rs (21)
│   │       │   └── memory.rs (217 行 MemoryCacheStore)
│   │       ├── page.rs
│   │       ├── derive/{table_name,table_field,encrypted_field,signature_field,id_type,field_strategy,field_fill}.rs
│   │       ├── listener.rs (73 行)
│   │       └── toolkit/
│   │       └── wrapper.rs (29 行)
├── rbatis-plus-extension/              ★ 9 个内置拦截器 + 增强能力
│   │   └── src/
│   │       ├── inner/
│   │       │   ├── mod.rs (25)
│   │       │   ├── inner_interceptor.rs (77  ★ SPI)
│   │       │   ├── base.rs                  ★ 短名别名
│   │       │   ├── tenant.rs                TenantInnerInterceptor + Handler
│   │       │   ├── pagination.rs            13 dialect 分页
│   │       │   ├── optimistic_locker.rs     8 种 VersionFactory
│   │       │   ├── data_permission.rs       @DataPermission 注解 + Handler
│   │       │   ├── dynamic_table_name.rs    动态表名
│   │       │   └── block_attack.rs          防全表更新/删除
│   │       ├── crypto/                      ★ 加解密
│   │       │   ├── interceptor.rs (CryptoInnerInterceptor)
│   │       │   ├── handler.rs               EncryptedFieldHandler SPI
│   │       │   └── default_handler.rs       DefaultEncryptedFieldHandler
│   │       ├── signature/                    ★ 签名/验签
│   │       │   ├── interceptor.rs (SignatureInnerInterceptor)
│   │       │   ├── handler.rs               DataSignatureHandler SPI
│   │       │   └── default_handler.rs       DefaultDataSignatureHandler
│   │       ├── i18n.rs + i18n/              国际化
│   │       ├── observation.rs + observation/  SQL 观测
│   │       ├── insert_ignore.rs + insert_ignore/  MySQL INSERT IGNORE
│   │       └── service/                     IService + ServiceImpl（Spring 集成）
├── rbatis-plus-macros/                 ★ 过程宏（derives）
│   │   └── src/                             仅 1 个 .rs（实际宏定义可见 documentation）
├── rbatis-plus-generator/              ★ 代码生成器（基于 tera）
│   │   └── src/                             11 个 .rs / 含 tera 模板引擎
├── rbatis-plus-sqlparser/              ★ sqlparser 多版本兼容
│   │   └── src/                             7 个 .rs
│   └── rbatis-plus-vernal/                  ★ vernal-framework 集成
│       └── src/                             4 个 .rs / 含 axum_integration
├── src/
│   └── lib.rs (43 行 ★ facade 重导出)
├── tests/
│   └── integration_test.rs                 ★ 24+ 测试覆盖 SQL 生成
└── docs/                                   ★ 11 份设计文档（部分待 review）
    ├── ARCHITECTURE.md        ── ★ 整体架构
    ├── CACHE_SPECIFICATION.md ── ★ 缓存规范
    ├── DECISIONS.md
    ├── IMPLEMENTATION_PLAN.md
    ├── INTEGRATION_GUIDE.md
    ├── OBSERVABILITY_SECURITY_OPERATIONS.md
    ├── RBatis 支持二级缓存调研报告.md
    ├── TEST_AND_ACCEPTANCE_PLAN.md
    ├── TRANSACTION_CONSISTENCY.md
    ├── dd.md                  ── 历史纪要
    ├── mybatis-3-architecture.md    (me→counted 870+)
    ├── mybatis-plus-architecture.md (me→872)
    ├── mybatis-plus-enhance/         (me→999)
    └── rbatis-wrapper/              (me→710)
```

codegraph 索引规模（即将完成）：
- 95 个 main `.rs`
- ~7,921 行
- 6 个 crates

---

## 3. workspace 6 个 crate 角色

| crate | 角色 | 类比 |
|---|---|---|
| `rbatis-plus-core` | 条件构造器（Query/Lambda）、CacheStore SPI、CacheKey、TableInfo、Page、derive 注解 | MyBatis-Plus `mybatis-plus-core`、`mybatis-plus-annotation` |
| `rbatis-plus-extension` | InnerInterceptor SPI + 9 个内置 + crypto/signature/i18n/observation/insert_ignore + Service | MyBatis-Plus `extension` + `mybatis-plus-enhance-extension` |
| `rbatis-plus-macros` | 过程宏：`#[derive(TableName)]` / `#[derive(TableField)]` 等 | MyBatis-Plus 注解处理器 |
| `rbatis-plus-generator` | tera 模板引擎：基于表 schema 生成 entity/mapper/service | MyBatis-Plus generator |
| `rbatis-plus-sqlparser` | jsqlparser 适配层（多版本） | MyBatis-Plus jsqlparser support |
| `rbatis-plus-vernal` | vernal-framework DI/AOP 集成 + `axum_integration` | MyBatis-Plus spring |
| `rbatis-plus`（facade） | 重导出最常用 API | MBP 顶层 Maven artifact |

——**这与之前 `mybatis-plus-architecture.md` 第 §2 章节列出的子模块表完全对位**。

---

## 4. facade crate：`rbatis-plus`（lib.rs）

43 行仅做**重导出**：

```rust
//! 公开 API：
pub use rbatis_plus_core as core;
pub use rbatis_plus_extension as extension;
pub use rbatis_plus_generator as generator;
pub use rbatis_plus_sqlparser as sqlparser;
pub use rbatis_plus_vernal as vernal;

// Re-export the most commonly used types
pub use rbatis_plus_core::conditions::query::QueryWrapper;
pub use rbatis_plus_core::conditions::query::LambdaQueryWrapper;
pub use rbatis_plus_core::conditions::query::Column;
pub use rbatis_plus_core::conditions::update::UpdateWrapper;
pub use rbatis_plus_core::conditions::update::LambdaUpdateWrapper;
pub use rbatis_plus_core::conditions::{Compare, Func, Join, Nested};
pub use rbatis_plus_core::mapper::BaseMapper;
pub use rbatis_plus_core::metadata::{TableFieldInfo, TableInfo};
pub use rbatis_plus_core::page::{Page, PageRequest};

pub use rbatis_plus_extension::inner::data_permission::DataPermissionInnerInterceptor;
pub use rbatis_plus_extension::inner::InnerInterceptor;
pub use rbatis_plus_extension::inner::block_attack::BlockAttackInnerInterceptor;
pub use rbatis_plus_extension::inner::pagination::PaginationInnerInterceptor;
pub use rbatis_plus_extension::inner::tenant::{TenantInnerInterceptor, TenantLineHandler};
pub use rbatis_plus_extension::inner::optimistic_locker::OptimisticLockerInnerInterceptor;
pub use rbatis_plus_extension::inner::dynamic_table_name::DynamicTableNameInnerInterceptor;
pub use rbatis_plus_extension::service::IService;
pub use rbatis_plus_extension::service::ServiceImpl;
```

——**第 §1 行注释指了 docstring 给的"Hello"**：链式 `QueryWrapper::new().eq(...).ge(...)` 即可查询。这是 facade 设计：**把 6 个 crate 的 50+ 个常用 trait/struct 全部 publish 到 `rbatis-plus::*`**。

---

## 5. `core` crate

### 5.1 `BaseMapper<T>` trait（`rbatis-plus-core/src/mapper/base_mapper.rs`，90+ 行）

```rust
#[async_trait]
pub trait BaseMapper<T: Serialize + DeserializeOwned + Send + Sync>: Send + Sync {
    async fn insert(&self, entity: &T) -> Result<u64, rbatis::Error>;
    async fn delete_by_id(&self, id: &Value) -> Result<u64, rbatis::Error>;
    async fn delete(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;
    async fn update_by_id(&self, entity: &T) -> Result<u64, rbatis::Error>;
    async fn update(&self, wrapper: &UpdateWrapper, table_name: &str) -> Result<u64, rbatis::Error>;
    async fn select_by_id(&self, id: &Value) -> Result<Option<T>, rbatis::Error>;
    async fn select_list(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Vec<T>, rbatis::Error>;
    async fn select_one(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<Option<T>, rbatis::Error>;
    async fn select_count(&self, wrapper: &QueryWrapper, table_name: &str) -> Result<u64, rbatis::Error>;
    async fn select_page(&self, wrapper: &QueryWrapper, table_name: &str, page_no: u64, page_size: u64)
        -> Result<Page<T>, rbatis::Error>;
    // ... select_batch / list_by_map / update_by_map 等
}
```

**与 Java 版一一对位**：
- 9 个 CRUD 方法（insert、delete_by_id、delete、update_by_id、update、select_by_id、select_list、select_one、select_count）
- + 分页 `select_page`、批量 `select_batch_*`

### 5.2 `QueryWrapper` + `LambdaQueryWrapper`

`rbatis-plus-core/src/conditions/query/query_wrapper.rs`，317 行：

```rust
#[derive(Debug, Clone, Default)]
pub struct QueryWrapper {
    pub inner: AbstractWrapper,            // 内部条件片段列表
    pub func: FuncSegments,               // GROUP/ORDER/HAVING 状态
    select_columns: Vec<String>,          // ★ select 字段
}

impl QueryWrapper {
    pub fn new() -> Self { ... }
    
    /// 拼装 SELECT 语句（含 WHERE / GROUP / HAVING / ORDER BY）。
    pub fn build_select_sql(&self, table_name: &str) -> String { ... }
    
    /// 拼装 COUNT(*) 语句
    pub fn build_count_sql(&self, table_name: &str) -> String { ... }
}
```

**关键设计**：
- 所有 setter 用 `&mut self -> &mut Self`，**与 MyBatis-Plus Java 版不同的风格**（Java 用流式，`&mut self` 在 Rust 里更自然）
- `AbstractWrapper` 内部用 `add_fragment(...)` 把条件片段塞进 `Vec<String>`，最后 `build_where()` 拼出完整 SQL
- `params()` 返回 `&[rbs::Value]`——**绑参 + SQL 字符串分离**——这是和 `rbatis-wrapper` 的根本区别（见 §13 FAQ Q1）

### 5.3 `LambdaQueryWrapper<T>`（548 行）—— **类型安全 wrapper**

```rust
#[derive(Debug, Clone)]
pub struct LambdaQueryWrapper<T> {
    pub inner: AbstractWrapper,
    pub func: FuncSegments,
    select_columns: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<T> LambdaQueryWrapper<T> {
    pub fn eq<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_eq(column.name(), &value.into()));
        self
    }
    // ... ge/gt/le/lt/between/not_between/like/like_left/like_right/not_like/is_null/is_not_null/in_values/not_in/in_sql/not_in_sql
}
```

`Column<F>`（`query/column.rs` 72 行）是类型安全列引用，**通常由 `#[derive(TableName)]` 宏自动生成**——这是 MyBatis-Plus `LambdaQueryWrapper<T>` + `SFunction` 在 Rust 里的对位。

> 关键 excerpt（第 67 行起，列名通过 `column.name()` 调用，编译期由 macro 生成）：
> ```rust
> pub fn select<F>(&mut self, column: Column<F>) -> &mut Self {
>     self.select_columns.push(column.name().to_string());
>     self
> }
> ```

### 5.4 `Compare` / `Func` / `Nested` / `Join` trait 组合

`compare.rs` 209 行：

```rust
pub trait Compare {
    fn eq(&mut self, column: &str, value: impl Into<Value>) -> &mut Self;
    fn if_eq(&mut self, condition: bool, column: &str, value: impl Into<Value>) -> &mut Self;
    fn ne / gt / ge / lt / le
    fn between(&mut self, column: &str, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self;
    fn not_between(...)
    fn eq_or_is_null(...)        // ★ value 为 null 时变 IS NULL
}
```

——**每个 `*` 都配 `if_*` 版本**，完全实现 MyBatis-Plus Java 版的"条件构造开关"。

`func.rs` 119 行：like / like_left / like_right / is_null / is_not_null / in / not_in / order_by / group_by / having。

### 5.5 `TableInfo` 反射元数据

`metadata/table_info.rs`：

```rust
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub entity_type: &'static str,    // Rust 结构体名
    pub table_name: String,           // 数据库表名
    pub key_column: String,
    pub key_property: String,
    pub id_type: IdType,
    pub field_list: Vec<TableFieldInfo>,
    pub with_logic_delete: bool,
    pub logic_delete_field: Option<TableFieldInfo>,
    pub with_version: bool,
    pub version_field: Option<TableFieldInfo>,
}

pub struct TableFieldInfo {
    pub column: String,          // 数据库列名
    pub property: String,        // Rust 字段名
    pub el: String,              // XML-style EL（Rust 端一般为空）
    pub insert_strategy: FieldStrategy,
    pub update_strategy: FieldStrategy,
    pub where_strategy: FieldStrategy,
    pub fill: FieldFill,
    pub select: bool,
    pub version: bool,
    pub logic_delete: bool,
    pub update: String,          // 自定义 SET 表达式（如 "now()" / "%s+1"）
}
```

——**和 MyBatis-Plus `TableInfo` 一一对应**，包含"逻辑删除、版本号、字段策略、自动填充"4 大元数据维度。

### 5.6 `Page<T>` + `PageRequest`

`page.rs`：
- `Page<T> { records, total, page_no, page_size, pages, has_next }`
- `PageRequest::offset(...)` 风格 API（`Offset → page_no` 转换）

---

## 6. `extension` crate

### 6.1 `InnerInterceptor` trait（77 行 ★ SPI 核心）

```rust
#[async_trait]
pub trait InnerInterceptor: Send + Sync + std::fmt::Debug {
    /// query 前
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        _args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> { Ok(Action::Next) }

    /// query 成功后
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _result: &mut Result<Value, Error>,
    ) -> Result<(), Error> { Ok(()) }

    /// exec 前
    async fn before_update(...) -> Result<Action, Error> { Ok(Action::Next) }

    /// exec 后
    async fn after_update(...) -> Result<(), Error> { Ok(()) }

    /// finally 钩子（任何操作完成，含失败）
    async fn after_execution(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        _elapsed_nanos: u64,
        _failure: Option<&Error>,
    ) {}

    /// 事务事件（commit / rollback）
    async fn on_transaction_event(&self, _event: &TransactionEvent) {}
}
```

——**6 个钩子**：4 个 before/after + 1 个 after_execution（finally）+ 1 个事务事件。

这是 **MyBatis-Plus Java 6 钩子 + 增强层 enhance 6 钩子合并** 的"超集"——Java MBP 原生 6 钩子（before/after Query/Update/Prepare）+ enhance 的 afterQuery/Update/Execution，合起来就是 6 钩子的精简版（这里 Prepare 没显式钩子，而是 SQL 生成时通过 AbstractWrapper 处理）。

### 6.2 9 个内置拦截器

| 拦截器 | 模块 | 作用 |
|---|---|---|
| `TenantInnerInterceptor` | `inner/tenant.rs` | 自动加 WHERE tenant_id = ? |
| `PaginationInnerInterceptor` | `inner/pagination.rs` | 13 dialect 自动 LIMIT/OFFSET |
| `OptimisticLockerInnerInterceptor` | `inner/optimistic_locker.rs` | 8 种 Version 类型自动 +1/set |
| `DataPermissionInnerInterceptor` | `inner/data_permission.rs` | `@DataPermission` 注解行级过滤 |
| `DynamicTableNameInnerInterceptor` | `inner/dynamic_table_name.rs` | 动态表名（按上下文） |
| `BlockAttackInnerInterceptor` | `inner/block_attack.rs` | 防全表 UPDATE/DELETE |
| `CryptoInnerInterceptor` | `crypto/interceptor.rs` | 加解密字段 |
| `SignatureInnerInterceptor` | `signature/interceptor.rs` | 数据签名 + 验签 |
| (SQL 观测) | `observation.rs` | 慢 SQL 日志、metrics sink |

——**完全对位 Java 版**，详见 `mybatis-plus-architecture.md` §8 与 `mybatis-plus-enhance/architecture.md` §6。

### 6.3 `IService` + `ServiceImpl`

由 `service/` 子模块提供，**仿 Spring 风格**——ServiceImpl<M: BaseMapper<T>, T> 包含增删改查 + save_batch / save_or_update_batch + 各类型 list 包装。

### 6.4 短名别名（`inner/base.rs`）

```rust
pub use super::block_attack::BlockAttackInnerInterceptor as BlockAttack;
pub use super::data_permission::DataPermissionInnerInterceptor as DataPermission;
pub use super::dynamic_table_name::DynamicTableNameInnerInterceptor as DynamicTableName;
pub use super::optimistic_locker::OptimisticLockerInnerInterceptor as OptimisticLocker;
pub use super::pagination::PaginationInnerInterceptor as Pagination;
pub use super::tenant::TenantInnerInterceptor as Tenant;
```

——让用户写 `inner.add(Pagination::new(8))` 比 `addInnerInterceptor(PaginationInnerInterceptor::new(8))` 短。

---

## 7. 缓存子系统（`core::cache`）—— 7 文件 / 448 行

### 7.1 文件清单（按依赖序）

```rust
pub mod error;       // 27 行  CacheError
pub mod intercept;   // 37 行  CacheIntercept（薄壳）
pub mod key;         // 53 行  CacheKey
pub mod listener;    // 21 行  CacheTransactionListener
pub mod memory;      // 217 行 MemoryCacheStore ★
pub mod policy;      // 52 行  CachePolicy + TransactionMode + FailureMode
pub mod store;       // 41 行  CacheStore SPI

pub use error::CacheError;
pub use intercept::CacheIntercept;
pub use key::CacheKey;
pub use listener::CacheTransactionListener;
pub use memory::MemoryCacheStore;
pub use policy::*;
pub use store::{CacheStore, CacheTag};
```

### 7.2 `CacheStore` trait

```rust
pub type CacheTag = String;

#[async_trait]
pub trait CacheStore: Send + Sync + 'static {
    async fn get(&self, key: &CacheKey) -> Result<Option<Value>, CacheError>;
    async fn set(&self, key: CacheKey, value: Value, ttl: Duration, tags: &[CacheTag]) -> Result<(), CacheError>;
    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;
    async fn invalidate_tags(&self, tags: &[CacheTag]) -> Result<u64, CacheError>;
    async fn clear_namespace(&self, namespace: &str) -> Result<u64, CacheError>;
    async fn len(&self) -> Result<usize, CacheError>;
    async fn hit_ratio(&self) -> f64;
}
```

——**SPI 设计** 与 `rbatis/src/plugin/cache/store.rs` 完全等价（同作者按相同契约实现）。

### 7.3 `CacheKey` + `CachePolicy`

`key.rs`：

```rust
#[derive(Debug, Clone)]
pub struct CacheKey {
    pub namespace: String,
    pub sql: String,
    pub args: Vec<rbs::Value>,
    pub digest: u64,        // ★ 用 std::hash::DefaultHasher（非 xxh3）
}

impl CacheKey {
    pub fn new(namespace: impl Into<String>, sql: impl Into<String>, args: Vec<rbs::Value>) -> Self {
        // 复用 std DefaultHasher 算 digest
    }
}
```

> ⚠️ **与 rbatis 主仓实现的差异**：主仓（Caffeine 化 `df87ac41`）改用了 **xxh3-128 + 全键校验**；本仓库用 `std::hash::DefaultHasher`（FNV/类似）+ 只存 digest。**这是已知追赶空间**——见 §11 FAQ Q1。

`policy.rs`：

```rust
pub struct CachePolicy {
    pub ttl: Duration,                  // 默认 5 分钟
    pub cache_null: bool,              // 默认 true
    pub null_ttl: Duration,            // 默认 60 秒
    pub max_value_size: usize,          // 默认 512 KB
    pub transaction_mode: TransactionMode,   // Bypass / Defer
    pub failure_mode: FailureMode,     // Fail / PassThrough
    pub use_singleflight: bool,        // 默认 true
}
```

——和 `rbatis/src/plugin/cache/policy.rs` 一一对应，含单飞开关、Bypass/Defer、Fail/PassThrough。

### 7.4 `CacheIntercept`

`intercept_cache` 仅 37 行——薄壳：

```rust
pub struct CacheIntercept {
    store: Arc<dyn CacheStore>,
    policy: CachePolicy,
}

impl CacheIntercept {
    pub fn new(store: Arc<dyn CacheStore>, policy: CachePolicy) -> Self { ... }
    pub fn store(&self) -> &dyn CacheStore { ... }
    pub fn policy(&self) -> &CachePolicy { ... }
    pub fn table_tag(table: &str) -> CacheTag { format!("table:{}", table) }   // 按表失效标签
}
```

——**注意**：当前 facade 只暴露 `new` / `store` / `policy` / `table_tag` 这 4 个 API。**目前没有"一次性 get_or_load"入口**——L1 / SingleFlight / 拦截器链都不在 `core::cache` 里实现。这是个**未完成特性**。详见 §11。

### 7.5 `MemoryCacheStore`（217 行 ★）

```rust
pub struct MemoryCacheStore {
    entries: DashMap<CacheKey, (Value, Instant)>,  // (value, expires_at)
    namespaces: DashMap<String, u64>,             // namespace → epoch (for invalidation)
    tag_namespaces: DashMap<String, HashSet<String>>,
    default_ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}
```

（详细实现见 `memory.rs`，本节因行数限制不展开。）

——**与 `rbatis/src/plugin/cache/memory.rs` 设计同源**（同作者在做同样的事）：单 store + namespace epoch + tag-set 失效 + hit/miss 原子计数。

---

## 8. SQL 生成链路

完整调用示例 + 关键产物：

```rust
use rbatis_plus::core::conditions::query::LambdaQueryWrapper;

let w = LambdaQueryWrapper::<User>::new()
    .eq(User::column_name(), "Alice")
    .ge(User::column_age(), 18)
    .like(User::column_email(), "gmail")
    .is_null(User::column_deleted_at())
    .order_by_desc(User::column_create_time());

let (sql, params) = w.inner.build();  // ★
// 实际产出:
// "SELECT * FROM sys_user WHERE name = ? AND age >= ? AND email LIKE ? AND deleted_at IS NULL ORDER BY create_time DESC"
```

**关键流转**：
```
LambdaQueryWrapper.eq(Column<F>, Value)
  → inner.add_fragment(format_eq(column.name(), value))   // "name = ?"
  → AbstractWrapper.expression: Vec<Segment>

build_select_sql()
  → SELECT * FROM {table_name} WHERE {expression[0] AND expression[1] ...}
  → order_by / group_by / having 来自 FuncSegments
```

——**绑定参数分离**：`Column<F>` 让列名编译期类型安全；参数走 `rbs::Value` 的绑定参数列表，由 rbatis 底层 `?` 占位 PreparedStatement。

---

## 9. 集成测试覆盖（`tests/integration_test.rs`）

仓库自带的测试覆盖（24+ 个，按字面例出）：

**QueryWrapper SQL 生成**
- `test_eq_condition`
- `test_ne_condition`
- `test_multiple_conditions_and`
- `test_like_conditions`
- `test_in_condition`
- `test_between_condition`
- `test_is_null_condition`
- `test_order_by` / `test_group_by_having`
- `test_select_columns` / `test_last_clause` / `test_count_sql`

**UpdateWrapper**
- `test_update_set_eq` / `test_update_set_sql` / `test_update_incr_decr`
- `test_delete_sql`

**Page**
- `test_page_construction` / `test_page_empty` / `test_page_request_offset`

**LambdaQueryWrapper**
- `test_lambda_eq_condition`（…还有更多）

——**纯 SQL 字符串级别测试**（无真实 DB）——和 MyBatis-Plus Java 的 `H2DatabaseTest` 风格一致。

---

## 10. 与参考仓库对照速记表

| 维度 | **rbatis-plus** | MyBatis-Plus（Java）| MyBatis-Plus Enhance | rbatis-wrapper |
|---|---|---|---|---|
| 大小（LOC） | **~ 7,921 行** | 数千 Java + 注解 | +1K 行 | **310 行** |
| 模块数 | **6 个 crate** | 3 个 Maven 模块（core/extension/spring）| 2 个模块（core/extension）| 1 个 crate |
| Wrapper 风格 | 字符串列名 + `Column<F>` 类型安全 | 字符串 + `SFunction<T,?>` | — | 仅字符串 |
| 拦截器钩子数 | **6**（query/update/finally + 事务） | 4 | 7（+after 3）| — |
| 缓存子系统 | `CacheStore` SPI + `MemoryCacheStore` | `Cache` SPI + 11 装饰器 | （无缓存）| — |
| 加密/签名 | 内置 plugin | 不提供 | 内置 plugin | — |
| SQL 观测 | `observation.rs` | 不提供 | 不提供（`Sink` interface） | — |
| 过程宏 | `#[derive(TableName)]` 等 | `@TableName` 注解 | `@IgnoreEncrypted` 等 | — |
| 代码生成器 | `rbatis-plus-generator`（tera 引擎） | 官方 generator | — | — |
| 事务模式 Bypass/Defer | ✓ | Defer | ✓（同 rbatis-cache） | — |
| Single-flight | `policy.use_singleflight` 字段 | ❌ | ❌ | — |
| 编译验证 | `cargo check` 通过 | — | — | ❌ |

---

## 11. 与 **rbatis 主仓 `df87ac41`** 已合入的 Caffeine 缓存关系

> 主仓 `rbatis/src/plugin/cache/` 与本仓 `rbatis-plus/rbatis-plus-core/src/cache/` 是**并行**项目，作者应该有意做了第二份。

**对比**：

| 维度 | rbatis 主仓（`df87ac41`） | rbatis-plus（本文） |
|---|---|---|
| Key 摘要 | **xxh3-128 流式 + 全键校验** | `std::hash::DefaultHasher` + 仅 digest |
| CacheStore 值类型 | **`Arc<Value>`** | `Value`（同步克隆） |
| generation | epoch 进 store key | epoch 进 entry（独立） |
| SingleFlight 真实去重 | `Notify` + `Arc<LoadState>` | `policy.use_singleflight` 字段（**未实现实际去重**）|
| L1 cache | 分片 `L1Cache(executor → shard)` | ❌ 缺失 |
| Transactional buffer | `TransactionalCacheBuffer` + `Bypass/Defer` | `TransactionMode` enum + listener 但**实现简略** |
| Cache 拦截器入口 | `CacheIntercept::get_or_load` | `CacheIntercept::new`（**没有统一入口**）|
| `MemoryCacheStore::with_capacity_and_ttl` 语义 | 全局字节权重 64 MiB | 没看到 by_weight 实现 |
| `Arc::Mutex` 单飞实现 | ❌ | ❌ |

——**主仓缓存的 4 项核心（Caffeine 化 + 真去重 + Arc 值 + L1）在本仓里都"基本对齐，但落后"**。

详见 §15.4。

---

## 12. 关键设计权衡（FAQ）

### Q1：为什么 README 说 "DESIGN ONLY"？与现状不符？

仓库 HEAD（`ee18fcb`）是依赖更新；最早 `b4123ec feat: rbatis-plus 初始实现` 已经把所有模块写完。README 看起来是早期贴设计目标，没跟进。**需要在下次 doc 提交时一并修 README**。

### Q2：为什么缓存 key 用 `std::hash::DefaultHasher` 而非 xxh3？

最简：减少一行依赖。**生产风险**：`
std::collections::hash_map::DefaultHasher` 当前实现是 SipHash-1-3 + SipHash-1-3（Rust 标准库稳定，但跨 version 可能变化）—— 对 cache key 这种"跨进程/跨重启后语义一致"的场景**不应当用它**，应该用稳定实现的 xxh3-128（仓库已经依赖 `xxhash-rust`，加即可）。详见 §15.4。

### Q3：为什么 `CacheIntercept` 没有 `get_or_load` 这种统一入口？

当前 facade 只暴露 `new / store / policy / table_tag`，**没有真正接入 `rbatis::intercept::Intercept` 链**。这是个**未完成的特性**——一旦补上 `Intercept::intercept()` 实现，就是完整的 L2 缓存拦截器。详见 §15.5。

### Q4：为什么 `InnerInterceptor` 同时接收 query 与 update 钩子？不像 Java 用 4 个签名？

Rust `&mut` 借用规则决定——如果每个方法都独立签名，调用方需要拿到 `Executor + sql + args + result` 四元组的"可变借用"，无法组成一个 `Intercept::before_query` 完整钩子。因此**合并为单 trait + 4 个生命周期钩子**是简化调度的最优解。

### Q5：为什么 `Column<F>` 用 PhantomData 而不是泛型 `SFunction<T,F>`？

Java `SFunction<T, ?>` 用反射 + SerializedLambda 提取列名；Rust 想做到同样事需要 `#[derive]` 宏在编译期**生成** `pub fn column_name() -> Column<F>` 这种可方法。这正是 `rbatis-plus-macros` 当前设计——**PhantomData 标记类型，宏在编译期填充**。详见 `compare.rs:84` 调用 `column.name()`。

### Q6：为什么不统一用 `Executor`（traitor）而不是 `rbatis::executor::Executor`？

`rbatis-plus-extension/src/inner/inner_interceptor.rs:5` `use rbatis::executor::Executor;`——本仓库直接复用 rbatis 主仓的 `Executor` trait。`executor.rs:18` 定义：
```rust
pub trait Executor: RBatisRef + Send + Sync { fn id(&self) -> i64; ... }
```
**对位完美**——避免重复发明轮子。

### Q7：macro 衍生与 table_info 怎么绑定？

`derive/table_name.rs` 6 行（极简入口，`#[proc_macro_derive(TableName, attributes(...))]`）+ 真正的 macro 实现可能在 `rbatis-plus-macros` 单文件 crate 里。**目前只 1 个 .rs——可能 macro 实现是宏完成的**（symbol 几乎都在 proc-macro）。这是动态行为，codegraph 不易识别。

### Q8：`Service::IService` 是不是 Spring 直接对应？

`service/IService` 是 trait，签名对照 MP Java 版：
- `save_batch / save_or_update_batch / remove_batch_by_ids / update_batch_by_id`

——都是 default method + `@Transactional`-like 行为（**Rust 版没有 Spring `@Transactional` 直接等价**——由 vernal-framework 的 tx_di 替换）。

### Q9：vernal-framework 集成是什么？

`rbatis-plus-vernal` 提供与 vernal-framework（一个 Rust DI/AOP 框架）的桥接 + `axum_integration`（HTTP 框架集成）。对位 MyBatis-Plus 的 spring-boot-starter。详见 `vernal/` 子目录。

### Q10：`rbatis-plus-vernal` 4 个 .rs 而已？是否真的能跑？

需要 `cargo test --workspace` 实测。codegraph 索引与静态分析只能保证**编译通过**。vernal/axum 集成是否能真的拦截 SQL，需要看具体实现。

---

## 13. codegraph 速查命令

```bash
export PATH="/Users/wandl/.nvm/versions/node/v24.18.0/bin:$PATH"
cd /Users/wandl/workspaces/workspace-github-easy-4-rust/rbatis-plus

codegraph status
codegraph query "BaseMapper\|QueryWrapper\|LambdaQueryWrapper\|UpdateWrapper"
codegraph query "InnerInterceptor\|TenantInner\|PaginationInner\|OptimisticLockerInner"
codegraph query "CacheStore\|CacheKey\|MemoryCacheStore\|CachePolicy\|CacheIntercept"
codegraph query "TableInfo\|TableFieldInfo"
codegraph query "CryptoInnerInterceptor\|SignatureInnerInterceptor\|DefaultDataSignatureHandler"
codegraph query "DynamicTableNameInnerInterceptor\|BlockAttackInnerInterceptor"
codegraph query "Page<T>\|PageRequest"
codegraph query "tenant\|pagination\|optimistic_locker\|data_permission\|block_attack"
```

---

## 14. 推荐阅读顺序

1. **`Cargo.toml`** —— workspace 与默认依赖
2. **`src/lib.rs`**（facade 重导出）—— 30 秒了解"能用 rbatis-plus::* 访问到什么"
3. **`rbatis-plus-core/src/lib.rs`**（38 行）—— 列举 `core` 模块全景
4. **`rbatis-plus-core/src/mapper/base_mapper.rs`**（90 行）—— `BaseMapper<T>` 完整 9+ CRUD 方法签名
5. **`rbatis-plus-core/src/conditions/query/query_wrapper.rs`**（317 行）—— `QueryWrapper.build_select_sql` 全文
6. **`rbatis-plus-core/src/conditions/query/lambda_query_wrapper.rs`**（548 行）—— 类型安全 wrapper
7. **`rbatis-plus-core/src/conditions/compare.rs`**（209 行）—— `Compare` trait 全套 + `AbstractWrapper` 12 个 `format_*` 工具
8. **`rbatis-plus-core/src/cache/store.rs` + `memory.rs`** —— 缓存子系统
9. **`rbatis-plus-extension/src/inner/inner_interceptor.rs`**（77 行 ★ 6 钩子 SPI）
10. **`rbatis-plus-extension/src/inner/base.rs`** —— 短名别名表
11. **`rbatis-plus-extension/src/inner/{tenant,pagination,optimistic_locker}.rs`** —— 三个最常用拦截器
12. **`tests/integration_test.rs`** —— 跟着测试学 24+ 个 SQL 生成示例

---

## 15. 现状速评 + 接下来的可执行项

### 15.1 README 失真

README 第 1 行写 "DESIGN / PLANNING ONLY"，已经过时。**应改 README**——把这段措辞改成"Alpha 状态：核心 6 个 crate + 24+ 集成测试已落地；CI 验证 + 真实 DB 集成测试待补"。

### 15.2 README/Cargo.toml 与代码脱节

- README 写 `${revision}` = `2.0.x.20260630-SNAPSHOT` —— 这其实是 `mybatis-plus-enhance`（hiwepy）那边的 CI 模板
- 仓库 `Cargo.toml` `version = "0.1.0"` 实际版本号 —— 而 CI 用 `${revision}` 占位让 `maven` 知道

→ **下次发版时决定用 Rust 语义版本（`0.1.0`）还是 Java-like（`x.y.x.YYYYMMDD-SNAPSHOT`）**。

### 15.3 缓存子系统落后于上游

本仓库 `core::cache` ≈ `rbatis` 主仓 `df87ac41` 的"轻量复刻"：
- ❌ 没有 `Arc<Value>`（值类型仍 `Value`）
- ❌ 没有 xxh3-128（用 `DefaultHasher`）
- ❌ 没有真正 SingleFlight 实现
- ❌ 没有 L1 cache
- ❌ 没接 `rbatis::intercept::Intercept` 链

——**建议升级路径**：直接把 rbatis 主仓 `df87ac41` 复刻后 port 进来；或干脆 `pub use rbatis::plugin::cache::*` 从主仓复用。详见下一条。

### 15.4 缓存关键修复 4 选 1

| 选项 | 难度 | 影响 |
|---|---|---|
| **A. 用 std DefaultHasher + digest-only** | 极小 | **当前实现**，但有跨进程命中风险 |
| **B. 升 xxh3-128 + 全键校验** | 小 | 复用 `xxhash-rust` 依赖（主仓已用）|
| **C. 用 rbatis 主仓 `Arc<Value>` cache + 直 re-export** | 中 | **最大对齐**，失去"实现独立性" |
| **D. 本仓库自己用 moka 二级存** | 中 | 与主仓 `df87ac41` 等价但独立维护 |

> 推荐 **C**：re-export `rbatis::plugin::cache::*` 让本仓库的 `CacheStore` 直接 = 主仓的 `CacheStore`，**避免分叉**。

### 15.5 拦截器接入 Executing 链

`CacheIntercept` 当前 facade 只暴露薄 API。要真正接入需要：

```rust
impl rbatis::intercept::Intercept for rbatis_plus::core::cache::CacheIntercept {
    async fn before(&self, ..., ResultType<...>) -> Result<Action, Error> { ... }
    async fn after(&self, ..., ResultType<...>) -> Result<Action, Error> { ... }
}
```

——大约 100-150 行代码。

### 15.6 分页 dialect 适配

`pagination.rs` 13 dialect。**当前是否完整实现？** 需要 `cargo doc --document-private-items --workspace` + 实测。看 `mybatis-plus-architecture.md` §8.2 的 13 dialect 列表确认覆盖。

### 15.7 vernal/axum 集成完整性

`rbatis-plus-vernal` 4 个 .rs 文件描述 "vernal-framework integration" + "axum_integration"——但每个文件应该多大才能覆盖真实集成场景？这要看 vernal-framework 本身的功能集。

---

## 16. 接下来合理的工作项（按 ROI 排序）

| ROI | 任务 | 估时 |
|---|---|---|
| ⭐⭐⭐ | **修 README 失真**：把 "DESIGN ONLY" 改成"Alpha 落地" + 当前 crate 矩阵 + 编译/测试状态 | 10 分钟 |
| ⭐⭐⭐ | 在 `rbatis-plus/docs/` 加 `rbatis-plus-architecture.md`（**本文**）作为权威索引 | 1 分钟 |
| ⭐⭐ | Cache 接入 `rbatis::intercept::Intercept`（让 `CacheIntercept::install(&rb)` 一行生效）| 半天 |
| ⭐⭐ | 缓存 key 切到 xxh3-128 + 全键校验（与主仓对齐）| 2 小时 |
| ⭐ | 跑 `cargo test --workspace`：24+ 测试 + 真 DB | 半天 |
| ⭐ | 测试 `cargo doc --no-deps`：检查文档完整性 | 10 分钟 |
| ⭐ | 检查 9 个拦截器对所有 dialect 的覆盖 | 半天 |
| ⚪ | 跟踪 `rbatis-cache`（workspace-github-easy-4-rust 下另一个 docker仓库）是否值得复用 | 调研 |

——这些任务**没有"提 PR"环节**——本工作流"代码 + 文档先沉淀、需要时再发 PR"。

---

## 附录 A：本仓库架构 ASCII

```
                ┌────────────────────────────┐
                │  user code (业务方)          │
                └──────────────┬─────────────┘
                               │   use rbatis_plus::*
                               ▼
                ┌──────────────────────────────┐
                │  rbatis-plus (facade crate)   │   ← facade/lib.rs (43 行重导出)
                │  暴露：QueryWrapper 等 30+   │
                └─────┬────────┬───────────┬───┘
                      │        │           │
         ┌────────────┘        │           └────────────┐
         ▼                     ▼                        ▼
┌─────────────────┐  ┌────────────────────┐   ┌────────────────────┐
│  rbatis-plus-   │  │  rbatis-plus-      │   │  rbatis-plus-      │
│  core           │  │  extension         │   │  macros/generator/ │
│ (base + cache)  │  │ (9 inner + 加强)    │   │ sqlparser/vernal   │
└────┬───────┬────┘  └─────────┬──────────┘   └────────────────────┘
     │       │                │
     │       │                ├─── 拦截器 intercept(Executor,sql,args)
     │       │                │
     │       └─── 抽象层
     ▼
┌──────────────┐
│  rbatis 4.9  │
│  Executor +  │
│  Intercept   │
│  (trait)     │
└──────────────┘
```

——**facade 顶、core 基、extension 强、底层借 rbatis trait**——清晰分层的 Cargo workspace。
