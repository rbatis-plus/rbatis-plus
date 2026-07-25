# RBatis Plus 事务与缓存一致性规范

> 状态：目标设计草案，尚未实现  
> 文档日期：2026-07-24  
> 上游基线（当前）：RBatis `master@4050edd3dad03a113b8bb4f5818a006f11f2da78`  
> 上游基线（旧）：RBatis `master@2df418feeab511c1899b2a110eef43228a1ad889`  
> 最新调研提交标题：`Limit decode fallback to single-column values`

## 0. 文档索引

| # | 文档 | 作用 |
| - | --- | --- |
| 0 | [`/README.md`](../README.md) | 项目门面 + Mermaid 总图 + 文档索引 |
| 1 | [`docs/RBatis 支持二级缓存调研报告.md`](./RBatis%20支持二级缓存调研报告.md) | 上游证据基线 |
| 2 | [`docs/ARCHITECTURE.md`](./ARCHITECTURE.md) | 分层与组件架构 |
| 3 | [`docs/IMPLEMENTATION_PLAN.md`](./IMPLEMENTATION_PLAN.md) | 实施计划、PR 系列、测试矩阵 |
| 4 | [`docs/CACHE_SPECIFICATION.md`](./CACHE_SPECIFICATION.md) | Key/Envelope/Codec/Tag 协议 |
| 5 | `docs/TRANSACTION_CONSISTENCY.md` | 本文档；事务语义 |
| 6 | [`docs/DECISIONS.md`](./DECISIONS.md) | 架构决策记录 |
| 7 | [`docs/INTEGRATION_GUIDE.md`](./INTEGRATION_GUIDE.md) | 集成与门禁模板 |
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](./OBSERVABILITY_SECURITY_OPERATIONS.md) | 可观测 / 安全 / 运维 |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](./TEST_AND_ACCEPTANCE_PLAN.md) | 验收计划与发布门 |

## 1. 目的与边界

本文定义 RBatis Plus 二级缓存面对普通执行器和事务执行器时的读、回填、写入失效、提交、回滚与异常语义。

当前仓库没有 Rust 实现。RBatis 上游基线尚未向缓存插件提供本文所需的完整显式事务上下文和可靠的 commit/rollback 生命周期事件。本文中的事件、状态机、类型和 API 都是目标草案；没有这些核心能力时，只能实现本文的**保守模式**，不能宣称满足**原生事务模式**。

缓存键、`rbs::Value`、envelope、tags、singleflight 和 stale-fill prevention 见 [CACHE_SPECIFICATION.md](./CACHE_SPECIFICATION.md)。整体设计、实施阶段、验收和决策分别见 [ARCHITECTURE.md](./ARCHITECTURE.md)、[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)、[TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md) 和 [DECISIONS.md](./DECISIONS.md)。现状依据见 [RBatis 支持二级缓存调研报告](./RBatis%20支持二级缓存调研报告.md)。

本文不替代数据库隔离级别、锁、幂等、outbox 或分布式事务。缓存只能避免它自身扩大可见性错误；它不能让数据库本身不提供的隔离语义变得更强。

## 2. 术语与不变量

### 2.1 术语

- **普通操作**：不属于显式数据库事务的 query/exec。
- **事务操作**：带可靠 `transaction_id` 且位于 begin 与终态之间的 query/exec。
- **共享 L2**：可能被其他连接、事务或进程读取的二级缓存。
- **pending invalidation**：事务内成功 DML 产生、等待事务终态处理的失效意图。
- **原生模式**：核心提供可靠事务身份和生命周期事件，commit 成功后失效。
- **保守模式**：缺少可靠生命周期事件时，事务不读写共享缓存，事务 DML 成功后立即失效。
- **数据库成功**：数据库驱动返回成功；对事务内 DML，这只表示语句成功，不表示事务已提交。
- **提交成功**：数据库 commit 操作明确返回成功。
- **终态未知**：连接丢失或超时导致客户端无法确定数据库是否提交。

### 2.2 必须保持的不变量

1. 未提交查询结果不得写入共享 L2。
2. 事务内 DML 成功不得被误当作事务提交成功。
3. rollback 成功不得执行仅由该事务产生的延迟失效。
4. commit 成功后必须处理该事务累计的全部失效意图，不能只处理最后一个语句。
5. cache hit 或 fill 不得跨 tenant、datasource、shard 或 tag generation。
6. 失效后开始之前的旧查询不得回填到新 generation。
7. 不能可靠识别事务时必须 bypass/保守处理，不能通过 `Executor::name()` 字符串或 `task_id != 0` 猜测。
8. 任何不确定语义必须显式降级和可观测，不能静默宣称一致。

## 3. 所需核心上下文与事件

原生模式至少需要稳定暴露以下逻辑信息：

```rust
pub struct ExecutionContext<'a> {
    pub executor_kind: ExecutorKind,
    pub operation_kind: OperationKind,
    pub task_id: i64,
    pub transaction_id: Option<TransactionId>,
    pub statement_id: Option<&'a str>,
    pub datasource_id: &'a str,
}

pub enum TransactionEvent {
    Began { tx_id: TransactionId },
    CommitSucceeded { tx_id: TransactionId },
    CommitFailed { tx_id: TransactionId, outcome: CommitOutcome },
    RollbackSucceeded { tx_id: TransactionId },
    RollbackFailed { tx_id: TransactionId, outcome: RollbackOutcome },
    Abandoned { tx_id: TransactionId },
}
```

这些签名只是表达所需语义，不是已存在 API。具体枚举、错误携带、异步回调方式、guard drop 行为和兼容层均为 TBD。

事件必须满足：

- `transaction_id` 在一个事务生命周期内唯一且稳定；
- begin 事件先于该事务的 query/exec；
- commit-success 只能在数据库确认成功后发出；
- rollback-success 只能在数据库确认成功后发出；
- 每个事务至多有一个终态通知；重复通知必须幂等；
- 连接错误、guard drop 和自动回滚路径必须有明确事件；
- 插件初始化晚于 begin 或在终态前被移除的行为必须禁止或定义；
- 生命周期事件失败不能篡改数据库已经发生的终态。

## 4. 支持模式

### 4.1 Native mode（原生模式）

前置条件：可靠的 executor kind、transaction ID、commit/rollback 成功事件、终态不明事件，以及可测试的回调顺序。

语义：

- 事务查询默认 bypass 共享 L2；
- 事务查询结果不回填共享 L2；
- 事务 DML 成功只累计 pending invalidation；
- commit 成功后执行累计失效；
- rollback 成功后丢弃累计失效；
- 终态未知时执行保守失效；
- commit 后缓存失效失败按配置报告降级，不能“撤销”已经成功的数据库提交。

原生模式是目标生产语义，但它不等于事务内 query cache。首版原生模式仍建议 transaction read bypass。

### 4.2 Conservative mode（保守模式）

适用于当前上游基线不能可靠通知事务终态的阶段。

语义：

- 能可靠识别为事务的查询必须 bypass 共享 L2；
- 不确定是否为事务的路径应整体 bypass，而非冒险使用缓存；
- 事务 DML 语句返回成功后立即失效显式 tags 或整个 namespace；
- rollback 不恢复已推进的 generation；
- 因而不会把未提交结果写入共享缓存，但回滚事务会造成不必要失效和命中率下降；
- 如果甚至不能可靠识别事务 DML，则缓存能力必须限制为手动失效/短 TTL，或禁止该执行路径使用缓存。

保守模式不得标记为“commit-aware”“transactionally precise”或同义能力。

### 4.3 模式选择

模式必须显式配置或由编译期可验证能力选择。不得运行时静默从原生模式退化为保守模式。若原生事件通道失效，插件必须按配置：

- 拒绝启动；或
- 明确进入 degraded 状态、停止使用共享缓存并报警。

最终配置 API 和默认模式为 TBD。在当前无实现状态下，文档只能推荐语义，不能承诺默认值已存在。

## 5. 普通操作读写矩阵

### 5.1 普通查询矩阵

| 场景 | 读共享 L2 | 查数据库 | 回填共享 L2 | 说明 |
|---|---:|---:|---:|---|
| policy 未启用 | 否 | 是 | 否 | 缓存默认是显式能力 |
| fresh hit | 是 | 否 | 否 | 返回 envelope 中的 owned `rbs::Value` |
| miss，DB 成功，generation 未变 | 是 | 是 | 是 | 回填前执行 stale-fill 检查 |
| miss，DB 成功，generation 已变 | 是 | 是 | 否 | 返回 DB 结果，丢弃旧回填 |
| miss，DB 失败 | 是 | 是 | 否 | DB 错误不得缓存 |
| cache get 失败，FailOpen | 尝试 | 是 | 视后端状态 | 必须记录降级 |
| cache get 失败，FailClosed | 尝试 | 否 | 否 | 返回缓存错误；是否允许由产品决定 |
| `FOR UPDATE`/锁查询 | 否 | 是 | 否 | 默认强制 bypass |
| 非确定性或依赖不完整 | 否 | 是 | 否 | 除非显式安全策略 |
| stale entry，未启用 SWR | 否 | 是 | 条件回填 | stale 视为 miss |
| stale entry，启用 SWR | 可选 | 后台/leader | 条件回填 | 功能和一致性验收尚 TBD |

### 5.2 普通写入矩阵

| DB exec 结果 | 失效行为 | 对调用方结果 | 一致性说明 |
|---|---|---|---|
| 失败 | 不失效 | 返回 DB 错误 | 没有已确认写入 |
| 成功，失效成功 | 推进 tags/namespace generation | 返回成功 | 后续键使用新 generation |
| 成功，失效失败，FailOpen | 记录严重降级并返回 DB 成功 | 返回成功 | 旧缓存可能存活到 TTL/外部修复 |
| 成功，失效失败，FailClosed | 返回复合/缓存错误 | 返回错误，但 DB 写入已发生 | 不能声称写入回滚；调用方重试有重复写风险 |
| DB 结果未知 | 保守失效 | 返回原始未知结果 | 假定写入可能已发生 |

对“数据库已成功但缓存失效失败”不存在通用的原子回滚。默认产品建议可用性优先时采用 FailOpen、短 TTL、报警与重试队列；需要更强保证时必须设计数据库 outbox/CDC，而不是把 cache error 伪装成数据库事务回滚。

## 6. 事务操作读写矩阵

### 6.1 原生模式

| 事务内操作 | 读共享 L2 | 写共享 L2 | pending invalidation | 说明 |
|---|---:|---:|---:|---|
| 普通 SELECT | 否 | 否 | 不变 | 保留 read-your-writes 和数据库隔离语义 |
| 重复 SELECT | 否 | 否 | 不变 | 首版不提供事务本地 query cache |
| SELECT FOR UPDATE/SHARE | 否 | 否 | 不变 | 必须直达数据库 |
| DML 失败 | 否 | 否 | 不新增 | 返回 DB 错误，事务是否可继续由驱动/DB 决定 |
| DML 成功，显式 flush tags | 否 | 否 | 合并 tags | 等待终态 |
| DML 成功，依赖未知 | 否 | 否 | 合并保守 namespace | 不做 SQL 猜测 |
| statement success 后再次 query | 否 | 否 | 保留 | 由数据库事务快照决定结果 |
| commit 成功 | 不适用 | 不适用 | 执行并清除 | 见第 8 节 |
| rollback 成功 | 不适用 | 不适用 | 丢弃 | 不推进 generation |

### 6.2 保守模式

| 事务内操作 | 读共享 L2 | 写共享 L2 | 失效行为 | 说明 |
|---|---:|---:|---|---|
| SELECT | 否 | 否 | 无 | 必须 bypass |
| 锁查询 | 否 | 否 | 无 | 必须直达 DB |
| DML 失败 | 否 | 否 | 不失效 | 已知语句失败 |
| DML 成功 | 否 | 否 | 立即失效 | 即使未来 rollback 也不恢复 generation |
| DML 结果未知 | 否 | 否 | 保守立即失效 | 写入可能已发生 |
| commit | 不适用 | 不适用 | 无额外精确动作 | 因为缺少可靠事件或已经失效 |
| rollback | 不适用 | 不适用 | 不恢复缓存 | 避免复活旧值 |

## 7. Pending invalidation 数据模型

每个原生事务必须有独立状态。逻辑草案：

```rust
pub struct PendingInvalidation {
    pub tx_id: TransactionId,
    pub datasource_id: DatasourceId,
    pub namespaces: OrderedSet<CacheNamespace>,
    pub tags: OrderedSet<CacheTag>,
    pub reason: InvalidationReason,
    pub state: PendingState,
}
```

最终容器类型、持久化和有序集合实现为 TBD。

要求：

- key 必须至少包含 cache instance/domain、datasource 和 transaction ID；
- 多次写入合并去重，namespace 失效可以覆盖同 namespace 下的具体 tags；
- 不同 datasource 的意图不得合并；
- 每事务 tags、namespace 和内存字节数必须有上限；
- 超限时升级为更保守的 namespace 失效，而不是丢弃后续 tags；
- 事务状态不能只按 `task_id` 保存，除非核心契约明确它就是稳定且唯一的 transaction ID；
- pending 状态必须在所有终态、超时和 shutdown 路径清理；
- `tx_id` 重用不得关联到旧 pending 状态。

仅在插件状态位于 `Active` 时，事务内成功 DML 才能增加失效意图。若 DML 本身失败，不增加该语句的意图；此前成功语句的意图仍保留，因为事务可能由调用方继续并提交。

## 8. Pending invalidation 状态机

```text
                         DML success
                    +--------------------+
                    |                    v
Begin -> Active --------------------> Active
          |  DML failure                |
          |  (keep prior intents)       | commit requested
          |                             v
          |                        CommitInFlight
          |                         /     |      \
          |          success ------      |       ------ unknown
          |             v                |               v
          |      Invalidating         failed-known   OutcomeUnknown
          |         /   \                |               |
          |   success   failure          v               v
          |      v         v        CommitFailed   ConservativeInvalidating
          |  Committed  CommittedWithInvalidationFailure
          |
          | rollback requested
          v
    RollbackInFlight
       /      |       \
 success   failed-known  unknown
    v          v           v
RolledBack  RollbackFailed OutcomeUnknown
                           |
                           v
                  ConservativeInvalidating
```

### 8.1 状态定义

| 状态 | 含义 | 可接受操作 |
|---|---|---|
| `Active` | 事务已开始且未请求终态 | query bypass、合并 DML 意图 |
| `CommitInFlight` | commit 已发起，结果尚未确定 | 不接受新语句；等待结果 |
| `Invalidating` | DB commit 已确认，正在推进 generation | 幂等重试失效 |
| `Committed` | commit 与失效均成功 | 清理状态 |
| `CommittedWithInvalidationFailure` | DB 已提交，失效未完全成功 | 报警、重试/修复；不得回滚事实 |
| `CommitFailed` | DB 明确未提交或 commit 明确失败 | 根据 outcome 决定保留/回滚；不得凭错误类型猜测 |
| `RollbackInFlight` | rollback 已发起 | 不接受新语句 |
| `RolledBack` | DB rollback 已确认 | 丢弃意图并清理 |
| `RollbackFailed` | rollback 明确失败但最终数据库状态需分类 | 进入保守处理或保持待人工处置 |
| `OutcomeUnknown` | 客户端无法判断 commit/rollback 结果 | 保守失效，不得丢弃为“已回滚” |
| `ConservativeInvalidating` | 为未知终态推进累计 generation | 幂等重试 |

最终 enum 名称可以变化，但语义不得弱化。

### 8.2 状态转换要求

- begin 重复通知必须被拒绝或幂等识别同一事务；
- `Active -> CommitInFlight` 后不能再接受 query/exec；
- commit 成功事件只能触发一次逻辑失效；底层 generation advance 必须支持幂等命令 ID 或等价机制；
- rollback 成功只丢弃该事务 pending，不能恢复任何此前已推进的 generation；
- 终态未知必须选择可能产生多余失效但不会保留潜在旧值的路径；
- 插件进程崩溃后若 pending 仅在内存中，commit 成功事件和失效意图可能同时丢失。这是原生模式的关键耐久性限制，必须通过第 11 节方案之一处理或明确降级。

## 9. Commit 语义

### 9.1 Commit 成功

正确顺序：

```text
1. 数据库 commit 返回明确成功。
2. 核心发出 CommitSucceeded(tx_id)。
3. 缓存协调器冻结并取得该 tx 的 pending invalidations。
4. 按 datasource/cache domain 合并并推进 generation。
5. 成功后标记 Committed 并清理 pending。
```

不能在数据库 commit 之前推进 generation 作为原生模式的正常路径，否则 rollback 会造成不必要失效。也不能在 commit 成功事件之后继续接收该事务 DML。

如果 pending 为空，commit-success 仍需清理事务状态，但不访问后端。

### 9.2 Commit 明确失败

“commit 返回错误”不自动等于“数据库保证未提交”。错误分类必须来自驱动/核心明确契约：

- **KnownNotCommitted**：可保留 pending 以等待显式 rollback，或按事务已终止规则清理；不得失效为必要条件；
- **OutcomeUnknown**：必须保守失效全部 pending；
- **ConnectionLost/Unclassified**：默认归入 OutcomeUnknown。

错误分类 API 当前不存在，属于 TBD。实现前若无法可靠分类，所有 commit failure 都必须按 outcome unknown 处理。

### 9.3 Commit 后失效失败

数据库 commit 已经不可由缓存层撤销。处理顺序：

1. 将状态记为 `CommittedWithInvalidationFailure`；
2. 记录 datasource、namespace/tag 安全标识和幂等 command ID；
3. 按有界退避重试；
4. 必要时暂停该 namespace 的 cache reads 或推进更粗粒度 epoch；
5. 向调用方返回何种结果由产品策略决定，但错误必须说明“数据库已提交，缓存失效失败”。

禁止返回一个看起来像数据库 rollback 的普通错误。调用方若盲目重试可能重复写入，因此公开错误类型必须携带 commit outcome。

## 10. Rollback、drop 与失败语义

### 10.1 Rollback 成功

rollback 明确成功时：

- 丢弃该事务 pending invalidations；
- 不推进 generation；
- 不写共享缓存；
- 清理 singleflight/事务附属状态；
- 重复 rollback-success 事件应幂等。

### 10.2 Rollback 失败

- 若驱动明确保证事务未提交且连接将关闭，可以清理 pending，但该保证必须写入适配器契约；
- 若最终状态未知，推进 pending 的保守失效；
- 不得因为方法名是 rollback 就假定失败时数据库一定未写入。

### 10.3 Guard drop 与自动回滚

RBatis transaction guard 的 drop/自动回滚路径必须在核心层映射成可观察的终态。Rust `Drop` 不能直接可靠执行任意 async 回调，因此具体设计为 TBD。可选方案包括显式异步关闭、由连接层发出终态事件，或后台终结任务。

在没有可靠完成通知时：

- 该路径不满足原生模式；
- 必须保守失效 pending，或禁止事务 guard 与缓存原生模式组合；
- 不能仅在内存 map 超时后静默删除 pending。

### 10.4 进程终止

如果事务 pending 只保存在进程内：

```text
DB commit succeeds -> process crashes -> invalidation never runs
```

将留下旧缓存直到 TTL。这意味着“commit-aware”不等于“crash-safe”。生产实现必须在能力声明中区分两者。

## 11. 耐久失效选项

实现必须在 [DECISIONS.md](./DECISIONS.md) 中选择一致性等级。

### 11.1 Best-effort commit hook

- pending 在内存；
- commit 成功后同步/异步失效；
- 崩溃窗口依赖 TTL 修复。

这是最简单方案，只能声明进程存活期间的 commit-aware 行为。

### 11.2 Durable invalidation journal

- commit 前或与业务事务配合持久记录失效意图；
- commit 后 worker 幂等推进 generation；
- 需要清理、重试、顺序和重复消费设计。

若 journal 不与数据库事务原子写入，仍存在双写窗口。

### 11.3 Transactional outbox / CDC

- 在业务数据库事务内写 outbox，或订阅 binlog/CDC；
- commit 后消费者推进 cache generation；
- 可以覆盖进程崩溃与部分外部写入；
- 引入数据库 schema、消费者和运维依赖。

该方案最强，但不应作为核心 crate 的隐式依赖。具体 outbox schema、CDC 平台和交付保证均为 TBD。

## 12. 失效命令的幂等与顺序

简单的 `INCR generation` 如果同一 commit 因重试执行两次，会产生额外 generation。额外推进不会复活旧值，但会降低命中并可能破坏审计一致性。目标后端应该支持：

```text
apply_invalidation(command_id, targets)
```

其中 `command_id` 对事务终态唯一。后端原子地记录已处理 command 并推进 targets，重复命令返回已处理结果。dedupe 记录的 TTL/容量不能短于最大重试窗口；具体方案为 TBD。

不同事务对同一 tag 的失效允许任意串行顺序，只要 generation 单调。批量 targets 应具有明确原子边界：

- 若后端能全量原子推进，则一次应用；
- 若只能逐目标推进，部分成功必须可恢复，且 cache reads 应避免把“未推进部分”误认为整体一致；
- Redis Cluster 跨 slot 原子性未解决前必须声明限制或使用 namespace 级单 slot 策略。

## 13. 与 stale-fill prevention 的交互

普通查询在开始数据库访问前读取 dependency generation snapshot。事务 commit 或保守失效推进 generation 后，旧查询的 compare-and-publish 必须失败。

关键竞争矩阵：

| 时序 | 允许返回给原查询 | 允许写共享 L2 |
|---|---|---|
| 查询完成并回填后，写事务 commit | 查询时结果 | 是；commit 随后推进 generation，使旧键不可达 |
| 写事务 commit 后，旧查询尝试回填 | 查询时结果 | 否；snapshot 不匹配 |
| 事务 DML 未 commit，外部普通查询 | 按 DB 隔离结果 | 仅按当前 generation 条件回填 |
| outcome unknown 后旧查询回填 | 查询时结果 | 否；必须先保守推进 generation |
| rollback 成功后旧查询回填 | 查询时结果 | 可以，前提是 generation 未被其他写推进 |

失效和回填的原子细节由 [CACHE_SPECIFICATION.md](./CACHE_SPECIFICATION.md) 定义。事务规范要求 commit 处理必须在向调用方宣称“缓存一致性动作完成”之前完成，除非配置明确选择异步 best-effort。

## 14. 外部写入与多节点

仅观察 RBatis 事务不能覆盖：

- 其他服务或语言客户端；
- 数据迁移和手工 SQL；
- 触发器/存储过程中的间接表写入；
- 管理工具和定时任务。

因此：

- 内存后端在多进程中默认只是局部一致；
- Redis 共享 generation 能让所有使用同一协议的节点看到失效，但仍看不到绕过协议的写入；
- Pub/Sub 不是持久事实源；
- 生产部署至少需要短 TTL、统一写入口、管理失效 API、outbox 或 CDC 中的一项；
- 对外一致性声明必须写明覆盖的写入来源。

## 15. 故障处理矩阵

| 故障点 | 原生模式动作 | 保守模式动作 |
|---|---|---|
| 无法取得 transaction ID | 停止/降级，不得猜测 | 整条路径 bypass，必要时 namespace 清理 |
| pending state 写入失败 | 令 DML 路径失败或立即保守失效 | 立即保守失效 |
| DB query 失败 | 不回填 | 不回填 |
| DB DML 失败 | 不增加新 pending | 不失效 |
| commit outcome unknown | 保守失效 pending | 已立即失效；再次失效须幂等 |
| rollback outcome unknown | 保守失效 pending | 已立即失效或保守失效 |
| cache invalidation timeout | 标记 commit 已发生、失效失败；重试/禁读 | 报警、重试/禁读 |
| process restart 丢失 pending | 按所选耐久等级处理；best-effort 依赖 TTL | 已执行的即时失效保留，尚未执行部分依赖 TTL |
| duplicate terminal event | 幂等拒绝或返回已处理 | 幂等处理 |
| transaction state 超限 | 升级为 namespace pending | 立即 namespace 失效 |

## 16. 可观测性与运维

至少需要：

- active transactions 和 pending target 数；
- pending state 年龄、内存和超限降级次数；
- commit/rollback outcome 分类；
- commit-to-invalidation 延迟；
- invalidation retry、partial failure 和 terminal failure；
- conservative mode immediate invalidation 次数；
- transaction query bypass 次数；
- outcome unknown 次数；
- crash recovery/outbox lag（如启用）；
- generation mismatch 导致的 stale-fill discard。

日志必须带安全的 transaction correlation ID、datasource ID、mode 和 command ID。不得记录 SQL 参数、缓存 payload 或凭据。告警至少覆盖 commit 后失效持续失败、unknown outcome 激增和 pending 状态无法清理。

## 17. 测试与验收要求

具体自动化安排见 [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md)。本规范要求至少覆盖：

### 17.1 普通操作

- 相同 query hit；写成功后 generation 变化；写失败不失效；
- 失效与慢查询回填竞争，旧 fill 被丢弃；
- cache backend failure 的 FailOpen/FailClosed 可观察结果；
- unknown DML outcome 执行保守失效。

### 17.2 原生事务

- 事务 query 始终 bypass 且不回填；
- 多次 DML 合并并去重 tags；
- DML 失败保留此前成功语句的 pending；
- commit 成功后才失效；
- rollback 成功丢弃 pending；
- commit 失败的 known-not-committed 与 unknown 分支；
- rollback failure/connection loss 的保守分支；
- commit 后失效失败返回“已提交”事实；
- 重复终态事件和重试命令幂等；
- guard drop、自动 rollback 和 executor cancellation；
- pending 超限升级为 namespace；
- transaction ID 重用不读取旧状态。

### 17.3 保守模式

- 事务 query 不读写共享缓存；
- 每次事务 DML 成功立即失效；
- 后续 rollback 不恢复旧 generation；
- 无法识别事务时路径拒绝缓存；
- 文档/指标明确模式不是 commit-precise。

### 17.4 崩溃与多节点

- commit 后、失效前模拟崩溃；
- durable journal/outbox（如实现）的重放；
- Redis 多节点同时失效与回填；
- Pub/Sub 丢消息时共享 generation 仍正确；
- cache backend 分区恢复后的旧 envelope 不可重新可达。

## 18. 能力声明

在发布说明和配置参考中，必须使用可验证的能力术语：

| 声明 | 最低条件 |
|---|---|
| `transaction-bypass` | 可靠识别事务，事务 query 不读写 L2 |
| `conservative-invalidation` | 事务 DML 成功后立即失效，rollback 不恢复 |
| `commit-aware-invalidation` | 可靠 commit-success 事件，commit 后应用 pending |
| `unknown-outcome-safe` | commit/rollback 不明时保守失效 |
| `crash-safe-invalidation` | 持久 journal/outbox/CDC 通过崩溃验收 |
| `multi-process-invalidation` | 共享 generation 或可靠广播，且通过多节点测试 |

未达到最低条件时不得使用对应声明。

## 19. 尚未决定的事项

1. RBatis 核心 V2 context/lifecycle hook 的最终 API 和兼容方式；
2. transaction ID 的生成、唯一范围与重用规则；
3. commit/rollback error 的可靠 outcome 分类；
4. guard drop 与异步自动回滚的通知机制；
5. pending state 是纯内存、journal 还是 outbox；
6. commit 后失效采用同步、异步或可配置语义；
7. invalidation command 的幂等存储与保留窗口；
8. Redis Cluster 下跨 target 原子性；
9. cache invalidation 失败时暂停读缓存的粒度；
10. 是否支持事务本地缓存，以及如何保证 read-your-writes；首版建议不支持；
11. savepoint/nested transaction 语义；在明确生命周期前必须视为不支持或整体按外层事务处理；
12. 分布式事务/两阶段提交语义；本文不承诺支持。

这些事项必须进入 [DECISIONS.md](./DECISIONS.md) 和 [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)。任何实现都应先更新相应文档，再声称行为已稳定。

## 20. MyBatis 对照参考

> MyBatis `TransactionalCache` 是“缓存写入两阶段”，不是数据库 XA/2PC。
> MyBatis 的事务处理围绕 `CachingExecutor` + `TransactionalCacheManager` 展开，提供：
> - 事务期间 `entriesToAddOnCommit` 暂存；
> - commit 时把暂存写回真实 `Cache`；
> - rollback 时丢弃暂存并释放 miss 锁。
> RBatis-Plus 计划在不依赖数据库 2PC 的前提下，复用相同的“缓存写入两阶段”思想，
> 并在 `commit-success` / `rollback` / `unknown-outcome` 三种终结下分别给出缓存动作。
> 本节仅作对照参考，所有 RBatis-Plus 能力均标注为 planned/TBD。
> MyBatis 行号引用本地代码：`/Users/wandl/workspaces/workspace-github/mybatis-3/src/main/java/org/apache/ibatis/...`。
> RBatis 行号引用本地代码：`/Users/wandl/workspaces/workspace-github/rbatis/src/...`。

### 20.1 事务终结的缓存动作

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `CachingExecutor.commit(boolean)` | `CachingExecutor.java:117-131` | `RBatisConnExecutor::begin(self)` 消耗连接，`RBatisTxExecutor::commit` 直接调用底层 | 计划 `TransactionListener::on_commit` 触发应用 `pending invalidation` |
| `CachingExecutor.rollback(boolean)` | `CachingExecutor.java:124-133` | 同上，rollback 不进入 `Intercept` 链 | 计划 `TransactionListener::on_rollback` 触发丢弃 pending |
| `TransactionalCache.commit` | `TransactionalCache.java:94-100, 113-122` | 无 | 计划 `commit-success` 应用 pending |
| `TransactionalCache.rollback` | `TransactionalCache.java:102-105, 124-133` | 无 | 计划 `rollback` 丢弃 pending |

### 20.2 事务内查询

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| 事务内默认查询 L2（除非 `useCache=false`） | `CachingExecutor.query` 行 99-108 | 无 | 计划事务 query bypass L2（事务内仅读 L1） |
| `ensureNoOutParams` | `CachingExecutor.java:135-145` | 无 | 计划显式 OUT 参数校验 |

### 20.3 L1 与本地缓存清理

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `BaseExecutor.update` 清理 L1 | `BaseExecutor.java:110-118` | `PageIntercept` 间接清理 | 计划 hook 在 `before` 失效 |
| `BaseExecutor.commit/rollback` 清理 L1 | `BaseExecutor.java:250-274` | 同上 | 同上 |
| `LocalCacheScope.STATEMENT` | `Configuration.java:568-574` | 无 | 计划 `Statement`/`Session`/`Transaction` 三级 scope |

### 20.4 L2 提交

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| 事务内写暂存 | `TransactionalCache.putObject` 行 78-81 | 无 | 计划 `pending invalidation` 暂存 |
| 事务终结回填 | `TransactionalCache.commit` 行 94-100 | 无 | 计划 `commit-success` 后 `apply_pending` |
| 失败回滚丢弃 | `TransactionalCache.rollback` 行 102-105 | 无 | 计划 `rollback` 丢弃 pending |
| miss 锁释放 | `TransactionalCache.unlockMissedEntries` 行 124-133 | 无 | 计划 stale-fill prevention |

### 20.5 事务后失效语义

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `useCache=false` 跳过 L2 | `MappedStatement.java:276-278` | 无 | 计划 `CachePolicy::bypass` |
| `flushCacheRequired` 强制清空 | `MappedStatement.java:272-274` | 无 | 计划 `evict` / `evict_all` |
| `flushCacheIfRequired` | `CachingExecutor.java:168-173` | 无 | 计划 hook 失效 |
| 外部 DML 清理 | 不支持 | 不支持 | 计划短 TTL、Pub/Sub、binlog CDC、admin invalidate |

### 20.6 事务 outcome 分类

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `commit(boolean required)` 双参 | `Executor.java:48-66` | 同 RBatis `RBatisTxExecutor::commit` | 计划不改变签名 |
| `rollback(boolean required)` 双参 | `Executor.java:51-51` | 同上 | 同上 |
| unknown outcome 处理 | 不区分 | 不区分 | 计划 `unknown-outcome-safe` 能力声明 |
| 失败回滚与事务分离 | `CachingExecutor.close(forceRollback)` 行 54-66 | RBatis `RBatisTxExecutorGuard::drop` 触发 | 计划区分显式失败与 guard 自动 |

### 20.7 对 RBatis-Plus 的直接启示

- `TransactionalCache` 的 `entriesToAddOnCommit` 思想直接对应 `pending invalidation`。
- `TransactionalCacheManager` 按 cache 实例维护事务缓冲，对应计划 `CacheManager::pending_for(tx_id)`。
- 事务终结的两阶段发布思想必须保留，但应增加 unknown-outcome-safe 与 fail-closed 可观察。
- L1 与 L2 都应受 `TransactionListener` 事件驱动，避免依赖 SQL `exec` 拦截。
- 事务内 query 应默认 bypass L2，与 MyBatis 默认行为不同，是 RBatis-Plus 的产品差异化点。

