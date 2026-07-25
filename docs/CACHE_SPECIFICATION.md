# RBatis Plus 二级缓存协议规范

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
| 4 | `docs/CACHE_SPECIFICATION.md` | 本文档；Key/Envelope/Codec/Tag 协议 |
| 5 | [`docs/TRANSACTION_CONSISTENCY.md`](./TRANSACTION_CONSISTENCY.md) | 事务读/回填/失效语义 |
| 6 | [`docs/DECISIONS.md`](./DECISIONS.md) | 架构决策记录 |
| 7 | [`docs/INTEGRATION_GUIDE.md`](./INTEGRATION_GUIDE.md) | 集成与门禁模板 |
| 8 | [`docs/OBSERVABILITY_SECURITY_OPERATIONS.md`](./OBSERVABILITY_SECURITY_OPERATIONS.md) | 可观测 / 安全 / 运维 |
| 9 | [`docs/TEST_AND_ACCEPTANCE_PLAN.md`](./TEST_AND_ACCEPTANCE_PLAN.md) | 验收计划与发布门 |

## 1. 范围与非目标

本文定义 RBatis Plus 查询结果二级缓存的目标协议，包括稳定缓存键、`rbs::Value` 负载、内存与 Redis 后端、失效标签、并发回填和版本升级规则。

当前仓库没有 Rust 实现。本文中的类型、crate 名、trait、方法签名和默认值都是待实现 API 草案，不表示 RBatis 上游或本仓库已经提供这些能力。依赖版本、序列化库、哈希算法实现和 Redis 客户端均为 TBD，实施时必须在 [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) 中锁定并验证。

本文不保证：

- 自动理解任意 SQL 的完整读写依赖；
- 感知绕过 RBatis 的数据库写入；
- 在没有事务生命周期钩子时提供提交边界一致性；
- 通过缓存替代数据库锁、隔离级别或业务幂等；
- 缓存最终泛型结果 `T`。

事务行为由 [TRANSACTION_CONSISTENCY.md](./TRANSACTION_CONSISTENCY.md) 定义。架构边界、验收条件和设计记录分别见 [ARCHITECTURE.md](./ARCHITECTURE.md)、[TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md) 和 [DECISIONS.md](./DECISIONS.md)。背景证据见 [RBatis 支持二级缓存调研报告](./RBatis%20支持二级缓存调研报告.md)。

## 2. 规范用语

本文使用以下约束词：

- **必须**：兼容实现不可违反；
- **应该**：除非有记录充分的理由，否则应遵守；
- **可以**：可选能力；
- **TBD**：实施前尚未决定，不能伪装成已支持行为。

协议涉及的字节编码都必须按本文定义处理。不得用 Rust `Debug`、`Display`、默认 `Hash`、进程随机种子、未固定选项的通用 JSON，或依赖容器迭代顺序生成跨进程缓存键。

## 3. 处理位置与数据模型

缓存拦截器必须位于所有会改变查询身份的 SQL 重写器之后，例如租户、分片、动态表和分页拦截器。它必须使用数据库实际执行前的最终 SQL和最终参数。日志与指标拦截器位于缓存前后属于可配置的可观测性决定，但不得改变缓存身份。

目标数据流如下：

```text
final SQL + canonical args + execution context
                    |
                    v
              logical identity
                    |
                    v
 namespace/tag generations + stable key protocol
                    |
          +---------+---------+
          |                   |
        hit                  miss
          |                   |
 owned rbs::Value       database query
                              |
                              v
                       guarded cache fill
```

缓存值必须表示数据库查询返回、进入泛型解码之前的 owned `rbs::Value`。实现不得保存拦截器参数中的借用引用。

## 4. 稳定缓存键协议

### 4.1 协议标识

首个目标协议命名为 `rbp-l2-key/v1`。Redis 可读前缀建议为：

```text
rbatis-plus:l2:k1:{namespace-token}:{digest}
```

`k1` 是键协议版本，不是 crate 版本、缓存 envelope 版本或业务数据版本。实现必须允许并行读取或清理旧协议，但不得用新算法继续写旧版本前缀。

`namespace-token` 必须是经过长度限制和安全字符编码的标识；原始 namespace 不满足后端键约束时必须使用其摘要。完整身份始终进入摘要输入，因此可读 token 不能成为唯一隔离边界。

### 4.2 规范帧

摘要输入必须是带域分隔和长度前缀的二进制帧，字段顺序固定：

| 序号 | 字段 | 要求 |
|---:|---|---|
| 1 | protocol | 固定 ASCII `rbp-l2-key/v1` |
| 2 | namespace | UTF-8，非空，由策略显式指定 |
| 3 | datasource identity | 稳定、不含凭据；不能使用连接对象地址 |
| 4 | driver/dialect | 稳定标识，例如数据库类型与必要的方言版本 |
| 5 | final SQL | SQL 重写完成后的 UTF-8 字节；v1 不做语义规范化 |
| 6 | arguments | 本文第 5 节的 typed canonical encoding |
| 7 | routing context | 排序后的显式键值，如 tenant、shard、read role |
| 8 | tag generations | 排序后的 `(tag, generation)` 对 |
| 9 | policy identity | 仅包含会改变结果语义的策略字段；TTL 不进入身份 |

每个字段必须编码为 `field-id || byte-length || bytes`。整数宽度、字节序和长度上限必须在实现前写入测试向量；当前具体宽度与哈希算法为 **TBD**。被选算法必须：

- 在受支持平台和进程间得到相同摘要；
- 至少提供 128 位有效碰撞强度，建议输出 256 位摘要；
- 有公开、固定的测试向量；
- 不依赖 Rust 标准库非稳定哈希；
- 协议变更时升级 `kN`。

如威胁模型包含攻击者构造碰撞或由摘要推断低熵敏感值，实现应该使用带部署密钥的 MAC/HMAC。密钥管理方案为 TBD。Redis key 和日志不得包含 SQL 参数明文、数据库凭据或租户秘密。

### 4.3 SQL 语义

v1 必须对最终 SQL 的原始 UTF-8 字节做身份计算，除非某个数据库适配器提供有版本的、经过方言测试的规范化器。不得默认折叠空白、修改大小写、删除注释或重排语句，因为这些操作可能改变字符串字面量、提示、参数位置或数据库行为。

同义 SQL 产生不同键是可接受的命中率损失；不同语义 SQL 产生相同键不可接受。

### 4.4 数据源和路由隔离

`datasource identity` 必须由部署显式配置，且在同一数据内容域内稳定。URL 中的用户名、密码、token 和临时主机凭据不得直接进入键。

以下上下文在影响结果时必须进入键：

- tenant ID；
- shard ID；
- database/schema；
- 主库、只读副本或一致性路由角色；
- 软删除或权限范围；
- locale、timezone 或其他数据库会话语义；
- 用户提供的自定义 cache vary 维度。

缺少必要上下文时，策略必须 bypass，不能退化为共享键。

## 5. Typed canonical arguments

### 5.1 总则

参数编码必须递归保留 `rbs::Value` 类型、顺序、容器边界和数值位型。以下值必须互不冲突：

```text
NULL
false
0 (各整数类型按明确规则区分或统一)
0.0
"0"
[0]
{"0": null}
```

顶层参数是有序列表，参数位置属于查询身份。编码器必须拒绝超出配置深度、元素数或字节数的输入，并使该查询 bypass；不得截断后继续缓存。

### 5.2 规范节点格式

每个节点编码为：

```text
type-tag || payload-length || payload
```

目标类型标签至少覆盖：

| 类型族 | 规范语义 |
|---|---|
| Null | 无 payload |
| Bool | 单字节 `0` 或 `1` |
| Signed integer | 固定规则的二进制补码；宽度规则必须锁定 |
| Unsigned integer | 固定规则的无符号编码；宽度规则必须锁定 |
| Float32/Float64 | IEEE 754 原始位型；`-0` 与 `+0` 不得悄悄合并 |
| Decimal | 规范化的 sign、coefficient、scale；具体映射 TBD |
| String | UTF-8 原字节，不做 Unicode normalization |
| Binary/Bytes | 原始字节 |
| Array | 元素数及按原顺序编码的节点 |
| Map/Object | 规范排序后的键值节点 |
| Extension | 显式扩展类型 ID、扩展协议版本和 payload |

实施前必须依据锁定的 `rbs` 版本核对实际 `rbs::Value` variants。本文没有声明当前不存在或名称不同的 variant 已经可用。任何无法无损归类的 variant 必须 bypass，直到获得显式、版本化编码。

### 5.3 数值和浮点要求

实现必须在 [DECISIONS.md](./DECISIONS.md) 中选择并记录以下策略：

1. 保留具体整数宽度，或按有符号/无符号族归一化；
2. `NaN` 是否允许作为 SQL 参数；若允许，是否保留 payload 位；
3. decimal 的 coefficient/scale 规范；
4. 时间、日期、UUID 等扩展类型的稳定映射。

在选择完成前，这些类型不是“自动安全支持”，编码器应采取 bypass。禁止经由 JSON number 或十进制字符串进行会丢失类型/精度的转换。

### 5.4 Map 顺序

Map 编码不能依赖插入顺序或哈希迭代顺序。每个键和值先独立规范编码，再按编码后的键字节词典序排序。重复的规范键必须报错并 bypass，因为排序后保留哪一个值不应取决于实现细节。

### 5.5 测试向量

实现完成前必须发布跨进程测试向量，至少覆盖：

- null、bool、所有受支持整数边界；
- `+0.0`、`-0.0`、无穷和允许的 NaN；
- UTF-8、多字节文本和嵌入 NUL；
- bytes 与同内容 string 的区分；
- 嵌套 array/map；
- map 不同插入顺序得到相同编码；
- 不同参数类型或顺序得到不同摘要；
- 扩展类型版本变化使键协议或类型版本变化。

## 6. `rbs::Value` 缓存语义

缓存命中必须恢复与数据库查询成功时交给后续 decode 路径等价的 owned `rbs::Value`。

实现必须遵守：

- 只缓存成功的 query result；错误不得缓存；
- `Value::Null`、空数组和“没有缓存项”是三个不同状态；
- 空结果是否缓存由 `cache_null`/empty policy 决定，并可使用较短 TTL；
- 最大逻辑值大小和最大编码后大小都必须受限；超限时返回数据库结果但跳过回填；
- 负载编码必须无损保留受支持的 Value 语义；不能依赖会损失整数宽度、二进制或扩展类型信息的 JSON；
- codec 选择及锁定版本为 TBD；候选库不是已承诺依赖；
- 解码失败、未知 codec 或未知 envelope 版本视为 miss，并按故障策略记录；不得把损坏 payload 交给应用解码。

缓存发生在泛型 `decode<T>` 之前，因此同一个查询键不包含 Rust 返回类型 `T`。调用方若使用不同目标类型解码同一数据库结果，其成功或失败应与无缓存路径保持一致。

## 7. 缓存 envelope 与版本管理

### 7.1 逻辑结构

内存和 Redis 后端应使用相同逻辑 envelope；内存实现可以保存结构体而不序列化。目标草案：

```rust
pub struct CacheEnvelope {
    pub envelope_version: u16,
    pub key_protocol: u16,
    pub codec: CacheCodecId,
    pub codec_version: u16,
    pub created_at_unix_ms: i64,
    pub fresh_until_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub fill_token: FillToken,
    pub dependency_snapshot: Vec<TagGeneration>,
    pub payload_checksum: Option<Vec<u8>>,
    pub payload: Vec<u8>,
}
```

该签名仅为目标草案。时间字段类型、校验算法、codec enum 和 token 宽度均为 TBD。

### 7.2 新鲜度

- `now < fresh_until`：fresh，可直接返回；
- `fresh_until <= now < expires_at`：stale，仅在明确启用 stale-while-revalidate 时可返回；
- `now >= expires_at`：expired，必须视为 miss；
- 时钟回拨或非法时间关系：视为无效 envelope。

TTL 的随机抖动必须只影响写入后的有效期，不进入逻辑键。软 TTL/后台刷新默认不启用，直到过期数据语义获得独立验收。

### 7.3 升级规则

- envelope 兼容增加字段时，读取器必须定义缺省语义；
- 不兼容变化必须升级 `envelope_version`；
- key 输入或 canonical args 变化必须升级 `key_protocol`；
- payload 表示变化必须升级 codec 或 codec version；
- 新写入只使用当前版本；旧版本可以在迁移窗口读取，但不得无界期双写；
- 未知版本必须 fail-open 为 miss，并产生有限速的指标/日志；
- 文档弃用旧版本，不删除其迁移说明。

## 8. 显式 namespace 与 tags

### 8.1 为什么要求显式标签

任意 SQL 的依赖无法可靠地从文本自动推断，特别是 CTE、JOIN、schema、触发器、存储过程和动态 SQL。因此精确失效必须来自显式 metadata，而不是未验证的 SQL 猜测。

目标策略示例仅表示意图，尚无宏或运行时 API：

```rust
CachePolicy::new("activity")
    .tags(["table:biz_activity"])
    .ttl(Duration::from_secs(60))
```

### 8.2 标签规范

- namespace 必须非空，并定义管理和保守清理边界；
- query tags 表示该结果依赖的版本；
- write/flush tags 表示成功写入可能改变的数据域；
- tag 必须来自受控、稳定命名，不得拼入未哈希的秘密；
- 推荐格式为 `kind:name`，例如 `table:biz_activity`、`aggregate:customer:42`；
- 用户输入进入高基数 tag 前必须有长度、字符和数量限制；
- query 的 tag 集合和 generation snapshot 必须排序去重；
- 无法声明完整依赖时必须选保守 namespace 失效或 bypass。

### 8.3 版本化失效

目标模型使用单调 generation：

```text
namespace generation N
query tag A generation 7
query tag B generation 12
logical key includes (N, A=7, B=12)
invalidate A -> atomically advance A to 8
```

失效不要求扫描并删除旧值；旧键等待 TTL/容量淘汰。generation 存储丢失或回退可能复活旧键，因此后端必须提供持久、原子、单调的更新语义，或使用不会复用的 epoch。精确 representation 和溢出策略为 TBD。

## 9. Singleflight 与 stale-fill prevention

### 9.1 Singleflight

同一完整逻辑键的并发 miss 应合并为一个 leader 数据库查询。其他 waiter 等待 leader，随后获得同一个成功值，或在 leader 失败后按策略重试/直查。

要求：

- singleflight 作用域至少是同一 `CacheIntercept`/进程；
- Redis 分布式锁是可选增强，不是 v1 正确性的前提；
- lock key 必须使用完整稳定摘要和独立域前缀；
- leader 取消、panic、超时和 DB 错误必须释放 waiter；
- 不得持有进程锁跨越无限期后端调用；
- waiter 数、等待时间和 leader 查询必须可观测；
- fail-open 不等于无限制并发穿透，应配置超时和退避；
- 分布式锁若实现，必须有租约所有权 token；仅 `DEL` 一个可能已过期并被他人取得的锁是错误实现。

### 9.2 防止失效后的旧查询回填

仅有 singleflight 不能阻止以下竞争：

```text
Q1 读取 tag generation=7，缓存 miss，开始慢查询
W1 成功提交，tag generation 递增为 8
Q1 返回旧快照结果并尝试回填
```

每次 fill 必须携带开始查询前读取的 dependency snapshot。写缓存前必须原子或等价地验证：

```text
current generations == dependency snapshot
```

如果任一 generation 已改变，必须丢弃回填，但仍把数据库结果返回给原调用者。这是 **stale-fill prevention**，不保证原查询在数据库隔离级别之外获得更新值。

内存后端可以在同一协调器锁/原子版本检查下完成 compare-and-publish。Redis 后端必须通过 Lua、事务性 CAS 或等价服务器端原子操作同时验证 generation 并写入 envelope；`GET generations` 后客户端单独 `SET` 存在 TOCTOU 竞争，不符合规范。

若后端不能提供验证能力，实现必须使用更保守的顺序方案或禁用回填；不得宣称满足原生一致性模式。

## 10. 内存后端行为

内存后端目标为进程内、同一缓存实例共享，不能跨进程自动一致。具体缓存库和版本为 TBD；Moka 只是候选，不是既有依赖。

必须具备：

- 有界容量或有界权重；
- TTL，推荐支持按 entry 过期；
- owned `rbs::Value` 或逻辑 envelope；
- 进程内 singleflight；
- generation 与 compare-and-publish；
- namespace/tag generation 操作；
- 容量淘汰不影响 generation 正确性；
- 命中、miss、回填、丢弃旧回填、淘汰和错误指标。

如果维护 `tag -> key set`，淘汰必须清理反向索引，否则该索引会无界增长。推荐优先使用 generation 方案，避免逐键扫描。generation 元数据本身仍需要容量策略；高基数动态 tag 必须有配额或禁用。

多进程部署使用纯内存后端时，每个进程只看见自己的 DML 事件。文档和配置必须明确这一限制；短 TTL 不是强一致保证。

## 11. Redis 后端行为

Redis 后端是独立可选 crate 的目标，不得把 Redis 客户端强制引入核心 crate。客户端、连接池、runtime features 和最低 Redis 版本均为 TBD。

目标 keyspace 使用不同域：

```text
rbatis-plus:l2:k1:...     cache values
rbatis-plus:l2:g1:...     namespace/tag generations
rbatis-plus:l2:sf1:...    optional distributed singleflight leases
rbatis-plus:l2:ch1:...    optional invalidation notifications
```

要求：

- value 使用带版本 envelope 的二进制安全存储和原子 TTL；
- generation 递增必须原子且单调；
- compare generations 与 publish fill 必须服务器端原子；
- Pub/Sub 只能作为降低本地陈旧窗口的提示，不能作为唯一真相，因为消息可能丢失；
- Redis unavailable 时按 `CacheFailureMode` 执行，默认建议 `FailOpen`，数据库路径仍受限流保护；
- Redis timeout、codec 错误和协议未知必须有低基数指标；
- TTL 应支持有界 jitter，避免同批键同时失效；
- 不得用 `KEYS` 或生产环境全量扫描实现正常写路径失效；
- 清 namespace 应优先推进 generation，而不是同步删除全部值；
- ACL、TLS、凭据轮换和集群拓扑属于部署配置，具体 API 为 TBD。

Redis Cluster 下，多 key Lua/CAS 需要 hash slot 设计。generation 与 value 是否放在相同 slot、是否采用 namespace hash tag，以及这对热点的影响均为 TBD，必须在实现前验证；在此之前不能声称支持 Redis Cluster 原子 stale-fill prevention。

## 12. 读写算法

### 12.1 普通查询

```text
1. 取得最终 SQL、canonical args、datasource 和 vary context。
2. 解析显式 CachePolicy；不完整或不可缓存则 bypass。
3. 读取 namespace/tag generation snapshot。
4. 构建稳定 key。
5. 读取并验证 envelope；fresh hit 直接返回 Value。
6. miss 时进入 singleflight。
7. leader 再次检查缓存，随后查询数据库。
8. 查询成功后检查空值和大小策略。
9. compare current generations with snapshot and publish。
10. generation 改变则丢弃 fill；始终返回数据库结果。
11. 查询或缓存失败时释放 waiter 并执行故障策略。
```

锁查询、非确定性查询、用户 bypass 以及缺少必要上下文的查询默认不缓存。

### 12.2 普通写入

```text
1. before 阶段解析显式 flush tags 或保守 namespace。
2. 执行数据库写入。
3. 仅数据库返回成功后推进 generation。
4. 数据库失败时不得按普通规则失效。
5. 失效后端失败时按配置 fail-open/fail-closed；必须记录一致性降级。
```

事务写入不能套用上述“DML 返回成功即视为提交”的规则，详见 [TRANSACTION_CONSISTENCY.md](./TRANSACTION_CONSISTENCY.md)。

## 13. 故障语义和可观测性

目标故障模式：

- `FailOpen`：缓存操作失败时访问数据库或跳过回填；可用性优先；
- `FailClosed`：缓存一致性操作失败时令业务操作返回错误；仅在部署明确接受时启用；
- `BypassOnUncertain`：上下文、编码或依赖不完整时不缓存，这是协议默认行为，不应计为后端故障。

至少记录以下低基数指标：

- hit、miss、stale hit、bypass reason；
- singleflight leader/waiter/timeout；
- fill success、oversize skip、generation mismatch discard；
- invalidation success/failure；
- envelope/codec/version error；
- backend latency 和 failure mode；
- transaction mode 与 pending invalidation 结果。

日志不能输出参数、完整缓存 payload 或包含秘密的原始键。可记录协议版本、namespace 的安全标识、摘要前缀、statement ID 和错误类别。

## 14. 兼容实现检查表

实现进入“可用”状态前必须满足：

- [ ] 锁定 `rbs`、哈希、codec、内存缓存和 Redis 客户端版本；
- [ ] 发布键协议和 typed canonical args 测试向量；
- [ ] 对所有受支持 `rbs::Value` variants 做无损 round-trip；
- [ ] 未支持类型明确 bypass；
- [ ] stable key 不泄露参数；
- [ ] 内存和 Redis 后端通过同一语义测试套件；
- [ ] singleflight 处理取消、超时和 leader 失败；
- [ ] stale-fill prevention 通过读写竞争测试；
- [ ] tag generation 不回退、不静默复用；
- [ ] Redis Cluster 支持范围有明确声明；
- [ ] 事务行为通过 [TRANSACTION_CONSISTENCY.md](./TRANSACTION_CONSISTENCY.md) 的矩阵；
- [ ] 验收结果记录在 [TEST_AND_ACCEPTANCE_PLAN.md](./TEST_AND_ACCEPTANCE_PLAN.md) 对应条目。

## 15. 尚未决定的事项

以下内容必须保持诚实标注，直到通过决策和测试：

1. Rust crate/API 的最终命名与 object-safe async trait 方案；
2. `rbs::Value` 目标版本及全部 variant 映射；
3. 摘要或 MAC 算法、帧长度宽度和端序；
4. payload codec、checksum 和 schema 演进工具；
5. memory cache 库、容量权重算法和 generation 元数据存储；
6. Redis 客户端、最低服务端版本、Cluster 原子脚本设计；
7. stale-while-revalidate 是否进入首版；
8. 分布式 singleflight 是否提供，以及锁租约/fencing 语义；
9. 高基数 tags 的配额和管理 API；
10. fail-closed 与业务事务提交后失效失败之间的产品语义。

这些决定应进入 [DECISIONS.md](./DECISIONS.md)，实施排序进入 [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)，不得仅存在于代码注释。

## 16. MyBatis 对照参考

> MyBatis 3 是 RBatis 缓存模型的直接参考。CodeGraph 给出：1,807 个类、6,075 个函数、88 个执行流。
> MyBatis 通过 `Cache` SPI、`CachingExecutor`、`TransactionalCache` 提供了 L1 + L2 + 事务后提交的三段式缓存。
> RBatis 当前没有等价实现，RBatis-Plus 计划复用相同的分层语义，并基于 `rbs::Value` 与事务事件扩展。
> 本节仅作对照参考，所有 RBatis-Plus 能力均标注为 planned/TBD。
> MyBatis 行号引用本地代码：`/Users/wandl/workspaces/workspace-github/mybatis-3/src/main/java/org/apache/ibatis/...`。
> RBatis 行号引用本地代码：`/Users/wandl/workspaces/workspace-github/rbatis/src/...`。

### 16.1 SPI 与注册

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `Cache` SPI | `Cache.java:19-27, 41-97` | 无 | 计划 `CacheStore` trait |
| Per-namespace Cache 实例 | `Configuration.java:771-773` | 无 namespace 概念 | 计划 `namespace` 字段 |
| 装饰器栈 | `LruCache` / `FifoCache` / `SoftCache` / `BlockingCache` / `TransactionalCache` | 无 | 计划 `MemoryCacheStore` + `RedisCacheStore` |
| `Configuration.addCache` | `Configuration.java:771-773` | `RBatis::set_intercepts` | `RBatisPlus` 注册 `CacheStore` |

### 16.2 CacheKey 协议

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| 累加 + checksum | `CacheKey.update` 行 74-84 | 字符串参数 | 计划稳定、版本化二进制协议 |
| 包含 msId + offset + limit + SQL + 参数 + env | `BaseExecutor.createCacheKey` 行 199-243 | 字符串 SQL + 参数 | 计划 versioned envelope，含 driver、改写后 SQL、canonical args、tenant、shard、version |
| 类型安全 | `CacheKey.update` 通过 TypeHandler 处理 null/非 null | RBatis 通过 `rbs::Value` 区分 | 计划 typed canonical encoding |

### 16.3 L1 与 L2 分层

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `BaseExecutor.localCache` | `BaseExecutor.java:54-72` | 无 | 计划 `MemoryCacheStore`（进程内） |
| `CachingExecutor` 装饰 | `CachingExecutor.java:38-46` | 无 | 计划 `CachingExecutor<E: Executor>` |
| 共享 L2 | `MappedStatement.getCache` | 无 | 计划 Redis 后端 |
| 多级缓存 | 不支持，MyBatis 官方仅两级 | 无 | 计划 L1 + L2 多级（见 §2.3） |

### 16.4 事务回填

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `entriesToAddOnCommit` | `TransactionalCache.java:78-81` | 无 | 计划 `pending invalidation` |
| `commit` 回填 delegate | `TransactionalCache.java:94-100, 113-122` | 无 | 计划 `commit-success` 后应用 pending |
| `rollback` 释放 miss 锁 | `TransactionalCache.java:102-105, 124-133` | 无 | 计划 `rollback` 丢弃 pending |
| `clearOnCommit` | `TransactionalCache.java:88-92` | 无 | 计划 `flush_pending_on_commit` 配置 |
| `TransactionalCacheManager` | `TransactionalCacheManager.java:25-55` | 无 | 计划 `CacheManager` |

### 16.5 参数编码

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `TypeHandlerRegistry` | `TypeHandlerRegistry.java:60-76, 93-176` | 编译期类型 + `rbs::Value` | 计划 typed canonical encoding |
| `DefaultParameterHandler.setParameters` | `DefaultParameterHandler.java:75-103, 104-177` | 编译期 SQL 生成 | 计划 typed canonical args |
| `null` 值 | `DefaultParameterHandler.java:115-122` | 通过 `rbs::Value::Null` | 计划 stable key 不泄露参数 |
| 嵌套对象 | `DefaultParameterHandler.java:108-114` | 编译期展开 | 计划 ParamMap 兼容 |

### 16.6 动态 SQL 与最终 SQL

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `DynamicSqlSource.getBoundSql` | `DynamicSqlSource.java:26-50` | 编译期 py_sql/html_sql | 计划 “最终 SQL 后构键” |
| `RawSqlSource` | `RawSqlSource.java:35-69` | 同上 | 同上 |
| `ProviderSqlSource` | `ProviderSqlSource.java:100-212` | 编译期生成 | 计划 `#[cacheable]` 元数据 |

### 16.7 缓存对象

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| 缓存结果对象 | `CachingExecutor.query` 行 99-108 | 无 | 计划缓存 owned `rbs::Value` |
| `EXECUTION_PLACEHOLDER` 防递归 | `BaseExecutor.query` 行 153-162 | 无 | 计划类似 placeholder |

### 16.8 缓存清理

| MyBatis 机制 | MyBatis 关键位置 | 当前 RBatis 等价 | RBatis-Plus planned |
|---|---|---|---|
| `flushCacheIfRequired` | `CachingExecutor.java:168-173` | 无 | 计划 hook 在 `before` 失效 |
| `useCache` / `flushCacheRequired` | `MappedStatement.java:268, 272-274` | 无 | 计划 `CachePolicy` |
| 事务提交后失效 | `CachingExecutor.commit` 行 119-122 | 无 | 计划 `commit-success` + `tag invalidation` |
| 版本化 tag | 无 | 无 | 计划 `tag version`（见 §5.2） |
| stale-fill prevention | 无 | 无 | 计划生成 comparison + 锁租约 |

