# RBatis-Plus 架构决策记录

- **日期**：2026-07-20
- **状态**：已接受
- **适用范围**：RBatis-Plus 整体架构与二级缓存设计

---

## ADR-001：上游保持薄，RBatis-Plus 承载缓存产品能力

**状态**：已接受

### 上下文

生产级缓存需要可靠的执行上下文、通用拦截钩子和事务生命周期事件。这些能力对 trace、metrics、tenant、audit 等插件同样有用；缓存 SPI、后端和策略则属于 RBatis-Plus 的产品能力。如果把完整缓存实现放入上游 `rbatis`，会扩大上游 API、依赖和维护范围。

### 决策

上游 `rbatis` 只提供通用、缓存无关的扩展基础，包括执行上下文、操作类型、稳定的 before/after/error 钩子，以及事务 begin/commit-success/rollback 事件。RBatis-Plus 提供 `CacheStore` SPI、`CacheIntercept`、缓存策略、Key/Envelope 协议、内存与 Redis 后端、宏元数据和可观测能力。

### 备选方案

1. 把缓存 SPI、拦截器和后端全部放入上游 `rbatis`
2. 不改上游，仅在 RBatis-Plus 中通过 `task_id`、执行器名称和 SQL 猜测上下文
3. 在 CRUD 宏中分别实现缓存，不建设统一执行层能力

### 后果

- 上游 API 保持通用且依赖较少
- RBatis-Plus 可以独立迭代缓存产品能力
- 生产级事务语义依赖上游接受少量通用增强

### 反转条件

当上游明确决定将二级缓存作为内建产品能力时，重新评估边界。

---

## ADR-002：缓存原始 `rbs::Value`

**状态**：已接受

### 上下文

MyBatis 缓存 `List<POJO>` 或 `Map`。RBatis 的统一中间表示是 `rbs::Value`。缓存 `Value` 而非泛型 `T` 有以下优势：
- 避免序列化/反序列化开销
- 与 RBatis 执行路径自然对齐
- 支持 `selectMaps` 等返回 `Map` 的场景

### 决策

缓存 `rbs::Value`，不缓存泛型 `T`。读取时从 `Value` 解码为目标类型。

### 后果

- 缓存命中时仍需解码（但比 DB 查询快）
- 不同目标类型共享同一缓存条目
- `selectMaps` 和 `selectObjs` 自然支持

---

## ADR-003：版本化标签失效

**状态**：已接受

### 上游文

扫描式失效（遍历所有键检查标签）在高 QPS 下不可接受。版本化失效通过递增标签版本号实现 O(1) 失效。

### 决策

- 每个标签维护一个版本号键：`version:tag:<ns>:<tag> -> u64`
- 缓存键构建时包含标签版本号
- 失效时原子递增版本号（`INCR`）
- 读取时比较版本号，不匹配则视为失效

### 后果

- 失效操作 O(1)
- 缓存键变长（包含版本号）
- 需要 Redis `INCR` 原子操作

---

## ADR-004：FailOpen 默认

**状态**：已接受

### 上下文

缓存后端不可用时的行为选择。FailOpen（继续查询数据库）比 FailClosed（中断请求）更安全。

### 决策

默认 `FailOpen`：缓存后端错误时记录 WARN 日志并继续查询数据库。可配置为 `FailClosed`。

### 后果

- 缓存后端故障不影响业务可用性
- 可能产生短暂的数据库压力增加
- 需要监控缓存错误率

---

## ADR-005：一文件一对象

**状态**：已接受

### 上下文

Java 的类/接口/枚举通常每个占一个文件。Rust 允许在一个文件中定义多个类型。为了保持与 Java 的映射关系清晰，便于方法级审计。

### 决策

每个 `.rs` 文件只对应一个 Java 对象。`mod.rs` 只做模块声明与 re-export，禁止定义类型/逻辑。`lib.rs` 只做 crate 门面，禁止堆放对象。

### 后果

- 文件数量较多
- 映射关系清晰
- 便于方法级审计

---

## ADR-006：复用上游 rbatis 主接口

**状态**：已接受

### 上下文

rbatis 上游已实现 `Intercept`、`CacheStore`、`CacheIntercept`、`TransactionListener` 等接口。重复实现会导致维护负担和兼容性问题。

### 决策

完全复用上游 rbatis 主接口，重导出而非重写。RBatis-Plus 只实现产品层（后端、策略、Key 构建等）。

### 后果

- 减少维护负担
- 与上游保持兼容
- 依赖上游接口稳定性

---

## ADR-007：Rust 化映射规则

**状态**：已接受

### 决策

| Java 概念 | Rust 等价 |
|---|---|
| Jackson | serde |
| Spring Boot | axum (rbatis-plus-vernal) |
| Quarkus | actix (rbatis-plus-vernal) |
| Spring 容器 | vernal 显式注册表 |
| ThreadLocal | Arc 共享上下文 |
| CopyOnWriteHashMap | DashMap |
| CompletableFuture | tokio JoinSet |
| 反射实例化 | trait 对象 + 工厂 |
| JNDI | vernal Provider |

---

## ADR-008：禁止 Git worktree

**状态**：已接受

### 上下文

沙箱环境周期性清空 `$HOME`，已摧毁过 4 个并行 worktree。

### 决策

严格禁止使用 Git worktree。不调用、不模拟、不变相创建。

---

## ADR-009：不动手发 PR 到上游

**状态**：已接受

### 上下文

rbatis / mybatis-plus / mybatis-plus-enhance / rbatis-wrapper 上游有自己的维护节奏和贡献规范。

### 决策

不在当前阶段向上述上游仓库发 PR。上游接口增强通过 fork + feature branch 实现。

---

## ADR-010：模板引擎四选一

**状态**：已接受

### 上下文

MyBatis-Plus Generator 支持多种模板引擎（FreeMarker、Velocity、Thymeleaf 等）。Rust 生态有对应的模板引擎。

### 决策

| Java 模板引擎 | Rust 对应 | 特点 |
|---|---|---|
| FreeMarker | Tera | 功能型 |
| Velocity | Handlebars | 克制型 |
| JSP/Thymeleaf | Askama | 编译期 |
| Twirl/JSX | maud | 编译期 HTML |

四选一，默认 Tera，可配置切换。
